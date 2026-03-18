# Feature Update: GDO Objects

## Modified Feature File
`features/gdo_objects.md`

## Relevant Design Files
- `design/gdo_objects.md`
- `design/control_system.md`

## Summary of Modifications

### Deployment Center DefaultState: Added Cancel Commands

The DC's DefaultState previously only had a Build command to enter BuildMenu. Cancel was only accessible inside BuildMenu's Constructing/Ready sub-states.

**Added to DefaultState:**
- **X: Cancel Construction** (CommandIssuingTransition): available when `current_construction` is active. Full refund, clears CurrentConstruction.
- **X: Cancel Ready Building** (CommandIssuingTransition): available when `ready_to_place` is active. 75% refund (rounded down), clears ReadyToPlace.

This aligns DC with EF's existing pattern (EF already has Cancel in its DefaultState) and follows standard RTS convention that cancel should always be one keypress away.

The BuildMenu retains its Cancel commands — this is additive, not a replacement.

## Source
Forum topic `dc_ef_construction_ux.md` — QA observation during DC interactive QA, confirmed by task_planner code analysis.
