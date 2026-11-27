# Camera

## Zoom

Fixed zoom level. The camera displays exactly 28 GridUnits horizontally. No zoom in or zoom out controls are available.

The number of GridUnits visible vertically is determined by the screen's aspect ratio minus the vertical space occupied by the HUD (top resource bar + bottom panel).

## Panning

### Middle-mouse grab
Holding the **middle mouse button** grabs the map: while held, the map follows the cursor (the point grabbed stays under the cursor), like dragging a page. Releasing the button stops panning. (User, 2026-11-23.)

### Arrow keys and screen edge
The **arrow keys** and moving the cursor to the **screen edge** also pan the camera, at a fixed speed (user, 2026-11-23). Speed value: TBD (tuning).

**The screen edge is the game window's edge** (designer, 2026-11-26, at the user's request after a playtest: in a window the cursor slid into the title bar and the panning stopped):
- **Scroll margin**: the camera pans while the cursor is within **8 pixels** of the window's edge (placeholder), so the edge is easy to hit.
- **The cursor stays in the game window** while the game is the active window, in full screen and in a window alike (with several monitors, full screen would otherwise let it slip onto the next screen). It is freed when the game stops being the active window (Alt-Tab, or the system switching windows); clicking back into the game confines it again.

## Window Mode
(user, 2026-11-26) The game starts in **borderless full screen** by default. **F11** or **Alt+Enter** switches between full screen and a window (keys: placeholder). In a window, the Screen edge rules above keep panning working.

### Centering
The camera pans via standard pan (currently implemented as instant snap to the target position). When the camera centers on a location — such as via Alt-click portrait in the SelectionPanel — it snaps instantly so the target is at the center of the viewport.

## Starting Position

Each player slot on a map can have an explicit starting camera position. At game start, the camera centers on:

1. The **map-defined starting camera position** for the local player's slot, if one is set.
2. Otherwise, the local player's **primary starting structure**:
   - GDO: Deployment Center
   - Syndicate: starting Tunnel
   - Cults: starting Recruitment Hall
   - Colonists: starting Robo Portal

## HUD Layout

- **Top**: Resource bar
- **Bottom**: Info/Selection/Command panel
