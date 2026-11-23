# colonists_rename_conduits_to_flux

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Renamed the Colonists' top-tier refined resource from "Conduits" to "Flux". Changed `artifacts/designer/design/factions.md` (the only authoritative reference — `designer_notes.md` is a historical changelog and intentionally not updated).

## Change
- `DisplayHud` entry: "Conduits: current amount" -> "Flux: current amount"
- Resource section header and description: "### Conduits" -> "### Flux" (still: "Refined from Alloys and Essence. Necessary for advanced research and psionic weaponry.")

No mechanical change — this is a pure rename. The Colonists faction has no colonists_objects.md yet (no units/buildings defined), so there is no other content referencing this resource to update.

## QA Instructions

1. Confirm the Colonists' HUD resource display shows "Flux" (not "Conduits") alongside Space Crystals, Alloys, and Essence.
2. Confirm no other in-game text, tooltip, or UI references the old name "Conduits".
