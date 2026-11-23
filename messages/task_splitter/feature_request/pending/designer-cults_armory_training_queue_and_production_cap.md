# cults_armory_training_queue_and_production_cap

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Resolved the Cults Armory's open training-queue questions and standardized production queue capacity across all factions.

## artifacts/designer/design/cults_objects.md — "Armory" section

**Resolved** (previously "Open Questions" — now removed):
- The Armory supports a queueable, serial training system: orders process one at a time, in FIFO order.
- Queuing a Train Soldier/Train Gunner order immediately deducts its Space Crystals cost AND reserves one Recruit from `StoredRecruits`, moving it into the new `TrainingQueue` entry. Only the front of the queue is actively training; later entries wait with their cost/Recruit already committed (mirrors how GDO's Barracks/SupplyTower and Syndicate's Hideout build queues deduct cost at queue-time, not at production-start).
- New `TrainingQueue` state: array of `{UnitType, ReservedRecruit}`, max 20 entries (see capacity note below). Replaced the old placeholder `TrainingQueue: TBD`.
- New `CurrentTrainingProgress` field tracks frames elapsed for the front queue entry.
- New **X: Cancel Production** command: removes the *last* entry in TrainingQueue (not necessarily the one currently training — same convention as GDO's Cancel Production), refunds its full Space Crystals cost, and returns its reserved Recruit to `StoredRecruits` (the Recruit stays inside the Armory, it is not ejected onto the field).
- `Eject All` now explicitly only affects `StoredRecruits` (the unreserved pool) — Recruits already reserved by a queued training order are unaffected and stay committed to their order.
- Q/W (Train Soldier/Train Gunner) commands now explicitly require: StoredRecruits not empty, TrainingQueue has fewer than 20 entries, and sufficient Space Crystals.

## Production queue capacity standardized to 20 (was 5)

Changed all existing production queues from a max of 5 entries to a max of 20 entries, for consistency with the new Armory queue:
- **artifacts/designer/design/gdo_objects.md**: Barracks `BuildQueue` (Peacekeeper production) and SupplyTower `BuildQueue` (Supply Chopper production) — both now max 20, "fewer than 20 entries" gating updated accordingly.
- **artifacts/designer/design/syndicate_objects.md**: Hideout `BuildQueue` (Agent/Guard production) — now max 20, "fewer than 20 entries" gating updated accordingly.

This is a straightforward numeric change — no other queue behavior (FIFO order, cost-deducted-at-queue-time, Cancel Production removes last entry) changed for these existing structures.

## QA Instructions

**Armory training queue:**
1. Build/select an Armory with Recruits in its `StoredRecruits` pool. Queue several Train Soldier / Train Gunner orders back to back. Confirm each queue action immediately deducts the correct Space Crystals cost and removes one Recruit from the visible stored count, even though only the first order is actively "in progress."
2. Confirm only one unit trains at a time (serial) — the next queued order doesn't start until the current one finishes.
3. Let a training order complete. Confirm the trained unit (Soldier/Gunner) exits from the Armory's exit side, and its reserved Recruit is now gone (not returned).
4. Queue up multiple orders, then issue Cancel Production (X). Confirm it removes the *most recently queued* entry (not the one currently training), refunds that entry's full Space Crystals cost, and its Recruit reappears in StoredRecruits (still inside the Armory — not ejected onto the field).
5. With training orders queued (Recruits reserved), issue Eject All. Confirm only unreserved Recruits in StoredRecruits are ejected; Recruits committed to queued orders stay put and their orders remain queued.
6. Confirm you cannot queue more than 20 training orders at once, and cannot queue if StoredRecruits is empty or Space Crystals are insufficient.

**Production queue cap (GDO Barracks, GDO Supply Tower, Syndicate Hideout):**
1. At each of these three structures, queue production orders and confirm you can queue up to 20 at once (previously capped at 5).
2. Confirm Cancel Production still removes the last-queued entry and refunds its full cost, unchanged from existing behavior — only the cap changed.


> Note (designer, 2026-10-18): the Syndicate "Headquarters" expansion has been renamed to "Hideout". All references in this message were updated accordingly — implement under the name Hideout.
