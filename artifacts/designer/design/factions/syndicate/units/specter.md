# Specter
(user, 2026-11-25; named by the user) Syndicate **bomber** carrying a **single payload**. It flies as a **Glider** (must keep moving), but **lifts off and lands vertically** through a Tier 3 Tunnel's air doors (factions/syndicate/tunnels.md > Tunnel > Look by tier).

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - Glider (with vertical lift-off and landing at the Tunnel air doors)
## Produced at - Aerospace Assembly
## TechPrerequisite - player owns at least one **Vault** and at least one **Chemical Plant** (user, 2026-11-25)

## Attack
- **A single payload**: one bomb per sortie. **Re-arming** (user, 2026-11-25, designer's reading of "requires a Vault"): it flies back and lands in any own Tunnel it can land in, i.e. a **Tier 3** Tunnel with air doors (not one mid-upgrade to Tier 3); inside the network it is re-armed, as long as the player owns a Vault **anywhere** in the network (Vault > Network-wide).
- **AoE damage**.
- **No turret** (user, 2026-11-28; Q0192; replaces "on a turret with very low range"): it drops the bomb from its base, **facing its target**, with very low range, so it must fly straight at its target and be nearly over it. It never drops on enemies it passes on a Move.
- **Can miss**: the bomb falls to a locked location, so targets can dodge (DoublyDisjointed, assumed). The **projectile travel time is relatively high**, almost all of it the **vertical fall**.
- **Out of ammo** (user, 2026-11-25): once its bomb is dropped, its default behavior is to **fly back to the nearest own Tier 3 Tunnel**, land through its air door and re-arm inside, **even if the player owns no Vault** (it then waits inside, unarmed, until a Vault exists). If there is **no Tier 3 Tunnel**, it **circles** where it is (the Glider idle behavior).
- **Attack orders and ammo** (user, 2026-11-25):
  - **Ordered to attack while unarmed**: it can't bomb, so carrying out the Attack means re-arming first. It **flies to the nearest own Tier 3 Tunnel** to re-arm.
  - **On entering the Tunnel its orders are wiped**: the Attack and anything queued after it are dropped, and it **stays inside**. Re-arming never resumes the attack on its own.
  - **After it bombs**: if the target is **destroyed**, the Attack is done and it goes on to its next queued order. With nothing queued it is idle, and an unarmed idle Specter flies home (Out of ammo, above). If the target **survives**, the Attack order still stands. Since it is now unarmed, that means going to re-arm, so it **ignores the orders queued after the Attack** and flies home, where they are wiped.
  - Example: Attack a target, then shift-queue a Move. If the bomb kills the target, it flies the Move (unarmed; confirmed by the user), then home. If the target survives, it never flies the Move: it goes home and stays inside.
  - With **no Tier 3 Tunnel**, an unarmed Specter **ignores Attack orders** (user, 2026-11-25): it has no ammo and no way to re-arm, so the Attack is dropped and it goes on to its next queued order. With nothing queued it circles (Out of ammo, above).
- Like every attack that can miss, it can **Attack Ground** (systems/combat.md > AttackType > CanTargetGround).
- TargetDomain: **Ground only** (user, 2026-11-25).

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 56x56 space units; MaxHP 140; PointArmor 1, FullArmor 1; SightRange 6 grid units; TunnelSpaceCost 3.
- GliderMovement: IdleSpeed 6, MaxSpeed 11 su/frame, Acceleration 0.5, Deceleration 0.5, MaxCentripetalAcceleration 0.5; idle circle radius 1 grid unit; SeparationRadius **64** su (raised from 40, Q0148; must exceed the 56 su Silhouette).
- **Lift-off and landing**: vertical, **24 frames** each at the air door (it can be hit during both, as an air unit).
- Turret: **none** (Q0192; was TurnAngle 20°). Bomb: Damage **400** total over the circle; AoERadius 80 su; Range 1 grid unit; a fixed **24-frame fall** (the dodge window); **Aim 2** (was 4; short so a run lines up smoothly, designer placeholder), Firing 2, Cooldown 4.
- **Re-arming**: 160 frames inside the network with a Vault (Vault > Throughput).
- Cost: Aerospace Assembly, 200 Space Crystals, 400 frames.
