use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;

const LOG_FILE_NAME: &str = ".minikv.log";

/// Agrega una operación al final del archivo de log (append-only).
pub fn add_operation(operation: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_NAME)?;
    writeln!(file, "{}", operation)?;
    Ok(())
}

/// Lee todas las operaciones del log validando el formato.
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

/// Trunca el archivo de log (lo vacía completamente).
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
            Ok(_) => {}
            Err(e) => panic!("Error en add_operation: {}", e),
        }

        let file = match File::open(LOG_FILE_NAME) {
            Ok(f) => f,
            Err(e) => panic!("Error abriendo archivo de log: {}", e),
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
                Err(e) => panic!("Error leyendo línea: {}", e),
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
            Ok(_) => {}
            Err(e) => panic!("Error en primer append: {}", e),
        }

        match add_operation("set clave2 valor2") {
            Ok(_) => {}
            Err(e) => panic!("Error en segundo append: {}", e),
        }

        match add_operation("set clave2") {
            Ok(_) => {}
            Err(e) => panic!("Error en tercer append: {}", e),
        }

        match read_all_operations() {
            Ok(ops) => {
                assert_eq!(ops.len(), 3, "Expected 3 operations, got: {:?}", ops);
            }
            Err(e) => panic!("Error al leer operaciones: {}", e),
        }

        clear_log();
    }

    #[test]
    fn test03_read_operations_nonexistent_file() {
        let _guard = get_lock().lock().unwrap();
        clear_log();
        match read_all_operations() {
            Ok(ops) => {
                assert_eq!(ops.len(), 0, "Expected empty list, got: {:?}", ops);
            }
            Err(e) => panic!("Error al leer operaciones inexistentes: {}", e),
        }
        clear_log();
    }

    #[test]
    fn test04_truncate_log() {
        let _guard = get_lock().lock().unwrap();
        clear_log();
        match add_operation("set clave1 valor1") {
            Ok(_) => {}
            Err(e) => panic!("Error en append antes de truncate: {}", e),
        }

        match truncate() {
            Ok(_) => {}
            Err(e) => panic!("Error al truncar log: {}", e),
        }

        match read_all_operations() {
            Ok(ops) => {
                assert_eq!(ops.len(), 0, "Expected empty log after truncate");
            }
            Err(e) => panic!("Error al leer log después de truncate: {}", e),
        }

        clear_log();
    }
}
