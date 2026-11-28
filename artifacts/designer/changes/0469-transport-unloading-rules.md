# 0469 — Transport unloading: crowded unloads, passenger pips, F on the move

- **Date**: 2026-11-28T10:00:00-05:00
- **Answers**: Q0191
- **Docs**: `factions/syndicate/units/subterranean_transport.md > Placeholder stats` (Unloading rules, shared by every transport)

## What changed and why
All three parts as built (user, option a each). They apply to every transport, the Supply Chopper and the Pilgrim included, unless that transport's doc says otherwise.
1. **Crowded unload:** passengers come out on the nearest free spots around the transport. Those with no free spot **stay aboard**; the player moves it and presses F again.
2. **Passenger pips:** shown to the **owner only**, so opponents can't count what's inside.
3. **F on the move:** the transport **stops first**, then unloads.

## Implementation impact
None. Remove the placeholder marks. The Pilgrim's own rule for Unload All on the runway side, with its crowded case, also follows point 1.

## Verification
1. Unload a full Transport with the ground around it packed. Only those with room come out; the rest stay and can be unloaded after moving.
2. An opponent selecting the Transport sees no pips.
3. Pressing F while it drives makes it stop, then unload.
