# Syndicate: Structures

## Tech Tree Overview

Complete list of Syndicate structures. "Tier" is the Tier Requirement checked against the Tunnel building the expansion; "Requires" is the TechPrerequisite, satisfied by a fully constructed structure anywhere in the Tunnel Network (see Tunnel Expansions > Expansion Requirements and core/tech_and_research.md > Tech Prerequisites).

| Structure | Type | Built by | Tier | Requires | Role |
|---|---|---|---|---|---|
| Tunnel | Surface | Agent | — (upgradeable T1 → T2 → T3 with Supplies) | — | Network entry/exit, hosts expansions, Tunnel Space |
| Hideout | Expansion | Tunnel | 1 | — | Primary infantry accommodations; produces Agents, Guards |
| Optics Lab | Expansion | Tunnel | 1 | — | Laser weaponry tech |
| Chemical Plant | Expansion | Tunnel | 1 | — | Explosives and propulsion tech |
| Vault | Expansion | Tunnel | 1 | Hideout | Equipment storage for re-arming forces |
| Vehicle Assembly | Expansion | Tunnel | 2 | Chemical Plant | Ground vehicle production |
| Forge | Expansion | Tunnel | 2 | Vehicle Assembly | Advanced alloys and materials |
| Null Chamber | Expansion | Tunnel | 2 | Hideout | Shielded crystal consumption; manned units exiting the Tunnel Network get a temporary buff (detection, sight, damage reduction) |
| Aerospace Assembly | Expansion | Tunnel | 3 | Forge | Aviation and orbital (satellite) production |
| Orbital Command | Expansion | Tunnel | 3 | Aerospace Assembly | Control/coordination of orbital forces |
| Laser Turret | Pop-up (neutral with its Tunnel, like expansions) | Tunnel | — | Optics Lab | Defensive structure, attacks ground and air; inside a Tunnel Area it is a hidden pop-up turret |

## Tunnel Expansions

Underground buildings constructed within a Tunnel's Tunnel Area. Expansions are invisible to enemies without detection and can be walked over by surface units.

**High armor underground** (user, 2026-11-25; Q0071): expansions are hit by area damage over them (systems/combat.md > AoEDamage) but have **high armor**. **Placeholder for every expansion: n = 6, i.e. PointArmor 6, FullArmor 6 x size in grid squares** (a 2x2 expansion: 6 / 24), following the building armor rule (core/entities.md > Structure Type > Building armor rule); this replaces the 1/4 listed in individual stat blocks. Numbers to be tuned.

**Not repaired; they regenerate** (user, 2026-11-26; Q0124; **Restoring HP, Regeneration**, systems/mechanics/hp_restoration.md): Agents can't repair expansions. Instead, a completed expansion **regenerates HP gradually** while it is part of a Tunnel system (its Tunnel is alive). Placeholder rate (designer): **1% of MaxHP per second**, always on, **including during combat** (user, 2026-11-26).

**No buildings on top of expansions, and no expansions under buildings** (user, 2026-11-25; Q0062): a surface structure and an underground expansion (any player's, including neutral ones) can never share a tile.
- **Surface buildings over a hidden expansion**: an enemy's placement is refused. A **direct placement** (e.g. a GDO building) shows its ghost **red at once**, which reveals that something is underground there; this is accepted, since GDO is limited by its build area. A Colonists **Warp In** is judged the same way: red at once (user, 2026-11-26; Q0080), since it too starts at the command with no walk. A **worker-built** building's ghost judges by what the player knows (so it stays green), and the order **fails when the worker arrives** (core/entities.md > Placement Validation > Worker-Built Structures).
- **Expansions under a surface building**: refused (see factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState > AwaitingPlacement (Expansion)).
- Units still walk freely over expansions.
- **No expansions under resources** (user, 2026-11-25; Q0062): a Space Crystals patch or Supply Delivery Station above blocks an expansion (core/entities.md > Placement Validation > All placement methods: no buildings on resources). All Syndicate units are produced by Tunnel expansions.

### Expansion Requirements

Each expansion has two independent kinds of requirement:

- **Tier Requirement — checked against the building Tunnel.** A Tunnel can only build expansions inside its own Tunnel Area, and the expansion's Tier Requirement must be met by **that specific Tunnel's** current tier. Other Tunnels' tiers are irrelevant. (E.g. a Tier 2 expansion can only be built by a Tier 2+ Tunnel.)
- **TechPrerequisite — checked across the whole Tunnel Network.** If an expansion requires other expansion type(s), those may be located in **any** Tunnel the player owns, regardless of that Tunnel's tier. (E.g. a Tier 2 expansion requiring a Hideout can be built by a Tier 2 Tunnel whose own Tunnel Area contains no Hideout, as long as a Hideout exists in some other Tunnel — even a Tier 1 one.)

TechPrerequisites follow the shared rules in core/tech_and_research.md > Tech Prerequisites: only fully constructed expansions count, the check happens only when construction is started, and losing a prerequisite never disables existing or in-progress expansions — it only blocks starting new ones.

### Rally Point Behavior

Each production expansion can have a rally point set. This determines what happens when a unit finishes production:

- **Rally point set on the surface**: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally point.
- **No rally point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel.
- **Rally point on any own Tunnel** (user, 2026-11-25; Q0064): the unit stays inside the network, like a rally on the parent Tunnel.
- **Rally point on an expansion** (user, 2026-11-25; Q0064): treated as the **surface location** above it: the unit auto-ejects from the parent Tunnel and walks to stand on top of the expansion.
- **Auto-ejected units** (those leaving by themselves because of a surface rally point; user, 2026-11-25; Q0064) join the parent Tunnel's **exit queue** in order with manual ejects, first come, first served. While waiting they don't appear in the EjectMenu.
- **What "moves to the rally point" means** (user, 2026-11-25; Q0064): the unit's default right-click action at the rally target (systems/controls/interface.md > CommandPanel > What a rally point makes a new unit do), e.g. an Agent rallied on a crystal patch gathers.

## Surface Defensive Structures

(user, 2026-11-26) The Syndicate's defensive structure, the **Laser Turret**, is **built only by Tunnels**, inside their Tunnel Area (factions/syndicate/structures/laser_turret.md); Agents build no surface structures other than Tunnels. It is only active inside its Tunnel's area; with its Tunnel lost it goes neutral like an expansion.

## Files

- `hideout.md` — **Hideout**: Infantry production expansion
- `optics_lab.md` — **Optics Lab**: Laser tech building
- `chemical_plant.md` — **Chemical Plant**: Explosives/propulsion tech building
- `vault.md` — **Vault**: Re-arming/recharging, network-wide
- `vehicle_assembly.md` — **Vehicle Assembly**: Ground vehicle production (Tier 2)
- `forge.md` — **Forge**: Advanced materials (Tier 2)
- `null_chamber.md` — **Null Chamber**: Exit buff for manned units
- `aerospace_assembly.md` — **Aerospace Assembly**: Aircraft and satellite production (Tier 3)
- `orbital_command.md` — **Orbital Command**: Orbital coordination; Prism prerequisite (Tier 3)
- `laser_turret.md` — **Laser Turret**: Pop-up defensive turret, built by Tunnels
