//! Capa de persistencia para el almacén clave-valor.
//! Este módulo provee dos mecanismos para durabilidad:
//!
//! - [`log`]: Registro de escritura para actualizaciones incrementales
//! - [`store`]: Gestión de snapshots para recuperación eficiente

pub mod log;
pub mod store;
