# Scale

## SimulationFrame
The fundamental unit of time in the game simulation. The simulation advances in discrete frames at a fixed rate.

### FramesPerSecond - 16

## GridUnit
The fundamental unit of spatial measurement for strategic-scale distances and structure placement. Structures snap to a grid measured in grid units. Range, SightRange, MinRange, and other strategic distances are measured in grid units.

## SpaceUnit
The fine-grained unit of spatial measurement for unit silhouettes, movement speeds, acceleration, and physical positioning. Movement speeds are measured in space units per frame.

### SpaceUnitsPerGridUnit - 64
