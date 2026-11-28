# Vault

Syndicate equipment storage warehouse for re-arming Syndicate forces.

Prerequisite for the **Shadebringer** and (with a Chemical Plant) the **Specter** (2026-11-25).

**Network-wide** (user, 2026-11-25): a Vault re-arms and recharges units **anywhere in the Tunnel Network**, not only units that entered through the Tunnel whose area it sits in. While the player owns a Vault, Shadebringers inside the Tunnel Network recharge their Sludge and Cover charges (factions/syndicate/units/shadebringer.md > Charges and recharging).

**Throughput limits** (user, 2026-11-25, to be designed): re-arming should probably be **limited**: only so many units re-armed at a time (per Vault), and each re-arming takes time. Placeholder rule and numbers below (to confirm).

## Entity Type - Structure Type (Underground)
## Tier Requirement - 1
## TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 2x2; MaxHP 400; PointArmor 6, FullArmor 24 (underground expansion, n = 6); Cost 150 Space Crystals; Build Time 400 frames (25 seconds).
- **Throughput**: each Vault re-arms or recharges **2 units at a time**; others wait their turn (network-wide, first come first served). Specter re-arming: 160 frames (10 s). A Shadebringer's tank (one, shared by Sludge and Cover; Q0190) refills at 10 per second. **Services run side by side** (user, 2026-11-26; Q0158): a unit's turns at a Vault and at a Null Chamber (and any repair) progress **at the same time**, in separate queues; e.g. a Specter is re-armed and buffed in 10 s, an empty Shadebringer in 20 s.
