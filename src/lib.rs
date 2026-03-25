pub mod log;
pub mod store;

use std::collections::HashMap;
use std::io::Result;

/// Errores que pueden ocurrir en el sistema.
#[derive(Debug, PartialEq)]
pub enum Error {
    NotFound,
    ExtraArgument,
    InvalidDataFile,
    InvalidLogFile,
    MissingArgument,
    UnknownCommand,
}

impl Error {
    /// Retorna el mensaje de error formateado.
    pub fn mensaje(&self) -> String {
        match self {
            Error::NotFound => "ERROR: NOT FOUND".to_string(),
            Error::ExtraArgument => "ERROR: EXTRA ARGUMENT".to_string(),
            Error::InvalidDataFile => "ERROR: INVALID DATA FILE".to_string(),
            Error::InvalidLogFile => "ERROR: INVALID LOG FILE".to_string(),
            Error::MissingArgument => "ERROR: MISSING ARGUMENT".to_string(),
            Error::UnknownCommand => "ERROR: UNKNOWN COMMAND".to_string(),
        }
    }
}

/// Estructura que representa el guardado de clave y valor.
pub struct KvStore {
    data: HashMap<String, String>,
}
impl Default for KvStore {
    fn default() -> Self {
        Self::new()
    }
}
impl KvStore {
    /// Crea un nuevo almacenamiento vacío en memoria.
    pub fn new() -> Self {
        KvStore {
            data: HashMap::new(),
        }
    }
    pub fn load() -> Result<Self> {
        // Cargar snapshot del archivo de datos
        let mut almacenamiento = match store::load_snapshot() {
            Ok(datos) => datos,
            Err(error) => {
                if error.to_string().contains("INVALID DATA FILE") {
                    return Err(error);
                }
                HashMap::new()
            }
        };

        // Leer y aplicar operaciones del log para reconstruir el estado actual
        let operaciones = log::leer_todas_las_operaciones()?;
        for operacion in operaciones {
            aplicar_operacion(&mut almacenamiento, &operacion);
        }

        Ok(KvStore {
            data: almacenamiento,
        })
    }
    /// Asocia un valor a una clave. Si el valor está vacío, elimina la clave.
    pub fn set(&mut self, clave: String, valor: String) -> Result<()> {
        let linea_log = if valor.is_empty() {
            // Unset: eliminar la clave
            format!("set \"{}\"", clave)
        } else {
            // Set: escapar comillas internas y guardar con formato entrecomillado
            let valor_escapado = valor.replace("\"", "\\\"");
            format!("set \"{}\" \"{}\"", clave, valor_escapado)
        };

        match log::agregar_operacion(&linea_log) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }

        Ok(())
    }
    /// Obtiene el valor asociado a una clave, None si no existe.
    pub fn get(&self, clave: &str) -> Option<String> {
        self.data.get(clave).cloned()
    }
    /// Retorna la cantidad de claves activas (con valor) en el almacenamiento.
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Verifica si el almacenamiento está vacío.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// Genera un snapshot del estado actual y trunca el log.
    pub fn snapshot(&self) -> Result<()> {
        match store::save_snapshot(&self.data) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        match log::truncar() {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        Ok(())
    }
}
/// Aplica una operación del log al HashMap reconstruyendo el estado.
/// Parsea el formato entrecomillado: set "clave" "valor"
fn aplicar_operacion(almacenamiento: &mut HashMap<String, String>, linea_operacion: &str) {
    if !linea_operacion.starts_with("set ") {
        return;
    }

    let resto = &linea_operacion[4..];
    let mut texto_clave = String::new();
    let mut texto_valor = String::new();
    let mut dentro_comillas = false;
    let mut caracter_escapado = false;
    let mut procesando_clave = true;
    let mut primer_espacio_encontrado = false;
    let caracteres: Vec<char> = resto.chars().collect();
    let mut indice = 0;

    while indice < caracteres.len() {
        let caracter = caracteres[indice];

        if caracter_escapado {
            if procesando_clave {
                texto_clave.push(caracter);
            } else {
                texto_valor.push(caracter);
            }
            caracter_escapado = false;
            indice += 1;
            continue;
        }

        if caracter == '\\' && dentro_comillas {
            caracter_escapado = true;
            if procesando_clave {
                texto_clave.push(caracter);
            } else {
                texto_valor.push(caracter);
            }
            indice += 1;
            continue;
        }

        if caracter == '"' {
            dentro_comillas = !dentro_comillas;
            indice += 1;
            continue;
        }

        if caracter == ' ' && !dentro_comillas && procesando_clave && !primer_espacio_encontrado {
            primer_espacio_encontrado = true;
            procesando_clave = false;
            indice += 1;
            continue;
        }

        if procesando_clave {
            texto_clave.push(caracter);
        } else {
            texto_valor.push(caracter);
        }
        indice += 1;
    }

    if texto_valor.is_empty() {
        // Unset: eliminar la clave del almacenamiento
        almacenamiento.remove(&texto_clave);
    } else {
        // Desescapar comillas internas
        let valor_desescapado = texto_valor.replace("\\\"", "\"");
        almacenamiento.insert(texto_clave, valor_desescapado);
    }
}

