# Art Direction

Decided with the user 2026-11-23 (answering Director question Q0001). The game is **2.5D pixel art / sprites**, in the spirit of StarCraft 1 and Age of Empires 2 (user's reference).

## Projection
**Square tile grid** (not an isometric diamond grid). The map is laid out on the same axis-aligned square grid as the simulation (core/scale.md > GridUnit). Sprites are drawn in a **3/4 (top-down, tilted) view**: objects show their top and their camera-facing (south) side.

## Sprite Production
Sprites are **pre-rendered from 3D models**: each unit and building is modeled in 3D, then rendered from the fixed game camera angle into pixel-art sprites. This is what makes the orientation and facing counts below affordable: an extra orientation or facing is just another render of the same model, so all orientations stay consistent with each other.

## Native Pixel Scale
- **NativePixelsPerTile - 32** (1 native pixel = 2 space units).
- The camera shows 28 grid units across (core/camera.md > Zoom), so native art is scaled up on screen: about 2.1× at 1920 px wide, 2.9× at 2560 px, 4.3× at 3840 px. At every common resolution the pixels stay visible, which keeps the pixel-art look. (The scaling method is a technical choice.)

## Building Orientations
All placement orientations stay available to the player (core/entities.md > Structure Type: 4 rotations × flip, up to 8 for ABCD). Because sprites are rendered from 3D models, **each distinct orientation gets its own render**. A flip is rendered from the mirrored model, so lighting stays correct, and no orientation is restricted for art reasons.

Distinct orientations per SymmetryType (= number of renders needed): AAAA 1, ABAB 2, AABB 4, AAAB 4, ABAC 4, AABC 8, ABCD 8.

## Unit Facings
A unit's facing is continuous in the simulation (turn rates, directional armor). Sprite facings only decide how smooth turning looks and have no gameplay effect.
- **Infantry: 16 facings.**
- **Vehicles, turrets and aircraft: 32 facings.** They turn slowly and visibly, so turret tracking and wheeled turning arcs need finer steps.

Every animation (move, attack, death, etc.) is rendered at the unit's facing count. Rendering each facing directly (rather than mirroring) keeps lighting consistent. Mirroring may be used as an optimization only if it doesn't visibly break the lighting.

## Elevation Display
AoE2-like sloped terrain (user's reference), built on corner heights (core/map.md > Elevation):
- **Slopes** are drawn as tilted terrain and **shaded by the direction they face**: lighter facing the light, darker facing away. This is how hills read on screen.
- **Higher terrain is drawn shifted up the screen** in proportion to its height. Units and structures are drawn at the terrain height of their position, so a unit climbing a hill visibly rises.
- **Cliffs** (non-Traversible tiles spanning a large height difference) are drawn as cliff faces.
- Vertical screen offset per height level: TBD (art tuning).
