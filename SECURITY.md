# Modelo De Seguridad

NocturneDTL asume que los operadores de protocolo deben mantener cuatro
invariantes:

1. Todo retiro agregado debe estar respaldado por reservas del vault del activo.
2. Un commitment consumido no puede volver a liquidarse en el dominio interno.
3. Los nullifiers deben ser unicos dentro del ledger y persistir entre ventanas.
4. La reconciliacion debe conservar la igualdad entre depositos, reservas y
   retiros ejecutados.

## Alcance De Revision

La revision debe incluir el crate Rust, el contrato JSON del binario, los tests
JavaScript, las politicas de ventana y la salida de reconciliacion. El sistema no
incluye red P2P, custodia real, integracion on-chain ni generacion de pruebas de
conocimiento cero reales.

## Validaciones Automatizadas

- `cargo test --locked` valida la API Rust.
- `node --test "tests/node/*.test.js"` valida flujos de caja negra.
- `bash scripts/ci.sh` ejecuta formato, build, tests y lint local.

## Gestion De Dependencias

Las dependencias son deliberadamente reducidas. `serde` define el contrato JSON,
`blake3` modela compromisos criptograficos deterministas y `thiserror` mantiene
un contrato de errores estable para integraciones.

## Reportes Internos

Los reportes deben incluir activo, ventana, identificadores de commitment,
recibos, snapshot de vaults y pasos de reproduccion. No incluir secretos de
apertura en canales compartidos.
