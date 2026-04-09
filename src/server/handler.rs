//! lee lineas con bufreader
//! para get lee directamente del rwlock<hashmap>
//! para set escribe al channel del writer
//! configura timeouts

use crate::server::writer::{WriteOperation, WriteResult};
use crate::{protocol, Command, CommandType, Error, KvStore};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{sync_channel, Sender};
use std::sync::{Arc, RwLock};

const READ_TIMEOUT_MS: u64 = 2_000;
const WRITE_TIMEOUT_MS: u64 = 2_000;

/// Maneja una conexión de cliente, procesando comandos hasta desconexión
///
/// # Errors
/// Devuelve un error si ocurre un problema de I/O al leer o escribir en el stream
///
/// # Ejemplo
/// ``` ignore
/// use minikv::server::handler;
/// use std::net::TcpStream;
/// let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
/// handler::handle_client(stream, &mut store);
/// ```
pub fn handle_client(
    stream: &TcpStream,
    store: &Arc<RwLock<KvStore>>,
    writer_tx: &Sender<WriteOperation>,
) -> std::io::Result<()> {
    configure_timeouts(stream)?;
    let reader = BufReader::new(stream);
    let mut stream = stream;
    for line in reader.lines() {
        let line = match line {
            Ok(content) => content,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::TimedOut {
                    return Err(std::io::Error::new(err.kind(), Error::Timeout.code()));
                }
                return Err(err);
            }
        };
        let response = process_line(&line, store, writer_tx);
        if let Err(err) = writeln!(stream, "{response}") {
            if err.kind() == std::io::ErrorKind::TimedOut {
                return Err(std::io::Error::new(err.kind(), Error::Timeout.code()));
            }
            return Err(err);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        Error::ConnectionClosed.code(),
    ))
}

fn configure_timeouts(stream: &TcpStream) -> std::io::Result<()> {
    let read_timeout = std::time::Duration::from_millis(READ_TIMEOUT_MS);
    let write_timeout = std::time::Duration::from_millis(WRITE_TIMEOUT_MS);
    stream.set_read_timeout(Some(read_timeout))?;
    stream.set_write_timeout(Some(write_timeout))?;
    Ok(())
}

/// Procesa una línea de comando, ejecutando la acción correspondiente
fn process_line(
    line: &str,
    store: &Arc<RwLock<KvStore>>,
    writer_tx: &Sender<WriteOperation>,
) -> String {
    let cmd = match Command::parse_from_string(line) {
        Ok(cmd) => cmd,
        Err(e) => return protocol::error(e),
    };
    execute_command(&cmd, store, writer_tx)
}

/// Ejecuta un comando, accediendo al store según el tipo de operación
fn execute_command(
    cmd: &Command,
    store: &Arc<RwLock<KvStore>>,
    writer_tx: &Sender<WriteOperation>,
) -> String {
    match cmd.get_type() {
        CommandType::Get => execute_get(cmd, store),
        CommandType::Set => execute_set(cmd, writer_tx),
        CommandType::Length => execute_length(store),
        CommandType::Snapshot => execute_snapshot(writer_tx),
    }
}

fn execute_get(cmd: &Command, store: &Arc<RwLock<KvStore>>) -> String {
    let key = match cmd.get_key() {
        Ok(k) => k,
        Err(e) => return protocol::error(e),
    };
    let Ok(guard) = store.read() else {
        return protocol::error(Error::ConnectionClosed);
    };
    match guard.get(key) {
        Some(value) => protocol::value(value),
        None => protocol::error(Error::NotFound),
    }
}
fn execute_set(cmd: &Command, writer_tx: &Sender<WriteOperation>) -> String {
    let key = match cmd.get_key() {
        Ok(k) => k,
        Err(e) => return protocol::error(e),
    };
    let value = match cmd.get_value() {
        Ok(Some(v)) => v,
        Ok(None) => "",
        Err(e) => return protocol::error(e),
    };
    send_and_wait(
        writer_tx,
        WriteOperation::Set {
            key: key.to_string(),
            value: value.to_string(),
            response: sync_channel(0).0,
        },
    )
}
fn execute_length(store: &Arc<RwLock<KvStore>>) -> String {
    let Ok(guard) = store.read() else {
        return protocol::error(Error::ConnectionClosed);
    };
    protocol::number(guard.len())
}
fn execute_snapshot(writer_tx: &Sender<WriteOperation>) -> String {
    send_and_wait(
        writer_tx,
        WriteOperation::Snapshot {
            response: sync_channel(0).0,
        },
    )
}
fn send_and_wait(writer_tx: &Sender<WriteOperation>, op: WriteOperation) -> String {
    let (resp_tx, resp_rx) = sync_channel(0);
    let op = attach_response_channel(op, resp_tx);
    if writer_tx.send(op).is_err() {
        return protocol::error(Error::ConnectionClosed);
    }
    match resp_rx.recv() {
        Ok(result) => format_write_result(result),
        Err(_) => protocol::error(Error::ConnectionClosed),
    }
}

fn attach_response_channel(
    op: WriteOperation,
    resp_tx: std::sync::mpsc::SyncSender<WriteResult>,
) -> WriteOperation {
    match op {
        WriteOperation::Set { key, value, .. } => WriteOperation::Set {
            key,
            value,
            response: resp_tx,
        },
        WriteOperation::Snapshot { .. } => WriteOperation::Snapshot { response: resp_tx },
    }
}

fn format_write_result(result: WriteResult) -> String {
    match result {
        Ok(()) => protocol::ok(),
        Err(err) => protocol::error(err),
    }
}
