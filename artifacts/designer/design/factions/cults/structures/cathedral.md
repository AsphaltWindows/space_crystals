# Cathedral

Tier 3 and last building of the Mystical/Human Enhancement branch. Trains Recruits into Acolytes, the Cults' spell caster, and unlocks the second level of the Temple's Augmentation improvement lines (Enhanced Constitution Level 2 and Improved Metabolism Level 2 require owning a Cathedral).

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Temple

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, entrance/exit sides, internal Recruit capacity, training queue size, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Training
Works like the Armory's Training: Recruits enter the Cathedral and join an internal pool; training orders are queueable and serial; queuing an order deducts its Space Crystals cost and reserves one Recruit; when the order finishes, the Recruit is consumed and the trained unit exits to the RallyPoint. The trained unit inherits the Recruit's AugmentationState (see Augmentation > Persistence).

Available unit types:
- **Acolyte** (consumes 1 Recruit; SpaceCrystalsCost: TBD, TrainingTime: TBD)
