# world/

World systems including map/tiles, resource nodes, factions, entity selection, and fog of war.

## Structure

- **types.rs** — TilePresetEnum, TilePreset, TilePlacement, Tile, GridMap, SpaceCrystalPatch, SupplyDeliveryStation, SelectionState, SelectionIndicator, DragBoxUI, FogOfWarMap, LastKnownStructures, ElevationMap, LastRecallState
- **utils.rs** — Map-space grid conversion, screen-space hit testing, box-selection priority helpers (BoxCandidate, SelectionTier, closest_to_center, classify_selection_tier), build placement validation, underground expansion placement validation
- **map.rs** — Grid spawning, tile type generation, tile hover system, fog of war vision update and rendering systems
- **faction.rs** — Player resource initialization, power grid, construction, production, placement, mining, tunnel construction/upgrade systems
- **resources.rs** — Resource node spawning, click/drag-box selection with 5-tier priority, control groups, SDS delivery timer
