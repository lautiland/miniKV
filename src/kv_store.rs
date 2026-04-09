//! Implementación del almacén clave-valor principal con persistencia.
//!
//! Este módulo provee la estructura principal `KvStore` que gestiona el
//! almacenamiento clave-valor en memoria con el registro de escritura y
//! soporte de snapshots para durabilidad.

use crate::persistence::{log, store};
use std::collections::HashMap;
use std::io::Result;

/// Estructura que representa el guardado de clave y valor.
/// El almacén mantiene un `HashMap` en memoria y sincroniza los cambios
/// a disco mediante el registro de escritura.
///
/// # Ejemplo
/// ```
/// use minikv::KvStore;
/// # let _ = std::fs::remove_file(".minikv.log");
/// # let _ = std::fs::remove_file(".minikv.data");
/// let mut store = KvStore::new();
/// store.set("name", "Alice").expect("Failed to set");
/// let loaded = KvStore::load().expect("Failed to load");
/// assert_eq!(loaded.get("name"), Some("Alice"));
/// # let _ = std::fs::remove_file(".minikv.log");
/// # let _ = std::fs::remove_file(".minikv.data");
/// ```
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
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::KvStore;
    /// let store = KvStore::new();
    /// assert_eq!(store.len(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        KvStore {
            data: HashMap::new(),
        }
    }

    /// Carga el estado desde los archivos de persistencia
    /// Carga el snapshot desde `.minikv.data` y reproduce las operaciones
    /// desde `.minikv.log` para reconstruir el estado actual.
    ///
    /// # Errors
    /// Retorna error si los archivos de datos o log son inválidos.
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::KvStore;
    /// # let _ = std::fs::remove_file(".minikv.log");
    /// # let _ = std::fs::remove_file(".minikv.data");
    /// let store = KvStore::load().expect("Failed to load");
    /// # let _ = std::fs::remove_file(".minikv.log");
    /// # let _ = std::fs::remove_file(".minikv.data");
    /// ```
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
    /// Los cambios se escriben al archivo de log inmediatamente para durabilidad.
    ///
    /// # Errors
    /// Retorna error si falla la escritura al log.
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::KvStore;
    /// # let _ = std::fs::remove_file(".minikv.log");
    /// # let _ = std::fs::remove_file(".minikv.data");
    ///
    /// let mut store = KvStore::new();
    /// store.set("key", "value").expect("Failed to set");
    /// store.set("key", "").expect("Failed to delete");  // Valor vacío elimina
    /// # let _ = std::fs::remove_file(".minikv.log");
    /// # let _ = std::fs::remove_file(".minikv.data");
    /// ```
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let linea_log = if value.is_empty() {
            // Unset: eliminar la clave
            self.data.remove(key);
            format!("set \"{key}\"")
        } else {
            // Set: quitar comillas internas y guardar con formato entrecomillado
            let value_sanitized = value.replace('"', "\\\"");
            self.data.insert(key.to_string(), value.to_string());
            format!("set \"{key}\" \"{value_sanitized}\"")
        };

        log::add_operation(&linea_log)?;
        Ok(())
    }

    /// Obtiene el valor asociado a una clave, `None` si no existe.
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::KvStore;
    /// let store = KvStore::new();
    /// assert_eq!(store.get("missing"), None);
    /// ```
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(String::as_str)
    }

    /// Retorna la cantidad de claves activas (con valor) en el almacenamiento.
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::KvStore;
    /// let store = KvStore::new();
    /// assert_eq!(store.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Verifica si el almacenamiento está vacío.
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::KvStore;
    /// let store = KvStore::new();
    /// assert!(store.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Genera un snapshot del estado actual y trunca el log.
    /// Escribe el estado actual a `.minikv.data` y limpia `.minikv.log`.
    ///
    /// # Errors
    /// Retorna error si falla la escritura del snapshot o el truncado del log.
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::KvStore;
    /// # let _ = std::fs::remove_file(".minikv.log");
    /// # let _ = std::fs::remove_file(".minikv.data");
    /// let store = KvStore::new();
    /// store.snapshot().expect("Failed to snapshot");
    /// # let _ = std::fs::remove_file(".minikv.log");
    /// # let _ = std::fs::remove_file(".minikv.data");
    /// ```
    pub fn snapshot(&self) -> Result<()> {
        store::save_snapshot(&self.data)?;
        log::truncate()?;
        Ok(())
    }
}

