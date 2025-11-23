#!/bin/bash

# Development script with hot reload using cargo-watch
# This script watches for changes in Rust source files and templates,
# automatically rebuilding and restarting the server

set -e

echo "🚀 Starting backend with hot reload..."
echo "📁 Watching for changes in:"
echo "   - src/**/*.rs"
echo "   - templates/**/*.html"
echo ""
echo "💡 Tip: Changes to templates will trigger a rebuild"
echo "   Press Ctrl+C to stop"
echo ""

# Check if running in a TTY (interactive terminal)
# Only use --clear if we have a TTY, otherwise it causes errors in background mode
CLEAR_FLAG=""
if [ -t 0 ]; then
    CLEAR_FLAG="--clear"
fi

# Use cargo-watch to watch for changes and rebuild/restart
# -x 'run --bin privatechefspace-backend' specifies which binary to run
# -w src watches the src directory
# -w templates watches the templates directory (Askama will recompile templates)
# -d 0.5 sets a 0.5 second delay before running (debounce)
# --clear clears the screen on each run (only if TTY available)
# --why shows why cargo-watch triggered
# RUST_LOG sets the log level

cargo watch \
  -x 'run --bin privatechefspace-backend' \
  -w src \
  -w templates \
  -d 0.5 \
  $CLEAR_FLAG \
  --why \
  -- RUST_LOG=info

