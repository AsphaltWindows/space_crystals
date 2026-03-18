#!/usr/bin/env bash
# check_pipeline.sh — Deterministic pipeline state checker and janitor.
# Archives fully-closed forum topics, then outputs one line per agent
# that has work waiting. Exit code 0 = work found, 1 = pipeline idle.

set -euo pipefail
cd "$(dirname "$0")"

WORK_FOUND=0

# (get_processed removed — agents now move processed files to archive directories)

# --- Helper: check if agent is already running ---
is_running() {
  local agent="$1"
  pgrep -f "claude --agent ${agent}" > /dev/null 2>&1
}

# --- Helper: get close votes from a forum topic file ---
get_close_votes() {
  local file="$1"
  # Extract lines between "# Close Votes" and next heading, matching "- agent_name" or "- [agent_name]"
  sed -n '/^# Close Votes/,/^#[^#]/p' "$file" 2>/dev/null | grep -oP '^\- \[?\K[^\]\s]+' || true
}

VOTING_AGENTS="designer product_analyst project_manager task_planner developer qa"

# ============================================================
# FORUM: Check for topics needing votes and archivable topics
# ============================================================
if [[ -d forum ]]; then
  for topic in forum/*.md; do
    [[ -f "$topic" ]] || continue
    topicname="$(basename "$topic")"
    votes=$(get_close_votes "$topic")

    # Check if all 6 voting agents have voted — if so, mark for archive
    all_voted=true
    for agent in $VOTING_AGENTS; do
      if ! echo "$votes" | grep -qx "$agent"; then
        all_voted=false
        break
      fi
    done

    if $all_voted; then
      mkdir -p forum_archive
      mv "forum/${topicname}" "forum_archive/${topicname}"
      echo "ARCHIVED $topicname"
      continue
    fi

    # Build list of agents who haven't voted
    missing=""
    for agent in $VOTING_AGENTS; do
      if ! echo "$votes" | grep -qx "$agent"; then
        missing="${missing:+$missing,}$agent"
      fi
    done
    voted_count=$(echo "$votes" | grep -c . || true)
    echo "FORUM_PENDING $topicname votes=${voted_count}/6 missing=$missing"
    WORK_FOUND=1
  done
fi

# ============================================================
# PRODUCT_ANALYST: Files in /design_updates (presence = unprocessed)
# ============================================================
if [[ -d design_updates ]]; then
  for f in design_updates/*.md; do
    [[ -f "$f" ]] || continue
    echo "LAUNCH product_analyst design_updates/$(basename "$f")"
    WORK_FOUND=1
  done
fi

# ============================================================
# PROJECT_MANAGER: Files in /feature_updates (presence = unprocessed)
# ============================================================
if [[ -d feature_updates ]]; then
  for f in feature_updates/*.md; do
    [[ -f "$f" ]] || continue
    echo "LAUNCH project_manager feature_updates/$(basename "$f")"
    WORK_FOUND=1
  done
fi

# ============================================================
# TASK_PLANNER: Files in /tickets (presence = unprocessed)
# ============================================================
if [[ -d tickets ]]; then
  for f in tickets/*.md; do
    [[ -f "$f" ]] || continue
    echo "LAUNCH task_planner tickets/$(basename "$f")"
    WORK_FOUND=1
  done
fi

# ============================================================
# DEVELOPER: Tasks in /developer_tasks
# ============================================================
if [[ -d developer_tasks ]] && ls developer_tasks/*.md &>/dev/null; then
  count=$(ls developer_tasks/*.md 2>/dev/null | wc -l)
  if [[ -f "developer_tasks/.blocked" ]]; then
    # .blocked lists dependencies still in /developer_tasks/. Check if any blocker
    # has been implemented (i.e., no longer in /developer_tasks/).
    any_cleared=false
    while IFS= read -r blocker; do
      [[ -z "$blocker" ]] && continue
      if [[ ! -f "developer_tasks/${blocker}" ]]; then
        any_cleared=true
        break
      fi
    done < "developer_tasks/.blocked"
    if ! $any_cleared; then
      # Also check if any task exists that is NOT listed in .blocked (new task arrived)
      blocked_list=$(cat "developer_tasks/.blocked")
      for f in developer_tasks/*.md; do
        [[ -f "$f" ]] || continue
        fname="$(basename "$f")"
        if ! echo "$blocked_list" | grep -qxF "$fname"; then
          any_cleared=true
          break
        fi
      done
    fi
    if $any_cleared; then
      echo "LAUNCH developer ${count} developer_tasks available (blocker cleared or new task)"
      WORK_FOUND=1
    else
      echo "BLOCKED developer ${count} developer_tasks waiting on dependencies"
    fi
  else
    # No .blocked file — developer is ready to pick tasks
    echo "LAUNCH developer ${count} developer_tasks available"
    WORK_FOUND=1
  fi
fi

# ============================================================
# QA: Launch for automated QA pass (agent decides what it can handle)
# ============================================================
if [[ -d qa_tasks ]] && ls qa_tasks/*.md &>/dev/null; then
  count=$(ls qa_tasks/*.md 2>/dev/null | wc -l)
  if [[ -f "qa_tasks/.blocked" ]]; then
    # Check if any qa_task exists that is NOT listed in .blocked (i.e., a new task arrived)
    any_new=false
    blocked_list=$(cat "qa_tasks/.blocked")
    for f in qa_tasks/*.md; do
      [[ -f "$f" ]] || continue
      fname="$(basename "$f")"
      if ! echo "$blocked_list" | grep -qxF "$fname"; then
        any_new=true
        break
      fi
    done
    if $any_new; then
      # Check if QA log already shows BLOCKED after the new task arrived — avoid relaunch loop
      qa_log="agent_logs/qa_log.md"
      if [[ -f "$qa_log" ]]; then
        recent_blocked=$(grep -c '^\#\#.*BLOCKED' <<< "$(head -60 "$qa_log")" || true)
        if [[ "$recent_blocked" -ge 2 ]]; then
          echo "BLOCKED qa ${count} qa_tasks (log shows ${recent_blocked} recent BLOCKED entries)"
        else
          echo "LAUNCH qa ${count} qa_tasks available (new task arrived)"
          WORK_FOUND=1
        fi
      else
        echo "LAUNCH qa ${count} qa_tasks available (new task arrived)"
        WORK_FOUND=1
      fi
    else
      echo "BLOCKED qa ${count} qa_tasks need interactive QA"
    fi
  else
    echo "LAUNCH qa ${count} qa_tasks available"
    WORK_FOUND=1
  fi
fi

# ============================================================
# QA_HUMAN_REVIEW: Tasks needing interactive human review
# ============================================================
if [[ -d qa_human_review ]] && ls qa_human_review/*.md &>/dev/null; then
  count=$(ls qa_human_review/*.md 2>/dev/null | wc -l)
  echo "HUMAN_REVIEW qa ${count} tasks awaiting human QA review"
  # Don't set WORK_FOUND — human review is user-initiated, not scheduled
fi

# ============================================================
# RUNNING: Check which agents are currently running
# ============================================================
for agent in designer product_analyst project_manager task_planner developer qa operator; do
  if is_running "$agent"; then
    echo "RUNNING $agent"
  fi
done

# ============================================================
# Summary exit
# ============================================================
if [[ $WORK_FOUND -eq 0 ]]; then
  echo "IDLE pipeline fully drained"
  exit 1
fi
exit 0