/// Maneja errores de carga centralizando la lógica de conversión.
fn gestionar_error_carga(error: std::io::Error) {
    let mensaje_error = error.to_string();
    if mensaje_error.contains("INVALID DATA FILE") {
        eprintln!("{}", Error::InvalidDataFile.mensaje());
    } else if mensaje_error.contains("INVALID LOG FILE") {
        eprintln!("{}", Error::InvalidLogFile.mensaje());
    } else {
        eprintln!("{}", Error::InvalidDataFile.mensaje());
    }
}

/// Ejecuta el comando SET: guarda o actualiza una clave-valor.
pub fn execute_set(clave: String, valor: String) {
    let mut almacenamiento = match KvStore::load() {
        Ok(almacen) => almacen,
        Err(error) => {
            gestionar_error_carga(error);
            return;
        }
    };

    match almacenamiento.set(clave, valor) {
        Ok(_) => println!("OK"),
        Err(error) => eprintln!("{}", error),
    }
}
/// Ejecuta el comando GET: recupera el valor de una clave.
pub fn execute_get(clave: String) {
    let almacenamiento = match KvStore::load() {
        Ok(almacen) => almacen,
        Err(error) => {
            gestionar_error_carga(error);
            return;
        }
    };
    match almacenamiento.get(&clave) {
        Some(valor) => println!("{}", valor),
        None => eprintln!("{}", Error::NotFound.mensaje()),
    }
}
/// Ejecuta el comando LENGTH: retorna la cantidad de claves con valor.
pub fn execute_length() {
    let almacenamiento = match KvStore::load() {
        Ok(almacen) => almacen,
        Err(error) => {
            gestionar_error_carga(error);
            return;
        }
    };
    println!("{}", almacenamiento.len());
}
/// Ejecuta el comando SNAPSHOT: persiste el estado actual y limpia el log.
pub fn execute_snapshot() {
    let almacenamiento = match KvStore::load() {
        Ok(almacen) => almacen,
        Err(error) => {
            gestionar_error_carga(error);
            return;
        }
    };
    match almacenamiento.snapshot() {
        Ok(_) => println!("OK"),
        Err(error) => eprintln!("{}", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn obtener_lock() -> &'static Mutex<()> {
        TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn cleanup() {
        let _ = std::fs::remove_file(".minikv.data");
        let _ = std::fs::remove_file(".minikv.log");
    }

    fn eliminar_comillas(s: &str) -> String {
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    }

    #[test]
    fn test01_eliminar_comillas() {
        assert_eq!(eliminar_comillas("\"hello world\""), "hello world");
        assert_eq!(eliminar_comillas("hello"), "hello");
        assert_eq!(eliminar_comillas("\"hello\""), "hello");
        assert_eq!(eliminar_comillas("\"\""), "");
    }

    #[test]
    fn test02_store_nuevo_vacio() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        cleanup();

        let store = KvStore::new();
        assert_eq!(store.len(), 0);
        assert_eq!(store.get("inexistente"), None);

        cleanup();
    }

    #[test]
    fn test04_set_get_valor() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        cleanup();

        let mut store = KvStore::new();
        match store.set("nombre".to_string(), "jose".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error al hacer set: {}", e),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("nombre"), Some("jose".to_string()));
                assert_eq!(loaded.len(), 1);
            }
            Err(e) => panic!("Error al cargar: {}", e),
        }

        cleanup();
    }

    #[test]
    fn test05_set_pisa_valor() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        cleanup();

        let mut store = KvStore::new();
        match store.set("nombre".to_string(), "maria".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en primer set: {}", e),
        }

        match store.set("nombre".to_string(), "juana".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en segundo set: {}", e),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("nombre"), Some("juana".to_string()));
                assert_eq!(loaded.len(), 1);
            }
            Err(e) => panic!("Error al cargar: {}", e),
        }

        cleanup();
    }

    #[test]
    fn test06_setear_vacio_elimina_clave() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        cleanup();

        let mut store = KvStore::new();
        match store.set("clave".to_string(), "valor".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en primer set: {}", e),
        }

        match store.set("clave".to_string(), "".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error al establecer valor vacío: {}", e),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("clave"), None);
                assert_eq!(loaded.len(), 0);
            }
            Err(e) => panic!("Error al cargar: {}", e),
        }

        cleanup();
    }

    #[test]
    fn test07_lenght_cuenta_claves() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        cleanup();

        let mut store = KvStore::new();
        match store.set("a".to_string(), "1".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en set de 'a': {}", e),
        }

        match store.set("b".to_string(), "2".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en set de 'b': {}", e),
        }

        match store.set("a".to_string(), "".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error al eliminar 'a': {}", e),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.len(), 1);
                assert_eq!(loaded.get("b"), Some("2".to_string()));
            }
            Err(e) => panic!("Error al cargar: {}", e),
        }

        cleanup();
    }

    #[test]
    fn test08_carga_store_despues_de_set_y_unset() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        cleanup();

        let mut store = KvStore::new();
        match store.set("frase".to_string(), "hola mundo".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en set de 'frase': {}", e),
        }

        match store.set("tmp".to_string(), "123".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en set de 'tmp': {}", e),
        }

        match store.set("tmp".to_string(), "".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error al eliminar 'tmp': {}", e),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("frase"), Some("hola mundo".to_string()));
                assert_eq!(loaded.get("tmp"), None);
                assert_eq!(loaded.len(), 1);
            }
            Err(e) => panic!("Error al cargar el store: {}", e),
        }

        cleanup();
    }

    #[test]
    fn test09_snapshot_persiste_estado_y_trunca_log() {
        let _guardia = match obtener_lock().lock() {
            Ok(guardia) => guardia,
            Err(_) => panic!("No se pudo adquirir el lock de prueba"),
        };
        cleanup();

        let mut store = KvStore::new();
        match store.set("a".to_string(), "uno".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en set de 'a': {}", e),
        }

        match store.set("b".to_string(), "dos palabras".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en set de 'b': {}", e),
        }

        match KvStore::load() {
            Ok(loaded_store) => match loaded_store.snapshot() {
                Ok(_) => {}
                Err(e) => panic!("Error al crear snapshot: {}", e),
            },
            Err(e) => panic!("Error al cargar antes de snapshot: {}", e),
        }

        match std::fs::metadata(".minikv.log") {
            Ok(metadata) => {
                let log_len = metadata.len();
                assert_eq!(log_len, 0);
            }
            Err(e) => panic!("Error al acceder al archivo .minikv.log: {}", e),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("a"), Some("uno".to_string()));
                assert_eq!(loaded.get("b"), Some("dos palabras".to_string()));
                assert_eq!(loaded.len(), 2);
            }
            Err(e) => panic!("Error al cargar el store: {}", e),
        }

        cleanup();
    }

    #[test]
    fn test10_aplica_operacion_con_comillas() {
        let mut data = HashMap::new();

        aplicar_operacion(&mut data, "set clave \"valor con espacios\"");

        assert_eq!(data.get("clave"), Some(&"valor con espacios".to_string()));
    }
}
