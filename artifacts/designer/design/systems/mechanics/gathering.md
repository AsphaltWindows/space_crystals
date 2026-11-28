# Mechanic: Gathering

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit; gathers the job rules previously written in the Colonists Drone's doc (and applied to the Syndicate Agent from there); no rule changed in the move. The rules below keep their original wording, written for the Drone; read "Drone" as "worker", "Egress Node or refinery" as "its drop-off", and "crystals" as "the resource".)

Two archetypes collect resources. A gatherer's doc says which it uses, its parameters, and its exceptions.

## Archetype: Worker round trip
A worker **picks up at X and delivers to Y**, repeating the trip: a Gather order sets the pick-up point; the drop-off is chosen by a rule (parameter).
- **Parameters**: CarryCapacity; MiningDuration (and PickUpDuration where it picks up from a building); DropOffDuration; MiningReach and DropOffReach (8 su placeholder for every worker); **which drop-off** (e.g. an assigned one; the nearest to the worker when it finishes mining); **patch sharing** (shared, or one worker per patch); **collision while gathering** (normal, or gathering workers pass through each other); drop-off occupancy (e.g. one at a time at a hatch).
- **Job rules** (below) apply unless the worker's doc says otherwise.

## Job rules
### Last load of a patch
(user, 2026-11-24) A Drone mining a patch with fewer crystals left than its CarryCapacity mines the full 32 frames and takes **what is left**; the patch is then depleted. Same as the GDO plate's last partial load (factions/gdo/economy.md > ExtractionPlate).

### Interrupted mining
(user, 2026-11-24) A Drone ordered away (or otherwise stopped) before its 32 MiningDuration frames finish **collects nothing**, and the patch is **freed immediately** for another Drone.

### Other orders keep the load
(user, 2026-11-24) A carrying Drone given any other order (Move, Stop, …) **keeps its load**. It delivers it at its next drop-off: on its next Gather trip (Gather while carrying a load below) or with a Drop Off order. (Deploying with a load: factions/colonists/economy.md > Beacon Deployment > Deploying with a load.)

### Who sees the load
(user, 2026-11-25; Q0052) A carrying Drone shows a **visual of what it is carrying** (e.g. a crystal load) on its model, seen by **anyone who can see the Drone**. Everything else is **owner-only**: **how much** it holds, **where it is taking it** (its drop-off) and **where it got it** (its source). Other players' InfoPanels for a Drone show none of this, and its command indicators are never shown to them.

### Gather while carrying a load
(user, 2026-11-24) A Drone already carrying a load that is ordered to Gather goes **straight to the new pick-up point**. On arrival it sees it is already loaded, **skips mining** (it doesn't occupy the patch) and returns to deliver as if it had just mined the load there. From then on it follows the normal loop for its new patch.

### Drop Off order
(user, 2026-11-24) An explicit **Drop Off** (right-click an own Egress Node or accepting refinery while carrying) is **only a delivery**, not a Gather: the Drone delivers its load and the order is **complete**. It then starts its next queued order (e.g. a Shift-queued build), or with nothing queued **becomes idle** at the drop-off point; it does not walk back to its patch. (If Drop Off counted as a Gather, an order Shift-queued behind it would wait until the field is mined out.)

**Empty Drones** (user, 2026-11-24): Drop Off is only resolved for a Drone that is carrying something. A right-click on an own Egress Node with an **empty** Drone is a plain **Move to the node** (it walks there and stops). Right-click resolves per Drone (systems/controls/interface.md > RightClickResolution), so in a mixed group the loaded Drones deliver and idle, and the empty ones walk to the node; none goes back to mining on its own.

### When a Gather ends
(user, 2026-11-24) A Gather order normally never ends: the Drone repeats its trip. It **ends on its own** only when the Drone runs out of crystals to mine: its patch is gone and no patch is left within its search radius. It first **delivers any load** it carries, then becomes idle (or starts its next queued order).

**Orders Shift-queued behind a Gather** wait until the Gather has ended this way, e.g. "mine this field out, then move to X". A Drone waiting at an occupied patch is still gathering. A Shift-queued Deploy behind a Gather therefore means "deploy once the field is mined out"; the user expects players rarely to do this. (Rejected: starting the queued order after the next drop-off, or after the current mining/delivery half of the trip, which could send the Drone off holding a load.)

Consequence: one patch yields at most 8 crystals per 32 frames (4 Space Crystals per second) from Drones. A patch saturates at roughly (round-trip time ÷ 2 seconds) Drones, since each Drone occupies it for 2 of its trip's seconds: about 1 Drone for a patch adjacent to the drop-off, about 2 at 2 grid units, about 2.5 at 3 grid units (ignoring acceleration; slightly more once Drone acceleration is accounted for). Example: 8 patches in a row with the Egress Node 2 grid units from the row (edge to edge, centred on the long side; factions/gdo/economy.md > Economy Targets (GDO vs Colonists)) saturate at about **24 working Drones** (3 per patch, 32 Space Crystals per second; measured, 2026-11-25, Q0072). **2 per patch (16 Drones, ≈25 per second) is the efficient point.** Measured quirk, accepted for now: Drones 17–20 add almost nothing (≈26 at 20), and 21–24 add the rest, because third Drones mostly wait their turn until every patch has one.


## Archetype: Stationary extractor
A structure placed on a resource that extracts it by itself at a rate (e.g. GDO Extraction Plates; factions/gdo/economy.md). Its last load of a patch is partial (as a worker's, above).

## Used by (index only)
Worker round trip: Colonists Drone (all job rules; one Drone per patch; pass-through; assigned drop-off), Syndicate Agent (job rules except patch occupancy and Set Drop-off; shared patches; nearest Tunnel hatch), Cults Recruit (shared patches; normal collision; nearest Storage; **whether the job rules apply to Recruits is not stated in its doc: open**). Stationary extractor: GDO Extraction Plate.
