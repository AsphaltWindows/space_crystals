# Q0154 — HoverCraft movement: DragMovement numbers for the Hornet, the Phantom, the Prism and the drone

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GK and SK, which register the new GDO and Syndicate kinds with their numbers, give these units option (a)'s numbers. GD (the Drone Operator and its drone), GH1 (the Airfield and the Hornet), SD (the Phantom) and SP (the Prism) build on them.
- **Docs**: `systems/units.md > DragMovement`, `> TurnRateMovement`, `> HoverCraft`; `factions/gdo/units/{hornet,drone_operator}.md > Placeholder stats`; `factions/gdo/units/supply_chopper.md > UnitBaseAttributes[HoverCraft]`; `factions/syndicate/units/{phantom,prism}.md > Placeholder stats`; `factions/colonists/units/drone.md > UnitBaseAttributes[HoverVehicle]`

## Question
Which movement model, with which numbers, do the Hornet, the Phantom and the Prism use? The same for the Drone Operator's drone, if it is a HoverCraft (its UnitBase is asked in the Drone Operator question).

## Context
- **The HoverCraft base** moves by DragMovement. It "Relies on high drag to decelerate passively", and "Can turn in place or while moving".
  - DragMovement: "MaxSpeed = (OmniDirectionalAcceleration + ForwardAcceleration) / DragRatio".
  - "Air units typically have high drag and high acceleration, relying on drag alone to stop. Ground hover units have lower drag and actively thrust in the opposing direction to decelerate and change direction."
- **Two units have DragMovement numbers:**
  - the Supply Chopper: ForwardAcceleration 0.9, OmniDirectionalAcceleration 0.1, DragRatio 0.1, TurnRate 10°/frame, so a top speed of 10 su/frame;
  - the Colonists' Drone, a ground HoverVehicle: 0.9, 0.3 and 0.15, top speed 8. Its note: "thrust and drag raised ×1.5 from 0.6 / 0.2 / 0.1, keeping top speed 8 but reaching it faster: ~65% after ~7 frames, near top speed after ~20 frames, instead of 10 and ~30. Snappier starts, stops and turns."
- **The other HoverCraft have TurnRateMovement's attributes** (TurnRate, Acceleration, Deceleration, MaxSpeed):
  - the Hornet: "MaxSpeed 9 su/frame; Acceleration 0.6; Deceleration 0.6; TurnRate 8°/frame";
  - the Phantom: "MaxSpeed 9 su/frame; Acceleration 0.8; Deceleration 0.8; TurnRate 8°/frame". It is "Relatively manoeuvrable.";
  - the drone: only "MaxSpeed 10 su/frame". The Prism: only "MaxSpeed 1 su/frame".
  - TurnRateMovement: "Fast acceleration and deceleration provide responsive, direct movement typical of most units in modern RTS games."
- These are the first HoverCraft in play, so the answer sets how GDO and Syndicate aircraft handle.
- The game runs 16 frames a second, and a GU is 64 su.

## Options
- **(a) DragMovement, with the given numbers mapped** (the placeholder):
  - OmniDirectionalAcceleration = the given Acceleration, ForwardAcceleration 0, DragRatio = Acceleration ÷ MaxSpeed, TurnRate as given. The top speed is then the given one. At top speed, drag alone slows the unit at the given Deceleration, then more gently as it slows.
  - The Hornet's DragRatio is then 0.067: lower than the Chopper's 0.1, and under half the ground-hovering Drone's 0.15.
  - A Hornet reaches about 64% of top speed in 15 frames (~1 s), and 90% in about 33 frames (~2 s).
  - Stopping by drag alone from 9 su/frame, it drifts about 126 su (~2 GU), and is still moving at about 1 su/frame after 2 s. A Phantom drifts about 92 su (~1.4 GU).
  - The drone and the Prism take the Hornet's Acceleration and TurnRate. The drone drifts about 157 su (~2.4 GU) from 10 su/frame. The Prism's DragRatio would be 0.6, so it stops almost at once.
- **(b) TurnRateMovement for these units, with the numbers as given:**
  - a Hornet reaches top speed in 15 frames (~1 s), and stops in 15 frames over about 65 su (~1 GU). A Phantom stops in about 11 frames over about 50 su;
  - no drift, though not instant as infantry are;
  - the HoverCraft base would list TurnRateMovement, or these units would be exceptions to it. The drone and the Prism would need an Acceleration and a Deceleration.
- **(c) DragMovement numbers given for each unit,** as for the Supply Chopper: high drag and high thrust, as "Air units typically have" reads. At the Chopper's numbers (thrust 1.0, DragRatio 0.1), a unit reaches about 65% of top speed in 10 frames, and drifts about 90 su (~1.4 GU) from 10 su/frame.

In every option the turn rate is the given one: at 8°/frame, a 90° turn takes about 11 frames.
