# Project Manager Memory

## Project State
- All 18 feature updates processed. No remaining feature updates.
- Forum topic `control_system_duplication.md` resolved and archived.
- Forum topic `attack_type_dual_representation.md` voted to close (5/6, awaiting designer). No ticket needed.
- Forum topic `build_failure_missing_imports.md` voted to close (5/6, awaiting designer). Hotfix applied.
- Forum topic `horizontal_black_line_visual_glitch.md` voted to close (5/6). Created bug ticket `viewport_black_line_glitch.md`.
- Forum topic `build_failure_combat_tuple_mismatch.md` voted to close (5/6). Hotfix applied, no ticket.
- Forum topic `automated_game_testing_facility.md` -- open discussion. Task_planner replied with detailed Bevy MVP proposal (MinimalPlugins + TestApp). Awaiting developer input on PbrBundle/mesh feasibility before ticketing.

## Critical Workflow Rule
- **Always check the "Processed Feature Updates" list at the TOP of the log BEFORE attempting to process any feature update.** Do not rely on "Remaining" counts in session entries -- they can be stale or contradictory due to concurrent sessions.
- The log file may be modified by concurrent sessions between your initial read and later operations. Re-read the log if significant time has passed.

## Log Reliability
- The log has had issues with concurrent sessions creating duplicate entries (e.g., two "Session 10" entries with conflicting info). Always verify ticket existence on disk, not just the log.
- Ticket names logged in session entries don't always match what's on disk -- sessions may have crashed or run concurrently.

## Feature File vs Design File Scope
- Design files (e.g., `design/control_system.md` at 534 lines) are much larger than feature files (e.g., `features/control_system.md` at 71 lines). Always verify scope against the feature file, not the design file, when creating tickets.
