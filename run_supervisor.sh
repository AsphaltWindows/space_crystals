#!/usr/bin/env bash
# run_supervisor.sh — Pure shell supervisor. Replaces the supervisor agent.
# Runs check_pipeline.sh, archives closed forum topics, launches agents.

cd "$(dirname "$0")"

# nvm is fragile under strict mode — source it first
export NVM_DIR="$HOME/.nvm"
if [ -s "$NVM_DIR/nvm.sh" ]; then
  . "$NVM_DIR/nvm.sh"
  nvm use --lts 2>/dev/null || true
fi

set -euo pipefail

mkdir -p agent_logs

LOGFILE="./agent_logs/supervisor_log.md"
ALLOWED="Bash,Read,Write,Edit,Glob,Grep"
ALL_AGENTS="designer product_analyst project_manager task_planner developer qa operator"
LOG_LIMIT=600

launch_agent() {
  local agent="$1"
  local prompt="$2"
  local logdest="./agent_logs/${agent}_transcript.log"

  CLAUDECODE= nohup claude --agent "${agent}" \
    -p "${prompt}" \
    --allowedTools "${ALLOWED}" \
    --permission-mode acceptEdits \
    < /dev/null >> "${logdest}" 2>&1 &
}

# --- Build the prompt string for an agent ---
get_prompt() {
  local agent="$1"
  local reason="$2"
  case "$reason" in
    prune)     echo "PRUNE" ;;
    forum)     echo "Execute your forum pass only. Do NOT enter an interactive session. If the forum is clear, log and exit." ;;
    pipeline)
      if [[ "$agent" == "qa" ]]; then
        echo "Execute your scheduled run. Do your forum pass, then run your automated QA pass on any automatable tasks in /qa_tasks. Do NOT start an interactive session."
      else
        echo "Execute your task"
      fi
      ;;
  esac
}

