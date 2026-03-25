use minikv::KvStore;
use std::sync::{Mutex, OnceLock};

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn cleanup() {
    let _ = std::fs::remove_file(".minikv.data");
    let _ = std::fs::remove_file(".minikv.log");
}

#[test]
fn integracion01_flujo_completo_set_snapshot_load() {
    let _guard = match get_lock().lock() {
        Ok(guard) => guard,
        Err(_) => panic!("No se pudo adquirir el lock de prueba"),
    };
    cleanup();

    let mut store = KvStore::new();

    match store.set("usuario".to_string(), "juan".to_string()) {
        Ok(_) => {}
        Err(e) => panic!("Error al hacer set de usuario: {}", e),
    }

    match store.set("email".to_string(), "juan@example.com".to_string()) {
        Ok(_) => {}
        Err(e) => panic!("Error al hacer set de email: {}", e),
    }

    match store.set("edad".to_string(), "25".to_string()) {
        Ok(_) => {}
        Err(e) => panic!("Error al hacer set de edad: {}", e),
    }

    match KvStore::load() {
        Ok(loaded_for_check) => {
            assert_eq!(
                loaded_for_check.len(),
                3,
                "El store debería tener 3 elementos"
            );
        }
        Err(e) => panic!("Error al cargar para verificar: {}", e),
    }

    match KvStore::load() {
        Ok(store_for_snapshot) => match store_for_snapshot.snapshot() {
            Ok(_) => {}
            Err(e) => panic!("Error al hacer snapshot: {}", e),
        },
        Err(e) => panic!("Error al cargar antes de snapshot: {}", e),
    }

    match KvStore::load() {
        Ok(loaded_store) => {
            assert_eq!(
                loaded_store.len(),
                3,
                "El store cargado debería tener 3 elementos"
            );
            assert_eq!(
                loaded_store.get("usuario"),
                Some("juan".to_string()),
                "usuario no coincide"
            );
            assert_eq!(
                loaded_store.get("email"),
                Some("juan@example.com".to_string()),
                "email no coincide"
            );
            assert_eq!(
                loaded_store.get("edad"),
                Some("25".to_string()),
                "edad no coincide"
            );
        }
        Err(e) => panic!("Error al cargar el store: {}", e),
    }

    cleanup();
}

#[test]
fn integracion02_flujo_con_operaciones_log_complejas() {
    let _guard = match get_lock().lock() {
        Ok(guard) => guard,
        Err(_) => panic!("No se pudo adquirir el lock de prueba"),
    };
    cleanup();

    let mut store = KvStore::new();

    match store.set(
        "nombre_completo".to_string(),
        "Juan Carlos Perez Rodriguez".to_string(),
    ) {
        Ok(_) => {}
        Err(e) => panic!("Error en set de nombre_completo: {}", e),
    }

    match store.set(
        "descripcion".to_string(),
        "Desarrollador de software con experiencia en Rust".to_string(),
    ) {
        Ok(_) => {}
        Err(e) => panic!("Error en set de descripcion: {}", e),
    }

    match store.set("ciudad".to_string(), "Buenos Aires".to_string()) {
        Ok(_) => {}
        Err(e) => panic!("Error en set de ciudad: {}", e),
    }

    match KvStore::load() {
        Ok(loaded_for_check) => {
            assert_eq!(
                loaded_for_check.len(),
                3,
                "El store debería tener 3 elementos después de los sets iniciales"
            );
        }
        Err(e) => panic!("Error al cargar para verificar: {}", e),
    }

    match store.set("ciudad".to_string(), "Córdoba".to_string()) {
        Ok(_) => {}
        Err(e) => panic!("Error al modificar ciudad: {}", e),
    }

    match KvStore::load() {
        Ok(loaded_for_check) => {
            assert_eq!(
                loaded_for_check.get("ciudad"),
                Some("Córdoba".to_string()),
                "ciudad no fue modificada correctamente"
            );
            assert_eq!(
                loaded_for_check.len(),
                3,
                "El store debería seguir teniendo 3 elementos"
            );
        }
        Err(e) => panic!("Error al cargar para verificar: {}", e),
    }

    match store.set("descripcion".to_string(), "".to_string()) {
        Ok(_) => {}
        Err(e) => panic!("Error al eliminar descripcion: {}", e),
    }

    match KvStore::load() {
        Ok(loaded_for_check) => {
            assert_eq!(
                loaded_for_check.len(),
                2,
                "El store debería tener 2 elementos después de eliminar uno"
            );
            assert_eq!(
                loaded_for_check.get("descripcion"),
                None,
                "descripcion debería estar eliminada"
            );
        }
        Err(e) => panic!("Error al cargar para verificar: {}", e),
    }

    match KvStore::load() {
        Ok(store_for_snapshot) => match store_for_snapshot.snapshot() {
            Ok(_) => {}
            Err(e) => panic!("Error al hacer snapshot: {}", e),
        },
        Err(e) => panic!("Error al cargar antes de snapshot: {}", e),
    }

    match KvStore::load() {
        Ok(loaded_store) => {
            assert_eq!(
                loaded_store.len(),
                2,
                "El store cargado debería tener 2 elementos"
            );
            assert_eq!(
                loaded_store.get("nombre_completo"),
                Some("Juan Carlos Perez Rodriguez".to_string()),
                "nombre_completo no coincide"
            );
            assert_eq!(
                loaded_store.get("ciudad"),
                Some("Córdoba".to_string()),
                "ciudad debería estar modificada a Córdoba"
            );
            assert_eq!(
                loaded_store.get("descripcion"),
                None,
                "descripcion no debería existir"
            );
        }
        Err(e) => panic!("Error al cargar el store en la integración: {}", e),
    }

    cleanup();
}
