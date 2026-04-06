//! Registro de escritura para operaciones clave-valor durables.
//!
//! Todas las modificaciones se añaden a `.minikv.log` antes de aplicarse
//! al almacén en memoria.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;

const LOG_FILE_NAME: &str = ".minikv.log";

/// Agrega una operación al final del archivo de log.
/// Las operaciones se escriben en el formato: `set "clave" "valor"`
///
/// # Errors
/// Retorna error si falla la apertura o escritura del archivo.
///
/// # Ejemplo
/// ```
/// add_operation(r#"set "name" "Alice""#).expect("Failed to log");
/// ```
pub fn add_operation(operation: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_NAME)?;
    writeln!(file, "{operation}")?;
    Ok(())
}

/// Lee todas las operaciones del log validando el formato.
/// Cada línea debe comenzar con `set ` o el archivo se considera inválido.
///
/// # Errors
/// Retorna error si el archivo tiene formato inválido.
///
/// # Ejemplo
/// ```
/// read_all_operations().expect("Failed to read log");
/// ```
pub fn read_all_operations() -> Result<Vec<String>> {
    let path = Path::new(LOG_FILE_NAME);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut operations = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with("set ") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "INVALID LOG FILE",
            ));
        }
        operations.push(line);
    }
    Ok(operations)
}

/// Trunca el archivo de log (lo vacía).
/// Se llama después de un snapshot exitoso para comenzar de cero.
///
/// # Errors
/// Retorna error si falla la creación del archivo vacío.
///
/// # Ejemplo
/// ```
/// truncate().expect("Failed to truncate log");
/// ```
pub fn truncate() -> Result<()> {
    std::fs::File::create(LOG_FILE_NAME)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_sync::get_lock;

    fn clear_log() {
        let _ = std::fs::remove_file(LOG_FILE_NAME);
    }

    #[test]
    fn test01_add_operation() {
        let _guard = get_lock().lock().unwrap();
        clear_log();
        match add_operation("set clave1 valor1") {
            Ok(()) => {}
            Err(e) => panic!("Error en add_operation: {e}"),
        }

        let file = match File::open(LOG_FILE_NAME) {
            Ok(f) => f,
            Err(e) => panic!("Error abriendo archivo de log: {e}"),
        };
        let reader = BufReader::new(file);
        let mut found = false;
        for line_result in reader.lines() {
            match line_result {
                Ok(line) => {
                    if line.contains("set clave1 valor1") {
                        found = true;
                        break;
                    }
                }
                Err(e) => panic!("Error leyendo línea: {e}"),
            }
        }
        assert!(found, "Content doesn't contain expected operation");

        clear_log();
    }

    #[test]
    fn test02_read_all_operations() {
        let _guard = get_lock().lock().unwrap();
        clear_log();
        match add_operation("set clave1 valor1") {
            Ok(()) => {}
            Err(e) => panic!("Error en primer append: {e}"),
        }

        match add_operation("set clave2 valor2") {
            Ok(()) => {}
            Err(e) => panic!("Error en segundo append: {e}"),
        }

        match add_operation("set clave2") {
            Ok(()) => {}
            Err(e) => panic!("Error en tercer append: {e}"),
        }

        match read_all_operations() {
            Ok(ops) => {
                assert_eq!(ops.len(), 3, "Expected 3 operations, got: {ops:?}");
            }
            Err(e) => panic!("Error al leer operaciones: {e}"),
        }

        clear_log();
    }

    #[test]
    fn test03_read_operations_nonexistent_file() {
        let _guard = get_lock().lock().unwrap();
        clear_log();
        match read_all_operations() {
            Ok(ops) => {
                assert_eq!(ops.len(), 0, "Expected empty list, got: {ops:?}");
            }
            Err(e) => panic!("Error al leer operaciones inexistentes: {e}"),
        }
        clear_log();
    }

    #[test]
    fn test04_truncate_log() {
        let _guard = get_lock().lock().unwrap();
        clear_log();
        match add_operation("set clave1 valor1") {
            Ok(()) => {}
            Err(e) => panic!("Error en append antes de truncate: {e}"),
        }

        match truncate() {
            Ok(()) => {}
            Err(e) => panic!("Error al truncar log: {e}"),
        }

        match read_all_operations() {
            Ok(ops) => {
                assert_eq!(ops.len(), 0, "Expected empty log after truncate");
            }
            Err(e) => panic!("Error al leer log después de truncate: {e}"),
        }

        clear_log();
    }

    #[test]
    fn test05_read_operations_invalid_format() {
        let _guard = get_lock().lock().unwrap();
        clear_log();

        // Escribir una línea que no empieza con "set "
        let mut file = std::fs::File::create(LOG_FILE_NAME).expect("No se pudo crear archivo");
        std::io::Write::write_all(&mut file, b"invalid_command clave valor\n")
            .expect("No se pudo escribir");

        let result = read_all_operations();
        assert!(result.is_err(), "Debería fallar con formato inválido");

        let error_msg = result.expect_err("Debería ser error").to_string();
        assert!(
            error_msg.contains("INVALID LOG FILE"),
            "El error debería indicar log inválido"
        );

        clear_log();
    }

    #[test]
    fn test06_add_multiple_operations_order() {
        let _guard = get_lock().lock().unwrap();
        clear_log();

        match add_operation("set a 1") {
            Ok(()) => {}
            Err(e) => panic!("Error en primer append: {e}"),
        }
        match add_operation("set b 2") {
            Ok(()) => {}
            Err(e) => panic!("Error en segundo append: {e}"),
        }
        match add_operation("set c 3") {
            Ok(()) => {}
            Err(e) => panic!("Error en tercer append: {e}"),
        }

        match read_all_operations() {
            Ok(ops) => {
                assert_eq!(ops.len(), 3);
                assert_eq!(ops.first(), Some(&"set a 1".to_string()));
                assert_eq!(ops.get(1), Some(&"set b 2".to_string()));
                assert_eq!(ops.get(2), Some(&"set c 3".to_string()));
            }
            Err(e) => panic!("Error al leer operaciones: {e}"),
        }

        clear_log();
    }
}
