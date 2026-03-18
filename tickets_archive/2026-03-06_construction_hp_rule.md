# Ticket: Implement ConstructionHP Rule for On-Site Structure Construction

## Current State
Structures that are built on-site (e.g., Syndicate Tunnels built by Agents) do not have a progressive HP system during construction. There is no mechanism to give a partially-built structure reduced HP proportional to its construction progress.

## Desired State
An opt-in ConstructionHP rule for Structure Types that are built on-site:

- **Formula**: HP during construction = `MaxHP x (10% + 90% x construction_progress)` where `construction_progress` is a value from 0.0 (just started) to 1.0 (complete).
- Structure starts at 10% of MaxHP when construction begins.
- HP increases linearly as construction progresses, reaching full MaxHP at completion.
- Partially-built structures can be attacked and destroyed before completion.
- This rule is opt-in — only structures that explicitly reference it use this behavior. Currently referenced by: Syndicate Tunnel (Agent construction flow).

## Justification
The entity system feature (`features/entity_system.md`, ConstructionHP Rule section) specifies this as an opt-in rule for structures built on-site. The Syndicate Agent construction flow (in `features/syndicate_objects.md`) already references this rule. Feature update: `feature_updates/2026-03-06_entity_system_construction_hp.md`.

## QA Steps
1. Begin constructing a Tunnel with a Syndicate Agent at an eligible location.
2. Immediately after construction starts, inspect the Tunnel's HP — verify it is approximately 10% of MaxHP (e.g., 60 HP for a T1 Tunnel with 600 MaxHP).
3. At approximately 50% construction progress, verify HP is approximately 55% of MaxHP (10% + 90% x 0.5 = 55%, e.g., ~330 HP for 600 MaxHP).
4. Attack the partially-built structure — verify it takes damage and its HP decreases.
5. Destroy the partially-built structure before completion — verify it is destroyed (removed from game) and the constructing Agent is freed (per Agent construction flow rules).
6. Build another structure to full completion — verify HP reaches full MaxHP upon completion.
7. Verify that structures NOT using the ConstructionHP rule (e.g., GDO buildings placed via Deployment Center) are unaffected — they should spawn at full HP immediately.

## Expected Experience
A newly started construction appears fragile — at only 10% HP, it can be destroyed quickly by an attacker. As construction progresses, the structure becomes increasingly durable. An attacker must decide whether to commit to destroying a partially-built structure early (when it's weak) or risk it completing (when it reaches full durability). Structures that don't use this rule behave unchanged.
