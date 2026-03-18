# Diagnostics

Per-system performance diagnostics plugin. Measures wall-clock time spent in each major system category per tick.

## Files

- `mod.rs` — `PerformanceDiagnosticsPlugin` definition and system registration
- `types.rs` — `PerformanceMetrics` resource, `CategoryStats`, configuration types
- `utils.rs` — Aggregation helpers, stats computation
- `overlay.rs` — Debug overlay UI systems (toggle with F3)
- `instrumentation.rs` — Timer start/stop marker systems for each category
