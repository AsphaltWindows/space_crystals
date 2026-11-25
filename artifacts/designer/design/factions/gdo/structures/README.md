# GDO: Structures

## GDOBuildArea
A single shared buildable area per GDO player. Seeded by each Deployment Center's and each Outpost's BuildRadiusExtension and grown by every placed GDO building's BuildRadiusExtension. Field Engineer structures (Bunker, First Aid Tent) have BuildRadiusExtension 0 and never extend it — the Prospector/Outpost is GDO's expansion tool. (Outposts can be deployed outside the current build area, so they seed new, possibly separate, parts of it.) When placing a building, at least 1 grid cell of the new building must be within the current build area. After placement, the build area expands outward from the placed building by that building's BuildRadiusExtension in all directions. **Shape: square** (user, 2026-11-24): a building adds every tile within BuildRadiusExtension tiles of its footprint, diagonals included, i.e. its footprint grown by that many tiles on every side (the starting Deployment Center, 4x4 with 12, gives a 28x28 square). This matches the game's other square areas on the square grid (e.g. Tunnel Area, Recruitment Area). **The area follows the current buildings** (user, 2026-11-24): it is always the union of the areas of the GDO buildings the player **currently** owns (Deployment Centers, Outposts and every other placed building), so it **shrinks** when a building is destroyed (unless another building also covers those tiles). Buildings already standing outside the shrunken area stay, and plates keep mining; only new placements need the area. Losing the Deployment Center doesn't wipe the area: the other buildings' areas still count (e.g. an Extraction Facility can still place plates within them).

## Files

- `barracks.md` — **Barracks**: Infantry production
- `deployment_center.md` — **DeploymentCenter**: Primary construction structure; build queue, cancellation
- `arsenal.md` — **Arsenal**: Combat research
- `career_campus.md` — **Career Campus**: Advanced personnel
- `factory.md` — **Factory**: Vehicle production
- `radar_station.md` — **Radar Station**: Reconnaissance
- `advanced_mechanics.md` — **Advanced Mechanics**: Vehicle upgrades
- `airfield.md` — **Airfield**: Aircraft production; Aircraft Support
- `industrial_powerplant.md` — **Industrial Powerplant**: Power generation
- `propulsion_labs.md` — **Propulsion Labs**: Advanced aircraft
- `defense_system.md` — **Defense System**: Constructs defenses
- `missile_turret.md` — **Missile Turret**: Defensive turret
- `bunker.md` — **Bunker**: Field Engineer-built garrison
- `first_aid_tent.md` — **First Aid Tent**: Field Engineer-built healing
- `outpost.md` — **Outpost**: Prospector-deployed build radius
