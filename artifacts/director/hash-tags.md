# Hash-tag registry (M6)

The sim hashes each optional piece of state under a numeric tag, written only while the state is present (so that no existing golden moves when a slice adds state). Slices running in parallel would otherwise pick the same next number, so the Director assigns the tags in each brief. At every merge, check the new tags against this list:
- `grep -n 'h.tag(' crates/sim/src/orders/hash.rs`
- the Object hash
- `Sim`'s hash in `crates/sim/src/sim.rs`

A gap is harmless: a slice that needs no new state leaves its number unused.

| Namespace | On master | Assigned (slice) |
|---|---|---|
| `Object`'s optional state | 9 buffs (M5-N5); 10 cooldown (GR); 11–13 stock, tether, drone (GD); 14 aircraft (GH1); 15 illusion_of (CM); 17 thrust (SP2a); 27 the Chopper's cargo and attachment, 28 the Tower's schedule (GS1) | 16 GA (unused); 18 ST1 (unused: the carrier needed no `Object` state); 19 GM; 20 GB; 21 SN (unused); 22 GP; 23 SL; 24 SB; 25 ST2; 26 GL (unused); 29 GS2, now R2's (the carrier archetypes build GS2's mechanics); 30 unused (S6a needed none); 31 S6b; 32 SR (the armed Prism, the lock); 33 SA; 34 SC; 35 CP |
| `OrderKind` (`orders/hash.rs`) | up to 21 (18 Rebase, 19 ReturnToBase, GH1; 21 VehicleDeploy, GA); 24 Thrust (SP2a); 25 UnloadAll (ST1); 33–35 PickUpSupplies, DropOffSupplies, AttachToTower (GS1); 20, 22 and 27 stay unused (CM, SN and GH2 needed none) | 23 GM (LayMine); 26 GB; 28 GP; 29 SL; 30 unused (SB needed none); 31 ST2; 32 GL (unused); 36–37 GS2 (Load, Unload All), now R2's; 38 unused (S6a needed none); 39 S6b (the Mirage step, 0464); 40 SR (Relay); 41 SA; 42 SC; 43 CP (Drop) |
| `Sim` (`sim.rs`) | 1 held; 2 projectiles; 3 clouds; 4 patches (SB: every `PatchKind`, smoke included) | 5 stays free (SC's smoke shares the patches); 6 SA (only if the falling bomb needs state outside the projectiles' tag 2) |
| `Mover` | 10 rise (SD); 12 unfold (SP phase 1); 13 approach (GL); 11 stays unused (GH2 needed none) | 14 ST2; 15 SA; 16 unused (GS1 needed none); 17 GS2, now R1's or R2's (whichever needs `Mover` state first); 18 CP (the runway landing, the fall) |
| `Behaviour` (`orders/state/hash.rs`) | up to 13 (13 VehicleDeploy, GA); 14, 16 and 20 stay unused (GH2, ST1 and GL needed none); 17 Thrust (SP2a) | 15 GM; 18 GB; 19 GP; 21 unused (GS1 needed none); 22 GS2, now R2's; 23 ST2; 24 SR; 25 SA; 26 unused (S6a needed none); 27 SL; 28 unused (SB needed none); 29 SC; 30 S6b; 31 unused (CM needed none); 32 CP |
| `Behaviour::Enter` | sub-tag 1 descent (SD) | |
| The attack behaviours' repositioning (S6a, `orders/aim.rs`) | `AttackOrder` sub-tag 5, `Pursuit` sub-tag 1, `AttackGround` sub-tag 3 | |
| `Place` (`network.rs`) | 0 Network; 1 Site; 2 Building; 3 Carrier (ST1) | |
| `Held` (`network.rs`) | 1 leaving | 2 SN (service) |
| `Service` (`network/services.rs`, SN's) | 0 chamber (SN); 1 stays unused (SB's Vault needed none: the tanks are its turn) | |

`AircraftState` (Object tag 14) has sub-tags of its own: GH2's `Landing` is sub-tag 1, only while set. GL needed no sub-tag (2 stays unused); SA takes 3.

**Other namespaces:** a slice that extends a hash not listed here takes its next free number, and says so in its report. Only slices running at the same time can collide, and the merge check catches it. Every M6 slice is briefed (2026-11-27); MQ adds a test that no two `OrderKind`s or optional `Object` states share a tag. M6-CP (the Pilgrim, 0462) was added later and took the next free numbers. A slice added after it takes `Object` 36, `OrderKind` 44, `Sim` 7, `Behaviour` 33 and `Mover` 19.
