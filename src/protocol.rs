//! Protocolo de respuestas para el servidor `MiniKV`.
//!
//! Las respuestas siguen el formato de la consigna:
//! - Éxito: `OK`
//! - Valores y números: se imprimen como texto sin prefijo
//! - Errores: `ERROR "<MOTIVO>"`

use crate::Error;

#[must_use]
pub fn ok() -> String {
    "OK".to_string()
}

#[must_use]
pub fn value(value: &str) -> String {
    value.to_string()
}

#[must_use]
pub fn error(error: Error) -> String {
    format!("ERROR \"{}\"", error.code())
}

#[must_use]
pub fn number(n: usize) -> String {
    n.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ok() {
        assert_eq!(ok(), "OK");
    }
    #[test]
    fn test_error() {
        assert_eq!(error(Error::NotFound), "ERROR \"NOT FOUND\"");
    }
    #[test]
    fn test_number() {
        assert_eq!(number(42), "42");
        assert_eq!(number(0), "0");
    }
    #[test]
    fn test_value() {
        assert_eq!(value("hello"), "hello");
        assert_eq!(value("hello world"), "hello world");
    }
}
