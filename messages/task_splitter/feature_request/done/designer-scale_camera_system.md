# scale-camera-system

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the foundational scale and camera systems as defined in `artifacts/designer/design/scale.md` and `artifacts/designer/design/camera.md`.

**Scale System (`scale.md`):**

- **SimulationFrame**: The game simulation advances in discrete frames at a fixed rate of **16 frames per second**.
- **GridUnit**: The fundamental unit of spatial measurement for strategic-scale distances and structure placement. Structures snap to a grid measured in grid units. Range, SightRange, MinRange, and other strategic distances are measured in grid units.
- **SpaceUnit**: Fine-grained unit for unit silhouettes, movement speeds, acceleration, and physical positioning. Movement speeds are measured in space units per frame. **64 SpaceUnits = 1 GridUnit**.

**Camera System (`camera.md`):**

- **Fixed Zoom**: The camera displays exactly **28 GridUnits horizontally**. No zoom in/out controls.
- The number of GridUnits visible vertically is determined by the screen's aspect ratio minus the vertical space occupied by the HUD (top resource bar + bottom panel).
- **HUD Layout**: Top = Resource bar, Bottom = Info/Selection/Command panel.

## QA Instructions

1. Verify the simulation runs at 16 frames per second (not render frames — simulation ticks).
2. Place a structure and verify it snaps to the grid in GridUnit increments.
3. Verify that 64 SpaceUnits equals 1 GridUnit (e.g., a unit with MaxSpeed of 64 space units/frame should traverse exactly 1 grid unit per frame).
4. Verify the camera shows exactly 28 GridUnits horizontally at any screen width.
5. Verify no zoom in/out controls exist (scroll wheel, pinch, etc. should do nothing).
6. Verify the HUD has a resource bar at the top and info/selection/command panel at the bottom.
7. Resize the window — verify the horizontal extent stays at 28 GridUnits and the vertical extent adjusts based on aspect ratio minus HUD space.
