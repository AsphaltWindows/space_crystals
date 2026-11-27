# 0425 — Shadebringer trails: lays while standing still; centre rule; toggles off in the network

- **Date**: 2026-11-26T18:35:00-05:00
- **Answers**: Q0159
- **Docs**: `factions/syndicate/units/shadebringer.md > Trail rules` (new)

## What changed and why
- **1 (b)**: a stationary Shadebringer with a toggle on **lays its effect where it stands** (and drains). User: "the smoke (and sludge) only lasts a certain amount of time, so standing still while letting them out can be a conscious decision."
- **2 (a)**: a unit is in sludge / inside smoke when its **centre** is inside.
- **3 (a)**: entering the Tunnel Network **switches both toggles off**.
- The user also confirmed: **an empty tank switches its toggle off** automatically (already in Charges and recharging).

## Implementation impact
Replaces placeholder 1 (a) ("lays only while moving"). 2 and 3 match the placeholders.

## Verification
- Park a Shadebringer with Cover on: smoke keeps appearing on its spot until the tank empties, then the toggle turns off by itself and the smoke clears 12 s later.
- A 36x36 unit overlapping the sludge's edge but with its centre outside isn't slowed.
- Drive a Shadebringer with both toggles on into a Tunnel, eject it: both toggles are off.
