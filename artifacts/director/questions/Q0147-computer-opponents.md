# Q0147 — Basic computer opponents for playtesting

- **Date**: 2026-11-26T13:22:49-05:00
- **Blocking**: no. The Director plans them as a playtest tool started from the command line, on Option (a)'s placeholders.
- **Docs**: none; the design has no computer players. Related: `core/game_rules.md > Defeat Condition`, `core/vision.md`.

## Question
The user wants basic AIs for the next playtest (2026-11-26): "A lot of timings are hard to test if switching between factions, so at that point i may as well be playing multiplayer." What should a computer player do, and what may it know?

## Context
What the Director plans (the technical side):
- A computer player can take any player's slot, for example `cargo run -- --factions cults,gdo --ai 2`. Two can play each other while the user watches.
- It gives the same commands a player gives, through the same rules, and the replay records them.
- The GDO's comes first, then the other three factions'.

The design points this raises:
1. **What it knows.**
   - **Fair:** it sees only what its own units see, and remembers buildings it has seen, as the fog does for a player. It also knows the map and where each player starts.
   - **Or** it knows the whole map at all times.
2. **How it plays.** Profiles, chosen on the command line:
   - **standard:**
     - its faction's opening, then workers up to what its field seats;
     - one or two production buildings, training its basic units without pause;
     - attacks in waves of about 8 units on the nearest enemy building it knows, then reinforces;
     - pulls its army home when its base is attacked.
   - **rush:** a small early army and an early first attack.
   - **passive:** builds its economy and army but never attacks, for racing economies.
3. **Build orders.** Which opening and units each faction plays. The starting point is the Director's scripted lines, for example the GDO's "Plate, Generator, Generator, Barracks" and the Cults' Two Halls line.
4. **What it won't do at first:**
   - spells and abilities (Shroud, the Shrine's casts, the Riot Shield);
   - expanding to a second field;
   - tech past its basic units;
   - difficulty levels.
5. **What it is:** a player-facing feature (a choice in game setup, difficulties, names), or a playtest tool only for now.

## Options
- **(a) A playtest tool for now** (the Director's default). It is built as above, with every behavior marked as a placeholder in the code. The Designer writes a design for computer players when they become a feature.
- **(b) The Designer specifies computer players now,** in a doc such as `core/computer_players.md`, and the Director builds to it. Until it arrives, the first slices proceed on (a)'s placeholders.
