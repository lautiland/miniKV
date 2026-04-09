//! lee lineas con bufreader
//! para get lee directamente del rwlock<hashmap>
//! para set escribe al channel del writer
//! configura timeouts

use crate::server::writer::WriteOperation;
use crate::{protocol, Command, CommandType, KvStore};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{sync_channel, Sender};
use std::sync::{Arc, RwLock};

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
    let reader = BufReader::new(stream);
    let mut stream = stream;
    for line in reader.lines() {
        let line = line?;
        let response = process_line(&line, store, writer_tx);
        writeln!(stream, "{response}")?;
    }
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
        Err(e) => return protocol::error(&e),
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
        Err(e) => return protocol::error(&e),
    };
    let Ok(guard) = store.read() else {
        return protocol::error("Error al acceder al store");
    };
    match guard.get(&key) {
        Some(value) => protocol::ok_value(&value),
        None => protocol::error("Clave no encontrada"),
    }
}
fn execute_set(cmd: &Command, writer_tx: &Sender<WriteOperation>) -> String {
    let key = match cmd.get_key() {
        Ok(k) => k,
        Err(e) => return protocol::error(&e),
    };
    let value = match cmd.get_value() {
        Ok(v) => v,
        Err(e) => return protocol::error(&e),
    };
    send_and_wait(
        writer_tx,
        WriteOperation::Set {
            key,
            value,
            response: sync_channel(0).0, // Canal sin buffer para sincronización
        },
    )
}
fn execute_length(store: &Arc<RwLock<KvStore>>) -> String {
    let Ok(guard) = store.read() else {
        return protocol::error("Error al acceder al store");
    };
    protocol::ok_value(&guard.len().to_string())
}
fn execute_snapshot(writer_tx: &Sender<WriteOperation>) -> String {
    send_and_wait(
        writer_tx,
        WriteOperation::Snapshot {
            response: sync_channel(0).0, // Canal sin buffer para sincronización
        },
    )
}
fn send_and_wait(writer_tx: &Sender<WriteOperation>, op: WriteOperation) -> String {
    let (resp_tx, resp_rx) = sync_channel(0); // Canal sin buffer para sincronización
    let op = match op {
        WriteOperation::Set { key, value, .. } => WriteOperation::Set {
            key,
            value,
            response: resp_tx,
        },
        WriteOperation::Snapshot { .. } => WriteOperation::Snapshot { response: resp_tx },
    };
    if writer_tx.send(op).is_err() {
        return protocol::error("Error al enviar operación al writer");
    }
    match resp_rx.recv() {
        Ok(result) => match result {
            Ok(msg) => protocol::ok_value(&msg),
            Err(err) => protocol::error(&err),
        },
        Err(_) => protocol::error("Error al recibir respuesta del writer"),
    }
}
