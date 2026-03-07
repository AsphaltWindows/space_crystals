# Combat Systems

Combat system implementations for the combat module.

## Files

- `core.rs` — Core combat systems: attack command processing, attack phase progression, auto-targeting, idle leash, damage application, dead entity removal
- `behaviors.rs` — Combat behavior systems: attacking objects, attacking locations, attack-move, patrol scanning, hold position engagement
- `types.rs` — Types specific to combat systems (convention file; most types in parent combat/types.rs)
- `utils.rs` — Shared helpers for combat systems (convention file; most utils in parent combat/utils.rs)
