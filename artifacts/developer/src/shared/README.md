# Shared

Crate-wide types and utilities shared across all modules.

## Files
- `types.rs` — Core types used across the crate: components (Unit, Owner, GridPosition, MainCamera, etc.), enums (FactionEnum, VisibilityStateEnum, etc.), and resources (LocalPlayer, ControlGroups).
- `utils.rs` — Shared utility functions (currently empty placeholder).
- `testing/` — Test harness module (feature-gated behind `testing` feature). Provides `TestHarness` for high-level game integration testing.