/// Fase del parser: qué parte del input estamos leyendo.
#[derive(PartialEq, Clone, Copy)]
enum ParsePhase {
    Key,
    Value,
}

/// Estado del parser para procesar caracteres.
struct ParseState {
    in_quotes: bool,
    escaped: bool,
    phase: ParsePhase,
}

impl Default for ParseState {
    fn default() -> Self {
        Self {
            in_quotes: false,
            escaped: false,
            phase: ParsePhase::Key,
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
        add_char(key, value, c, state.phase == ParsePhase::Key);
        state.escaped = false;
        return;
    }

    if c == '\\' && state.in_quotes {
        state.escaped = true;
        add_char(key, value, c, state.phase == ParsePhase::Key);
        return;
    }

    if c == '"' {
        state.in_quotes = !state.in_quotes;
        return;
    }

    if c == ' ' && !state.in_quotes && state.phase == ParsePhase::Key {
        state.phase = ParsePhase::Value;
        return;
    }

    add_char(key, value, c, state.phase == ParsePhase::Key);
}

/// Agrega un caracter a la clave o al valor según corresponda.
fn add_char(key: &mut String, value: &mut String, c: char, is_key: bool) {
    if is_key {
        key.push(c);
    } else {
        value.push(c);
    }
}

/// Aplica una operación del log al `HashMap` reconstruyendo el estado.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_sync::get_lock;

    fn cleanup() {
        let _ = std::fs::remove_file(".minikv.data");
        let _ = std::fs::remove_file(".minikv.log");
    }

    #[test]
    fn test01_new_store_empty() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let store = KvStore::new();
        assert_eq!(store.len(), 0);
        assert_eq!(store.get("inexistente"), None);

