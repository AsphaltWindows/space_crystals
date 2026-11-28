#!/bin/bash
# Usage: watch_budget.sh <agent-id> <calls>. The Director runs it in the background on each phase's launch, with <calls>
# at 80% of the phase's budget. It exits, printing one line, when the agent's transcript reaches <calls> API calls; the
# Director then tells the agent to finish at a green point and hand off. It gives up after 4 hours. Stop it with
# TaskStop when the agent finishes first.
ID=${1:?agent id}
N=${2:?calls}
T=~/.claude/projects/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/subagents/agent-$ID.jsonl
S=/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad
end=$((SECONDS + 4 * 3600))
while [ $SECONDS -lt $end ]; do
    c=$(python3 "$S/tools/ctx_stats.py" "$T" 2>/dev/null | grep -oE 'API calls [0-9]+' | grep -oE '[0-9]+')
    if [ -n "$c" ] && [ "$c" -ge "$N" ]; then
        echo "AGENT $ID AT $c API CALLS (threshold $N)"
        exit 0
    fi
    sleep 60
done
echo "AGENT $ID: the watch expired after 4 hours"
