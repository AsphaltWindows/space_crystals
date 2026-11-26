# 0370 — Doc reorganisation: four long files split (no design change)

- **Date**: 2026-11-27T03:40:00-05:00
- **Answers**: —
- **Docs**: see the mapping below; `README.md` (index); `factions/cults/structures/README.md > Files`. Cross-references updated in about 25 files.

## What changed and why
The user asked for the longest files to be split. **No rule, number or wording changed**: every original line is in exactly one new file. A script checked this. Only file titles changed, plus one "See also" line per file, and moved sub-sections are one heading level higher.

| Old location | New file |
|---|---|
| `systems/controls/selection_and_hud.md` > ControlState, Selection, SelectionGroup, BoxSelection, MapClickSelection, ControlGroups, GroupCycling | `systems/controls/selection.md` |
| … > Key Rebinding, ObjectInterfaceState, the transitions, CommandPanel (incl. rally points), DefaultState, RightClickResolution, AwaitingTarget, CursorTarget, BasicCombatUnitInterfaceState | `systems/controls/interface.md` |
| … > HUD Bar, SelectionPanel, PointerDisplayType, CommandIndicators | `systems/controls/hud.md` |
| `systems/controls/behaviors.md` > Core principle … PathKnowledge | stays in `systems/controls/behaviors.md` |
| … > MovingToLocation … StoppingBehavior | `systems/controls/command_behaviors.md` |
| … > BaseActionChannels … TurretInactive | `systems/controls/action_channels.md` |
| `factions/syndicate/tunnels.md` > Tunnel Network; Tunnel (stats, starting conditions, tiers); Construction and Upgrade Rules; Cost Scaling | stays in `factions/syndicate/tunnels.md` |
| … > Tunnel > Tunnel ObjectInterfaceState (Eject and Expand menus, expansion placement, expansions under construction) | `factions/syndicate/tunnel_menus.md` |
| … > Tunnel > Tunnel Area (display, Non-Overlap Rule), Tunnel Space | `factions/syndicate/tunnel_area_and_space.md` |
| `factions/cults/economy.md` > the RecruitmentHall section | `factions/cults/structures/recruitment_hall.md` |
| `factions/cults/economy.md` > Storage | `factions/cults/structures/storage.md` |

`selection_and_hud.md` no longer exists.

**Fix found along the way:** the Recruitment Hall's `## RecruitmentHall` heading had been deleted by accident in the 0346 edit. Its section had since read as part of "Economy check". In the new file it again has its own title. Its content is unchanged.

## Implementation impact
- None in behaviour.
- If code comments or the Director's notes cite the old paths or sections, update them using the table above.
- New section paths:
  - `systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange`
  - `systems/controls/interface.md > CommandPanel > Rally points on enemy objects`
  - `factions/syndicate/tunnel_area_and_space.md > Tunnel Space > Production and Tunnel Space`

## Verification
—
