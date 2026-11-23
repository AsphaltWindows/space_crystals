# cults_recruit_outside_construction

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Reworked how Cults Recruits construct buildings, and added a matching cancel-construction option for the Syndicate Agent's initial Tunnel build. Two design docs changed:

## artifacts/designer/design/cults_objects.md — "Cults Building Mechanics" section

**Old behavior**: Recruits assigned to a build entered the structure and were consumed on completion; if cancelled, all assigned Recruits were returned; Recruits could not be selectively pulled off a build in progress.

**New behavior**:
- Recruits build from OUTSIDE the structure — they are never consumed and are not "inside" the building.
- Recruits are vulnerable to attack while constructing (same exposure as the structure itself, which follows the existing ConstructionHP Rule).
- Build speed scales proportionally with the number of Recruits currently assigned to the site.
- Recruits can be freely reassigned to other orders at any time mid-construction. Removing a Recruit immediately reduces the assigned count and slows progress proportionally.
- If all Recruits are pulled off a site, progress stalls at its current value (no decay/regression) until Recruits resume or the building is cancelled.
- On completion, all participating Recruits remain on the field as ordinary, free Recruits — available for new orders immediately.
- Assist Construction remains a separate command from Build/Construct (Build starts a new structure; Assist Construction joins an existing in-progress one). Both now describe Recruits working from outside.
- NEW: **Cancel Construction** command (X key), available only while a Cults building is under construction. Refunds 75% of the building's Space Crystals cost (rounded down). Destroys the partially-built structure with a cosmetic-only rubble animation (no gameplay effect). Any Recruits on-site are unaffected by the cancellation — they simply go idle and remain available for orders.

This affects the Recruitment Center, Storage, and Armory (all three currently use the ConstructionHP Rule and are built via this Recruit mechanic).

## artifacts/designer/design/syndicate_objects.md — Tunnel "Tunnel Construction Flow" section

Added a new outcome for the Agent-built initial Tunnel construction (previously only had "completes" and "destroyed" outcomes):
- NEW: Player can now cancel an in-progress initial Tunnel construction. The Agent survives and emerges at the location (identical to the existing destruction-outcome behavior). A cosmetic rubble animation plays. Unlike destruction, the player receives a 100% refund of the Supplies cost (destruction still results in full loss, unchanged).
- NEW: **Cancel Construction** command (X key) added to the Tunnel's ObjectInterfaceState, available only during initial construction (Agent inside, not yet operational). This is mutually exclusive with the existing **Cancel Upgrade** command (also X key), since a Tunnel is never simultaneously under initial construction and mid-upgrade — the two states don't overlap.

Note: this Tunnel refund percentage (100%) is intentionally different from the Cults building refund (75%) — this was a deliberate design decision, not an inconsistency.

## QA Instructions

**Cults Recruit construction:**
1. Select one or more Recruits, issue a Construct command for a Recruitment Center/Storage/Armory, and place it. Confirm the Recruits walk to the site and remain visibly outside it (not disappearing/entering) while a partially-built structure appears and gains HP over time.
2. Confirm the partially-built structure can be damaged/destroyed by enemy attacks during construction (per the existing ConstructionHP Rule).
3. With multiple Recruits assigned, confirm construction completes faster than with a single Recruit (proportional speed-up).
4. Mid-construction, select one assigned Recruit and give it a different order (e.g., move elsewhere). Confirm: that Recruit leaves, remaining Recruits keep building, and overall progress speed drops accordingly (not to zero unless it was the last one).
5. Pull all Recruits off a site mid-construction. Confirm construction progress freezes (does not decay or regress) until a Recruit resumes work.
6. Let a construction complete. Confirm the Recruits involved remain as normal, orderable Recruits (not consumed/removed).
7. Start a construction, then issue Cancel Construction (X). Confirm: 75% of the Space Crystals cost is refunded (rounded down), the partially-built structure is destroyed with a rubble animation, and any Recruits that were building it are left idle and orderable (not returned/respawned — they were never removed from the field).

**Syndicate Tunnel construction cancel:**
1. Order an Agent to build a Tunnel. While it's under construction (Agent inside, untargetable), select the partially-built Tunnel and issue Cancel Construction (X).
2. Confirm: the Agent emerges and survives (same visual/behavior as when a partially-built Tunnel is destroyed by an enemy), a rubble animation plays, and 100% of the Supplies cost spent is refunded to the player.
3. Confirm this new Cancel Construction option is NOT available on an already-operational Tunnel (where only Cancel Upgrade should appear, and only while an upgrade is in progress).
