#!/usr/bin/env bash
# Launch the Director session with its notification hooks.
cd "$(dirname "$0")/.."
exec claude --agent director --settings .claude/director.settings.json "$@"
