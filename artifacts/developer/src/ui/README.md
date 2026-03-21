# ui/

User interface systems for Space Crystals RTS.

## Structure

- **types.rs** — HUD component markers and state types: HudPanel, MinimapSection, UnitsGridSection, MinimapTile, MinimapUnit, MinimapContainer, UnitIcon, UnitHealthBar, ResourceBarField (per-faction resource display), StructureIcon, StructureHealthBar, CommandPanelSection, ObjectInterfaceState (generalized state machine replacing CommandPanelState), StructureMenuState, CursorTarget/CursorTargetEnum (per-frame cursor classification), InterfaceTransition, CommandButtonAction, CommandPanelTarget, PointerDisplayType
- **utils.rs** — UI helpers: health bar coloring, minimap tile coloring, cursor-over-UI tracking, ray-AABB intersection for 3D entity click detection
- **hud.rs** — HUD setup, minimap rendering, selected units/structures grid display, faction-aware resource bar (GDO/Syndicate/Cults/Colonists)
- **command_panel.rs** — Command panel: structure build menus, production controls, construction progress, button interactions, unit command grid (CommonCommands vs GroupCommands), cursor target updates, Tab group cycling
- **pointer.rs** — Pointer display indicator: colored overlay that tracks cursor position, updates color based on PointerDisplayType (move/attack/patrol/gather/enter etc.), hidden during placement mode and when cursor is over UI
- **menu/** — Faction selection menu (game start screen)
