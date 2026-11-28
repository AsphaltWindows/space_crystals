# Mechanic: Carrying Units

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit (user, 2026-11-28: "a few archetypes ... independently defined from the units"). Gathers rules previously written in individual carriers' docs; no rule changed in the move.)

A **carrier** takes other units aboard, moves them, and lets them out. A carrier's doc says which **archetype** it uses, gives the **parameters**, and lists only its **exceptions**. Everything else follows this doc.

Not covered here: **garrisoning** (occupants fire from inside) and buildings that hold units for a service (e.g. a tunnel network, training or augmentation buildings). Those are separate mechanics.

## Parameters (every carrier)
- **Capacity**: how many places; each passenger takes one place unless the carrier says otherwise (e.g. one vehicle takes all places).
- **Who can board**: which unit types.
- **Load and unload inputs** it supports (Loading, Unloading below).
- **Archetype-specific parameters** (each archetype lists them).

## Archetypes

### Ground carrier
A ground unit that loads and unloads **where it stands**.
- **Loading state**: an optional condition (e.g. "above ground only" for a unit with an underground mode). It can't load or unload outside it.
- Unload All is **untargeted**: everyone comes out around it, where it is.

### Landing carrier
An aircraft that **lands to load and to unload**, then **takes off by itself** once loading or unloading is done.
- **Where it lands**: only on ground that infantry can walk on, with no building or unit in the way; a blocked spot means the nearest free spot.
- **Landing and taking off take a set time each** (parameter); it is an **air target** while landing and taking off and a **ground target** while landed.
- **Enter** makes it land **where it is**; the units walk to it.
- **Load** (optional input): it flies to the unit, lands beside it, and the unit walks in.
- Unload All is **targeted**: left-click a spot; it flies there, lands, and everyone comes out around it.

### Docked carrier
An aircraft that loads and unloads **only while landed on a given own structure** (parameter: which structure type, e.g. a runway), going there when ordered.
- **Unload All**, while docked: everyone comes out **along a given side** of the structure (parameter).
- It doesn't take off by itself; any order makes it take off.
- How it lands on and leaves the structure is the structure's (or the aircraft's) own landing rules.

### Paradrop (an addition to an aircraft carrier)
**Drop (targeted)**: left-click a spot anywhere; the carrier flies there and, on reaching it, **lets every passenger out in quick succession** while flying on (parameter: frames between passengers), so they come down in a short line along its path.
- **Falling**: each passenger falls for a set time (parameter) before landing. While falling it is an **air target** with its normal HP and armor, and **can't act**.
- **Landing spot**: below where it jumped; if that ground isn't walkable for it, the nearest walkable spot.
- A Drop with no passengers just flies to the spot. Drop Shift-queues like any command.

## Loading
- **Enter** (systems/controls/commands.md > Enter): right-click the carrier with own eligible units selected. The units **walk to it** and each **boards instantly on reaching it** (no per-unit boarding time).
- **Load** (only where the archetype or carrier gives it): with the carrier selected, right-click an own eligible unit.
- **Units that don't fit stay outside**, idle.

## Unloading
(user, 2026-11-28; Q0191)
- **Unload All** (hotkey **F** unless the carrier says otherwise) lets everyone out.
- **On the move, it stops first**: the passengers come out once the carrier is at rest (or landed).
- Passengers come out on the **nearest free spots** around the carrier (or along the side the archetype names).
- **Crowded**: passengers with no free spot **stay aboard**; the player moves the carrier and unloads again.
- Unload All Shift-queues like any command.

## Passengers
- **Can't fire or cast** from inside: they are carried, not fighting (user, 2026-11-26; Q0166).
- **Keep counting** against their faction's capacity (Unit Control, Tunnel Space, Beacon Capacity), aboard or not.
- **Are not on the map**: they can't be targeted or damaged while aboard, and nothing around the carrier affects them (e.g. a carrier inside a tunnel network doesn't put its passengers in the network; Q0166).
- **Passenger count**: a selected carrier shows **one pip per passenger**, **to its owner only** (user, 2026-11-28; Q0191).

## Carrier destroyed
**Everyone aboard dies with it** (user, 2026-11-25/26), wherever it is (landed, flying, underground). A carrier whose passengers should survive must say so as an exception.

## Used by (index only)
Subterranean Transport (Ground carrier), Supply Chopper (Landing carrier), Pilgrim (Docked carrier + Paradrop).
