# Hash-tag registry (M6)

The sim hashes each optional piece of state under a numeric tag, written only while the state is present (so that no existing golden moves when a slice adds state). Slices running in parallel would otherwise pick the same next number, so the Director assigns the tags in each brief. At every merge, check the new tags against this list:
- `grep -n 'h.tag(' crates/sim/src/orders/hash.rs`
- the Object hash
- `Sim`'s hash in `crates/sim/src/sim.rs`

A gap is harmless: a slice that needs no new state leaves its number unused.

| Namespace | On master | Assigned (slice) |
|---|---|---|
| `Object`'s optional state | 9 buffs (M5-N5); 10 cooldown (GR); 11–13 stock, tether, drone (GD); 14 aircraft (GH1) | 15 CM; 16 GA; 17 SP; 18 ST1; 19 GM; 20 GB; 21 SN (only if it adds Object state); 22 GP; 23 SL; 24 SB; 25 ST2; 26 GL; 27–28 GS1 (the Chopper's cargo and attachment, the Tower's schedule); 29 GS2; 30 S6a; 31 S6b |
| `OrderKind` (`orders/hash.rs`) | up to 19 (18 Rebase, 19 ReturnToBase, GH1) | 20 CM; 21 GA; 22 SN; 23 GM (LayMine); 24 SP (Thrust); 25 ST1; 26 GB; 27 GH2; 28 GP; 29 SL; 30 SB; 31 ST2; 32 GL; 33–35 GS1 (PickUpSupplies, DropOffSupplies, AttachToTower); 36–37 GS2 (Load, Unload All); 38 S6a; 39 S6b |
| `Sim` (`sim.rs`) | 1 held; 2 projectiles; 3 clouds | 4 SB (sludge patches); 5 SC (smoke) |
| `Mover` | 10 rise (SD) | |
| `Behaviour::Enter` | sub-tag 1 descent (SD) | |

GH2 extends `AircraftState` inside Object tag 14, and needs no new tag. Slices not yet briefed (GS2, SR, SA, SC, MQ) take the next free numbers when their briefs are written.
