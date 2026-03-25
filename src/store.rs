use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;

const DATA_FILE_NAME: &str = ".minikv.data";

/// Guarda un snapshot (copia del estado actual) en el archivo de datos.
pub fn save_snapshot(almacenamiento: &HashMap<String, String>) -> Result<()> {
    let mut archivo = File::create(DATA_FILE_NAME)?;
    for (clave, valor) in almacenamiento {
        let valor_escapado = valor.replace("\"", "\\\"");
        writeln!(archivo, "\"{}\" \"{}\"", clave, valor_escapado)?;
    }
    Ok(())
}

/// Carga el snapshot del archivo de datos validando el formato.
pub fn load_snapshot() -> Result<HashMap<String, String>> {
    let ruta = Path::new(DATA_FILE_NAME);
    let mut almacenamiento = HashMap::new();

    if !ruta.exists() {
        return Ok(almacenamiento);
    }

    let contenido = std::fs::read_to_string(ruta)?;
    for linea in contenido.lines() {
        if linea.trim().is_empty() {
            continue;
        }

        let partes: Vec<&str> = linea.splitn(2, "\" \"").collect();
        if partes.len() != 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "INVALID DATA FILE",
            ));
        }

        let clave = partes[0].trim_start_matches('"').to_string();
        let mut valor = partes[1].trim_end_matches('"').to_string();

        // Desescapar comillas internas
        valor = valor.replace("\\\"", "\"");

        almacenamiento.insert(clave, valor);
    }
    Ok(almacenamiento)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn obtener_lock() -> &'static Mutex<()> {
        TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn limpiar(ruta: &str) {
        let _ = std::fs::remove_file(ruta);
    }

    #[test]
    fn test01_guarda_y_carga_snapshot() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        limpiar(DATA_FILE_NAME);

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

        limpiar(DATA_FILE_NAME);
    }

    #[test]
    fn test02_carga_snapshot_archivo_inexistente() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        limpiar(DATA_FILE_NAME);

        match load_snapshot() {
            Ok(data) => {
                assert_eq!(data.len(), 0, "El snapshot debería estar vacío");
            }
            Err(e) => panic!("Error al cargar snapshot inexistente: {}", e),
        }

        limpiar(DATA_FILE_NAME);
    }
}
