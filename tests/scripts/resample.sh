#!/bin/bash

set -e

usage() {
    echo "Usage: $0 <input.wav>"
}

if [ "$#" -ne 1 ]; then
    usage
    exit 1
fi

INPUT_WAV="$1"

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

echo "Running resampling test on file: $INPUT_WAV"
cargo run --bin resample -- "$INPUT_WAV"