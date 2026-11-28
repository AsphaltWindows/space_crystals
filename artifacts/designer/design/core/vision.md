# Vision

## Vision
The fog of war system that controls what each player can see on the map. Every tile on the map has one of three visibility states per player. Vision is provided by owned Object Instances (units and structures) based on their SightRange, measured from the object's **centre** (a disc of radius SightRange; target scans use the same measure, systems/controls/behaviors.md > BaseAutoTargeting; Q0190).

## VisibilityState
The visibility of a tile from the perspective of a specific player.

### Value - VisibilityStateEnum (Unexplored | Explored | Visible)

## Unexplored
The tile has never been within the SightRange of any of the player's units or structures. The tile is fully black — terrain, structures, and units are all hidden.

## Explored
The tile was previously within the SightRange of one of the player's units or structures, but is not currently. Terrain is shown. Structures and resources are shown in their last-known state (may have been destroyed or changed since). Enemy units are not shown.

### Explored Selectability
Objects in Explored areas are **not selectable**, with one exception:
- **SupplyDeliveryStation**: Selectable by GDO players only. The GDO has a lore/strategic relationship with supply stations — supplies are intended for GDO forces, and the SupplyTower scheduled delivery mechanic requires planning around delivery timing even without active vision. Non-GDO players cannot select SupplyDeliveryStations in Explored areas.

### Targeting remembered objects
(user, 2026-11-24) Remembered objects (structures, resource patches and other objects shown in their last-known state in Explored areas) **can be targeted** by right-click and by targeted commands (Attack, Gather, Move to object, rally points), even though they are not selectable.
- The order goes to the object's **last-known position**. When that spot comes into sight, the memory updates: if the object is **still there**, the order carries on normally (attack it, gather it, …); if it is **gone**, the order ends the way it would for a vanished target (e.g. Gather: the redirect rules, factions/colonists/units/drone.md > Gathering > Patch Occupancy; Attack: the unit stops there and takes its next queued order or idles).
- Nothing about the real state leaks: the cursor, the command and any rally marker treat the remembered object as present until the player sees otherwise.

## Starting exploration
(user, 2026-11-26) A game option: the map **starts explored** (fog of war still on). **Default: on.**
- With it on, every tile starts **Explored** for every player instead of Unexplored. What each player remembers at 0:00: the **terrain** and the **neutral objects as they are at the start** (Space Crystal patches with their starting amounts, Supply Delivery Stations and other neutral objects). **No player's objects** are remembered, so bases still have to be scouted. Memories update as usual when an area comes into sight.
- With it off, the map starts Unexplored (black), as described above.
- **Where it is set**: in the future, the **game creation screen**; for now a **command-line flag** (like the computer players, core/game_rules.md > Computer players (playtest tool)). Restart (core/game_rules.md > Playtest menu (temporary)) keeps the setting.

## Visible
The tile is currently within the SightRange of one of the player's units or structures. Everything on the tile is shown in real time — terrain, structures, and units.

### Fog delay
(user, 2026-11-24) When a tile leaves the SightRange of all the player's units and structures, it **stays Visible for 0.5 seconds (8 frames), placeholder**, before turning Explored. If it comes back into sight within that time, it simply stays Visible.
- Fog closes in softly behind moving units.
- A target at the very edge of sight doesn't blink in and out of view, so chases at the edge of sight stay smooth (this replaces a separate "grace period" for lost targets in AttackingObject / Follow; Q0018).
- No information leak: during the delay the tile is genuinely Visible, so units only act on what the player can see.

## Attacks from outside vision
(user, 2026-11-24)
- **Shots are drawn where they travel through vision**: a shot, tracer or projectile is visible to a player only along the part of its path that crosses tiles **Visible** to that player. A shot fired from the fog at your unit is seen coming out of the fog edge, which shows its direction. **No exceptions** (user, 2026-11-26; Q0083; replaces the Q0056/Q0075 exception that drew a revealed attacker's shots at you along their whole path): every shot, tracer or projectile, whoever fires it and at whatever, is drawn **only over your Visible tiles**. You see the attacker only when it is revealed by damaging you, and you see only the visible part of its projectiles.
- **The attacker is revealed**: a unit or structure that attacks one of your units or structures while outside your vision becomes **visible to you** for **1.5 seconds (24 frames), placeholder**, after each shot; a unit that keeps firing stays visible. **For projectiles and area damage** (user, 2026-11-26; Q0083) the reveal happens **when the damage lands**, not at the throw, and only to the owners of the objects it **damaged**; a dodged projectile or a blast that damages nothing reveals nothing. The attacker is revealed only if it is still alive. Only the attacker itself is revealed, not the area around it. **A hit absorbed by a Shroud cloud** counts as landing on the object it was aimed at, so it reveals the attacker to that object's owner (user, 2026-11-27; Q0143; factions/cults/units/disciple.md > Spells > Shroud).
- While revealed it is a normal visible object for you: it can be seen, selected and **targeted** (e.g. your units can shoot back if in range).
- Applies to every attacker, including long-range ones (e.g. the Artillery Rig).
- **Colour** (user, 2026-11-26; Q0090): a shot is always drawn in the **shooter's player colour**, even when the viewer doesn't see the shooter.
- **Marks seen late** (user, 2026-11-26; Q0090): a death marker or blast mark is shown on any of the viewer's Visible tiles while it is still fading, even if the viewer didn't see the event itself.
- Stealthed attackers: TBD when stealth and detection are designed.

## ElevationModifier
Sight range and attack range are modified by relative elevation between the source and target. This modifier applies when both the source and target are ground or underground **objects: units and structures alike** (structures added 2026-11-23, so a defensive structure on high ground gets the bonus and one on low ground suffers the penalty). Air units are exempt from elevation modifiers in both directions (neither gain nor suffer them).

### Elevation used: each object's **elevation level** = terrain height at its position rounded down (core/map.md > Elevation > Elevation level). For a surface structure this is simply the level of its flat footprint
### Higher ground: +1 to sight range and attack range against lower-elevation targets
### Lower ground: -1 to sight range and attack range against higher-elevation targets
### Equal elevation: no modifier
### Binary: any difference in elevation level triggers the modifier regardless of the size of the gap
### Air exempt: air units ignore elevation modifiers entirely, both as source and target
### Underground units: use the elevation level of the terrain above them
