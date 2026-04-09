//! Listener TCP para el servidor de `MiniKV`

use crate::server::{handler, writer};
use crate::{protocol, Error, KvStore};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::thread;

/// Inicia el servidor TCP y maneja las conexiones entrantes.
///
/// # Errors
/// Devuelve un error si no se puede iniciar el servidor o si ocurre un error al manejar una conexión.
///
/// # Ejemplo
/// ``` ignore
/// use minikv::server::listener;
/// listener::start("127.0.0.1:8080");
/// ```
pub fn start(addr: &str) -> std::io::Result<()> {
    let listener = bind_listener(addr)?;
    println!("Servidor escuchando en {addr}");
    let store = Arc::new(RwLock::new(map_storage_result(KvStore::load())?));
    let (writer_tx, writer_rx) = channel();
    start_writer_thread(&store, writer_rx);
    accept_connections(&listener, &store, &writer_tx);
    Ok(())
}

fn bind_listener(addr: &str) -> std::io::Result<std::net::TcpListener> {
    std::net::TcpListener::bind(addr)
        .map_err(|_| std::io::Error::other(Error::ServerSocketBinding.code()))
}

fn start_writer_thread(
    store: &Arc<RwLock<KvStore>>,
    writer_rx: std::sync::mpsc::Receiver<writer::WriteOperation>,
) {
    let writer_store = Arc::clone(store);
    thread::spawn(move || {
        writer::start(&writer_store, &writer_rx);
    });
}

fn accept_connections(
    listener: &std::net::TcpListener,
    store: &Arc<RwLock<KvStore>>,
    writer_tx: &std::sync::mpsc::Sender<writer::WriteOperation>,
) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => spawn_handler_thread(store, writer_tx, stream),
            Err(_) => println!("{}", protocol::error(Error::ConnectionClosed)),
        }
    }
}

fn spawn_handler_thread(
    store: &Arc<RwLock<KvStore>>,
    writer_tx: &std::sync::mpsc::Sender<writer::WriteOperation>,
    stream: std::net::TcpStream,
) {
    let handler_store = Arc::clone(store);
    let handler_writer_tx = writer_tx.clone();
    thread::spawn(move || {
        if let Err(e) = handler::handle_client(&stream, &handler_store, &handler_writer_tx) {
            let err = map_connection_error(&e);
            println!("{}", protocol::error(err));
        }
    });
}

fn map_storage_result(result: std::io::Result<KvStore>) -> std::io::Result<KvStore> {
    match result {
        Ok(store) => Ok(store),
        Err(err) => Err(map_storage_error(&err)),
    }
}

fn map_storage_error(err: &std::io::Error) -> std::io::Error {
    let message = err.to_string();
    if message.contains(Error::InvalidDataFile.code()) {
        return std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            Error::InvalidDataFile.code(),
        );
    }
    if message.contains(Error::InvalidLogFile.code()) {
        return std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            Error::InvalidLogFile.code(),
        );
    }
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        Error::InvalidDataFile.code(),
    )
}

fn map_connection_error(err: &std::io::Error) -> Error {
    if err.kind() == std::io::ErrorKind::TimedOut {
        return Error::Timeout;
    }
    Error::ConnectionClosed
}
