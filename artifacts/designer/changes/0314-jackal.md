# 0314 — The buggy is the Jackal: ground and air, 2 Recruits; Chop Shop works like the Armory

- **Date**: 2026-11-26T14:55:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/units/jackal.md` (renamed from `buggy.md`); `factions/cults/structures/chop_shop.md > Production`; `factions/cults/units/README.md`; `README.md`

## What changed and why
User:
- **Name:** the light vehicle (0313) is named **Jackal**.
- **Targets:** it fires at **ground and air** (Universal).
- **Cost in Recruits:** it consumes **2 Recruits**, so its Unit Control cost is 2.
- **Chop Shop production works like the Armory:**
  - Recruits enter an internal pool;
  - orders are serial and queueable;
  - an order deducts its crystals and reserves the vehicle's number of Recruits, and can only be queued if enough are free;
  - the Recruits are consumed on completion;
  - Eject All and Cancel work as at the Armory.

## Implementation impact
Concept only. TBD:
- the Chop Shop's capacity, queue and sides;
- the Jackal's numbers.

## Verification
—
