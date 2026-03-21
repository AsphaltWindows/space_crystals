# ui/

User interface systems for Space Crystals RTS.

## Structure

- **types.rs** — HUD component markers and state types: HudPanel, MinimapSection, UnitsGridSection, MinimapTile, MinimapUnit, MinimapContainer, UnitIcon, UnitHealthBar, ResourceBarField (per-faction resource display), StructureIcon, StructureHealthBar, CommandPanelSection, ObjectInterfaceState (generalized state machine replacing CommandPanelState), StructureMenuState, CursorTarget/CursorTargetEnum (per-frame cursor classification), InterfaceTransition, CommandButtonAction, CommandPanelTarget, PointerDisplayType, PointerIndicator
- **utils.rs** — UI helpers and utility systems: health bar coloring, minimap tile coloring, cursor-over-UI tracking, ray-AABB intersection for 3D entity click detection, pointer indicator spawn/update systems with color mapping
- **hud.rs** — HUD setup, minimap rendering, selected units/structures grid display, faction-aware resource bar (GDO/Syndicate/Cults/Colonists)
- **command_panel.rs** — Command panel: structure build menus, production controls, construction progress, button interactions, unit command grid (CommonCommands vs GroupCommands), cursor target updates, Tab group cycling, pointer display type resolution
- **menu.rs** — Faction selection menu (game start screen)
