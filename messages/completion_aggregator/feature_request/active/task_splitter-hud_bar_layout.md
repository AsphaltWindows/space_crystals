# hud_bar_layout

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

# hud_bar_layout

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added HUD Bar layout design to `artifacts/designer/design/control_system.md`. The bottom of the screen is a horizontal strip called the HUD Bar, divided into five fixed-position sections (left to right): Minimap (always visible), Portrait (visible with 1+ selected), InfoPanel (visible with 1+ selected), SelectionPanel (visible with 2+ selected), CommandPanel (visible with 1+ player-owned selected). All sections share the same vertical height. Positions are locked — panels appear/disappear in place but never shift when adjacent panels show or hide. Empty slots remain as reserved space.

## QA Instructions

1. Launch the game and observe the bottom of the screen — the HUD Bar should be visible as a horizontal strip.
2. With nothing selected, only the Minimap section should show content. The other four slots should be present as empty reserved space (no content, but space is held).
3. Select one player-owned unit — Minimap, Portrait, InfoPanel, and CommandPanel should all show content. SelectionPanel slot should remain empty but in its fixed position.
4. Select two or more player-owned units — all five sections should now show content.
5. Select a single enemy or neutral object — Minimap, Portrait, and InfoPanel should show. CommandPanel should be hidden (empty slot). SelectionPanel should be hidden (empty slot).
6. Deselect everything — verify that only Minimap shows content and no panels shift position during any transition.
7. Rapidly change selection states (single unit, multi unit, enemy, nothing) and confirm no panel ever slides or shifts horizontally — they always appear/disappear in their fixed slots.
