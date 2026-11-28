# 0480 — Mechanics doc: Construction

- **Date**: 2026-11-28T13:25:00-05:00
- **Answers**: Q0193 (migration, part 8)
- **Docs**:
  - `systems/mechanics/construction.md` (new)
  - `README.md > systems/mechanics/`
  - `factions/cults/economy.md > Cults Building Mechanics` (intro line)
  - `factions/gdo/units/field_engineer.md > Building`
  - `factions/syndicate/units/agent.md > Building`

## What changed and why
The eighth mechanics doc.

**Common rules:**
- paid when the order is given, refunded in full before the start;
- a site does nothing but Cancel, counts for the Defeat Condition and follows the ConstructionHP Rule;
- the cancel refund is a parameter;
- builders go on to queued orders.

**Archetypes:**
- **Builders outside:** the Cults Recruits' rules, generalised to "builders":
  - builder-frames that scale linearly, and Assist;
  - stalling when builders are pulled off;
  - where builders work;
  - start and blocked-site rules;
  - group refunds;
  - Cancel.
- **Builder inside:** the Agent and the Field Engineer:
  - one builder, inside, untargetable;
  - comes out at the centre when the site is destroyed or cancelled;
  - where it goes on completion is a parameter.
- **Placed by a constructor:** the GDO Deployment Center and the Syndicate menus.

Placement validation and ConstructionHP stay in core/entities.md. Transformations stay in their own docs: the Beacon, Warp In, and the Prospector's deploy.

**No rule changed.** The Cults section keeps its wording and rationale as the Cults' specifics, under a Uses line.

## Implementation impact
One construction module with these archetypes. No behaviour changes.

## Verification
No gameplay change.
