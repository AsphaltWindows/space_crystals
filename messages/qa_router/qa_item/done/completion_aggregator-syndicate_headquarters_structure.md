# syndicate_headquarters_structure

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# syndicate-headquarters-structure

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the Headquarters underground expansion structure stats as defined in `artifacts/designer/design/syndicate_objects.md` under Headquarters.

**NOTE: The HQ production interface was already sent (designer-hq-production-interface). This request covers the STRUCTURE STATS and construction details.**

The Headquarters is a Tier 1 Tunnel expansion that produces Agents and Guards. The Syndicate player starts with one pre-built in their starting Tunnel.

**Entity Type:** Structure Type (Underground)
**Faction:** TheSyndicate

**Stats:**
- Size: 2x2 grid units
- Tier Requirement: 1 (can be built in any Tier 1+ Tunnel)
- Cost: 200 Space Crystals
- Build Time: 400 frames (25 seconds)
- HP: 400
- PointArmor: 1
- FullArmor: 4

**HeadquartersInstanceState:**
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 5)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

**Production Catalog:**
| Unit | Cost | Build Time |
|------|------|-----------|
| Agent | 100 Space Crystals | 160 frames (10s) |
| Guard | 125 Space Crystals | 120 frames (7.5s) |

**Starting Condition:** Syndicate player begins with one pre-built Headquarters in their starting Tunnel.

## QA Instructions

1. Start as Syndicate — verify a pre-built Headquarters exists in the starting Tunnel.
2. Build a new Headquarters from a Tunnel's ExpandMenu — verify 200 crystals deducted, 25-second build time.
3. Verify HQ occupies 2x2 grid units within the Tunnel Area.
4. Verify HQ has 400 HP, 1/4 armor.
5. Queue an Agent (Q) — verify 100 crystals deducted, 10-second build time.
6. Queue a Guard (W) — verify 125 crystals deducted, 7.5-second build time.
7. Fill the build queue to 5 — verify further production commands are unavailable.
8. Cancel last queue entry (X) — verify full cost refunded.
9. Verify HQ can be built in any Tier 1+ Tunnel (not restricted to higher tiers).
10. Verify HQ is an underground structure (invisible to enemies without detection).
