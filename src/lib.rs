pub mod log;
pub mod store;

/// Módulo de sincronización para tests que acceden a archivos compartidos.
#[doc(hidden)]
#[cfg(test)]
pub mod test_sync {
    use std::sync::{Mutex, OnceLock};
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    pub fn get_lock() -> &'static Mutex<()> {
        TEST_LOCK.get_or_init(|| Mutex::new(()))
    }
}

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
    pub fn msg(&self) -> String {
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
        let mut storage = match store::load_snapshot() {
            Ok(data) => data,
            Err(err) => {
                if err.to_string().contains("INVALID DATA FILE") {
                    return Err(err);
                }
                HashMap::new()
            }
        };

        // Leer y aplicar operaciones del log para reconstruir el estado actual
        let operations = log::read_all_operations()?;
        for op in operations {
            apply_operation(&mut storage, &op);
        }

        Ok(KvStore { data: storage })
    }
    /// Asocia un valor a una clave. Si el valor está vacío, elimina la clave.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let linea_log = if value.is_empty() {
            // Unset: eliminar la clave
            format!("set \"{}\"", key)
        } else {
            // Set: quitar comillas internas y guardar con formato entrecomillado
            let value_sanitized = value.replace("\"", "\\\"");
            format!("set \"{}\" \"{}\"", key, value_sanitized)
        };

        log::add_operation(&linea_log)?;
        Ok(())
    }
    /// Obtiene el valor asociado a una clave, None si no existe.
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
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
        store::save_snapshot(&self.data)?;
        log::truncate()?;
        Ok(())
    }
}

/// Estado del parser para procesar caracteres.
struct ParseState {
    in_quotes: bool,
    escaped: bool,
    parsing_key: bool,
    found_space: bool,
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
            in_quotes: false,
            escaped: false,
            parsing_key: true,
            found_space: false,
        }
    }
}

/// Parsea una línea de operación y extrae clave y valor.
/// Formato esperado: "clave" "valor" o "clave" (para unset)
fn parse_kv(text: &str) -> (String, String) {
    let mut key = String::new();
    let mut value = String::new();
    let mut state = ParseState::default();

    for c in text.chars() {
        process_char(&mut key, &mut value, &mut state, c);
    }

    (key, value)
}

/// Procesa un caracter y actualiza el estado del parser.
fn process_char(key: &mut String, value: &mut String, state: &mut ParseState, c: char) {
    if state.escaped {
        add_char(key, value, c, state.parsing_key);
        state.escaped = false;
        return;
    }

    if c == '\\' && state.in_quotes {
        state.escaped = true;
        add_char(key, value, c, state.parsing_key);
        return;
    }

    if c == '"' {
        state.in_quotes = !state.in_quotes;
        return;
    }

    if c == ' ' && !state.in_quotes && state.parsing_key && !state.found_space {
        state.found_space = true;
        state.parsing_key = false;
        return;
    }

    add_char(key, value, c, state.parsing_key);
}

/// Agrega un caracter a la clave o al valor según corresponda.
fn add_char(key: &mut String, value: &mut String, c: char, is_key: bool) {
    if is_key {
        key.push(c);
    } else {
        value.push(c);
    }
}

/// Aplica una operación del log al HashMap reconstruyendo el estado.
/// Parsea el formato entrecomillado: set "clave" "valor"
fn apply_operation(storage: &mut HashMap<String, String>, op_line: &str) {
    if !op_line.starts_with("set ") {
        return;
    }

    let (key, value) = parse_kv(&op_line[4..]);

    if value.is_empty() {
        storage.remove(&key);
    } else {
        let value_sanitized = value.replace("\\\"", "\"");
        storage.insert(key, value_sanitized);
    }
}

/// Maneja errores de carga centralizando la lógica de conversión.
fn error_load_handle(error: std::io::Error) {
    let error_msg = error.to_string();
    if error_msg.contains("INVALID DATA FILE") {
        eprintln!("{}", Error::InvalidDataFile.msg());
    } else if error_msg.contains("INVALID LOG FILE") {
        eprintln!("{}", Error::InvalidLogFile.msg());
    } else {
        eprintln!("{}", Error::InvalidDataFile.msg());
    }
}

/// Ejecuta el comando SET: guarda o actualiza una clave-valor.
pub fn execute_set(key: String, value: String) {
    let mut storage = match KvStore::load() {
        Ok(sto) => sto,
        Err(err) => {
            error_load_handle(err);
            return;
        }
    };

    match storage.set(key, value) {
        Ok(_) => println!("OK"),
        Err(error) => eprintln!("{}", error),
    }
}
/// Ejecuta el comando GET: recupera el valor de una clave.
pub fn execute_get(key: String) {
    let storage = match KvStore::load() {
        Ok(sto) => sto,
        Err(err) => {
            error_load_handle(err);
            return;
        }
    };
    match storage.get(&key) {
        Some(value) => println!("{}", value),
        None => eprintln!("{}", Error::NotFound.msg()),
    }
}
/// Ejecuta el comando LENGTH: retorna la cantidad de claves con valor.
pub fn execute_length() {
    let storage = match KvStore::load() {
        Ok(sto) => sto,
        Err(err) => {
            error_load_handle(err);
            return;
        }
    };
    println!("{}", storage.len());
}
/// Ejecuta el comando SNAPSHOT: persiste el estado actual y limpia el log.
pub fn execute_snapshot() {
    let storage = match KvStore::load() {
        Ok(sto) => sto,
        Err(err) => {
            error_load_handle(err);
            return;
        }
    };
    match storage.snapshot() {
        Ok(_) => println!("OK"),
        Err(err) => eprintln!("{}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_sync::get_lock;

    fn cleanup() {
        let _ = std::fs::remove_file(".minikv.data");
        let _ = std::fs::remove_file(".minikv.log");
    }

    fn remove_quotes(s: &str) -> String {
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    }

    #[test]
    fn test01_remove_quotes() {
        assert_eq!(remove_quotes("\"hello world\""), "hello world");
        assert_eq!(remove_quotes("hello"), "hello");
        assert_eq!(remove_quotes("\"hello\""), "hello");
        assert_eq!(remove_quotes("\"\""), "");
    }

    #[test]
    fn test02_new_store_empty() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let store = KvStore::new();
        assert_eq!(store.len(), 0);
        assert_eq!(store.get("inexistente"), None);

        cleanup();
    }

    #[test]
    fn test04_set_get_value() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        match store.set("nombre".to_string(), "jose".to_string()) {
            Ok(_) => {}
            Err(e) => panic!("Error en set: {}", e),
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
    fn test05_set_overwrite_value() {
        let _guard = get_lock().lock().unwrap();
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
    fn test06_set_empty_remove_key() {
        let _guard = get_lock().lock().unwrap();
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
    fn test07_length_count_keys() {
        let _guard = get_lock().lock().unwrap();
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
    fn test08_load_store_after_set_unset() {
        let _guard = get_lock().lock().unwrap();
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
    fn test09_snapshot_persist_and_truncate_log() {
        let _guard = get_lock().lock().unwrap();
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
    fn test10_apply_operation_with_quotes() {
        let mut data = HashMap::new();

        apply_operation(&mut data, "set clave \"valor con espacios\"");

        assert_eq!(data.get("clave"), Some(&"valor con espacios".to_string()));
    }
}
