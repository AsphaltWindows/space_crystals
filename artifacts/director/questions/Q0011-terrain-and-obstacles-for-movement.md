# Q0011: Terrain and obstacle rules for movement

- **Date**: 2026-11-23T15:48:00-05:00
- **Blocking**: no. Proceeding with the defaults below. The pathfinding work starts now on these defaults, and each rule is a single switch in the code.
- **Docs**: `core/map.md > Tile`, `> TilePreset`, `systems/units.md > UnitBase` (RuggedTerrain), `> UnitCollision`, `core/map.md > SpaceCrystalsPatch`, `> SupplyDeliveryStation`

## Question
1. Do crystal patches and Supply Delivery Stations block ground units? Collision names only units and "structures".
2. Is a Rugged tile impassable for a unit whose UnitBase has RuggedTerrain = false? The docs imply this but never state it.
3. When a unit plans a route, does it know about enemy structures its owner has never seen? Alternatively it plans as if unseen tiles were open, then finds out on arrival.
4. For later domains: do underground units collide with each other? Can air units cross every tile, including the map edge area?

## Options
1. **A (default)**: yes, their footprints block ground units. **B**: units walk over them.
2. **A (default)**: yes, impassable for those bases. **B**: passable, perhaps slower. Slower would need a speed rule.
3. **A (default)**: routes use the true map, so units go around unseen structures. This is simplest, and RTS players rarely notice. **B**: routes use only what the owner has seen, so units discover blocked routes and re-route. This is more realistic and costs more.
4. **Default**: underground units collide with each other the same way ground units do. Air units cross everything inside the map. This isn't needed for M1.
