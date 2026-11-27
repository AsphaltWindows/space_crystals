# Shadebringer
(user, 2026-11-25) Syndicate **support vehicle**. Wheeled. **No attack and no turret.** Has two **toggleable abilities**, Sludge and Cover, each switched on and off by the player; while one is on, the Shadebringer leaves its effect behind it as it drives.

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - WheeledVehicle (exits only through Tier 2+ Tunnels)
## Produced at - Vehicle Assembly
## TechPrerequisite - player owns at least one **Vault**
## Groupable - true (assumed)
## AttackAttributes - none

## Sludge (toggle)
While on, the Shadebringer leaves a **trail of sludge** on the ground in its wake. Each patch of sludge lasts **X seconds** and then disappears.
- **Infantry and non-hover vehicles** that step into sludge are **slowed by Y%** for **Z seconds**.
- **Mechs** are slowed like vehicles (user, 2026-11-25).
- **Drill Units** are slowed while on the surface, and unaffected while underground (user, 2026-11-25).
- Not affected: hover units (HoverVehicle, HoverCraft) and air units.
- **Affects everyone** (user, 2026-11-25): enemy, allied and the player's own units alike. The Shadebringer has to be driven so it doesn't trap its own army.
- **The Shadebringer itself** (user, 2026-11-25): the sludge is spread **behind** it, so laying the trail doesn't slow it. But it is **not immune**: if it drives back over sludge (its own or another Shadebringer's), it is slowed like any other vehicle.
- X, Y, Z, trail width, refresh: placeholders below.

## Cover (toggle)
While on, the Shadebringer leaves a **stack/wall of smoke** in its wake. Each piece of smoke **lasts a set number of seconds** (placeholder below) and then clears. The smoke **blocks line of sight into it and behind it** for every unit except **satellites**: it's not possible to see into the smoke, or past it, from the other side.
- This is the **first sight-blocking mechanic** in the game: elsewhere vision is a plain radius (core/vision.md).
- **Units inside the smoke can't see anything** (user, 2026-11-25): a unit standing in smoke provides no vision at all, not even of the smoke around it.
- **Structures too** (user, 2026-11-25): the smoke blocks **buildings'** sight the same way as units'. Only satellites see through it.
- Smoke duration and width: placeholders below.

## Both at once
(user, 2026-11-25) Sludge and Cover are independent toggles and **can both be on at the same time**: one pass then leaves a trail that both slows and blinds, draining both tanks.

## Trail rules
(user, 2026-11-26; Q0159)
- **Standing still lays too**: a Shadebringer with a toggle on that isn't moving keeps laying its effect **where it stands** (and keeps draining). Since sludge and smoke only last a set time, parking to hold a spot is a conscious choice: e.g. a full Cover tank holds one spot of smoke for about 22 s (10 s of tank, then the last smoke's 12 s).
- **In it = its centre is in it**: a unit is in sludge, or inside smoke, when its **centre** is inside the trail, as for Rugged terrain, the Shrine and the First Aid Tent. (Area damage still goes by overlap.)
- **Entering the Tunnel Network switches both toggles off**: it comes out with both off; the player switches them on again.
- **An empty tank switches its toggle off** (as in Charges and recharging): it never stays "on" with nothing left.

## Charges and recharging
(user, 2026-11-25) Sludge and Cover each have their **own, separate charge**. The charges are **recharged only inside the Tunnel Network**, and only while the player owns a **Vault** (this is the Vault's "re-arming" role). Away from the tunnels, a Shadebringer that has used up a charge can't use that ability until it goes back underground.
- **Each charge is a tank** (user, 2026-11-25) that **drains while its toggle is on**. Using an ability for a short while costs only a little. When a tank runs **empty**, that ability switches off and can't be switched back on until the tank has some charge again.
- Inside the Tunnel Network (with a Vault), the tanks **refill gradually**, partly used ones included.
- Tank size, drain, refill, minimum charge: placeholders below.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 56x40 space units; MaxHP 160; PointArmor 2, FullArmor 2; SightRange 6 grid units; TunnelSpaceCost 2.
- FixedTurnRadiusMovement: MinimumTurnRadius 80 su; ForwardMaxSpeed 8 su/frame; ForwardAcceleration 1; ReverseMaxSpeed 3; ReverseAcceleration 0.5; Deceleration 1.
- **Sludge**: each patch lasts **X = 10 s**; units are slowed by **Y = 40%** for **Z = 2 s** after leaving it, and the slow **refreshes while they stay in it**; trail width 48 su.
- **Cover**: each piece of smoke lasts **12 s**; the wall is **64 su (1 grid unit)** wide and blocks sight at any height except satellites'.
- **Tanks**: 100 each; drain **10 per second** while on (10 s of use from full); refill **5 per second** inside the network with a Vault; a toggle can be switched on with **any charge** left in its tank (user, 2026-11-27).
- Toggles follow the mode-switch rule (keep orders, not queueable). Hotkeys: **R** Sludge, **F** Cover.
- Cost: Vehicle Assembly, 150 Space Crystals, 320 frames.

## ObjectInterfaceState
Move-type commands like a combat unit without an attack, plus the two toggles (hotkeys and toggle rule: placeholders below; the Riot Shield/Convert rule, mode switches keep orders and are not queueable, is the likely model).
