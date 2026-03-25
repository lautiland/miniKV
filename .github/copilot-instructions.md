# Instrucciones para Copilot - Proyecto MiniKV

## Descripción del Proyecto
MiniKV es un almacén de clave-valor en Rust con persistencia en archivos usando un log de operaciones y snapshots. Es un proyecto educativo enfocado en calidad de código, testing robusto y cumplimiento de estándares.

## Restricciones Críticas

### Manejo de Errores - REGLA FUNDAMENTAL
- **NUNCA** usar `panic!()`, `unwrap()`, `expect()`
- **NUNCA** usar `exit()`
- **NUNCA** usar módulo `mem`
- **NUNCA** usar bloques `unsafe`
- Todo error debe manejarse idiomáticamente con `Result<T>` y `match`
- El programa debe finalizar normalmente sin salidas abruptas

### Gestión de Memoria
- Evitar `.clone()` y `.copy()` en estructuras principales de datos
- Usar referencias (`&`) y ownership correctamente
- Las funciones pueden tener máximo 30 líneas - si exceden, particionar en varias funciones
- Sin crates externos - solo usar la biblioteca estándar

## Convenciones de Testing

### Ejecución de Tests
- **SIEMPRE** ejecutar tests unitarios con: `cargo test --lib -- --test-threads=1`
- **SIEMPRE** ejecutar tests de integración con: `cargo test --test integracion -- --test-threads=1`
- Los tests se ejecutan secuencialmente para evitar conflictos con archivos compartidos (`.minikv.log`, `.minikv.data`)
- Los tests también deben evitar `unwrap()`, `panic!()` y `expect()` en el código principal

### Estructura de Tests
- Tests unitarios en módulos `#[cfg(test)]` dentro de cada archivo:
  - `src/lib.rs` → tests del KvStore
  - `src/log.rs` → tests de operaciones de log
  - `src/store.rs` → tests de snapshot
  - `src/main.rs` → tests de parsing de comandos
- Tests de integración en `tests/integracion.rs` → flujos completos de uso

### Nomenclatura de Tests
- Usar nombres descriptivos en **español**
- Formato: `test{NN}_{descripcion}` donde NN es número secuencial
- Ejemplos: `test01_eliminar_comillas`, `test02_store_nuevo_vacio`, `test03_lee_operaciones_archivo_inexistente`

## Manejo de Errores en Tests

### ✅ CORRECTO
```rust
match some_operation() {
    Ok(value) => {
        assert_eq!(value, expected);
    }
    Err(e) => panic!("Descripción del error: {}", e),
}
```

### ❌ NO HACER
```rust
let value = some_operation().unwrap();  // ❌ Evitar unwrap()
match some_operation() {
    Ok(value) => assert_eq!(value, expected),
    Err(_) => assert!(false, "mensaje"),  // ❌ Usar panic!() en lugar de assert!(false, ...)
}
```

## Cumplimiento de Clippy

### Regla Fundamental
- **NUNCA** usar `#[allow(...)]` para silenciar warnings de clippy
- **SIEMPRE** corregir el problema subyacente, no el síntoma

### Ejecución de Clippy
```bash
cargo clippy -- -D warnings  # Strict mode
cargo fmt                     # Formatear código
```

### Warnings Comunes a Evitar
- `assertions_on_constants` → usar `panic!()` en tests en lugar de `assert!(false, ...)`
- `dead_code` → remover código sin usar
- `unused_imports` → remover imports no necesarios
- Spacing y formatting → usar `cargo fmt`

## Requerimientos Técnicos

### Versión de Rust
- **OBLIGATORIO**: Rust 1.94 (versión estable)
- Usar solo biblioteca estándar (std)
- Sin dependencias externas (sin crates)

### Compilación y Validación
- **NO** debe compilar con warnings
- `cargo clippy -- -D warnings` debe pasar 100%
- `cargo fmt` debe dejar el código formateado
- `cargo build` debe completar sin errores

### Documentación
- Usar `///` para documentación de funciones públicas
- Usar `///` para documentación de tipos y módulos
- Seguir el estándar de cargo doc
- La documentación debe ser clara y concisa

### Estructura de Módulos
- Cada tipo de dato en su propio módulo/archivo
- Modularidad clara: `log`, `store`, operaciones en `lib.rs`
- Importes ordenados alfabéticamente

