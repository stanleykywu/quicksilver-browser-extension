#!/bin/bash
set -e

# build the python bindings with uv sync
# use --reinstall to ensure changes to the bindings are picked up
uv sync --reinstall
echo "Python bindings built successfully"
# run the fakeprint example to ensure the bindings work
echo "Running test..."
uv run tests/scripts/pybindings.py
