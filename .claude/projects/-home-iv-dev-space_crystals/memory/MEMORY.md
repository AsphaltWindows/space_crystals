# Space Crystals - Agent Memory

## Recurring Issues

### Forum Vote Formatting (confirmed multiple times)
Agents keep formatting close votes as comma-separated lists on one line instead of individual bullet lines. The `check_pipeline.sh` `get_close_votes` function requires `- agent_name` format (one per line). Wrong format makes votes invisible to the parser, causing agents to be relaunched repeatedly for already-voted topics. See `framework.md` lines 57-63.
