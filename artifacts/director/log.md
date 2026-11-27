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
- **M2 complete** at `3d98e29`: J2's final QA merged, with 926 tests, a 24-step walkthrough to VICTORY, clean perf, and `docs/playtest/m2.md`. Next: the M1/M2 playtest with the user, and the M3 planning spike.


## 2026-11-24 to 2026-11-25: M3 complete; M4 Syndicate through wave 6 (catch-up entry)
- **M3 Colonists complete** at `a777ae3` (M3-L5), except 0203, which waits for the Rocketman's MinRange value. See plan.md > Built and `m3-plan.md`.
- **M4 planned** (`m4-plan.md`, revision 2). The user asked for organized, modular code (0282; "Modules now, data files at playtest"). M4-R1 and M4-R2 reorganized the sim and the client into per-object modules and tuning modules, with no golden moved.
- **M4 slices merged:**
  - A `fcd06dc`, D `edc30e11` (Fable), C `f67df212` (Fable), B `35e19634`, I `9aca8972`;
  - R1 `8eab2290`, R2 `c60b8d0f`, J2 `c5910e4d`, O `356d1a96`, E `4933e094`, H `f456b0ca`;
  - J1 `dd14dc38`, which with H is **the Syndicate's first playable**;
  - F `2cbb777e` (wave 5 complete), I2 `4f31587f`, O2 `2e0f31f4`.

  The count is 1685 tests at O2.
- **Running:** K (prerequisites, the Optics Lab, the Chemical Plant), G (Tunnel upgrades) and L (the Syndicate oracle and goldens, started early). S1's brief is written and launches after K.
- **The Agents' jam (Q0076).** The user asked for costs (0286, 0287). A Fable investigation found:
  - two Agents each want the spot the other stands on, so it isn't head-on;
  - infantry sidestep already exists in the code;
  - only pass-through reaches 12.5 SC/s at 0–2 GU.

  This went to the Designer as Q0088, with the costs per option. L reports the 0–2 GU rows as blocked until the answer.
- **Questions filed this stretch:** up to Q0088 (the next is Q0089). Design changes were triaged up to 0293.
- **Lessons:**
  - `cd` into a worktree without a subshell moves the session's cwd. It happened three times; use `( cd $WT && ... )` or `git -C`.
  - The plan helper `setcell` must restore the trailing ` |` when it replaces a row's last cell.

## 2026-11-25 (continued): M4 complete; M5 planned and started
- **Merged:**
  - M4-J3, the Tunnel Area display (`5e2e9419`);
  - M4-S1, the upright Laser Turret (`50f51141`);
  - M4-V, one per-viewer event filter (`dbefb552`);
  - M4-S2, the pop-up turret, on Fable (`0cee8bc4`);
  - M4-M, QA (`4e3fdd4a`). **M4 (the Syndicate) is complete.**
- **Follow-ups from M4's QA:**
  - **M4-W:** four Agents at one crystal hatch gridlock for good. Fable is debugging it.
  - **M4-Z:** strays in marks and test helpers.
- **Questions:**
  - 0303 answered Q0089 as built.
  - Q0090 (perception of unseen events) and Q0091 (the pop-up turret's details) are filed as built.
- **M5:**
  - Planning spike M5-P drafted the plan, adopted as `artifacts/director/m5-plan.md`, and Q0092–Q0108 are filed from its drafts.
  - **The user decided** that the first playtest is hot-seat, from the command line, with no AI and no start screen.
  - Triaged:
    - 0302 (Militant) and 0304 (the Gunner TailDisjointed: a new slice, O3);
    - 0305–0308 (Disciple, and its cloud concept: gated N4).
  - Wave 1: M5-A is running. R waits for Z, to keep three builds at most.

## 2026-11-26 to 2026-11-27: M5's content complete; computer players; M6 planned
- **Merged** (the plan lists each with its details):
  - **M5 content:**
    - E, T, F2, J1, F3, Y, J2, Z1, L, G2, U1, N8, N2, N1a, Z2, N9, S1, N1b, M1, N4a, N3, S2a, S2b, N4b, Z3, N5, S3, S4, H1, P3, S5, S6 and H2;
    - V1, N7, MM, V2, S7 (from the user's first playtest), K, N6, MM2, V3, S8, V4, N10, Ma and N11 (`30a50ae5`), **the last content slice**.
  - **Computer players** (the user asked for them for the next playtest): A1 (the framework and the GDO), A2a–c (the other factions) and A2d (the rush profile, and economies that spend their income).
  - Tests went from about 2300 to 2747, with 23 goldens.
- **Running:** M5-X1, the sim's ruleset, alone. Then X2 (data files) and Mb (the QA); M6 starts at X1's merge with GK and SK, whose briefs are drafted.
- **The user** (2026-11-26): "I might not be able to playtest for the next 24 hours so just continue with the milestones work", and "Don't forget to commit and push frequently!" M6's plan (`m6-plan.md`, from the M6-P spike) is adopted.
- **Questions filed:** Q0109–Q0180. Open: Q0176 (turning at rest), Q0177 (the default rally holds three), Q0178 (crushing while stopping; the alert), Q0179 (the Hand of Heaven's back-off and hull aim) and Q0180 (Inspire's faster attack). Changes triaged up to 0447.
- **Process:**
  - M5's M was split into Ma (the generated playtest docs, beside the content) and Mb (the QA, after X2).
  - Branches with a "(wip)" or "(in progress)" commit, or with merges that carry code, go in as one attributed squash (K, N10, N11).
  - The computer players' 192-row `ai_games` table (since A2d) is the reference every slice is compared with.

## 2026-11-27 (continued): M6 under way
- **Merged:** GK, SK, SV, GD, GR, GF, SD, GH1, then GA (`df49b7f9`), SN (`c5469988`) and CM (`f4651a44`). M5's X1, X2 and Mb landed before them, and with Mb the playtest build. There are 31 goldens, and the 192 computer-player rows are unchanged through every merge.
- **Running:** S6a (the answer sweep: 0453, 0454, 0460, 0461 and Z4), GH2 (landing and rearming, with 0458's burst), and SB (the Vault and the Sludge).
- **Every M6 brief is drafted,** GS2, SR, SA, SC and MQ included. GS split into GS1 (the supply loop) and GS2 (the Chopper's transport).
- **Design changes triaged to 0461.** 0457–0461 answered Q0182–Q0184 and Q0186, and went into the unlaunched briefs. Q0187 (the Null Chamber's buff) and Q0188 (Mirage's buffs and Shift) were filed.
- **Process:**
  - A hash-tag registry, `artifacts/director/hash-tags.md`, assigns each parallel slice its tags in every namespace.
  - Counts that both sides change the same way merge silently wrong: the merger sets them by hand (an insight, and a line in m6-common).
  - ST1 waits for GH2, so that the per-object domain check isn't built three times at once (GH2, ST1 and S6a's 0461).
