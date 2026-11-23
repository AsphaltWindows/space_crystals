# syndicate_rename_headquarters_to_hideout

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Rename the Syndicate underground Tunnel expansion "Headquarters" to "Hideout". This is a pure rename — no stat, cost, production, or behavior changes.

**artifacts/designer/design/syndicate_objects.md** changes:
- Section `### Headquarters` renamed to `### Hideout`. Description updated: the Hideout is the primary accommodations for Syndicate infantry — a Tier 1 Tunnel expansion that produces Agents and Guards.
- `HeadquartersInstanceState` -> `HideoutInstanceState`; `ObjectInterfaceState[Headquarters]` -> `ObjectInterfaceState[Hideout]`.
- Tunnel section: the Syndicate starts with one Tier 1 Tunnel and one pre-built **Hideout** expansion.

Everything else is unchanged: 2x2, Tier Requirement 1, 200 Space Crystals / 400 frames, 400 HP, 1/4 armor, produces Agent (100 crystals / 160 frames) and Guard (125 crystals / 120 frames), BuildQueue max 20, RallyPoint behavior, Q/W/X/C hotkeys.

Rename every user-visible occurrence (display name, InfoPanel/SelectionPanel text, ExpandMenu entry, tooltips) and the code identifiers (enum variants, types, components, tests) so code matches the design doc terminology. Note: several pending feature_requests previously referenced "Headquarters"; they have been edited in place to say Hideout.

## QA Instructions

1. Start a game as the Syndicate. Confirm the starting Tunnel contains a pre-built expansion named **Hideout** (no "Headquarters" anywhere).
2. Select the Hideout. Confirm the InfoPanel/SelectionPanel shows the name "Hideout", and the command panel still shows Q: Build Agent, W: Build Guard, X: Cancel Production, C: Set Rally Point, all working exactly as before (same costs: Agent 100, Guard 125; same build times).
3. Select the Tunnel, open B: Expand Tunnel. Confirm the expansion is listed as "Hideout" and can be placed and built (200 Space Crystals, 25 seconds) as before.
4. Confirm starting Tunnel Space is still 11 (Hideout still occupies 2x2).
5. Search all in-game text/tooltips you encounter during the session: the word "Headquarters" should not appear anywhere.
