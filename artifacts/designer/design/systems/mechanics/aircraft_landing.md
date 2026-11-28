# Mechanic: Aircraft Landing and Rearming

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit; gathers rules previously written in the Airfield's, Airstrip's, Sparrow's, Dragonfly's, Specter's, Supply Chopper's and Pilgrim's docs; no rule changed in the move.)

Two parts: **how an aircraft lands and takes off** (three landing archetypes), and **how a limited-ammo aircraft gets rearmed** (two rearming archetypes). An aircraft's doc (and its base structure's doc) says which archetypes it uses, with parameters and exceptions.

## Landed aircraft (every archetype)
- **Air target while landing and taking off**, **ground target while landed** (user, 2026-11-23/26; Q0162): landed, it can be hit only by anti-ground weapons (TargetDomain Ground or Universal); from the moment it starts to take off it is an air target again.
- **Landed aircraft hold fire** (user, 2026-11-28; Q0189): they fire only in the air.
- **Out of the air separation** while landed (designer, Q0189): flying aircraft don't push landed ones, nor the reverse.
- **Landed on a structure that is destroyed** (user, 2026-11-26, Q0162; **for every structure**, user, 2026-11-28): aircraft landed on a structure when it is destroyed are **destroyed with it** (e.g. a Pilgrim on an Airstrip, with its passengers); one still **landing** counts as landed, one already **taking off** escapes.

## Landing archetypes

### Vertical
Lands on a **spot** (a slot or door on a structure, or open ground) straight down, and takes off straight up.
- Parameters: **landing time** and **take-off time** (frames).
- HoverCraft: hover over the spot, then descend.
- **A Glider with vertical landing** (e.g. at an Airfield slot or a tunnel air door; user, 2026-11-26; Q0162): it approaches at IdleSpeed, stops over the spot while it lands, takes off straight up and leaves at IdleSpeed. Landed, it doesn't move. **Approach** (designer placeholder, Q0192): it slows from MaxSpeed to IdleSpeed 4 GU before the spot, lines up by flying straight on while the spot is within its tightest turn (overflying, it comes round again), and lands facing the way it came.
- **No free spot** at its base: a Glider circles over the structure's centre until one frees (Q0192).

### Runway
Lands on and takes off along a **runway** (a structure with a long axis; the runway's doc names its ends, e.g. B and C).
- **Take-off run**: it speeds up along the runway toward the far end and lifts off. **The run always finishes** (user, 2026-11-26; Q0170): an order given during the run waits for its end and then carries on.
- **Landing**: it lines up with the runway and touches down along it (parameter: landing time).
- **Occupancy**: a parameter of the runway (e.g. one landed aircraft at a time; while occupied, finished aircraft wait at 100%, as behind a blocked exit, core/entities.md > Exit Doorway).

### Field landing
Lands on **open ground** where it is ordered (used by Landing carriers, systems/mechanics/carrying.md): only on ground infantry can walk on, with no building or unit in the way; a blocked spot means the nearest free spot. Parameters: landing and take-off time.

## Rearming archetypes
For aircraft with **limited ammo**: once out, they must go to a base to rearm. **Rearm time is per unit of ammo** (RearmTimePerAmmo on each aircraft; user, 2026-11-23): rearming takes RearmTimePerAmmo × missing ammo.

### Assigned base
(user, 2026-11-23/26/28) Each aircraft is **assigned** to one base structure and rearms only there.
- **Assignment**: to the structure that built it. A base holds a set number of assignments (parameter), and as many landing spots. **Reassigning**: right-click another own base with room (its assignments plus its queued aircraft fewer than the maximum); the aircraft is reassigned and flies there. **No manual unassigning.** A base can't queue an aircraft that would exceed its assignments.
- **Base destroyed**: its aircraft become **unassigned**, with no automatic reassignment; an unassigned aircraft can't rearm or return until reassigned.
- **Automatic return** when out of ammo: it flies to its base, lands and rearms. Any other order interrupts it (it stays out of ammo). **It replaces all the aircraft's orders** (user, 2026-11-28; Q0195): orders queued behind the one it ran out in are **dropped**, as for the Network base; it rearms fully and stays landed until given new orders.
- **Return to Base** (hotkey F, placeholder; or right-click its own base): fly there, land and rearm.
- **Shift-queued orders run as soon as it has landed** (user, 2026-11-28; Q0189): taking off with the ammo rearmed so far.
- **An order during rearming** (user, 2026-11-26; Q0162): takes off at once with the ammo rearmed so far.
- **After rearming it stays landed**; it doesn't go back unless ordered.
- **New aircraft** appear **landed on a free spot** and take off for the base's rally point; with no rally point they stay landed. No free spot: the finished aircraft waits inside (Q0189).

### Network base
(user, 2026-11-25) The aircraft rearms by **landing into a network** of the owner's structures (e.g. any own tunnel with air doors) and being rearmed **inside**, by a service anywhere in the network (parameter: which service structure, e.g. a Vault).
- **Out of ammo**: it flies to the **nearest** structure it can land in, lands, and is rearmed inside; with **none**, it circles where it is. It lands even if no rearming service exists yet, and waits inside unarmed until one does.
- **Ordered to attack while unarmed**: it goes to rearm first; **on entering, its orders are wiped**, and it stays inside (rearming never resumes the attack by itself). With no structure to land in, it **ignores Attack orders**.
- **After its attack**: if the target is destroyed, it goes on to its next queued order; if the target survives, the Attack still stands, so it ignores later queued orders and goes home to rearm.

## Used by (index only)
Vertical + Assigned base: Hornet, Dragonfly (GDO Airfield). Vertical + Network base: Specter (Syndicate tunnel air doors). Vertical, no rearming: Phantom, Prism (tunnel air doors). Runway: Sparrow (take-off only; never lands), Pilgrim (Cults Airstrip). Field landing: Supply Chopper.
