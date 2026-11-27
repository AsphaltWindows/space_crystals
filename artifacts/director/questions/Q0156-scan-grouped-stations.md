# Q0156 — Scan with several Radar Stations selected

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GR, the Radar Station and Scan, builds the placeholder in Options.
- **Docs**: `factions/gdo/structures/radar_station.md > Placeholder stats`; `systems/controls/selection.md > No smart-casting`; `factions/cults/structures/shrine.md > Placeholder stats`

## Question
With several Radar Stations selected, does one press of Scan scan from all of them?

## Context
- "Scan (hotkey Q, targeted: left-click any point on the map): reveals a circle of radius 6 grid units for 8 seconds (128 frames); cooldown 60 seconds (960 frames); free. Each Radar Station has its own cooldown." The Radar Station is "Groupable true".
- **The rules as they stand:** "A command issued from the command panel (hotkey or button) goes to every instance it is issued to". "The game never hands a group's command to just one member on the player's behalf". "To use a Groupable unit's ability on only one instance, the player selects that instance alone".
- **Ungroupable casters:** Groupable is false for "caster-type units (one cast should spend one unit's energy, not the whole group's)", so "the player casts with one unit, presses Tab to the next, and casts again, with no mouse reselection".
  - The Shrine, a building that casts: "Groupable false (user, 2026-11-26; Q0118: each press casts from one Shrine)".

## Options
- **(a) All of them, as the rules stand** (the placeholder): two selected stations scan the same point, and both cool down for 60 s. To scan two places, the player selects each station alone.
- **(b) The Radar Station becomes Ungroupable,** as the Shrine is: each press scans from one station and Tab moves to the next, so Q, Tab, Q scans two places.
