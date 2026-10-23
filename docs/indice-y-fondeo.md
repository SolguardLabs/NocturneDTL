# Índice por ventanas y financiación

El rendimiento de una posición se expresa como un índice acumulado por ventana, no como un saldo adicional emitido en el momento del depósito. `LedgerConfig.yield_curve` define una tasa base y, opcionalmente, tasas específicas en `scheduled_rates_bps`. La tasa asignada a la ventana `w` se aplica al índice de `w-1`:

```text
I₁ = 1.000.000.000.000
r_w = tasa programada para w, o tasa base
I_w = floor(I_(w-1) × (10.000 + r_w) / 10.000)
valor_en_destino = floor(valor_nota × I_destino / I_referencia)
```

Todas las cantidades de notas y de reserva se expresan en unidades enteras del activo. `BasisPoints` limita cada tasa a 10.000 puntos y el índice rechaza épocas fuera del máximo configurado o un desbordamiento de 128 bits. Las operaciones de rotación deben avanzar de ventana y respetar `max_rollover_gap` y `max_notes_per_position`.

```mermaid
sequenceDiagram
    participant T as Tesorería
    participant V as Vault DTL
    participant N as Posición y nota
    participant I as Curva de índice
    participant A as Cuenta interna
    T->>V: fund_yield(71.200 unidades)
    N->>I: consultar I₁ e I₃
    I-->>N: I₁=10¹²; I₃=1,0712×10¹²
    N->>N: archivar compromiso anterior
    N-->>A: spend(nota valorada)
    A->>V: withdraw(1.071.200 unidades)
    V-->>A: recibo de retiro
```

Ejemplo: una nota de 1.000.000 unidades abierta en la ventana 1 se valora en la ventana 3. Si la tasa de la ventana 2 es 300 bps y la de la 3 es 400 bps, `I₂=1.030.000.000.000` e `I₃=1.071.200.000.000`. Su valor de referencia en la ventana 3 es 1.071.200 unidades. Para liquidar esa obligación sin depender de otras posiciones, la tesorería aporta 71.200 unidades; así la reserva pasa de 1.000.000 a 1.071.200. El redondeo se aplica en cada paso de composición y en la valoración.

Un escenario puede fijar la curva mediante `config`:

```json
{
  "config": {
    "yield_curve": {
      "scheduled_rates_bps": { "2": 300, "3": 400 }
    }
  },
  "operations": [
    { "op": "open_window", "window": 3, "anchor": "cierre-3" },
    { "op": "deposit", "note": "origen", "owner": "ana", "account": "ana-main", "asset": "DTL", "amount": 1000000, "position": "ana-1", "window": 1, "blinding": "apertura-ana-1" },
    { "op": "fund_yield", "operator": "operator.local", "asset": "DTL", "amount": 71200, "window": 3 },
    { "op": "roll", "source": "origen", "note": "cierre", "window": 3, "blinding": "apertura-ana-3" },
    { "op": "spend", "note": "cierre", "recipient": "ana-main", "memo": "liquidación" },
    { "op": "withdraw", "account": "ana-main", "asset": "DTL", "amount": 1071200, "destination": "dtl:ana", "window": 3 },
    { "op": "reconcile" }
  ]
}
```

El identificador `operator.local` del ejemplo procede de la configuración por defecto y representa al operador del motor local. El recibo de financiación acredita capital al vault; no acredita ninguna cuenta de usuario. Un retiro solo se ejecuta si el saldo interno y la reserva disponible lo cubren. La reconciliación nominal debe leerse junto con la exposición por posición; ninguna de las dos, aislada, certifica la capacidad de atender obligaciones futuras.
