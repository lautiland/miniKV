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
    let _guard = get_lock().lock().unwrap();
    cleanup();

    let mut store = KvStore::new();
    store.set("usuario", "juan").unwrap();
    store.set("email", "juan@example.com").unwrap();
    store.set("edad", "25").unwrap();

    let loaded = KvStore::load().unwrap();
    assert_eq!(loaded.len(), 3);

    let loaded = KvStore::load().unwrap();
    loaded.snapshot().unwrap();

    let loaded = KvStore::load().unwrap();
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded.get("usuario"), Some("juan".to_string()));
    assert_eq!(loaded.get("email"), Some("juan@example.com".to_string()));
    assert_eq!(loaded.get("edad"), Some("25".to_string()));

    cleanup();
}

#[test]
fn integracion02_flujo_con_operaciones_log_complejas() {
    let _guard = get_lock().lock().unwrap();
    cleanup();

    let mut store = KvStore::new();
    store
        .set("nombre_completo", "Juan Carlos Perez Rodriguez")
        .unwrap();
    store
        .set(
            "descripcion",
            "Desarrollador de software con experiencia en Rust",
        )
        .unwrap();
    store.set("ciudad", "Buenos Aires").unwrap();
    assert_eq!(KvStore::load().unwrap().len(), 3);

    store.set("ciudad", "Córdoba").unwrap();
    let reloaded = KvStore::load().unwrap();
    assert_eq!(reloaded.get("ciudad"), Some("Córdoba".to_string()));

    store.set("descripcion", "").unwrap();
    let reloaded = KvStore::load().unwrap();
    assert_eq!(reloaded.len(), 2);
    assert_eq!(reloaded.get("descripcion"), None);

    KvStore::load().unwrap().snapshot().unwrap();

    let loaded = KvStore::load().unwrap();
    assert_eq!(
        loaded.get("nombre_completo"),
        Some("Juan Carlos Perez Rodriguez".to_string())
    );
    assert_eq!(loaded.get("ciudad"), Some("Córdoba".to_string()));
    assert_eq!(loaded.get("descripcion"), None);

    cleanup();
}
