# gdo_rename_powerplant_to_generator

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

GDO's PowerPlant entity has been renamed to Generator in artifacts/designer/design/gdo_objects.md. This is a pure rename — no stat changes (still Size 2x2, MaxHP 350, PointArmor 1, FullArmor 4, SightRange 3, BuildRadiusExtension 1, Power +20). All references were updated: the entity's own section header, DeploymentCenter's Constructs list entry, DeploymentCenter's BuildMenu command entry (CurrentConstruction = Generator), and the enum name used throughout.

This rename is part of a larger GDO tech tree expansion session (10 new buildings were stubbed into gdo_objects.md: Arsenal, Career Campus, Factory, Radar Station, Advanced Mechanics, Airfield, Industrial Powerplant, Propulsion Labs, Defense System, Missile Turret). Those 10 are description + Faction + TechPrerequisite only — no stat blocks (HP, size, cost, ObjectInterfaceState) yet, so they are NOT actionable for implementation in this request. Do not create entities for them yet; a future feature_request will follow once each is fully stated.

One important note on sequencing: SupplyTower's design-doc prerequisite has been changed from "player owns at least one Generator" to "player owns at least one Radar Station" (per the new tech tree). However, Radar Station has no stats yet and cannot be implemented. DO NOT change SupplyTower's build-gating logic in code as part of this request — leave it functionally gated on Generator (previously PowerPlant, unchanged behavior, just following the rename) until a future feature_request introduces a fully-stated Radar Station and explicitly asks for the gating swap.

## QA Instructions

1. Search the codebase for "PowerPlant" and confirm every reference (component/entity type, build menu button, enum variants, tests, UI strings) has been renamed to "Generator". The game should build and run with no leftover "PowerPlant" naming anywhere.
2. In-game: select the Deployment Center, open the build menu, and confirm the button now reads "Generator" (not "Power Plant"). Building it should behave exactly as the old Power Plant did (same cost 150 Space Crystals, same build time 10 seconds, same +20 Power once placed).
3. Confirm the Supply Tower still requires owning a Generator to build (same gating as before under its old PowerPlant name) — this request does not change that gating to Radar Station yet, since Radar Station doesn't exist as an entity.
4. Confirm no other GDO building behavior changed — Barracks, Extraction Facility, Extraction Plate, Deployment Center, Supply Tower, Supply Chopper should all function identically to before this change, just with the renamed Generator reference where applicable.
