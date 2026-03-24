#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 2 ]; then
    echo "Usage: vote_close.sh <topic-file> <agent-name>"
    echo "Example: vote_close.sh forum/open/2024-01-15-designer-api-ambiguity.md designer"
    exit 1
fi

TOPIC_FILE="$1"
AGENT_NAME="$2"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FULL_PATH="$ROOT_DIR/$TOPIC_FILE"

if [ ! -f "$FULL_PATH" ]; then
    echo "Error: Topic file not found: $FULL_PATH"
    exit 1
fi

# Check if agent already voted
if grep -q "^VOTE:${AGENT_NAME}$" "$FULL_PATH"; then
    echo "Agent '$AGENT_NAME' has already voted to close this topic."
    exit 0
fi

# Add vote after the "## Close Votes" line
sed -i "/^## Close Votes$/a VOTE:${AGENT_NAME}" "$FULL_PATH"

echo "Vote added for '$AGENT_NAME'."

# Check if all required nodes have voted
# Nodes with close_vote_required: false in pipeline.yaml are excluded
PIPELINE="$ROOT_DIR/pipeline.yaml"
REQUIRED_NODES=""
IN_NODES_SECTION=false
CURRENT_NAME=""
CURRENT_CLOSE_VOTE=""

while IFS= read -r line; do
    if echo "$line" | grep -q '^[a-z_]*:'; then
        if echo "$line" | grep -q '^nodes:'; then
            IN_NODES_SECTION=true
        else
            # Flush last node before leaving section
            if $IN_NODES_SECTION && [ -n "$CURRENT_NAME" ] && [ "$CURRENT_CLOSE_VOTE" != "false" ]; then
                REQUIRED_NODES="$REQUIRED_NODES $CURRENT_NAME"
            fi
            IN_NODES_SECTION=false
        fi
        continue
    fi
    [ "$IN_NODES_SECTION" = true ] || continue

    if echo "$line" | grep -q '^\s*-\?\s*name:'; then
        # Flush previous node
        if [ -n "$CURRENT_NAME" ] && [ "$CURRENT_CLOSE_VOTE" != "false" ]; then
            REQUIRED_NODES="$REQUIRED_NODES $CURRENT_NAME"
        fi
        CURRENT_NAME=$(echo "$line" | sed 's/.*name:\s*//' | tr -d ' ')
        CURRENT_CLOSE_VOTE=""
    elif echo "$line" | grep -q '^\s*close_vote_required:'; then
        CURRENT_CLOSE_VOTE=$(echo "$line" | sed 's/.*close_vote_required:\s*//' | tr -d ' ')
    fi
done < "$PIPELINE"
# Flush final node
if $IN_NODES_SECTION && [ -n "$CURRENT_NAME" ] && [ "$CURRENT_CLOSE_VOTE" != "false" ]; then
    REQUIRED_NODES="$REQUIRED_NODES $CURRENT_NAME"
fi

ALL_VOTED=true
for NODE in $REQUIRED_NODES; do
    if ! grep -q "^VOTE:${NODE}$" "$FULL_PATH"; then
        ALL_VOTED=false
        break
    fi
done

if [ "$ALL_VOTED" = true ]; then
    CLOSED_DIR="$ROOT_DIR/forum/closed"
    mv "$FULL_PATH" "$CLOSED_DIR/"
    echo "All required nodes voted. Topic moved to closed."
fi
