use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;

const DATA_FILE_NAME: &str = ".minikv.data";

/// Guarda un snapshot (copia del estado actual) en el archivo de datos.
pub fn save_snapshot(storage: &HashMap<String, String>) -> Result<()> {
    let mut file = File::create(DATA_FILE_NAME)?;
    for (key, value) in storage {
        let special_value = value.replace("\"", "\\\"");
        writeln!(file, "\"{}\" \"{}\"", key, special_value)?;
    }
    Ok(())
}

/// Carga el snapshot del archivo de datos validando el formato.
pub fn load_snapshot() -> Result<HashMap<String, String>> {
    let path = Path::new(DATA_FILE_NAME);
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut storage = HashMap::new();

    for line_result in reader.lines() {
        let linea = line_result?;
        if linea.trim().is_empty() {
            continue;
        }
        let (clave, valor) = parse_data_line(&linea)?;
        storage.insert(clave, valor);
    }
    Ok(storage)
}

/// Parsea una línea del archivo de datos y extrae clave y valor.
fn parse_data_line(line: &str) -> Result<(String, String)> {
    let parsed_line_parts: Vec<&str> = line.splitn(2, "\" \"").collect();
    let (key_raw, value_raw) = match parsed_line_parts.as_slice() {
        [c, v] => (*c, *v),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "INVALID DATA FILE",
            ))
        }
    };
    let key = key_raw.trim_start_matches('"').to_string();
    let value = value_raw.trim_end_matches('"').replace("\\\"", "\"");
    Ok((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_sync::get_lock;

    fn cleanup(path: &str) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_save_and_load_snapshot_success() {
        let _guard = get_lock().lock().unwrap();
        cleanup(DATA_FILE_NAME);

        let mut data = HashMap::new();
        data.insert("clave1".to_string(), "valor1".to_string());
        data.insert("clave2".to_string(), "valor2".to_string());

        match save_snapshot(&data) {
            Ok(_) => {}
            Err(e) => panic!("Error al guardar snapshot: {}", e),
        }

        match load_snapshot() {
            Ok(loaded_data) => {
                assert_eq!(
                    loaded_data.get("clave1"),
                    Some(&"valor1".to_string()),
                    "clave1 no coincide"
                );
                assert_eq!(
                    loaded_data.get("clave2"),
                    Some(&"valor2".to_string()),
                    "clave2 no coincide"
                );
            }
            Err(e) => panic!("Error al cargar snapshot: {}", e),
        }

        cleanup(DATA_FILE_NAME);
    }

    #[test]
    fn test_load_snapshot_nonexistent_file() {
        let _guard = get_lock().lock().unwrap();
        cleanup(DATA_FILE_NAME);

        match load_snapshot() {
            Ok(data) => {
                assert_eq!(data.len(), 0, "El snapshot debería estar vacío");
            }
            Err(e) => panic!("Error al cargar snapshot inexistente: {}", e),
        }

        cleanup(DATA_FILE_NAME);
    }
}
