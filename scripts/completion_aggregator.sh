#!/usr/bin/env bash
set -euo pipefail

# Completion Aggregator Script Node
#
# Tracks developer task completions against feature_tasks manifests.
# When all tasks for a feature are complete, produces a qa_item via send_message.sh.
#
# Message types consumed:
#   - feature_request: The original feature description + QA instructions (held in active)
#   - feature_tasks:   Manifest mapping feature_request -> developer_task filenames (held in active)
#   - task_completion:  Marker that a developer_task is done (matched and moved to done)
#
# Message type produced:
#   - qa_item: Original feature_request content forwarded to qa_router

ROOT_DIR="$1"
NODE_NAME="$2"

MSG_DIR="$ROOT_DIR/messages/$NODE_NAME"
FEATURE_REQ_DIR="$MSG_DIR/feature_request"
FEATURE_TASKS_DIR="$MSG_DIR/feature_tasks"
TASK_COMPLETION_DIR="$MSG_DIR/task_completion"
SEND_MSG="$ROOT_DIR/scripts/send_message.sh"
LOG_FILE="$ROOT_DIR/artifacts/$NODE_NAME/log.md"

log() {
    echo "[$NODE_NAME] $1"
    echo "- $(date -Iseconds): $1" >> "$LOG_FILE"
}

# --- Step 1: Move new feature_requests and feature_tasks from pending to active ---

for f in "$FEATURE_REQ_DIR/pending"/*.md; do
    [ -f "$f" ] || continue
    mv "$f" "$FEATURE_REQ_DIR/active/"
    log "Activated feature_request: $(basename "$f")"
done

for f in "$FEATURE_TASKS_DIR/pending"/*.md; do
    [ -f "$f" ] || continue
    mv "$f" "$FEATURE_TASKS_DIR/active/"
    log "Activated feature_tasks: $(basename "$f")"
done

# --- Step 2: Move new task_completions from pending to active ---

for f in "$TASK_COMPLETION_DIR/pending"/*.md; do
    [ -f "$f" ] || continue
    mv "$f" "$TASK_COMPLETION_DIR/active/"
    log "Activated task_completion: $(basename "$f")"
done

# --- Step 3: Check each active feature_tasks manifest for completion ---

for manifest in "$FEATURE_TASKS_DIR/active"/*.md; do
    [ -f "$manifest" ] || continue

    manifest_name="$(basename "$manifest")"

    # Parse the feature_request filename from the manifest
    feature_req_file=""
    in_feature_request=false
    while IFS= read -r line; do
        if echo "$line" | grep -q "^## Feature Request"; then
            in_feature_request=true
            continue
        fi
        if $in_feature_request; then
            # First non-empty line after the heading is the filename
            trimmed="${line#"${line%%[![:space:]]*}"}"
            trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
            if [ -n "$trimmed" ] && ! echo "$trimmed" | grep -q "^##"; then
                feature_req_file="$trimmed"
                break
            fi
            # Stop if we hit another heading
            if echo "$line" | grep -q "^##"; then
                break
            fi
        fi
    done < "$manifest"

    if [ -z "$feature_req_file" ]; then
        log "WARNING: Could not parse feature_request filename from $manifest_name"
        continue
    fi

    # Parse developer_task filenames from the manifest
    tasks=()
    in_tasks=false
    while IFS= read -r line; do
        if echo "$line" | grep -q "^## Developer Tasks"; then
            in_tasks=true
            continue
        fi
        if $in_tasks; then
            # Stop at the next heading
            if echo "$line" | grep -q "^##"; then
                break
            fi
            # Parse bullet lines: "- filename.md"
            if echo "$line" | grep -q "^- "; then
                task_file="${line#- }"
                task_file="${task_file#"${task_file%%[![:space:]]*}"}"
                task_file="${task_file%"${task_file##*[![:space:]]}"}"
                if [ -n "$task_file" ]; then
                    tasks+=("$task_file")
                fi
            fi
        fi
    done < "$manifest"

    if [ ${#tasks[@]} -eq 0 ]; then
        log "WARNING: No developer_tasks found in $manifest_name"
        continue
    fi

    # Check if all tasks have corresponding task_completion files in active/
    # The task_completion filename is: developer-{task_slug}.md
    # The developer_task filename is: task_splitter-{task_slug}.md
    # So we need to map task_splitter-{slug} -> developer-{slug}
    all_complete=true
    for task_file in "${tasks[@]}"; do
        # Extract the slug: strip the producing agent prefix and dash
        task_slug="$(echo "$task_file" | sed 's/^[^-]*-//')"
        completion_file="developer-${task_slug}"

        if [ ! -f "$TASK_COMPLETION_DIR/active/$completion_file" ]; then
            all_complete=false
            break
        fi
    done

    if $all_complete; then
        log "All tasks complete for $manifest_name — producing qa_item"

        # Find the corresponding feature_request in active/
        if [ ! -f "$FEATURE_REQ_DIR/active/$feature_req_file" ]; then
            log "WARNING: feature_request $feature_req_file not found in active/"
            continue
        fi

        # Derive the feature slug from the feature_request filename
        # e.g., designer-add_syndicate_tunnels.md -> add_syndicate_tunnels
        feature_slug="$(echo "$feature_req_file" | sed 's/^[^-]*-//; s/\.md$//')"

        # Extract Content and QA Instructions from the feature_request
        # (everything from ## Content onward, skipping metadata)
        content="$(sed -n '/^## Content$/,$ p' "$FEATURE_REQ_DIR/active/$feature_req_file")"

        # Send via send_message.sh
        "$SEND_MSG" "$NODE_NAME" "qa_router" "qa_item" "$feature_slug" "$content"

        log "Produced qa_item: $feature_slug"

        # Move everything to done
        mv "$FEATURE_REQ_DIR/active/$feature_req_file" "$FEATURE_REQ_DIR/done/"
        mv "$manifest" "$FEATURE_TASKS_DIR/done/"

        for task_file in "${tasks[@]}"; do
            task_slug="$(echo "$task_file" | sed 's/^[^-]*-//')"
            completion_file="developer-${task_slug}"
            if [ -f "$TASK_COMPLETION_DIR/active/$completion_file" ]; then
                mv "$TASK_COMPLETION_DIR/active/$completion_file" "$TASK_COMPLETION_DIR/done/"
            fi
        done

        log "Moved all messages to done for feature: $feature_slug"
    fi
done

log "Aggregator pass complete"
