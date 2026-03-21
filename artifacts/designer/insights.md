# Designer Insights

## Table of Contents — Design Documents

| File | Summary |
|------|---------|
| `camera.md` | Camera system design |
| `combat.md` | Attack attributes, attack phases, damage calculation, locomotion/orientation constraints |
| `control_system.md` | ControlState, Selection, ObjectInterfaceState, CommandPanel, SelectionPanel, PointerDisplayTypes, unit commands, base/turret behaviors |
| `designer_notes.md` | Designer notes |
| `design_questions.md` | Open design questions |
| `entities.md` | Entity hierarchy (tiles, objects, structures, resources), fog of war, placement validation |
| `factions.md` | All 4 factions (GDO, Syndicate, Cults, Colonists) and their resources |
| `gdo_objects.md` | GDO units/structures: Peacekeeper, PowerPlant, Barracks, DeploymentCenter, ExtractionFacility, ExtractionPlate, SupplyTower, SupplyChopper |
| `scale.md` | Simulation frame rate (16 FPS), grid units, space units (64 per grid unit) |
| `syndicate_objects.md` | Syndicate units/structures: Tunnel Network, Tunnel, Headquarters, Guard, Agent |
| `to_be_converted.md` | Content awaiting conversion to design doc format |
| `units.md` | Unit base types, movement models, turret attributes, collision |

## Urgent Forum Questions

(None)

## Pending Design Review

(All design content has been sent as feature_requests into the pipeline. 26 new feature requests sent this session covering all remaining design docs.)

## Loose Ends

- **Lore**: User wants to do a lore session before diving into Cults faction design. Opening question asked: "What's the setting? What are Space Crystals, and why are these factions fighting over them?" No answer yet — resume here next session.
- **Cults faction design**: Blocked on lore session. After lore, resume with: Recruitable terrain (special tile type or created by Cult action?), how Recruits spawn (passive at Recruitment Centers or active mechanic?). Existing design in factions.md covers identity, resources (Space Crystals, Unit Control), and the territorial scaling concept.

## Insights

- When batching many feature_requests, note overlap with already-completed pipeline work (check `messages/task_splitter/feature_request/done/`) and flag it in the message content so downstream agents don't duplicate effort.
- The gdo_objects.md SupplyChopper truncation has been resolved. Key decisions: drop-off at non-attached towers is touch-and-go; PickUpSupplies and AttachToTower gated by not-carrying-units; DropOffSupplies gated by carrying supplies; automated scheduled departures don't break attachment.
- Full design doc coverage achieved: 26 feature requests sent covering scale, camera, tiles, fog of war, resources, factions, unit bases, movement, collision, combat, locomotion constraints, control system, commands, behaviors, turret system, auto-targeting, action channels, command indicators, all GDO objects, all Syndicate objects, and framework systems.
