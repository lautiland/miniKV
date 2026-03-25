use std::fs::OpenOptions;
use std::io::{Result, Write};
use std::path::Path;

const LOG_FILE_NAME: &str = ".minikv.log";

/// Agrega una operación al final del archivo de log (append-only).
pub fn agregar_operacion(operacion: &str) -> Result<()> {
    let mut archivo = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE_NAME)?;
    writeln!(archivo, "{}", operacion)?;
    Ok(())
}

/// Lee todas las operaciones del log validando el formato.
pub fn leer_todas_las_operaciones() -> Result<Vec<String>> {
    let ruta = Path::new(LOG_FILE_NAME);

    if !ruta.exists() {
        return Ok(Vec::new());
    }

    let contenido = std::fs::read_to_string(ruta)?;

    for linea in contenido.lines() {
        if linea.trim().is_empty() {
            continue;
        }
        if !linea.starts_with("set ") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "INVALID LOG FILE",
            ));
        }
    }

    Ok(contenido
        .lines()
        .map_while(|linea| {
            if linea.trim().is_empty() {
                None
            } else {
                Some(linea.to_string())
            }
        })
        .collect())
}

/// Trunca el archivo de log (lo vacía completamente).
pub fn truncar() -> Result<()> {
    std::fs::File::create(LOG_FILE_NAME)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn obtener_lock() -> &'static Mutex<()> {
        TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn limpiar_log() {
        let _ = std::fs::remove_file(LOG_FILE_NAME);
    }

    #[test]
    fn test01_agregar_operacion() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        limpiar_log();
        match agregar_operacion("set clave1 valor1") {
            Ok(_) => {}
            Err(e) => panic!("Error en agregar_operacion: {}", e),
        }

        match std::fs::read_to_string(LOG_FILE_NAME) {
            Ok(content) => {
                assert!(
                    content.contains("set clave1 valor1"),
                    "Content doesn't contain expected operation"
                );
            }
            Err(e) => panic!("Error leyendo archivo de log: {}", e),
        }

        limpiar_log();
    }

    #[test]
    fn test02_lee_todas_las_operaciones() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        limpiar_log();
        match agregar_operacion("set clave1 valor1") {
            Ok(_) => {}
            Err(e) => panic!("Error en primer append: {}", e),
        }

        match agregar_operacion("set clave2 valor2") {
            Ok(_) => {}
            Err(e) => panic!("Error en segundo append: {}", e),
        }

        match agregar_operacion("set clave2") {
            Ok(_) => {}
            Err(e) => panic!("Error en tercer append: {}", e),
        }

        match leer_todas_las_operaciones() {
            Ok(ops) => {
                assert_eq!(ops.len(), 3, "Expected 3 operations, got: {:?}", ops);
            }
            Err(e) => panic!("Error al leer operaciones: {}", e),
        }

        limpiar_log();
    }

    #[test]
    fn test03_lee_operaciones_archivo_inexistente() {
        limpiar_log();
        match leer_todas_las_operaciones() {
            Ok(ops) => {
                assert_eq!(ops.len(), 0, "Expected empty list, got: {:?}", ops);
            }
            Err(e) => panic!("Error al leer operaciones inexistentes: {}", e),
        }
        limpiar_log();
    }

    #[test]
    fn test04_trunca_log() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        limpiar_log();
        match agregar_operacion("set clave1 valor1") {
            Ok(_) => {}
            Err(e) => panic!("Error en append antes de truncate: {}", e),
        }

        match truncar() {
            Ok(_) => {}
            Err(e) => panic!("Error al truncar log: {}", e),
        }

        match leer_todas_las_operaciones() {
            Ok(ops) => {
                assert_eq!(ops.len(), 0, "Expected empty log after truncate");
            }
            Err(e) => panic!("Error al leer log después de truncate: {}", e),
        }

        limpiar_log();
    }
}
