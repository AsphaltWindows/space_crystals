# Cathedral

Tier 3 and last building of the Mystical/Human Enhancement branch. Trains Recruits into Disciples, the Cults' spell caster, and unlocks the second level of the Temple's Augmentation improvement lines (Enhanced Constitution Level 2 and Improved Metabolism Level 2 require owning a Cathedral).

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Temple

Size, armor and other stats: see Placeholder stats below. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Training
Works like the Armory's Training: Recruits enter the Cathedral and join an internal pool; training orders are queueable and serial; queuing an order deducts its Space Crystals cost and reserves one Recruit; when the order finishes, the Recruit is consumed and the trained unit exits to the RallyPoint. The trained unit inherits the Recruit's AugmentationState (see Augmentation > Persistence).

Available unit types:
- **Disciple** (consumes 1 Recruit; SpaceCrystalsCost: 75, TrainingTime: 320 frames; placeholder)

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Size 2x3; SymmetryType ABAC; EntranceSide B, ExitSide C (like the Armory); MaxHP 400; PointArmor 1, FullArmor 6; SightRange 4 grid units; Groupable **false** (user, 2026-11-26; Q0105: Cults production buildings aren't groupable).
- Cost and time: Recruit > Constructs (250 SC, 640 frames).
- InternalRecruitCapacity 10, counting reserved Recruits; so at most 10 orders queued (bounded by capacity: reserved units count toward it; Q0104).
- **Disciple**: 1 Recruit, 75 Space Crystals, 320 frames (20 seconds).
- Panel like the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits, Release and Eject All): **Q: Train Disciple**, **A: Release** in the kind menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**.
