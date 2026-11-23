#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PIPELINE="$ROOT_DIR/pipeline.yaml"

LOCKFILE="$ROOT_DIR/.scheduler.lock"
exec 9>"$LOCKFILE"
if ! flock -n 9; then
    echo "Another scheduler is already running for this project ($ROOT_DIR). Exiting."
    exit 1
fi

declare -A RUNNING_PIDS
declare -A NODE_COMMANDS
declare -A NODE_CLOSE_VOTES

while true; do

# Parse pipeline.yaml once per cycle
NODES=""
NODE_COMMANDS=()
NODE_CLOSE_VOTES=()
INTERVAL="20"

IN_NODES=false
CURRENT_NAME=""

while IFS= read -r line; do
    # Top-level keys
    if echo "$line" | grep -q '^[a-z_]*:'; then
        # Flush previous node when leaving nodes section
        if $IN_NODES && [ -n "$CURRENT_NAME" ]; then
            NODES="$NODES $CURRENT_NAME"
        fi
        if echo "$line" | grep -q '^nodes:'; then
            IN_NODES=true
        else
            IN_NODES=false
        fi
        if echo "$line" | grep -q '^scheduler_interval:'; then
            INTERVAL=$(echo "$line" | sed 's/.*scheduler_interval:\s*//' | tr -d ' ')
        fi
        CURRENT_NAME=""
        continue
    fi
    $IN_NODES || continue

    if echo "$line" | grep -q '^\s*-\?\s*name:'; then
        # Flush previous node
        if [ -n "$CURRENT_NAME" ]; then
            NODES="$NODES $CURRENT_NAME"
        fi
        CURRENT_NAME=$(echo "$line" | sed 's/.*name:\s*//' | tr -d ' ')
    elif [ -n "$CURRENT_NAME" ]; then
        if echo "$line" | grep -q '^\s*scheduled:'; then
            val=$(echo "$line" | sed 's/.*scheduled:\s*//' | tr -d ' ')
            if [ "$val" = "false" ]; then
                CURRENT_NAME=""
            fi
        elif echo "$line" | grep -q '^\s*command:'; then
            NODE_COMMANDS[$CURRENT_NAME]=$(echo "$line" | sed 's/.*command:\s*//')
        elif echo "$line" | grep -q '^\s*close_vote_required:'; then
            NODE_CLOSE_VOTES[$CURRENT_NAME]=$(echo "$line" | sed 's/.*close_vote_required:\s*//' | tr -d ' ')
        fi
    fi
done < "$PIPELINE"
# Flush final node
if $IN_NODES && [ -n "$CURRENT_NAME" ]; then
    NODES="$NODES $CURRENT_NAME"
fi

INTERVAL="${INTERVAL:-20}"

if [ -z "$NODES" ]; then
    echo "No scheduled nodes found."
    sleep "$INTERVAL"
    continue
fi

for NODE in $NODES; do
    # Check if node is already running
    if [ -n "${RUNNING_PIDS[$NODE]:-}" ]; then
        if kill -0 "${RUNNING_PIDS[$NODE]}" 2>/dev/null; then
            echo "[$NODE] Already running (PID ${RUNNING_PIDS[$NODE]}), skipping."
            continue
        else
            unset "RUNNING_PIDS[$NODE]"
        fi
    fi

    HAS_WORK=false

    # Check forum topics if close_vote_required is not false
    if [ "${NODE_CLOSE_VOTES[$NODE]:-}" != "false" ]; then
        for TOPIC in "$ROOT_DIR"/forum/open/*.md; do
            [ -f "$TOPIC" ] || continue
            if ! grep -q "^VOTE:${NODE}$" "$TOPIC"; then
                HAS_WORK=true
                break
            fi
        done
    fi

    # Check pending messages across all message-type subdirectories
    if [ "$HAS_WORK" = false ]; then
        NODE_MSG_DIR="$ROOT_DIR/messages/${NODE}"
        if [ -d "$NODE_MSG_DIR" ]; then
            for TYPE_DIR in "$NODE_MSG_DIR"/*/pending; do
                [ -d "$TYPE_DIR" ] || continue
                if ls "$TYPE_DIR"/*.md &>/dev/null; then
                    HAS_WORK=true
                    break
                fi
            done
        fi
    fi

    if [ "$HAS_WORK" = false ]; then
        echo "[$NODE] No pending work."
        continue
    fi

    echo "[$NODE] Work found, launching."

    NODE_CMD="${NODE_COMMANDS[$NODE]:-}"
    if [ -z "$NODE_CMD" ]; then
        echo "[$NODE] No command defined in pipeline.yaml, skipping."
        continue
    fi

    # Launch node in background, detached from terminal
    mkdir -p "$ROOT_DIR/logs"
    (
        exec 9>&-
        cd "$ROOT_DIR"
        export NODE_ROOT="$ROOT_DIR"
        export NODE_NAME="$NODE"
        eval "exec $NODE_CMD"
    ) </dev/null &>>"$ROOT_DIR/logs/${NODE}.log" &
    RUNNING_PIDS[$NODE]=$!

done

echo "Scheduler pass complete."
sleep "$INTERVAL"
done
