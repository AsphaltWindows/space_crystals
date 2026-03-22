# Designer Insights

## Table of Contents — Design Documents

| File | Summary |
|------|---------|
| `camera.md` | Camera system design |
| `combat.md` | Attack attributes, attack phases, damage calculation, locomotion/orientation constraints |
| `control_system.md` | ControlState, Selection, ObjectInterfaceState, CommandPanel (ownership guard), SelectionPanel, PointerDisplayTypes, unit commands, base/turret behaviors |
| `designer_notes.md` | Designer notes |
| `design_questions.md` | Open design questions |
| `entities.md` | Entity hierarchy (tiles, objects, structures, resources), fog of war, placement validation |
| `factions.md` | All 4 factions (GDO, Syndicate, Cults, Colonists) and their resources. GDO Power penalty covers buildings and Extraction Plates. |
| `gdo_objects.md` | GDO units/structures: Peacekeeper, PowerPlant, Barracks, DeploymentCenter (now builds ExtractionFacility), ExtractionFacility, ExtractionPlate (Power -3), SupplyTower, SupplyChopper |
| `scale.md` | Simulation frame rate (16 FPS), grid units, space units (64 per grid unit) |
| `cults_objects.md` | Cults buildings/mechanics: Recruitment Center, Storage, Armory (trains Recruits into Soldiers/Gunners), Recruit building mechanics (consumption, cancellation, assist construction) |
| `syndicate_objects.md` | Syndicate units/structures: Tunnel Network, Tunnel, Headquarters, Guard, Agent |
| `to_be_converted.md` | Content awaiting conversion to design doc format |
| `lore.md` | Background lore: Space Crystals origin/properties, faction backstories (GDO, Syndicate, Cults, Colonists) |
| `units.md` | Unit base types, movement models, turret attributes, collision |

## Urgent Forum Questions

(none)

## Pending Design Review

(none)

## Loose Ends

- **Cults Armory open questions**: Training queue (one at a time vs queueable?), cancel mid-training behavior (Recruit back? Crystals refunded? Both?), parallel training or serial only. All TBD values (costs, times, HP/armor) still need numbers.
- **Cults unit definitions needed**: Soldier and Gunner (trained at Armory) need full unit definitions — base type, movement, attack, stats. Recruit also needs a formal unit definition (light infantry base, mining behavior, build behavior, enter-armory behavior).
- **Cults faction design**: Additional structures beyond RecruitmentCenter, Storage, Armory still to be explored.

## Insights

- When batching many feature_requests, note overlap with already-completed pipeline work (check `messages/task_splitter/feature_request/done/`) and flag it in the message content so downstream agents don't duplicate effort.
- The gdo_objects.md SupplyChopper truncation has been resolved. Key decisions: drop-off at non-attached towers is touch-and-go; PickUpSupplies and AttachToTower gated by not-carrying-units; DropOffSupplies gated by carrying supplies; automated scheduled departures don't break attachment.
- Full design doc coverage achieved: 26 feature requests sent covering scale, camera, tiles, fog of war, resources, factions, unit bases, movement, collision, combat, locomotion constraints, control system, commands, behaviors, turret system, auto-targeting, action channels, command indicators, all GDO objects, all Syndicate objects, and framework systems.
- Design gaps from `to_be_converted.md` can still surface (e.g., EF was always intended as a DC construct but was missed in formalization). Cross-check `to_be_converted.md` when forum topics flag missing features.
