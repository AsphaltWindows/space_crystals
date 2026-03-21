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

- **Pointer display types**: RESOLVED. 8 types designed (Inactive, Move, Attack, AttackGround, Patrol, GatherResources, ReturnResources, Enter) with full resolution rules for DefaultState, AwaitingTarget, and AwaitingPlacement contexts. Feature request sent. Supply Tower ScheduleDeliveries invalid target now shows Inactive pointer.
- Alt-click camera behavior: should it pan smoothly or snap instantly? (from `unit-control-selection-ui`)
- Agent UnitBase is documented as HeavyInfantry but forum topic 2 mentions LightInfantry in the key questions. Design says HeavyInfantry — confirm this is intentional.
- Tunnel transit tier requirements: design says "Tier 1+: Infantry (Heavy Infantry)" but the parenthetical only mentions Heavy Infantry, not Light Infantry. The BasicCombatUnitInterfaceState right-click resolution for Enter says "Syndicate units only" — does this include all Syndicate unit bases or just those meeting tier requirements?

## Insights

- When batching many feature_requests, note overlap with already-completed pipeline work (check `messages/task_splitter/feature_request/done/`) and flag it in the message content so downstream agents don't duplicate effort.
- The gdo_objects.md SupplyChopper truncation has been resolved. Key decisions: drop-off at non-attached towers is touch-and-go; PickUpSupplies and AttachToTower gated by not-carrying-units; DropOffSupplies gated by carrying supplies; automated scheduled departures don't break attachment.
- Full design doc coverage achieved: 26 feature requests sent covering scale, camera, tiles, fog of war, resources, factions, unit bases, movement, collision, combat, locomotion constraints, control system, commands, behaviors, turret system, auto-targeting, action channels, command indicators, all GDO objects, all Syndicate objects, and framework systems.
