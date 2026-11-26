# 0381 — Cults economy check: the realistic opening should overtake by ~4:00; measure out to 6:00

- **Date**: 2026-11-27T10:35:00-05:00
- **Answers**: Q0135 (with 0380)
- **Docs**: `factions/cults/economy.md > Economy check` (scenario 2)

## What changed and why
User: "The bigger storage should help. But also let's go with B and see how economies compare on a longer time line".
- **Ideal-mining misses:** answered by the 3x2 Storage (0380); re-measure with it.
- **Realistic opening target (option b):** expanding early is an investment. It may fall behind early, but should **overtake ideal mining by about 4:00**.
- **Longer timeline:** measure both Cults scenarios, the GDO and the Colonists at **1:30, 2:48, 4:00, 5:00 and 6:00**, and report the comparison. The user will set further targets from it.

## Implementation impact
- Re-run the oracle with the 3x2 Storage and the new drop-off choice (0375, nearest to the Recruit).
- Add the 4:00–6:00 checkpoints for all four economies, and pin the results.
- Don't tune Cults numbers yet: report first, including whether ideal mining still lands at ~0.9 by 2:48. A new question with the table is the expected answer.

## Verification
- The oracle reports the five checkpoints for each of the four lines.
- It flags whether the realistic opening has overtaken ideal mining by 4:00.
