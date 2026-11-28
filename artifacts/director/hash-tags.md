# Hash-tag registry (M6)

The sim hashes each optional piece of state under a numeric tag, written only while the state is present (so that no existing golden moves when a slice adds state). Slices running in parallel would otherwise pick the same next number, so the Director assigns the tags in each brief. At every merge, check the new tags against this list:
- `grep -n 'h.tag(' crates/sim/src/orders/hash.rs`
- the Object hash
- `Sim`'s hash in `crates/sim/src/sim.rs`

A gap is harmless: a slice that needs no new state leaves its number unused.

| Namespace | On master | Assigned (slice) |
|---|---|---|
| `Object`'s optional state | 9 buffs (M5-N5); 10 cooldown (GR); 11–13 stock, tether, drone (GD); 14 aircraft (GH1); 15 illusion_of (CM) | 16 GA (unused); 17 SP; 18 ST1; 19 GM; 20 GB; 21 SN (unused); 22 GP; 23 SL; 24 SB; 25 ST2; 26 GL; 27–28 GS1 (the Chopper's cargo and attachment, the Tower's schedule); 29 GS2; 30 unused (S6a needed none); 31 S6b; 32 SR (the armed Prism, the lock); 33 SA; 34 SC; 35 CP |
| `OrderKind` (`orders/hash.rs`) | up to 21 (18 Rebase, 19 ReturnToBase, GH1; 21 VehicleDeploy, GA); 20, 22 and 27 stay unused (CM, SN and GH2 needed none) | 23 GM (LayMine); 24 SP (Thrust); 25 ST1; 26 GB; 28 GP; 29 SL; 30 SB; 31 ST2; 32 GL; 33–35 GS1 (PickUpSupplies, DropOffSupplies, AttachToTower); 36–37 GS2 (Load, Unload All); 38 unused (S6a needed none); 39 S6b (the Mirage step, 0464); 40 SR (Relay); 41 SA; 42 SC; 43 CP (Drop) |
| `Sim` (`sim.rs`) | 1 held; 2 projectiles; 3 clouds | 4 SB (sludge patches); 5 SC (smoke, unless it lives in SB's patch store); 6 SA (only if the falling bomb needs state outside the projectiles' tag 2) |
| `Mover` | 10 rise (SD); 11 stays unused (GH2 needed none) | 12 SP; 13 GL; 14 ST2; 15 SA; 16 GS1; 17 GS2; 18 CP (the runway landing, the fall) |
| `Behaviour` (`orders/state/hash.rs`) | up to 13 (13 VehicleDeploy, GA); 14 stays unused (GH2 needed none) | 15 GM; 16 ST1; 17 SP; 18 GB; 19 GP; 20 GL; 21 GS1; 22 GS2; 23 ST2; 24 SR; 25 SA; 26 unused (S6a needed none); 27 SL; 28 SB; 29 SC; 30 S6b; 31 unused (CM needed none); 32 CP |
| `Behaviour::Enter` | sub-tag 1 descent (SD) | |
| The attack behaviours' repositioning (S6a, `orders/aim.rs`) | `AttackOrder` sub-tag 5, `Pursuit` sub-tag 1, `AttackGround` sub-tag 3 | |
| `Place` (`network.rs`) | 0 Network; 1 Site; 2 Building | 3 ST1 (Carrier) |
| `Held` (`network.rs`) | 1 leaving | 2 SN (service) |
| `Service` (`network/services.rs`, SN's) | 0 chamber (SN) | 1 SB (the Vault's turn) |

`AircraftState` (Object tag 14) has sub-tags of its own: GH2's `Landing` is sub-tag 1, only while set. GL and SA extend it after GH2, one at a time, with sub-tags 2 and 3.

**Other namespaces:** a slice that extends a hash not listed here takes its next free number, and says so in its report. Only slices running at the same time can collide, and the merge check catches it. Every M6 slice is briefed (2026-11-27); MQ adds a test that no two `OrderKind`s or optional `Object` states share a tag. M6-CP (the Pilgrim, 0462) was added later and took the next free numbers. A slice added after it takes `Object` 36, `OrderKind` 44, `Sim` 7, `Behaviour` 33 and `Mover` 19.
