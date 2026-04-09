pub mod command;
pub mod error;
pub mod kv_store;
pub mod persistence;
pub mod protocol;
pub mod server;

pub use command::{Command, CommandType};
pub use error::Error;
pub use kv_store::KvStore;

/// Módulo de sincronización para tests que acceden a archivos compartidos.
#[doc(hidden)]
pub mod test_sync {
    use std::sync::{Mutex, OnceLock};
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    pub fn get_lock() -> &'static Mutex<()> {
        TEST_LOCK.get_or_init(|| Mutex::new(()))
    }
}
