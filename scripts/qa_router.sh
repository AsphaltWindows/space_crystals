#!/usr/bin/env bash
set -euo pipefail

# QA Router Script Node
#
# Routes qa_items to manual_qa or automatic_qa based on whether the QA
# instructions can be performed automatically with current tooling.
#
# Decision logic:
#   1. Read auto_capabilities.txt for patterns that indicate automatable steps
#   2. Extract QA Instructions from the qa_item
#   3. If ALL non-empty QA instruction lines match at least one capability pattern,
#      route to automatic_qa
#   4. Otherwise route to manual_qa
#
# With no capabilities defined (all lines commented), everything goes to manual_qa.

ROOT_DIR="$1"
NODE_NAME="$2"

MSG_DIR="$ROOT_DIR/messages/$NODE_NAME"
QA_ITEM_DIR="$MSG_DIR/qa_item"
MANUAL_QA_DIR="$ROOT_DIR/messages/manual_qa/qa_item"
AUTO_QA_DIR="$ROOT_DIR/messages/automatic_qa/qa_item"
CAPABILITIES_FILE="$ROOT_DIR/artifacts/$NODE_NAME/auto_capabilities.txt"
LOG_FILE="$ROOT_DIR/artifacts/$NODE_NAME/log.md"

log() {
    echo "[$NODE_NAME] $1"
    echo "- $(date -Iseconds): $1" >> "$LOG_FILE"
}

# Load capability patterns (skip comments and empty lines)
capabilities=()
if [ -f "$CAPABILITIES_FILE" ]; then
    while IFS= read -r line; do
        trimmed="$(echo "$line" | xargs)"
        # Skip empty lines and comments
        [ -z "$trimmed" ] && continue
        echo "$trimmed" | grep -q "^#" && continue
        capabilities+=("$trimmed")
    done < "$CAPABILITIES_FILE"
fi

# Process each pending qa_item
for item in "$QA_ITEM_DIR/pending"/*.md; do
    [ -f "$item" ] || continue

    item_name="$(basename "$item")"
    mv "$item" "$QA_ITEM_DIR/active/"
    active_item="$QA_ITEM_DIR/active/$item_name"

    # Extract QA Instructions section
    qa_lines=()
    in_qa=false
    while IFS= read -r line; do
        if echo "$line" | grep -q "^## QA Instructions"; then
            in_qa=true
            continue
        fi
        if $in_qa; then
            # Stop at next heading
            if echo "$line" | grep -q "^## "; then
                break
            fi
            trimmed="$(echo "$line" | xargs)"
            # Collect non-empty lines
            if [ -n "$trimmed" ]; then
                qa_lines+=("$trimmed")
            fi
        fi
    done < "$active_item"

    # Decide routing
    route="manual_qa"

    if [ ${#capabilities[@]} -gt 0 ] && [ ${#qa_lines[@]} -gt 0 ]; then
        all_match=true
        for qa_line in "${qa_lines[@]}"; do
            line_matched=false
            for pattern in "${capabilities[@]}"; do
                if echo "$qa_line" | grep -qi "$pattern"; then
                    line_matched=true
                    break
                fi
            done
            if ! $line_matched; then
                all_match=false
                break
            fi
        done

        if $all_match; then
            route="automatic_qa"
        fi
    fi

    # Route the qa_item
    if [ "$route" = "automatic_qa" ]; then
        cp "$active_item" "$AUTO_QA_DIR/pending/$item_name"
        log "Routed to automatic_qa: $item_name"
    else
        cp "$active_item" "$MANUAL_QA_DIR/pending/$item_name"
        log "Routed to manual_qa: $item_name"
    fi

    mv "$active_item" "$QA_ITEM_DIR/done/"
done

log "Router pass complete"
