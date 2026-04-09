//! `MiniKV` - Un almacén clave-valor simple con persistencia
//!
//! Esta biblioteca provee un almacén clave-valor ligero basado en archivos con
//! Write-Ahead Logging (WAL) para durabilidad y soporte de snapshots para
//! recuperación eficiente.
//!
//! # Características
//!
//! - **Almacenamiento en memoria**: Operaciones rápidas clave-valor usando `HashMap`
//! - **Persistencia**: Write-ahead logging asegura durabilidad
//! - **Snapshots**: Snapshots periódicos optimizan el tiempo de recuperación
//! - **Interfaz CLI**: Interfaz de línea de comandos simple para operaciones básicas
//!
//! # Arquitectura
//!
//! El almacén mantiene dos archivos:
//! - `.minikv.data`: Snapshot del estado actual
//! - `.minikv.log`: Write-ahead log de operaciones desde el último snapshot
//!
//! Al iniciar, el almacén carga el snapshot y reproduce el log para
//! reconstruir el estado actual.
//!
//! # Ejemplo
//!
//! ```
//! use minikv::KvStore;
//! let mut store = KvStore::new();
//! store.set("key", "value").expect("Failed to set value");
//! let store = KvStore::load().expect("Failed to load store");
//! assert_eq!(store.get("key"), Some("value".to_string()));
//! ```

use minikv::{
    execute_get, execute_length, execute_set, execute_snapshot, Command, CommandType, Error,
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command: Result<Command, String> = Command::new(&args);
    match command {
        Ok(cmd) => match (cmd.cmd_type(), cmd.get_key(), cmd.get_value()) {
            (CommandType::Set, Ok(clave), Ok(valor)) => execute_set(&clave, &valor),
            (CommandType::Get, Ok(clave), _) => execute_get(&clave),
            (CommandType::Length, _, _) => execute_length(),
            (CommandType::Snapshot, _, _) => execute_snapshot(),
            (_, _, _) => println!("{}", Error::NotFound.msg()),
        },
        Err(e) => println!("{e}"),
    }
}
