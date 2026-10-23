# NocturneDTL

![NocturneDTL — Solguard Labs](assets/banner.png)

NocturneDTL es un motor contable en Rust para posiciones privadas sobre un libro DTL. Representa depósitos mediante notas y compromisos deterministas, revaloriza derechos entre ventanas con un índice acumulado, consume notas mediante anuladores y liquida retiros contra reservas por activo. El binario ejecuta escenarios JSON completos sin depender de servicios externos.

> Esta distribución es un motor local: no incorpora red P2P, contratos on-chain, custodia externa ni pruebas de conocimiento cero. Ninguna operación mueve activos fuera del proceso.

## Arquitectura

```mermaid
flowchart LR
    I[Escenario JSON] --> S[Orquestador]
    S --> W[Ventanas y políticas]
    W --> Y[Índice acumulado y tasas]
    S --> P[Notas y compromisos]
    Y --> P
    P --> N[Anuladores]
    N --> A[Cuentas internas]
    T[Tesorería: fondeo de rendimiento] --> V[Vault por activo]
    A --> V[Vault por activo]
    V --> R[Reconciliación]
    V --> L[Perfil de liquidez]
    R --> O[Resultado JSON]
    L --> O
```

`src/scenario.rs` mantiene el contrato de entrada/salida; `src/ledger.rs` coordina transiciones; `src/privacy.rs` construye compromisos; `src/yield_curve.rs` calcula el índice; `src/reconcile.rs` publica saldos agregados. `src/liquidity.rs` calcula capacidad bajo estrés sin modificar el ledger.

## Puesta en marcha

Requiere Rust compatible con `Cargo.toml` y Node.js 22 para las pruebas de integración.

```bash
cargo build --locked
cargo run --locked --quiet -- --json < escenario.json
bash scripts/ci.sh
```

Ejemplo de `escenario.json`:

```json
{
  "operations": [
    { "op": "deposit", "note": "n1", "owner": "alice", "account": "alice-main", "asset": "DTL", "amount": 350, "position": "p1", "window": 1, "blinding": "apertura-local-1" },
    { "op": "spend", "note": "n1", "recipient": "alice-main", "memo": "liquidación interna" },
    { "op": "withdraw", "account": "alice-main", "asset": "DTL", "amount": 200, "destination": "dtl:alice", "window": 1 },
    { "op": "reconcile" }
  ]
}
```

La salida incluye `notes`, `receipts`, `snapshot`, `reconciliations` y `liquidity_profiles`. Las cantidades son unidades enteras del activo; la escala decimal la define su registro.

## Modelo económico

Por activo se observa `entradas al vault = retiros + reservas`: las entradas incluyen depósitos de posiciones y aportaciones de tesorería identificadas por recibo. Para una ventana `w`, el índice se compone como `I_w = floor(I_(w-1) × (10.000 + r_w) / 10.000)`, con `I_1 = 10¹²`. Las tasas `r_w` se expresan en puntos básicos y pueden programarse por ventana. Una rotación posterior emite una nota valorada con el cociente de índices y archiva el compromiso anterior; el consumo acredita saldo interno antes de permitir el retiro. La financiación de tesorería es una operación separada: la existencia de un índice no crea por sí sola reservas.

El perfil de liquidez aplica un recorte de estrés de 2.000 puntos básicos: `reserva_estresada = floor(reserva × 8.000 / 10.000)`. La capacidad observada es el mínimo entre esa reserva y el límite por solicitud de retiro. Es una proyección informativa, no una autorización.

La conservación agregada es una señal operacional, no una prueba de solvencia de cada posición. La revisión de autorizaciones, dominio de anuladores y exposición individual se hace por separado.

## Documentación

- [Arquitectura y estados](docs/arquitectura.md)
- [Modelo económico y límites](docs/modelo-economico.md)
- [Índice por ventanas y financiación](docs/indice-y-fondeo.md)
- [Operación y validación](docs/operacion.md)
- [Política de seguridad](SECURITY.md)

## Versionado

La rama `production`, el tag anotado `v2.0.0` y la publicación `Production 2.0.0` identifican la misma revisión del motor local. Véase [CHANGELOG.md](CHANGELOG.md).