        cleanup();
    }

    #[test]
    fn test02_set_get_value() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        match store.set("nombre", "jose") {
            Ok(()) => {}
            Err(e) => panic!("Error en set: {e}"),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("nombre"), Some("jose"));
                assert_eq!(loaded.len(), 1);
            }
            Err(e) => panic!("Error al cargar: {e}"),
        }

        cleanup();
    }

    #[test]
    fn test03_set_overwrite_value() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        match store.set("nombre", "maria") {
            Ok(()) => {}
            Err(e) => panic!("Error en primer set: {e}"),
        }

        match store.set("nombre", "juana") {
            Ok(()) => {}
            Err(e) => panic!("Error en segundo set: {e}"),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("nombre"), Some("juana"));
                assert_eq!(loaded.len(), 1);
            }
            Err(e) => panic!("Error al cargar: {e}"),
        }

        cleanup();
    }

    #[test]
    fn test04_set_empty_remove_key() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        match store.set("clave", "valor") {
            Ok(()) => {}
            Err(e) => panic!("Error en primer set: {e}"),
        }

        match store.set("clave", "") {
            Ok(()) => {}
            Err(e) => panic!("Error al establecer valor vacío: {e}"),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("clave"), None);
                assert_eq!(loaded.len(), 0);
            }
            Err(e) => panic!("Error al cargar: {e}"),
        }

        cleanup();
    }

    #[test]
    fn test05_length_count_keys() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        match store.set("a", "1") {
            Ok(()) => {}
            Err(e) => panic!("Error en set de 'a': {e}"),
        }

        match store.set("b", "2") {
            Ok(()) => {}
            Err(e) => panic!("Error en set de 'b': {e}"),
        }

        match store.set("a", "") {
            Ok(()) => {}
            Err(e) => panic!("Error al eliminar 'a': {e}"),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("a"), None);
                assert_eq!(loaded.get("b"), Some("2"));
                assert_eq!(loaded.len(), 1);
            }
            Err(e) => panic!("Error al cargar: {e}"),
        }

        cleanup();
    }

    #[test]
    fn test06_load_store_after_set_unset() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        match store.set("frase", "hola mundo") {
            Ok(()) => {}
            Err(e) => panic!("Error en set de 'frase': {e}"),
        }

        match store.set("tmp", "123") {
            Ok(()) => {}
            Err(e) => panic!("Error en set de 'tmp': {e}"),
        }

        match store.set("tmp", "") {
            Ok(()) => {}
            Err(e) => panic!("Error al eliminar 'tmp': {e}"),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("frase"), Some("hola mundo"));
                assert_eq!(loaded.get("tmp"), None);
                assert_eq!(loaded.len(), 1);
            }
            Err(e) => panic!("Error al cargar el store: {e}"),
        }

        cleanup();
    }

    #[test]
    fn test07_snapshot_persist_and_truncate_log() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        store.set("a", "uno").unwrap();
        store.set("b", "dos palabras").unwrap();

        let loaded = KvStore::load().unwrap();
        loaded.snapshot().unwrap();

        let metadata = std::fs::metadata(".minikv.log").unwrap();
        assert_eq!(metadata.len(), 0);

        let loaded = KvStore::load().unwrap();
        assert_eq!(loaded.get("a"), Some("uno"));
        assert_eq!(loaded.get("b"), Some("dos palabras"));
        assert_eq!(loaded.len(), 2);

        cleanup();
    }

    #[test]
    fn test08_apply_operation_with_quotes() {
        let mut data = HashMap::new();

        apply_operation(&mut data, "set clave \"valor con espacios\"");

        assert_eq!(data.get("clave"), Some(&"valor con espacios".to_string()));
    }

    #[test]
    fn test09_is_empty_new_store() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let store = KvStore::new();
        assert!(store.is_empty());

        cleanup();
    }

    #[test]
    fn test10_is_empty_after_set() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        match store.set("clave", "valor") {
            Ok(()) => {}
            Err(e) => panic!("Error en set: {e}"),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert!(!loaded.is_empty());
            }
            Err(e) => panic!("Error al cargar: {e}"),
        }

        cleanup();
    }

    #[test]
    fn test11_set_get_value_with_escaped_quotes() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        let value_with_quotes = "dijo \"hola\" y se fue";
        match store.set("frase", value_with_quotes) {
            Ok(()) => {}
            Err(e) => panic!("Error en set: {e}"),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("frase"), Some(value_with_quotes));
            }
            Err(e) => panic!("Error al cargar: {e}"),
        }

        cleanup();
    }

    #[test]
    fn test12_set_get_key_with_spaces() {
        let _guard = get_lock().lock().unwrap();
        cleanup();

        let mut store = KvStore::new();
        match store.set("mi clave", "mi valor") {
            Ok(()) => {}
            Err(e) => panic!("Error en set: {e}"),
        }

        match KvStore::load() {
            Ok(loaded) => {
                assert_eq!(loaded.get("mi clave"), Some("mi valor"));
            }
            Err(e) => panic!("Error al cargar: {e}"),
        }

        cleanup();
    }

    #[test]
    fn test13_apply_operation_with_escaped_quotes() {
        let mut data = HashMap::new();

        apply_operation(&mut data, "set \"clave\" \"valor con \\\"comillas\\\"\"");

        assert_eq!(
            data.get("clave"),
            Some(&"valor con \"comillas\"".to_string())
        );
    }

    #[test]
    fn test14_apply_operation_unset() {
        let mut data = HashMap::new();
        data.insert("clave".to_string(), "valor".to_string());

        apply_operation(&mut data, "set \"clave\"");

        assert_eq!(data.get("clave"), None);
    }

    #[test]
    fn test15_parse_kv_simple() {
        let (key, value) = parse_kv("\"clave\" \"valor\"");
        assert_eq!(key, "clave");
        assert_eq!(value, "valor");
    }

    #[test]
    fn test16_parse_kv_empty_value() {
        let (key, value) = parse_kv("\"clave\"");
        assert_eq!(key, "clave");
        assert_eq!(value, "");
    }

    #[test]
    fn test17_parse_kv_with_spaces() {
        let (key, value) = parse_kv("\"mi clave\" \"mi valor\"");
        assert_eq!(key, "mi clave");
        assert_eq!(value, "mi valor");
    }
}
