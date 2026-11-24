# Director Log

## 2026-11-23 (continued): M0-C, M0-D, review; M0-E and M1-A started
- **M0-C map editor** merged (`a4a0945`). The screenshots were sent to the user.
- **Design changes 0001–0019 triaged.** Q0001–Q0006 are all answered (0002, 0003/0004, 0007, 0008/0009/0012, 0010/0011, 0013). Q0006 (flat tiles for map resources) was filed and then answered with "yes".
- **Review of M0 + M0-C** by a read-only agent: 11 findings, no determinism holes. Findings 1/3/4/6 were folded into M0-D; the rest went to M0-E and M1-A.
- **M0-D corner-height terrain** merged (`cffb46f`, 203 tests). It also covered 0007, 0010, 0013 and the 0012 symmetry transforms. The screenshots were sent to the user.
- **Two agents launched in parallel:** M0-E (map/setup split, map types, generated symmetry, custom presets, editor review fixes) and M1-A (shared input layer, bindings table, command gate, client replay tests).

## 2026-11-23 to 2026-11-24: M1 complete; M2 built (catch-up entry)
- **M1 (units move and fight) completed** at `80b3d88` (M1-H3), after R, G1–G3 and H2. The user was asked to playtest it. See plan.md > Built for the slices.
- **M2 planning.** A read-only spike wrote `m2-plan.md`. Q0022–Q0030 were filed, and design answers 0077–0105 arrived while slices ran. Running agents were re-briefed through SendMessage.
- **M2 slices merged:** A `cbeb804`, E `f76db38` (Fable), C `0f8cc70`, B `39bcf66`, G `69e80a1`, D2 `1a64e75` (Fable), F1 `d5a7d29`, D1 `fda74d3`, H1 `eccaaa6`, F2 `e3f67a9`, I `6ae2b02`, J1 `c2b2a17`, H2 `2d507b4`. **M2 has been playable end to end since `2d507b4`** (923 tests). M2-J2, the final QA, is running alone for clean perf numbers.
- **Director decisions:**
  - T9 split per capability;
  - F split into F1 and F2, J into J1 and J2;
  - recipe timing: an order applied in tick t progresses that tick, and a recipe of N frames is usable from t+N;
  - approved the `dev_skirmish` GDO layout.
- **Questions filed:** Q0020–Q0036. Q0035 covers the rally UI; Q0036 covers game-end edge cases.
- **Lessons** (in the insights):
  - perf numbers and wall-clock tests are unreliable under parallel builds;
  - a shared temp directory caused the replay flake;
  - never start a command with a bare `cd`;
  - the Bevy `Text2d` layout quirk.

