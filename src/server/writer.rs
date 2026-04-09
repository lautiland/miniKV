//! recibe operaciones de `mpsc::channel`
//! es el único que escribe al kvstore
//! maneja persistencia (oplog, snapshot)

use crate::{Error, KvStore};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, RwLock};

/// Operación de escritura que el writer recibe para ejecutar en el store
pub enum WriteOperation {
    Set {
        key: String,
        value: String,
        response: SyncSender<WriteResult>,
    },
    Snapshot {
        response: SyncSender<WriteResult>,
    },
}

/// Resultado de una operación de escritura, que el writer envía de vuelta al handler
pub type WriteResult = Result<(), Error>;

/// Inicia el writer, que procesa operaciones de escritura recibidas por el canal
/// El writer es el único que modifica el store, asegurando consistencia y manejo de persistencia
pub fn start(store: &Arc<RwLock<KvStore>>, rx: &Receiver<WriteOperation>) {
    loop {
        let Ok(op) = rx.recv() else { break };
        match op {
            WriteOperation::Set {
                key,
                value,
                response,
            } => {
                let result = execute_set(&key, &value, store);
                let _ = response.send(result);
            }
            WriteOperation::Snapshot { response } => {
                let result = execute_snapshot(store);
                let _ = response.send(result);
            }
        }
    }
}

fn execute_set(key: &str, value: &str, store: &Arc<RwLock<KvStore>>) -> WriteResult {
    let Ok(mut store) = store.write() else {
        return Err(Error::ConnectionClosed);
    };
    store.set(key, value).map_err(|_| Error::InvalidLogFile)
}

fn execute_snapshot(store: &Arc<RwLock<KvStore>>) -> WriteResult {
    let Ok(store) = store.read() else {
        return Err(Error::ConnectionClosed);
    };
    store.snapshot().map_err(|_| Error::InvalidDataFile)
}
