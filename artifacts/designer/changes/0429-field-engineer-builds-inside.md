# 0429 — The Field Engineer builds inside the site, alone (like an Agent); 75% cancel refund; Ungroupable

- **Date**: 2026-11-26T19:15:00-05:00
- **Answers**: Q0163
- **Docs**: `factions/gdo/units/field_engineer.md > Building` (new), `> Placeholder stats` (Groupable, panel); `factions/gdo/structures/bunker.md`, `first_aid_tent.md > Placeholder stats`; `core/entities.md > Object Type > Groupable`

## What changed and why
User: "More like an agent builds a tunnel. Disappears (goes inside) until the building is complete. No help." Then: it comes back out on completion (a), cancelling refunds 75% (a), and it becomes Ungroupable (a), as the Agent.
- Flow: order (paid then; full refund if it ends before construction starts) → walks to the site → goes inside, untargetable and unseen → site under the **ConstructionHP Rule** (10% start, damage stays) → **completed**: comes out through the Bunker's door (side B) or at the Tent's edge on the side it came from (designer placeholder), idles.
- **Site destroyed**: Engineer comes out alive at the footprint's centre; cost lost.
- **Cancel** (X on both buildings): Engineer comes out at the centre; **75% refund, rounded down** (75 / 56).
- Site does nothing but Cancel; counts as a structure for defeat.
- **One Engineer per site**, no help; **Groupable false**.

## Implementation impact
Replaces placeholder 1 (a) (Recruit-style group building). The Field Engineer's Groupable changes true → false. The Construct menu's Bunker and Tent are no longer "greyed until designed".

## Verification
- Engineer builds a Bunker: it walks there and vanishes; the Bunker rises from 10% HP over 20 s; enemies can't target the Engineer; it walks out the door when done.
- Select two Engineers: two groups; W, Q, place: only the active one goes; Tab, repeat for the second.
- Destroy a site mid-build: the Engineer appears at its centre.
- Cancel a Bunker: 75 SC back, Engineer at the centre.
