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
    store.set("a", "1").unwrap();
    store.set("b", "2").unwrap();
    store.set("c", "3").unwrap();
    store.set("d", "4").unwrap();
    store.set("e", "5").unwrap();

    let loaded = KvStore::load().unwrap();

    loaded.snapshot().unwrap();

    let loaded = KvStore::load().unwrap();
    assert_eq!(loaded.len(), 5);
    assert_eq!(loaded.get("a"), Some("1".to_string()));
    assert_eq!(loaded.get("b"), Some("2".to_string()));
    assert_eq!(loaded.get("c"), Some("3".to_string()));
    assert_eq!(loaded.get("d"), Some("4".to_string()));
    assert_eq!(loaded.get("e"), Some("5".to_string()));

    cleanup();
}
