#!/bin/bash
set -e

usage() {
    echo "Usage: $0 <web|core> <input.wav>"
}

if [ "$#" -ne 2 ]; then
    usage
    exit 1
fi

MODE="$1"
INPUT_WAV="$2"

case "$MODE" in
    web)
        BUILD_ARGS=(--profile profiling --features web --bin profile)
        ;;
    core)
        BUILD_ARGS=(--profile profiling --bin profile)
        ;;
    *)
        echo "Invalid mode: $MODE"
        usage
        exit 1
        ;;
esac

if [ ! -f "$INPUT_WAV" ]; then
    echo "Input file not found: $INPUT_WAV"
    usage
    exit 1
fi

if [ ! -r "$INPUT_WAV" ]; then
    echo "Input file is not readable: $INPUT_WAV"
    exit 1
fi

case "$INPUT_WAV" in
    *.wav|*.WAV) ;;
    *)
        echo "Input file must be a .wav file: $INPUT_WAV"
        usage
        exit 1
        ;;
esac

# check that samply is installed
if ! command -v samply &> /dev/null
then
    echo "samply not found, installing..."
    cargo install samply
else
    echo "samply found, skipping installation"
fi
echo "Building the binary..."
cargo build "${BUILD_ARGS[@]}" # build with optimizations and debug info
echo "Running the profiler..."
# run the profiler
samply record ./target/profiling/profile "$INPUT_WAV"
