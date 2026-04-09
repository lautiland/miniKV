//! Gestión de snapshots para recuperación eficiente del estado.
//!
//! El archivo de snapshot (`.minikv.data`) contiene el estado completo en un
//! punto en el tiempo, permitiendo recuperación rápida sin reproducir todo el log.

use std::collections::HashMap;
use std::fs::File;
use std::hash::BuildHasher;
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;

const DATA_FILE_NAME: &str = ".minikv.data";

/// Guarda un snapshot en el archivo de datos.
/// Cada línea se escribe en el formato: `"clave" "valor"`
///
/// # Errors
/// Retorna error si falla la creación o escritura del archivo.
///
/// # Ejemplo
/// ``` ignore
/// use std::collections::HashMap;
/// use minikv::persistence::store::save_snapshot;
/// let mut data = HashMap::new();
///
/// data.insert("clave".to_string(), "valor".to_string());
/// save_snapshot(&data).expect("Failed to save");
/// ```
pub fn save_snapshot<S: BuildHasher>(storage: &HashMap<String, String, S>) -> Result<()> {
    let mut file = File::create(DATA_FILE_NAME)?;
    for (key, value) in storage {
        let special_value = value.replace('"', "\\\"");
        writeln!(file, "\"{key}\" \"{special_value}\"")?;
    }
    Ok(())
}

/// Carga el snapshot del archivo de datos validando el formato.
///
/// # Errors
/// Retorna error si el archivo tiene formato inválido.
///
/// # Ejemplo
/// ``` ignore
/// use std::collections::HashMap;
/// use minikv::persistence::store::load_snapshot;
/// let data = load_snapshot().expect("Failed to load");
/// assert_eq!(data.get("clave"), Some(&"valor".to_string()));
/// ```
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

/// Estado del parser para líneas de datos.
struct DataLineParser {
    key: String,
    value: String,
    in_quotes: bool,
    escaped: bool,
    parsing_key: bool,
    quote_count: usize,
}

impl DataLineParser {
    fn new() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            in_quotes: false,
            escaped: false,
            parsing_key: true,
            quote_count: 0,
        }
    }

    fn push_char(&mut self, c: char) {
        if self.parsing_key {
            self.key.push(c);
        } else {
            self.value.push(c);
        }
    }

    fn process_char(&mut self, c: char) {
        if self.escaped {
            self.push_char(c);
            self.escaped = false;
        } else if c == '\\' {
            self.escaped = true;
        } else if c == '"' {
            self.in_quotes = !self.in_quotes;
            if !self.in_quotes {
                self.quote_count += 1;
                if self.parsing_key {
                    self.parsing_key = false;
                }
            }
        } else if self.in_quotes {
            self.push_char(c);
        }
    }
}

/// Parsea una línea del archivo de datos y extrae clave y valor.
fn parse_data_line(line: &str) -> Result<(String, String)> {
    let mut parser = DataLineParser::new();
    for c in line.chars() {
        parser.process_char(c);
    }
    if parser.key.is_empty() || parser.quote_count < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "INVALID DATA FILE",
        ));
    }
    Ok((parser.key, parser.value))
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
            Ok(()) => {}
            Err(e) => panic!("Error al guardar snapshot: {e}"),
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
            Err(e) => panic!("Error al cargar snapshot: {e}"),
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
            Err(e) => panic!("Error al cargar snapshot inexistente: {e}"),
        }

        cleanup(DATA_FILE_NAME);
    }

    #[test]
    fn test_load_snapshot_invalid_format() {
        let _guard = get_lock().lock().unwrap();
        cleanup(DATA_FILE_NAME);

        // Escribir un archivo con formato inválido
        let mut file = std::fs::File::create(DATA_FILE_NAME).expect("No se pudo crear archivo");
        std::io::Write::write_all(&mut file, b"linea sin formato correcto\n")
            .expect("No se pudo escribir");

        let result = load_snapshot();
        assert!(result.is_err(), "Debería fallar con formato inválido");

        let error_msg = result.expect_err("Debería ser error").to_string();
        assert!(
            error_msg.contains("INVALID DATA FILE"),
            "El error debería indicar archivo inválido"
        );

        cleanup(DATA_FILE_NAME);
    }

    #[test]
    fn test_save_load_snapshot_with_quotes_in_value() {
        let _guard = get_lock().lock().unwrap();
        cleanup(DATA_FILE_NAME);

        let mut data = HashMap::new();
        data.insert("frase".to_string(), "dijo \"hola\"".to_string());

        match save_snapshot(&data) {
            Ok(()) => {}
            Err(e) => panic!("Error al guardar snapshot: {e}"),
        }

        match load_snapshot() {
            Ok(loaded_data) => {
                assert_eq!(
                    loaded_data.get("frase"),
                    Some(&"dijo \"hola\"".to_string()),
                    "El valor con comillas no coincide"
                );
            }
            Err(e) => panic!("Error al cargar snapshot: {e}"),
        }

        cleanup(DATA_FILE_NAME);
    }
}
