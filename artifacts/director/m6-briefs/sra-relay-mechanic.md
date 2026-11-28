# Task SRa: The Relay, as a sim mechanic (phase 1 of M6-SR)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, in `crates/sim` only, and your budget is 120 tool calls, a hard limit.** Phase SRb, after you, builds the client.

## Where things stand
- **The Laser Battery** (SLa, SLb, SLc):
  - Its deployed type, `LaserBatteryDeployed`, is immobile, with a 360° turret and Air targets only.
  - The battery is its `EnergyDef`. The surge is `SurgeDef` on `AttackAttrs::surge`, with `crate::surge` (`used`, `damage`, `spend`).
  - Amplify is `Command::Ability(Ability::Amplify)`, and its mark is `Object::amplified`.
  - SLc's handoff: FILLED IN AT LAUNCH.
- **The Prism** (SP): a unit launched from a Tunnel's air door, with Thrust and the one-way rule. It has no attack yet.
- **Structures that attack** pick their targets in `combat/structures.rs` (plan decision T49). The armed Prism picks as they do.

## The design
- `factions/syndicate/units/laser_battery.md > Relay (ability, deployed only)`, whole, and the Relay line of its **Placeholder stats** paragraph: Battery-to-Prism range 8 GU, Prism-to-Prism link range 8 GU, the armed Prism's attack Range 6 GU.
- `factions/syndicate/units/prism.md`: what it says about being armed (`grep -n -i 'armed\|relay' ...`).

## Goal
Shape everything by def parts, so that nothing tests a unit's kind.
1. **The def parts:**
   - a relaying part on the deployed Battery's def (its reach to a Prism);
   - a relay part on the Prism's def (its link range, and the armed attack: Range 6 GU, ground and air, FullyConnected);
   - cite each number; add the X1/X2 tests for the new fields.
2. **The order:** `OrderKind::Relay { prism }`, in new `orders/relay.rs`, hashed as **`OrderKind` tag 40** (`orders/hash.rs`). The gate checks:
   - every unit given it has the relaying part, else `Reject::LacksAbility`;
   - every selected Battery has a Prism of its owner within reach;
   - the target is reachable from every one of them, through the owner's Prisms, by links within range: a breadth-first search.
   
   Add a new `Reject`, with its text, for a target that can't be reached.
3. **The chain,** in new `relay.rs`: derive it each tick from positions, only for players with a relay running.
   - Each Battery locks onto its nearest Prism, then follows the search's path to the target.
   - **A broken chain** (a Prism on it destroyed, or a link out of range) ends the order, and the Batteries go back to ordinary deployed behaviour.
4. **While relaying,** the Batteries are locked in. They make no attacks of their own and take no other orders, except:
   - Stop, which ends the relay;
   - Relay to another target;
   - Amplify, only while the target Prism has an active target. Refuse it otherwise, with a `Reject`.
5. **The armed Prism** is armed while a relay targets it.
   - It attacks anything its player sees within its range, ground or air, picking as T49's structures do, at the surge cadence.
   - Each of its surges deals the sum of each relaying Battery's own surge, from that Battery's own charge (`surge::used` and `damage`, Amplify's mark included). It spends each Battery's charge (`surge::spend`).
6. **Directing it:** an Attack given to an armed Prism holds over its own picks. Refuse it to a Prism that isn't armed.
7. **The hash and saves:** derive what can be derived. New `Object` state (the lock, the armed state) hashes and saves only while present, under **Object tag 32**.
8. **Perf:** the chain costs work only for players with a relay. If it adds per-tick work for every unit, measure `movement_perf` against master's binary (`target/perf/<master's hash>`).
9. **The golden `syndicate_relay`,** made as the other Syndicate goldens are (`crates/sim/tests/golden/main.rs`):
   - two deployed Batteries relay through a chain of three Prisms, and the armed Prism fires at a Peacekeeper;
   - a Prism on the chain is then destroyed, which ends the relay.
   
   No other golden may move.

## Tests
In a new `crates/sim/tests/relay/`:
- **the gate:** with two Batteries and a chain of three Prisms, the end of the chain can be targeted, and a Prism off it can't. If one Battery has no Prism within reach, no Prism can be targeted;
- **the armed Prism** hits a Peacekeeper 6 GU away that any own unit sees. Each surge is the sum of the Batteries' surges, and each Battery's charge falls;
- **Amplify** while relaying, with an active target: every Battery's whole charge goes into one surge. Without an active target, it is refused;
- **a broken chain:** a Prism on it destroyed, or a link out of range, ends the relay, and the Batteries go back to their deployed behaviour;
- **directing:** an Attack on an armed Prism holds over its picks. A Prism that isn't armed refuses it;
- **while relaying,** the Batteries never fire on their own.

## Don't
- No client: no panel, targeting, highlights or beams. Those are SRb's.
- Don't change the surge, Amplify or the battery (SLa's), or the Prism's thrust and launch (SP's), beyond what the relay reads.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff** for SRb, at most 6 lines:
- the order, and its `Reject`s with their texts;
- what the client reads to highlight the reachable Prisms and draw the beams;
- the armed state;
- where a relayed surge's hit lands, for SC's smoke rule (0433: a Battery firing from smoke is revealed for 1.5 s after each hit);
- the hash tags used.
