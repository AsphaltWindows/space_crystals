#!/usr/bin/env bash
set -euo pipefail

# QA Build Artifact Script
# Builds a self-contained binary (no dynamic linking) and saves it
# in a timestamped directory under qa_artifacts/.

# Determine project root (parent of scripts/ directory)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT/artifacts/developer"

# Parse arguments (QA task filenames)
QA_TASKS=("$@")

# Build without dynamic linking but keep diagnostics enabled
echo "Building release-debug binary (diagnostics enabled, no dynamic linking)..."
if ! cargo build --no-default-features --features diagnostics; then
    echo ""
    echo "ERROR: cargo build failed. No artifact was created."
    echo "       The 'latest' symlink (if it exists) still points to the previous successful build."
    exit 1
fi

# Create timestamped directory
TIMESTAMP="$(date +%Y-%m-%d_%H%M%S)"
ARTIFACT_DIR="$PROJECT_ROOT/artifacts/manual_qa/qa_artifacts/$TIMESTAMP"
mkdir -p "$ARTIFACT_DIR"

# Copy binary
BINARY_SRC="$PROJECT_ROOT/artifacts/developer/target/debug/space_crystals"
if [ ! -f "$BINARY_SRC" ]; then
    echo "ERROR: Binary not found at $BINARY_SRC"
    exit 1
fi
cp "$BINARY_SRC" "$ARTIFACT_DIR/space_crystals"
chmod +x "$ARTIFACT_DIR/space_crystals"

# Gather git metadata
GIT_COMMIT="$(git rev-parse HEAD 2>/dev/null || echo 'unknown')"
GIT_DIRTY="$(git diff --quiet 2>/dev/null && echo 'clean' || echo 'dirty')"
GIT_BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')"
BUILD_TIME="$(date -Iseconds)"

# Write manifest
{
    echo "Build Manifest"
    echo "=============="
    echo ""
    echo "Git commit:  $GIT_COMMIT"
    echo "Git state:   $GIT_DIRTY"
    echo "Git branch:  $GIT_BRANCH"
    echo "Build time:  $BUILD_TIME"
    echo ""
    echo "QA Tasks:"
    if [ ${#QA_TASKS[@]} -eq 0 ]; then
        echo "  (none specified)"
    else
        for task in "${QA_TASKS[@]}"; do
            echo "  - $task"
        done
    fi
} > "$ARTIFACT_DIR/manifest.txt"

# Update latest symlink
ln -sfn "$TIMESTAMP" "$PROJECT_ROOT/artifacts/manual_qa/qa_artifacts/latest"

# Success output
echo ""
echo "Build artifact saved to: artifacts/manual_qa/qa_artifacts/$TIMESTAMP/"
echo "Symlink updated:         artifacts/manual_qa/qa_artifacts/latest -> $TIMESTAMP"
echo ""
echo "Manifest:"
cat "$ARTIFACT_DIR/manifest.txt"
echo ""
echo "To run the game:"
echo "  cd $PROJECT_ROOT/artifacts/developer && $PROJECT_ROOT/artifacts/manual_qa/qa_artifacts/latest/space_crystals"
echo ""
echo "IMPORTANT: Run from artifacts/developer/ so Bevy can find the assets/ directory."
