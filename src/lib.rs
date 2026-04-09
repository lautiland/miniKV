pub mod command;
pub mod error;
pub mod kv_store;
pub mod persistence;
pub mod protocol;
pub mod server;

pub use command::{Command, CommandType};
pub use error::Error;
pub use kv_store::KvStore;
/// Maneja errores de carga centralizando la lógica de conversión.
fn error_load_handle(error: &std::io::Error) {
    let error_msg = error.to_string();
    if error_msg.contains("INVALID DATA FILE") {
        println!("{}", Error::InvalidDataFile.msg());
    } else if error_msg.contains("INVALID LOG FILE") {
        println!("{}", Error::InvalidLogFile.msg());
    } else {
        println!("{}", Error::InvalidDataFile.msg());
    }
}

/// Ejecuta el comando SET: guarda o actualiza una clave-valor.
pub fn execute_set(key: &str, value: &str) {
    let mut storage = match KvStore::load() {
        Ok(sto) => sto,
        Err(err) => {
            error_load_handle(&err);
            return;
        }
    };

    match storage.set(key, value) {
        Ok(()) => println!("OK"),
        Err(error) => println!("{error}"),
    }
}

/// Ejecuta el comando GET: recupera el valor de una clave.
pub fn execute_get(key: &str) {
    let storage = match KvStore::load() {
        Ok(sto) => sto,
        Err(err) => {
            error_load_handle(&err);
            return;
        }
    };
    match storage.get(key) {
        Some(value) => println!("{value}"),
        None => println!("{}", Error::NotFound.msg()),
    }
}

/// Ejecuta el comando LENGTH: retorna la cantidad de claves con valor.
pub fn execute_length() {
    let storage = match KvStore::load() {
        Ok(sto) => sto,
        Err(err) => {
            error_load_handle(&err);
            return;
        }
    };
    println!("{}", storage.len());
}

/// Ejecuta el comando SNAPSHOT: persiste el estado actual y limpia el log.
pub fn execute_snapshot() {
    let storage = match KvStore::load() {
        Ok(sto) => sto,
        Err(err) => {
            error_load_handle(&err);
            return;
        }
    };
    match storage.snapshot() {
        Ok(()) => println!("OK"),
        Err(err) => println!("{err}"),
    }
}

/// Módulo de sincronización para tests que acceden a archivos compartidos.
#[doc(hidden)]
pub mod test_sync {
    use std::sync::{Mutex, OnceLock};
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    pub fn get_lock() -> &'static Mutex<()> {
        TEST_LOCK.get_or_init(|| Mutex::new(()))
    }
}
