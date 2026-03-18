# Feature Update: Camera and Viewport

**Date**: 2026-03-07
**Feature file**: `features/camera_and_viewport.md`
**Design sources**: `design/camera.md`

## Summary

New feature file created for camera and viewport behavior.

## Modifications

- **New feature file**: `features/camera_and_viewport.md`
- Fixed zoom at 28 GridUnits horizontal (1792 SpaceUnits)
- No zoom controls
- Vertical coverage derived from aspect ratio minus HUD
- HUD layout: resource bar top, info/selection/command panel bottom, no side panels

## Open Questions Raised
- Panning method not yet designed (deferred)
- Minimum resolution and HUD sizing unspecified