### Limitaciones de Código
- Máximo 30 líneas por función
- Si necesita más, particionar en funciones auxiliares
- Sin bloques `unsafe`
- Sin uso del módulo `mem`

## Formato de Persistencia - ESTRICTAMENTE REQUERIDO

### Archivo `.minikv.log`
- Append-only log de operaciones
- Formato: una operación por línea
- Ejemplos:
  ```
  set clave1 valor1
  set clave2 valor2
  set clave2
  set clave1
  ```
- Solo se permite escribir al final, nunca modificar operaciones previas

### Archivo `.minikv.data`
- Snapshot del estado completo
- Formato: `clave=valor`, una entrada por línea
- Ejemplos:
  ```
  clave1=valor1
  clave2=valor2
  ```
- Se sobrescribe completamente al hacer snapshot
- IMPORTANTE: Los valores NO deben contener espacios sin comillas

## Comportamiento de Comandos

### Comando `set`
- `minikv set <clave> <valor>` → Asocia valor a clave
- `minikv set <clave>` → Desasocia valor (unset) - escribe línea vacía en log
- Siempre retorna `OK`
- Las operaciones se escriben en el log, NO en memoria directamente

### Comando `get`
- Retorna el valor si existe
- Retorna `NOT FOUND` si no existe o fue eliminado

### Comando `length`
- Retorna cantidad de claves CON valor (las eliminadas no cuentan)

### Comando `snapshot`
- Compacta el log y persiste el estado
- Trunca el archivo `.minikv.log` después de guardar snapshot

## Documentación

### Comentarios
- Usar `///` para funciones públicas
- Usar `//` para comentarios internos
- Ser breve pero descriptivo

### Ejemplo
```rust
/// Carga el KvStore desde disco (snapshot + log)
pub fn load() -> Result<KvStore> {
    // Cargar snapshot
    let mut data = store::load_snapshot().unwrap_or_default();
    
    // Aplicar operaciones del log
    // ...
}
```

## Flujo de Trabajo

### Antes de Commit
1. Correr tests: `cargo test --lib -- --test-threads=1`
2. Correr clippy: `cargo clippy -- -D warnings`
3. Formatear: `cargo fmt`
4. Verificar sin warnings

### Creación de Nuevos Tests
1. Agregar test en el módulo correspondiente
2. Nombrar con numeración consecutiva
3. Usar `match` para error handling
4. Llamar `cleanup()` al inicio
5. Verificar que pase con `--test-threads=1`

## Comandos Útiles

```bash
# Testing
cargo test --lib -- --test-threads=1           # Tests unitarios
cargo test --bin minikv -- --test-threads=1    # Tests de main.rs
cargo test --test integracion -- --test-threads=1  # Tests integración

# Validación
cargo clippy -- -D warnings                     # Lint con modo strict
cargo fmt                                        # Formatear código

# Combinado
cargo test --lib -- --test-threads=1 && \
cargo test --bin minikv -- --test-threads=1 && \
cargo test --test integracion -- --test-threads=1 && \
cargo clippy -- -D warnings && \
cargo fmt
```

## Estructura de Código

### Importes
- Ordenar alfabéticamente
- Agrupar por: stdlib → crate → local
- Ejemplo:
```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;

use crate::log;
use crate::store;
```

### Match Expressions
- Preferir `match` sobre `if let` para múltiples casos
- Usar destructuring cuando sea posible
- Siempre cubrir todos los casos

### Error Handling Público
- Definir enum `Error` con todos los casos posibles
- Implementar método `mensaje()` para mensajes de error
- Retornar `Result<T>` (no `Option<T>` en funciones públicas)

### Arquitectura de Datos
- **KvStore struct**: Contiene estado en memoria (HashMap)
- **Módulo log**: Funciones para operaciones con `.minikv.log`
- **Módulo store**: Funciones para operaciones con `.minikv.data`
- **Load operation**: Carga snapshot + aplica operaciones del log secuencialmente

## Notas Importantes

- Este es un proyecto **educativo** enfocado en calidad
- La **integridad del código** es prioritaria sobre velocidad
- Todos los tests deben pasar **sin excepciones**
- Clippy debe estar 100% limpio (sin warnings ni errors)
- Los tests son parte del contrato del código, no se deben remover sin justificación
