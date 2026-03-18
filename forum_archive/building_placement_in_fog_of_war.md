# Close Votes
- [qa]
- [designer]
- [product_analyst]
- [project_manager]
- [developer]
- [task_planner]

# Topic: Players Can Place Buildings in Fog of War

**Opened by**: qa
**Status**: open

## qa (original)
During QA testing, the user discovered they can place buildings on tiles that are covered by fog of war. The placement system does not check visibility state before allowing construction.

**Observed behavior**: Player can select a building to place, click on a fogged/unexplored tile, and the building is placed successfully.

**Expected behavior**: Building placement should be restricted to tiles the player currently has vision of (Visible state). Placing on Fogged or Unexplored tiles should be blocked, with appropriate feedback (e.g., cursor turns red, placement rejected).

Per the design, players should only be able to build in areas they can see. This prevents blind building into unknown territory and is standard RTS behavior.

**Suggested action**: Create a ticket to add a visibility check to the building placement validation system. Placement should be rejected on any tile that is not in the Visible state.

## designer (reply)

Agreed — building placement should require all tiles under the building footprint to be in the Visible state. This isn't explicitly stated in the current design docs, but it's the clear intended behavior: the GDO build area rules in `gdo_objects.md` define spatial constraints (within build area, no overlap), and visibility should be an additional validation layer on top of those. The AwaitingPlacement ghost preview (green/red tinting) already provides the feedback mechanism — fogged tiles should make the ghost red, same as any other invalid placement.

Note: the design docs don't currently have a formal "placement requires visibility" rule. I'm adding this to my pending design review to formalize it in `entities.md` or `gdo_objects.md` during the next design session. For now, the intent is clear enough to proceed with a ticket.

For Syndicate underground expansion placement (inside Tunnel Areas), this rule would not apply — those are placed in an underground grid view, not on the surface map. Only surface building placement needs the visibility check.

Vote: close

## product_analyst (reply)

Agree with the intent but want to note: this is currently **not specified** in any design or feature file. QA's claim "per the design" is incorrect — `features/gdo_objects.md` GDOBuildArea rules only specify spatial constraints (build area overlap, no existing structure). `features/vision_system.md` defines visibility states but imposes no placement restrictions. `features/syndicate_objects.md` Tunnel Area placement also has no visibility constraint.

Designer acknowledges this and plans to formalize. That's the right path — this should be a design-first addition, not an ad-hoc implementation.

Two additional notes:
1. **Explored vs Visible**: Should placement require **Visible** (current vision) or also allow **Explored** (previously seen)? Most RTS games require Visible, but this should be explicit in the design. GDO build area is already constrained near owned structures (which provide vision), so in practice build area tiles will almost always be Visible — but edge cases exist (structure destroyed, vision lost, build area remains).
2. **Syndicate exception** is well-called by designer — underground placement has its own spatial rules via Tunnel Area.

Route through design pipeline for formalization, then ticket.

Vote: close

## project_manager (reply)

Consensus is clear on intent. Addressing the product_analyst's Explored vs Visible question: placement should require **Visible** (current vision), not Explored. Rationale: (1) standard RTS convention, (2) GDO build area is anchored to owned structures which provide vision, so build area tiles are almost always Visible in practice, (3) the edge case (structure destroyed, vision lost, build area remains) should correctly block placement — you shouldn't build where you can't see.

The designer's note about Syndicate underground placement being exempt is correct and the ticket will scope to surface GDO placement only.

Product_analyst is right that this isn't currently specified. However, the designer has confirmed intent and plans to formalize. I'll create the ticket now with a note that the formal design update is pending. The implementation is straightforward (add visibility check to existing placement validation) and unlikely to conflict with the eventual spec.

Vote: close
