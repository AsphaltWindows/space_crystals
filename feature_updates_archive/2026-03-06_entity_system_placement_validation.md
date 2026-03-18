# Feature Update: Entity System — Placement Validation

**Date**: 2026-03-06
**Feature file**: `features/entity_system.md`
**Design sources**: `design/entities.md` (Placement Validation section)
**Triggered by**: `design_updates/2026-03-06_pending_review_formalization.md`

## Summary

Added Placement Validation section to entity_system feature spec, defining two validation models for structure placement:

1. **Direct Placement** (GDO buildings, Tunnel underground expansions): All footprint tiles validated at confirmation. Surface buildings require Visible state; underground expansions use underground spatial rules only. Standard spatial checks (Buildable, no overlap, faction constraints) apply.

2. **Worker-Built Structures** (Agent building Tunnel): Build command queued without visibility check. Validation on worker arrival (Buildable, unoccupied, faction constraints). Failure cancels command; worker idles. No visibility requirement.
