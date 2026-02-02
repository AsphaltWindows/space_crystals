# Running Space Crystals RTS

**Last Updated**: 2026-02-01

## Quick Start

```bash
cd /home/iv/dev/space_crystals
cargo run
```

## Build Commands

### Development Mode (Default)
```bash
cargo run              # Build and run
cargo build            # Build only
./target/debug/space_crystals  # Run existing build
```

### Release Mode (Optimized)
```bash
cargo run --release    # Slower build, faster runtime
cargo build --release
./target/release/space_crystals
```

### Other Useful Commands
```bash
cargo check            # Fast syntax checking without full build
cargo clean            # Remove build artifacts (WARNING: slow rebuild)
```

## Build Configuration

- **Dynamic Linking**: Enabled by default for fast incremental builds
- **Build Time**: 3-4 seconds (incremental)
- **Dependencies**: Bevy 0.14 game engine
- **Optimization**: opt-level 1 for project, opt-level 3 for dependencies

## Game Window

- **Title**: "Space Crystals RTS"
- **Resolution**: 1280x720 pixels
- **Fullscreen**: No (windowed mode)

## In-Game Controls

### Camera Controls
| Key | Action |
|-----|--------|
| W / ↑ | Move camera forward |
| S / ↓ | Move camera backward |
| A / ← | Move camera left |
| D / → | Move camera right |
| Q | Zoom out (camera up) |
| E | Zoom in (camera down) |

### Selection Controls
| Input | Action |
|-------|--------|
| Left Click | Select entity (unit or resource) |
| Ctrl + Left Click | Toggle entity in selection (multi-select) |
| Click + Drag | Create drag-box to select multiple entities |
| Click Empty Space | Deselect all |

## Current Game State

### Map
- 20x20 grid (400 tiles)
- 5 tile types with color-coding:
  - **Plane** (light green): Buildable, traversible
  - **Rugged Terrain** (brown): Not buildable, traversible, rugged
  - **Cliff** (gray): Not buildable, not traversible
  - **Mountain** (dark gray): Fully impassable
  - **Water** (blue): Not traversible, not recruitable

### Resources
- 4 Space Crystal Patches
- Glowing cyan/blue crystals
- Amounts: 5000, 3500, 2000, 4200

### Units
- 5 test units spawned
- 2 Player 0 units (blue)
- 2 Player 1 units (red)
- 1 Neutral unit (gray)
- Mix of infantry (capsules) and vehicles (cubes)

## System Information

**Current System**:
- OS: Linux (kernel 6.8.0-90-generic)
- GPU: NVIDIA GeForce GTX 1050 with Max-Q Design
- Graphics API: Vulkan
- Driver: NVIDIA 535.288.01

**Requirements**:
- Rust toolchain (cargo 1.93.0+)
- Graphics card with Vulkan support
- ~500MB disk space for dependencies and build artifacts

## Troubleshooting

### "No space left on device" error
- Check disk space: `df -h`
- Clean old builds if needed: `cargo clean` (WARNING: requires full rebuild)

### Window doesn't appear
- Check GPU drivers are installed
- Ensure Vulkan is available: `vulkaninfo` (if installed)

### Build fails
- Ensure Rust is up to date: `rustup update`
- Check Cargo.toml hasn't been modified incorrectly
- Try `cargo clean && cargo build`

## Console Logging

The game logs useful information to the console:
- Grid and tile spawn confirmation
- Resource and unit spawn counts
- Selection changes with entity counts
- Tile and entity information when clicked

## Performance Notes

- **Frame Rate**: Should be smooth on current hardware
- **Build Performance**: Dynamic linking provides ~4x faster rebuild times
- **Memory**: Bevy manages entity memory efficiently via ECS
- **Entity Count**: Currently 409 entities (400 tiles + 4 resources + 5 units)

## Future Enhancements

As development continues, this document will be updated to reflect:
- Additional controls (unit commands, building placement, etc.)
- New game modes or scenarios
- Performance optimization tips
- Multiplayer connection instructions (if applicable)
