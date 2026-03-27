#!/bin/bash
set -e

EXT_NAME="quicksilver"
OUT_ZIP="$EXT_NAME.zip"

zip -r "$OUT_ZIP" \
  chrome/manifest.json \
  chrome/*.js \
  chrome/*.html \
  chrome/pkg \
  chrome/LICENSE.md \
  chrome/assets/icon*.png