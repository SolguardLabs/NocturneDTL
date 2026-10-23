# Operación y validación

Instala Rust compatible con `Cargo.toml` y Node.js 22. No se requiere base de datos ni RPC. Desde la raíz:

```bash
cargo build --locked
cargo run --locked --quiet -- --json < escenario.json
bash scripts/ci.sh
```

La validación completa ejecuta formato, build, tests Rust, Clippy con warnings como errores, recuento de líneas informativo y tests Node.js. Cada test Node invoca el binario y comprueba JSON o error esperado. Para aislar una regresión: `cargo test --locked` o `node --test tests/node/aggregate_reconciliation.test.js`.

## Secuencia de revisión

1. Revisar `Cargo.lock`, código y políticas antes de ejecutar un escenario nuevo.
2. Comparar recibos con `operations` y `snapshot.clock`.
3. Comprobar conservación por activo y exposición por posición; ninguna sustituye a la otra.
4. Examinar `liquidity_profiles` como análisis posterior, no autorización de pagos.
5. Comparar el calendario de tasas con los índices de las ventanas y los recibos de `fund_yield`.
6. Registrar SHA y repetir pruebas sobre el mismo árbol antes de publicar el tag.

No almacenar aperturas reales, credenciales, claves privadas ni datos de terceros en escenarios versionados. El destino de un retiro es una cadena de salida del motor local; no se transmite una transacción a una red.
