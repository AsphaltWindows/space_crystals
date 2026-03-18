# Ticket: Tunnel ExpandMenu Hotkeys Not Functional

## Current State
After selecting a Tunnel and entering the Expand menu (B), keyboard hotkeys do not trigger expansion selection. Players must use the mouse to click expansion types, breaking the keyboard workflow that works consistently in other command panel states (DefaultState A/B/C, etc.).

## Desired State
Expansion types listed in the ExpandMenu respond to keyboard hotkeys following the standard command panel convention. Each expansion type button in the ExpandMenu should have an assigned hotkey (A, B, C, etc.) matching its position, consistent with how DefaultState commands work. The back/cancel hotkey (Z per `tickets/2026-03-06_back_button_hotkey_consistency.md`) should also work to return to DefaultState.

## Justification
Discovered during QA session 2026-03-08 (forum topic `qa_session_2026_03_08_issues.md`, issue #4). The Tunnel ObjectInterfaceState is specified in `features/syndicate_objects.md` (lines 56-81) and ticketed in `tickets/2026-03-06_tunnel_object_interface_state.md`. The ExpandMenu describes click-based interaction, but the standard command panel pattern uses keyboard hotkeys for all buttons. This inconsistency breaks the keyboard workflow and is likely a wiring bug in the ExpandMenu implementation rather than a design omission.

## QA Steps
1. [human] Select a Tunnel and press B to enter ExpandMenu — verify the menu opens
2. [human] Verify each expansion type button displays a hotkey label (A, B, C, etc.)
3. [human] Press the hotkey for an available expansion — verify AwaitingPlacement activates (same as clicking)
4. [human] Press Z (back) in ExpandMenu — verify return to DefaultState
5. [human] Press the hotkey for a greyed-out/unavailable expansion — verify nothing happens (no crash, no state change)

## Expected Experience
The ExpandMenu behaves like every other command panel state: buttons display hotkey labels, pressing the key activates the command. Players who use keyboard-driven workflows can navigate the full Tunnel interface without touching the mouse (B to enter expand, A/B/C to pick expansion type, arrow keys or mouse to place, Enter or click to confirm).
