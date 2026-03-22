# add_cults_armory

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# add_cults_armory

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added the Armory building to the Cults faction in `artifacts/designer/design/cults_objects.md`.

The Armory is a 3x2 training building (ABCB symmetry) where Recruits enter through an entrance side and trained units exit from the opposite side. Key design:

- **Internal Recruit pool**: Holds up to 10 Recruits
- **Training**: Consumes one internal Recruit + Space Crystals to produce a Soldier or Gunner
- **Eject All**: Sends all stored Recruits back out the exit side unchanged
- **ObjectInterfaceState**: Rally point (C), Eject All (E), Train Soldier (Q), Train Gunner (W)
- **ConstructionHP Rule applies**

Note: Several values are TBD (costs, training times, HP/armor) and some mechanics are still open (queue vs one-at-a-time, cancel behavior, parallel training). These will be resolved in a follow-up session. Downstream agents should implement the structural framework but can leave TBD values as placeholder constants.

## QA Instructions

1. Place a Cults Armory building on the map (3x2 footprint)
2. Verify it renders with distinct entrance (A side) and exit (C side) on the short ends, with matching long sides (B)
3. Select Recruits and order them to enter the Armory — they should walk to the entrance side and enter, incrementing the internal StoredRecruits count (visible in info panel), up to a max of 10
4. Select the Armory and verify the command panel shows: Train Soldier (Q), Train Gunner (W), Eject All (E), Set Rally Point (C)
5. With Recruits stored, press Q to train a Soldier — verify a Recruit is consumed from the pool, Space Crystals are deducted, and after the training period a Soldier exits from the exit side toward the rally point
6. Repeat with W for Gunner
7. Press E to Eject All — verify stored Recruits exit one at a time in rapid succession from the exit side as normal Recruits
8. Verify Train commands are greyed out when StoredRecruits is empty or Space Crystals are insufficient
9. Verify Eject All is greyed out when StoredRecruits is empty
