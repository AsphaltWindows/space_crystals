#!/usr/bin/env bash

# Run the QA build artifact from the developer's directory
# so Bevy can find assets/
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT/artifacts/developer"
"$PROJECT_ROOT/artifacts/manual_qa/qa_artifacts/latest/space_crystals" > "$PROJECT_ROOT/run_qa.log" 2>&1
