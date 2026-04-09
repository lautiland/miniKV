//! Listener TCP para el servidor de `MiniKV`

use crate::server::{handler, writer};
use crate::KvStore;
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
    let listener = std::net::TcpListener::bind(addr)?;
    println!("Servidor escuchando en {addr}");
    // Cargar el store al iniciar el servidor
    let store = Arc::new(RwLock::new(KvStore::load()?));

    let (writer_tx, writer_rx) = channel();
    let writer_store = Arc::clone(&store);
    thread::spawn(move || {
        writer::start(&writer_store, &writer_rx);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let handler_store = Arc::clone(&store);
                let handler_writer_tx = writer_tx.clone();
                thread::spawn(move || {
                    if let Err(e) =
                        handler::handle_client(&stream, &handler_store, &handler_writer_tx)
                    {
                        eprintln!("Error al manejar conexión: {e}");
                    }
                });
            }
            Err(e) => eprintln!("Error al aceptar conexión: {e}"),
        }
    }
    Ok(())
}
