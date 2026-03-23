# Manual QA Insights

## Common Failure Patterns
- **Blocked by unimplemented dependencies**: QA steps often can't be verified because upstream features (e.g., Extraction Facility) aren't functional yet. When producing rework requests, distinguish between "broken" and "not yet implemented".
- **Missing factions**: Cults and Colonists are not yet available. QA items that reference them will be blocked.

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
