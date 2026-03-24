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

# Parse a field value for the current node from pipeline.yaml
# Reads from the block of lines between the node's "- name:" and the next "- name:" or end of nodes section
get_node_field() {
    local node_name="$1"
    local field="$2"
    local in_node=false
    local in_nodes=false

    while IFS= read -r line; do
        if echo "$line" | grep -q '^nodes:'; then
            in_nodes=true
            continue
        fi
        # Exit nodes section on next top-level key
        if $in_nodes && echo "$line" | grep -q '^[a-z_]*:'; then
            break
        fi
        $in_nodes || continue

        # Detect node start
        if echo "$line" | grep -q '^\s*-\?\s*name:'; then
            local this_name
            this_name=$(echo "$line" | sed 's/.*name:\s*//' | tr -d ' ')
            if [ "$this_name" = "$node_name" ]; then
                in_node=true
            else
                $in_node && break
            fi
            continue
        fi
        $in_node || continue

        if echo "$line" | grep -q "^\s*${field}:"; then
            echo "$line" | sed "s/.*${field}:\s*//"
            return
        fi
    done < "$PIPELINE"
}

while true; do

# Parse scheduled node names from pipeline.yaml
NODES=""
IN_NODES_SECTION=false
CURRENT_NAME=""
CURRENT_SCHEDULED=""

while IFS= read -r line; do
    if echo "$line" | grep -q '^[a-z_]*:'; then
        if echo "$line" | grep -q '^nodes:'; then
            IN_NODES_SECTION=true
        else
            # Flush last node before leaving section
            if $IN_NODES_SECTION && [ -n "$CURRENT_NAME" ] && [ "$CURRENT_SCHEDULED" != "false" ]; then
                NODES="$NODES $CURRENT_NAME"
            fi
            IN_NODES_SECTION=false
        fi
        continue
    fi
    $IN_NODES_SECTION || continue

    if echo "$line" | grep -q '^\s*-\?\s*name:'; then
        # Flush previous node
        if [ -n "$CURRENT_NAME" ] && [ "$CURRENT_SCHEDULED" != "false" ]; then
            NODES="$NODES $CURRENT_NAME"
        fi
        CURRENT_NAME=$(echo "$line" | sed 's/.*name:\s*//' | tr -d ' ')
        CURRENT_SCHEDULED=""
    elif echo "$line" | grep -q '^\s*scheduled:'; then
        CURRENT_SCHEDULED=$(echo "$line" | sed 's/.*scheduled:\s*//' | tr -d ' ')
    fi
done < "$PIPELINE"
# Flush final node
if $IN_NODES_SECTION && [ -n "$CURRENT_NAME" ] && [ "$CURRENT_SCHEDULED" != "false" ]; then
    NODES="$NODES $CURRENT_NAME"
fi

if [ -z "$NODES" ]; then
    echo "No scheduled nodes found."
    INTERVAL=$(grep '^\s*scheduler_interval:' "$PIPELINE" | sed 's/.*scheduler_interval:\s*//' | tr -d ' ')
    INTERVAL="${INTERVAL:-20}"
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
    CLOSE_VOTE=$(get_node_field "$NODE" "close_vote_required")
    if [ "$CLOSE_VOTE" != "false" ]; then
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

    # Read command from pipeline.yaml
    NODE_CMD=$(get_node_field "$NODE" "command")

    if [ -z "$NODE_CMD" ]; then
        echo "[$NODE] No command defined in pipeline.yaml, skipping."
        continue
    fi

    # Launch node in background, detached from terminal
    mkdir -p "$ROOT_DIR/logs"
    (
        cd "$ROOT_DIR"
        export NODE_ROOT="$ROOT_DIR"
        export NODE_NAME="$NODE"
        eval "exec $NODE_CMD"
    ) </dev/null &>"$ROOT_DIR/logs/${NODE}.log" &
    RUNNING_PIDS[$NODE]=$!

done

echo "Scheduler pass complete."
INTERVAL=$(grep '^\s*scheduler_interval:' "$PIPELINE" | sed 's/.*scheduler_interval:\s*//' | tr -d ' ')
INTERVAL="${INTERVAL:-20}"
sleep "$INTERVAL"
done
