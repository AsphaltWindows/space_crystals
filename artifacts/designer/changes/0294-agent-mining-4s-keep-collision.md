# 0294 — Agent mining 4 s; Agents keep collision; reference distance 3 GU; moving-unit sidestep documented

- **Date**: 2026-11-26T11:00:00-05:00
- **Answers**: Q0076, Q0088 (except Q0076's two "also to confirm" points, still with the user)
- **Docs**: `factions/syndicate/units/agent.md > Gathering > Space Crystals` (MiningDuration, table, economy check targets, rationale, Close patches and traffic); `systems/units.md > Ground Collision` (moving units steer around blockers)

## What changed and why
User, after reviewing Q0088's measurements and the geometry:
- **MiningDuration 80 → 64 frames (4 s)**, equal to drop-off.
  - 2 Agents could saturate only with no walking at all (a patch touching the hatch).
  - A patch lasts only about 2 minutes at full intake, so the **third Agent is always needed** in practice.
  - 3 Agents saturate out to about 3 GU, and 4 out to about 6 GU.
  - The ceiling is unchanged at 12.5 SC/s. The table is recomputed.
- **Agents keep collision.** No pass-through: the Director's option (c), with this new mining time.
  - How far a starting Tunnel sits from its crystals is up to the map maker.
  - A little player intervention to get Agents mining well is acceptable (Core principle).
- **Economy check:**
  - the opening target ("3 Agents around 0:40") is now measured with the patch **3 GU** from the hatch (was 1 GU);
  - the table is checked **from 3 GU outward**. Rows 0–2 are theoretical, since collision can cost income there (designer's reading of "leave it to the map makers").
- **Stage 0:** `systems/units.md` now describes the moving-unit sidestep that M1 built. The user wants this behavior in general. Vehicle grazing comes with vehicles. 1a (a wider step) is not adopted.

## Implementation impact
- Agent MiningDuration becomes 64 frames.
- The reference layouts and `dev_skirmish` put the patch 3 GU from the hatch.
- The oracle checks rows ≥ 3 GU at ±15%, and the 12.5 ceiling exactly.
- No movement code changes.
- **Replaces** the 5 s mining rationale ("2 Agents never fully saturate") and 0259's 1 GU opening layout.

## Verification
At 3 GU, 3 Agents reach about 12.5 SC/s; at 4 GU about 11.3; at 6 GU, 4 Agents saturate.
