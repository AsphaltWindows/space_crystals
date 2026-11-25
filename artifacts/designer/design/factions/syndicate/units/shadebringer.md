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
- TBD: X, Y, Z; trail width; whether the slow refreshes while a unit stays in the sludge.

## Cover (toggle)
While on, the Shadebringer leaves a **stack/wall of smoke** in its wake. Each piece of smoke **lasts a set number of seconds** (TBD) and then clears. The smoke **blocks line of sight into it and behind it** for every unit except **satellites**: it's not possible to see into the smoke, or past it, from the other side.
- This is the **first sight-blocking mechanic** in the game: elsewhere vision is a plain radius (core/vision.md).
- **Units inside the smoke can't see anything** (user, 2026-11-25): a unit standing in smoke provides no vision at all, not even of the smoke around it.
- **Structures too** (user, 2026-11-25): the smoke blocks **buildings'** sight the same way as units'. Only satellites see through it.
- TBD: the smoke's duration and width/height.

## Both at once
(user, 2026-11-25) Sludge and Cover are independent toggles and **can both be on at the same time**: one pass then leaves a trail that both slows and blinds, draining both tanks.

## Charges and recharging
(user, 2026-11-25) Sludge and Cover each have their **own, separate charge**. The charges are **recharged only inside the Tunnel Network**, and only while the player owns a **Vault** (this is the Vault's "re-arming" role). Away from the tunnels, a Shadebringer that has used up a charge can't use that ability until it goes back underground.
- **Each charge is a tank** (user, 2026-11-25) that **drains while its toggle is on**. Using an ability for a short while costs only a little. When a tank runs **empty**, that ability switches off and can't be switched back on until the tank has some charge again.
- Inside the Tunnel Network (with a Vault), the tanks **refill gradually**, partly used ones included.
- TBD: tank size, drain rate, refill rate (numbers); whether a nearly empty tank can be switched on (minimum charge).

All numbers (HP, armor, sight, speed, turn radius, TunnelSpaceCost, X, Y, Z, smoke duration, cost, build time) and hotkeys: TBD.

## ObjectInterfaceState
Move-type commands like a combat unit without an attack, plus the two toggles (hotkeys TBD). Whether toggles are queueable and whether they keep orders: TBD (the Riot Shield/Convert rule, mode switches keep orders and are not queueable, is the likely model).
