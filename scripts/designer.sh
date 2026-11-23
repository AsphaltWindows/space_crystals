#!/usr/bin/env bash
# Launch the Designer session with its notification hooks.
cd "$(dirname "$0")/.."
exec claude --agent designer --settings .claude/designer.settings.json "$@"
