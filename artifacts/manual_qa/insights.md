# Manual QA Insights

## Common Failure Patterns
- **Blocked by unimplemented dependencies**: QA steps often can't be verified because upstream features (e.g., Extraction Facility) aren't functional yet. When producing rework requests, distinguish between "broken" and "not yet implemented".
- **Missing factions**: Cults is selectable but has no starting buildings/units — effectively non-functional. Colonists not available. GDO and Syndicate are playable. QA items that reference Cults/Colonists will be blocked.
- **Attack commands broken**: Right-clicking enemy units/buildings causes the unit to stop in place instead of pursuing and attacking. This cascades to rally point attacks and any feature depending on attack commands. Rework filed (unit_command_system_r1).
- **Command hotkeys not functional**: None of the command hotkeys (Q/W/E/A/S/D) respond for unit commands. Blocks testing HoldPosition, Stop, Patrol, Reverse, AttackGround, AttackMove.
- **Fog of war selection bug**: Objects in explored-but-not-visible areas are selectable and show full info. Forum topic filed. Affects resource nodes and likely all object types.

## QA Environment Notes
- **Build script issue**: `scripts/build_qa_artifact.sh` requires `diagnostics` feature in Cargo.toml — forum topic filed, fix in progress. Workaround: build with `cargo build --no-default-features` and manually copy binary to `artifacts/manual_qa/qa_artifacts/`.
- **Running the game**: Must run from `artifacts/developer/` directory so Bevy finds assets. Use `scripts/run_qa.sh` or `cd artifacts/developer && ../../artifacts/manual_qa/qa_artifacts/latest/space_crystals`.
- **Camera start position**: Camera doesn't center on starting structures for Syndicate (forum topic filed). May affect other factions too.

## Known Bugs (forum topics filed)
- **Enemies don't attack by default**: Enemy units are passive, don't engage player units in range.
- **Can control enemy units/buildings**: Player can issue commands to enemy objects — ownership check missing.
- **Building selection hitbox too small**: Must click near center of building to select it.
- **Elevation rendering still broken**: Cuboid mesh approach didn't fix the visual issues. Rework sent requesting flat rendering fallback (all tiles at same Y height, preserve ElevationMap data). This blocks most visual QA until resolved.

## Process Notes
- Ask user how to start the game / what's available before diving into steps — saves time on blocked steps.
- When multiple steps are blocked for different reasons, separate them clearly in the rework request (bugs vs not-yet-implemented).
- Only LightInfantry available as a unit type — many unit-related QA items will be heavily blocked.
- User can work around "enemies don't attack" by controlling enemy units themselves (due to ownership bug).
- GDO Barracks production/queue/cancel works well — Peacekeepers can be produced. Barracks hotkeys (Q to build, X to cancel, C for rally) work even though unit command hotkeys don't.
- Build script is now working without workarounds.
- **Tunnel placement broken**: Agent's Build Tunnel (A) command doesn't show ghost placement overlay. Blocks all Syndicate construction/tunnel QA.
- **Right-click Gather broken**: Right-clicking crystal fields with Agents does nothing. Blocks all Syndicate resource gathering QA.
- **Attack-Move doesn't maintain target lock**: Units fire once then resume moving instead of staying on target. Targeted attack (A+click unit, right-click unit) still broken.
- **Many QA items share root causes**: Tunnel placement, targeted attack, and right-click gather failures cascade across many features. Prioritize fixing these three systems to unblock the most QA items.
