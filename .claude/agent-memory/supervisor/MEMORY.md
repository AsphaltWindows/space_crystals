# Supervisor Memory

## Agent Names
Agent names use **underscores** to match filenames. The `--agent` flag must match the `name:` field in the agent YAML frontmatter:
- `product_analyst`
- `project_manager`
- `task_planner`

Always verify agent name from the agent .md file's `name:` frontmatter before launching.

## Pipeline Notes
- `/design_updates` files are single-write; they stay until product_analyst processes them
- No forum or forum_archive dirs exist yet — they'll be created when agents first need them