while true; do
  TIMESTAMP="$(date '+%Y-%m-%d %H:%M:%S')"
  echo "=== Supervisor cycle: ${TIMESTAMP} ==="

  # Run the pipeline checker
  PIPELINE_OUTPUT=$(./check_pipeline.sh 2>&1 || true)

  # =============================================
  # 1. STATE — current state of every agent
  # =============================================
  echo ""
  echo "-- State --"

  # Collect running agents
  declare -A AGENT_STATE
  for agent in $ALL_AGENTS; do
    AGENT_STATE[$agent]="idle"
  done

  for agent in $ALL_AGENTS; do
    if pgrep -f "claude --agent ${agent}" > /dev/null 2>&1; then
      AGENT_STATE[$agent]="running"
    fi
  done

  # Check log sizes
  declare -A AGENT_LOG_LINES
  for agent in $ALL_AGENTS; do
    logpath="./agent_logs/${agent}_log.md"
    if [[ -f "$logpath" ]]; then
      AGENT_LOG_LINES[$agent]=$(wc -l < "$logpath")
    else
      AGENT_LOG_LINES[$agent]=0
    fi
  done

  for agent in $ALL_AGENTS; do
    printf "  %-20s  state=%-8s  log=%s lines\n" "$agent" "${AGENT_STATE[$agent]}" "${AGENT_LOG_LINES[$agent]}"
  done

  # =============================================
  # 2. DESIRED — what each agent would be told to do
  # =============================================
  echo ""
  echo "-- Desired --"

  # Determine desired action for each agent
  declare -A AGENT_DESIRED
  for agent in $ALL_AGENTS; do
    AGENT_DESIRED[$agent]=""
  done

  # Prune takes priority
  for agent in $ALL_AGENTS; do
    if [[ ${AGENT_LOG_LINES[$agent]} -gt $LOG_LIMIT ]]; then
      AGENT_DESIRED[$agent]="prune (${AGENT_LOG_LINES[$agent]} lines)"
    fi
  done

  # Pipeline work (only if not already marked for prune)
  LAUNCH_AGENTS=$(echo "$PIPELINE_OUTPUT" | grep "^LAUNCH " | awk '{print $2}' | sort -u || true)
  BLOCKED_AGENTS=$(echo "$PIPELINE_OUTPUT" | grep "^BLOCKED " || true)

  for agent in $LAUNCH_AGENTS; do
    if [[ -z "${AGENT_DESIRED[$agent]}" ]]; then
      reason=$(echo "$PIPELINE_OUTPUT" | grep "^LAUNCH ${agent} " | head -1 | cut -d' ' -f3-)
      AGENT_DESIRED[$agent]="pipeline: ${reason}"
    fi
  done

  # Blocked agents (only if not already marked for something)
  if [[ -n "$BLOCKED_AGENTS" ]]; then
    while IFS= read -r line; do
      agent=$(echo "$line" | awk '{print $2}')
      reason=$(echo "$line" | cut -d' ' -f3-)
      if [[ -z "${AGENT_DESIRED[$agent]}" ]]; then
        AGENT_DESIRED[$agent]="blocked: ${reason}"
      fi
    done <<< "$BLOCKED_AGENTS"
  fi

  # Forum votes (override blocked status — agents must vote even if their pipeline work is blocked)
  FORUM_MISSING=$(echo "$PIPELINE_OUTPUT" | grep "^FORUM_PENDING " | grep -oP 'missing=\K\S+' || true)
  for agent in $ALL_AGENTS; do
    if echo "$FORUM_MISSING" | tr ',' '\n' | grep -qx "$agent"; then
      if [[ -z "${AGENT_DESIRED[$agent]}" || "${AGENT_DESIRED[$agent]}" == blocked:* ]]; then
        AGENT_DESIRED[$agent]="forum votes pending"
      fi
    fi
  done

  for agent in $ALL_AGENTS; do
    desired="${AGENT_DESIRED[$agent]:-none}"
    printf "  %-20s  %s\n" "$agent" "$desired"
  done

  # =============================================
  # 3. ACTIONS — what's actually being launched
  # =============================================
  echo ""
  echo "-- Actions --"

  ACTIONS=""
  ANY_ACTION=false

  for agent in $ALL_AGENTS; do
    desired="${AGENT_DESIRED[$agent]}"
    [[ -z "$desired" ]] && continue

    # Skip blocked agents
    if [[ "$desired" == blocked:* ]]; then
      continue
    fi

    # Skip if already running
    if [[ "${AGENT_STATE[$agent]}" == "running" ]]; then
      echo "  ${agent}: SKIP (already running, would: ${desired})"
      ACTIONS="${ACTIONS}\n- Skipped ${agent} (already running, would: ${desired})"
      continue
    fi

    # Determine prompt
    if [[ "$desired" == prune* ]]; then
      prompt=$(get_prompt "$agent" "prune")
    elif [[ "$desired" == pipeline:* ]]; then
      prompt=$(get_prompt "$agent" "pipeline")
    elif [[ "$desired" == "forum votes pending" ]]; then
      # Don't double-launch if already launching for pipeline or prune
      prompt=$(get_prompt "$agent" "forum")
    else
      continue
    fi

    echo "  ${agent}: LAUNCH — ${desired}"
    launch_agent "$agent" "$prompt"
    ACTIONS="${ACTIONS}\n- Launched ${agent} (${desired})"
    ANY_ACTION=true
  done

  if [[ "$ANY_ACTION" == false ]]; then
    echo "  (nothing to launch)"
    ACTIONS="${ACTIONS}\n- Nothing to launch"
  fi

  # --- Append to supervisor log ---
  {
    echo ""
    echo "## ${TIMESTAMP}"
    echo -e "$ACTIONS" | sed '/^$/d'
  } >> "$LOGFILE"

  echo ""
  echo "=== Cycle done. Sleeping 30s ==="
  sleep 30

  # Clean up associative arrays for next iteration
  unset AGENT_STATE AGENT_LOG_LINES AGENT_DESIRED
done
