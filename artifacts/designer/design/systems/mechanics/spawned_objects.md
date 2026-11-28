# Mechanic: Stocks and Spawned Objects

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit; gathers rules previously written in the Drone Operator's, Twister's and Disciple's docs; no rule changed in the move.)

Some units **create other objects** in the field: from a **stock** they carry and refill by purchase, or directly by an ability. A unit's doc says which archetypes it uses, its parameters, and its exceptions.

## Stock and purchase
- **Parameters**: maximum stock; starting stock when the unit is made; purchase cost (paid at the press) and time.
- **Purchase** is an ability of the carrier ("Buy ..."/"Build ..."): it adds one to the stock after its time. **When it can run** is a parameter:
  - **Queues up to the free stock**: presses queue one after another, each paid at its press, up to the room left (e.g. the Twister's Buy Mine, Q0160);
  - **Only once the previous object has ended**: no purchase while one is out, so each use is followed by at least the build time without one (e.g. the Drone Operator's Build Drone, Q0155).
- **Using the stock** is another ability (launch, lay), with its own targeting.

## Common rules for spawned objects
- They are the owner's **units**, **selectable by their owner** (others only once they can see them), with their own panels.
- They use **no capacity** (Unit Control 0 or the faction's equivalent).
- **Creating one doesn't change the selection** (Q0155): the creator stays selected.

## Archetypes

### Piloted object
(the Drone Operator's drone) A separate unit **controlled directly by the player**, tied to its creator.
- Parameters: **one active at a time**; what the creator can't do meanwhile (e.g. can't move); **how it ends**, e.g. leaving a **leash range** around the creator (measured edge to edge), a **battery** time limit, the **creator's death**; whether ending makes it explode; whether it can return (no recall: spent once launched).

### Deployed object
(the Twister's mine; user, 2026-11-26; Q0160) Placed by its creator at a location (the creator drives there, stops briefly to lay it, then is free). Closer to a **permanently deployed unit** than to a structure: **not a structure** (doesn't count for the Defeat Condition).
- Parameters: hidden or not (detection not yet designed); a **trigger** (e.g. enemy infantry or vehicles entering an activation radius, after a delay; who doesn't trigger it); its effect (e.g. a ground-only area explosion with friendly fire; friendly units don't trigger it); what happens if destroyed by damage (e.g. simply removed, no explosion); its sight and armor.

### Copy
(the Disciple's illusions; user, 2026-11-27) **Copies of the creator** placed beside it, looking like it to enemies.
- Parameters: number and placement; what is copied (current HP, MaxHP, speed, timed effects for their remaining time); what they can't do (e.g. cast); damage taken multiplier; how the owner tells them apart; **when they end** (their death, the creator's death, the creator recasting).

## Used by (index only)
Stock and purchase: Drone Operator (drones), Twister (mines). Piloted object: Drone Operator's drone. Deployed object: Twister's mine. Copy: Disciple's Mirage illusions.
