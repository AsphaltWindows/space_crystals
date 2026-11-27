# Q0183 — Idle vehicles stall a Tunnel's exit: of five Bikes ejected, two come out

- **Date**: 2026-11-27T09:36:06-05:00
- **Blocking**: no. M6-SV merges as built, and a test records what happens. It stays so until the answer.
- **Docs**:
  - `factions/syndicate/tunnels.md > Side A`: "exiting units never block each other, and an army enters or leaves single file";
  - `factions/syndicate/structures/README.md > Rally Point Behavior`;
  - `systems/units.md > UnitCollision > Ground Collision` ("Vehicles"; design change 0440);
  - design changes 0446 and 0447, and Q0177 (the same stall at the Chop Shop's default rally point).

## Question
How do vehicles leaving a Tunnel get clear of its exit, when an idle wheeled or tracked vehicle doesn't yield (0440)?

## Context
M6-SV made the Assault Bike live. It is the first vehicle to come out of a Tunnel: a Tier 2+ Tunnel's Side A exit path.

**What happens:**
- **Five Bikes ejected with no rally point and no order:**
  - The first comes out and stops just past the end of the exit path, idle.
  - The second stops behind it, over the exit spot, since the first doesn't yield (0440).
  - The other three stay in the exit queue for good, until the player moves the two on.
- **With a rally point straight ahead, about three Bike lengths out:** four Bikes line up back from the point, the last over the exit spot, and the fifth stays inside. This is Q0177's case at the Chop Shop.
- **With a rally point off to one side:** all five come out.
- **Infantry don't stall,** since idle infantry yield.

0446 gave the Chop Shop and the Factory a default rally point for this, and left the Syndicate's Tunnels out. Every Syndicate vehicle will meet it: the Bike, the Shadebringer, the Laser Battery and the Transport.

## Options
1. **(a) A Tunnel sends an ejected vehicle with no order a set distance out,** like 0446's default rally point. It's small to build: a stop point per eject. A long straight line still stalls at the Nth vehicle, as Q0177 asks.
2. **(b) An idle vehicle in front of an exit yields to a vehicle coming out of it.** This is an exception to 0440 near an exit; "near" needs a size. It would cover the Chop Shop's and the Factory's doors too.
3. **(c) A vehicle whose stop is taken parks beside it,** at the nearest free spot, rather than in a line back toward the exit (Q0177's option 3). This changes how a vehicle's Move ends when its goal is taken.
4. **(d) As built:** the exit queue waits, and the player moves the vehicles on, as 0447 says for a cleared rally point ("the doorway stall is the player's to handle"). A feedback line could say why the eject waits.
