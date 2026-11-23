# colonists_shields_and_prospector_beacon_deployment

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Introduced two new Colonists mechanics: a faction-wide Shields stat, and the Drone-to-Beacon deployment mechanic (how Beacons come into existence). This is the first real content for the Colonists faction — a brand new `artifacts/designer/design/colonists_objects.md` file was created, and `artifacts/designer/design/combat.md` was updated with a new Shields damage-resolution rule.

Note: this unit was originally named "Prospector" and was renamed to "Drone" in a later session, before this message was ever picked up by task_splitter. All references below already use the final name — implement as "Drone".

## New: Shields (combat.md)
A new stat, `MaxShields`, exists on every Colonists unit and building (no other faction has this — it's specific to Colonists). Key rules:
- **ShieldRegenRate**: 4 Shields per second (0.25/frame), constant base rate shared by all Colonists units/buildings. Regenerates continuously, not gated by a delay after taking damage. Upgradeable later via Colonists tech (not yet defined — separate future discussion).
- **Damage resolution order**: incoming damage hits Shields FIRST, at full value — PointArmor/FullArmor do NOT reduce damage against Shields. Only once Shields are depleted does the overflow damage proceed to HP, where it goes through the existing SingleTargetDamage/AoEDamage armor formulas as normal (PointArmor for single-target, FullArmor for AoE).
- If Shields fully absorb an incoming hit, no HP is lost and armor is never consulted for that hit.

This was added as a new "## Shields" section in `combat.md`, right after the existing AoEDamage section.

## New: Drone -> Beacon Deployment (colonists_objects.md, new file)
Colonists build their foundational structure (the Beacon) by having a Drone transform directly into it, rather than building a separate structure from outside like Cults Recruits or Syndicate Agents do. Key rules:
- **Deploy Command**: select a Drone, choose Deploy, left-click a ground location. The Drone walks there, then begins deploying in place.
- **Deployment Duration**: 48 frames (3 seconds).
- **HP/Shields interpolation during deployment**: both HP and Shields linearly interpolate from the Drone's own MaxHP/MaxShields toward the Beacon's MaxHP/MaxShields as progress goes from 0.0 to 1.0 over the 48 frames.
- **Vulnerability**: the transforming unit is vulnerable to attack for the whole duration. If destroyed mid-deployment, both the Drone and the in-progress Beacon are lost entirely.
- **Cancellable**: the player can cancel any time before completion — the unit reverts cleanly to a normal Drone, no loss.
- **One-way once complete**: a fully-formed Beacon can never revert back into a Drone.
- **No placement spacing restriction**: Beacons can be placed directly adjacent to other Beacons (this deliberately overrides an old pre-formalization note in `to_be_converted.md` that had described a minimum spacing radius — that rule is now superseded).

The new `colonists_objects.md` file also stubs out Drone and Beacon entity blocks (Entity Type, Faction, Destructible) with MaxHP/MaxShields/PointArmor/FullArmor/SightRange/Groupable/BeaconCapacity all marked TBD — these numeric stats were NOT decided this session and are explicitly left open for a future session. Movement, attack, gathering behavior, Size/SymmetryType, and full ObjectInterfaceState for both are also TBD.

## QA Instructions

**Shields (general mechanic — test against any Colonists unit once one exists with concrete stats, or verify the rule is implemented generically and ready to apply):**
1. Confirm Colonists units/buildings have a visible Shields value separate from HP in their InfoPanel.
2. Deal damage less than current Shields to a Colonists unit — confirm only Shields decrease, HP is untouched, and the damage taken ignores that unit's PointArmor/FullArmor entirely (full damage value subtracted from Shields).
3. Deal damage exceeding current Shields — confirm Shields drop to 0 and the overflow amount is then reduced by the unit's PointArmor (single-target) or FullArmor (AoE) before being subtracted from HP.
4. Let a Colonists unit sit without taking damage — confirm Shields regenerate continuously at 4/second (0.25/frame) rather than jumping after a delay.

**Drone -> Beacon Deployment:**
1. Select a Drone, issue Deploy, click a ground location. Confirm it walks there and begins an in-place transformation.
2. Confirm the transformation takes 48 frames (3 seconds) total, and that both the unit's HP and Shields values visibly interpolate from the Drone's stats toward the Beacon's stats over that time (not an instant swap).
3. Attack the transforming unit mid-deployment and destroy it — confirm both the Drone and the partially-formed Beacon are lost (no partial refund, no surviving unit).
4. Start a deployment and cancel it before completion — confirm the unit reverts to a fully-functional, undamaged Drone (assuming it wasn't damaged during deployment).
5. Let a deployment complete fully — confirm there is no way to convert the resulting Beacon back into a Drone.
6. Place two Beacons directly next to each other (adjacent tiles) — confirm placement is allowed with no minimum spacing enforcement.

Note: since Drone/Beacon's own MaxHP/MaxShields/PointArmor/FullArmor values are still TBD in the design doc, QA of the interpolation math (item 2) may need placeholder/test values until those are formalized in a future feature_request.
