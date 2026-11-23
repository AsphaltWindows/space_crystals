# add_cults_armory_r1

## Metadata
- **From**: automatic_qa
- **To**: qa_router

## Content

**Note from automatic_qa**: This item CANNOT be verified automatically. ALL QA steps require interactive gameplay (starting a game as Cults faction, placing buildings, training units, verifying in-game behavior). No automated tests cover these scenarios. Returning for manual QA.

## Content

The Cults Armory feature cannot be tested because the Cults faction has no starting buildings or units. While Cults is selectable as a faction, the game starts with nothing on the map, making it impossible to place or interact with any Cults buildings.

**Blocked**: All 9 QA steps — cannot place an Armory, cannot produce Recruits, cannot test any Armory functionality.

**Prerequisite**: Cults faction needs a functional starting condition (starting buildings and units) before any Cults-specific features can be QA'd.

## QA Instructions

1. Start a game as Cults faction — verify starting buildings and units are present.
2. Place a Cults Armory building (3x2 footprint) and verify all command panel actions work: Train Soldier (Q), Train Gunner (W), Eject All (E), Set Rally Point (C).
3. Verify Recruits can enter the Armory, training consumes Recruits + Space Crystals, and trained units exit toward the rally point.
