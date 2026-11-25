# Tech Prerequisites and Research

## Tech Prerequisites

A structure type may declare a **TechPrerequisite**: one or more structure types the player must own before construction of that structure can be started (e.g. "player owns at least one Barracks"). This applies to all factions.

### Counting Rule
A structure only counts toward a TechPrerequisite once it is **fully constructed and present on the map**:
- **GDO**: the building must have been placed. A completed building sitting in a constructor's ReadyToPlace slot does not count.
- **Syndicate**: the underground expansion must have finished construction.
- **Colonists**: the Warp In transformation must have completed. A Beacon still mid-transformation does not count. (Most Colonists buildings have no TechPrerequisites; the exceptions are defensive structures such as the Skylance and Plasma Turret — see factions/colonists/structures/README.md > Tech Model. The counting rule also applies to Colonists Unit Prerequisites.)
- **Cults**: the Recruits must have finished constructing the building.

Structures under construction (or awaiting placement) never satisfy a TechPrerequisite.

### Check Timing
The TechPrerequisite is checked only at the moment construction is **started** (queued at the constructor, Warp In begun, expansion construction begun, etc.). Options whose TechPrerequisite is not met are unavailable in the build menu.

### Losing a Prerequisite
If the player loses their last qualifying prerequisite structure:
- Already-completed structures that required it keep functioning normally.
- Constructions already started (including GDO buildings in progress or ReadyToPlace) continue and can be completed/placed normally.
- New constructions that require it cannot be started until the player again owns a qualifying (fully constructed) prerequisite structure.

### Unit Prerequisites
A unit type's production option may also declare a TechPrerequisite: the structure(s) the player must own before that unit can be queued at its production building (e.g. the Cults Gunner, trained at the Armory, requires a Gunworks; Cults advanced aircraft, built at the Airstrip, require a Scrap Hangar). The unit is still produced at its normal production building — the prerequisite structure only unlocks the option. The same rules apply as for structures:
- **Counting Rule**: identical to the structure counting rule above.
- **Check Timing**: checked only when the unit is queued. Production options whose TechPrerequisite is not met are unavailable on the production building's panel.
- **Losing a Prerequisite**: units already queued or in production continue and complete normally; new ones cannot be queued until the player again owns a qualifying prerequisite structure.

## Research

A structure type may offer **Research**: technologies that a player pays for once at that structure and that then apply to that player's eligible objects. The shared rules below apply to all factions. Individual factions may have exceptions, listed under Faction Exceptions.

### Shared Rules
1. **Once per player, applies everywhere**: a technology is researched once, at a specific structure type, and on completion applies to all of that player's eligible objects — both existing ones and ones created later (unless a faction exception says otherwise).
2. **Cost and time**: each technology has a resource cost and a research time. The cost is deducted when the technology is queued.
3. **Serial queue**: a research structure researches one technology at a time. Additional technologies can be queued behind it and are processed in queue order.
4. **No duplicates**: a technology cannot be queued or in progress at more than one structure at the same time. Once completed, it no longer appears in any research menu.
5. **Levels**: a multi-level research line is a chain of separate technologies, each requiring the previous level.
6. **Cancel**: cancelling a queued or in-progress technology refunds its full cost (standard **X: Cancel Upgrade** slot — removes the last entry in the research queue).
7. **Destruction**: if a research structure is destroyed, all of its in-progress and queued research is cancelled, with a full refund of each — the same as cancelling.
8. **Permanence**: completed research is permanent. Losing the structure afterward does not revoke it.

### Faction Exceptions

The Colonists have no exceptions: they follow the shared rules exactly.

#### Cults — stat upgrades are not retroactive
For the Cults, research that improves unit stats applies only to units that receive their stats **after** the research completes. Units that already exist are unaffected. The moment that counts depends on how the stat is applied:
- **Augmentation improvements** (Temple): the moment the unit's augmentation finishes at the Altar (see factions/cults/augmentation.md > Augmentation > Level Locking).
- **Other stat upgrades** (not yet designed): the moment the unit finishes training.

#### Cults — research is staffed by Recruits
A Cults research building only makes research progress while Recruits are inside it. Only Recruits can staff research (other Cults infantry cannot), mirroring the rule that only Recruits build.
- **BaseResearchTime**: each Cults technology's research time is defined as the time with exactly 1 Recruit inside.
- **Linear scaling**: effective research time scales down linearly with the number of Recruits inside — 2 Recruits halve it, 3 reduce it to a third, and so on (limited only by the building's staffing capacity). Same rule as Cults BaseConstructionTime Scaling.
- **Not consumed**: staffing Recruits are not used up by research.
- **Pauses when empty**: if no Recruits are inside, the in-progress technology pauses at its current progress (no decay) until Recruits return. Queued research waits likewise.
- Cost is still deducted on queue (shared rule 2); cancel and destruction refund in full (shared rules 6–7).
