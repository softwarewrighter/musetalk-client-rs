#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 3 ]; then
  echo "Usage: $0 <input_video> <desired_length_tenths> <output_video> [fps]"
  echo "Example: $0 idle.mp4 103 out.mp4 30"
  exit 1
fi

IN="$1"
TENTHS="$2"
OUT="$3"
FPS="${4:-}"   # optional

# Convert tenths → seconds
TARGET_SEC=$(awk -v t="$TENTHS" 'BEGIN { printf "%.10f\n", t / 10.0 }')

# Get input video duration
SRC_SEC=$(ffprobe -v error -show_entries format=duration \
  -of csv=p=0 "$IN")

# Compute stretch factor
FACTOR=$(awk -v tgt="$TARGET_SEC" -v src="$SRC_SEC" \
  'BEGIN { printf "%.10f\n", tgt / src }')

echo "Input video length : $SRC_SEC sec"
echo "Target video length: $TARGET_SEC sec"
echo "Stretch factor     : $FACTOR"

# Build filter
if [ -n "$FPS" ]; then
  VF="setpts=${FACTOR}*PTS,fps=${FPS}"
else
  VF="setpts=${FACTOR}*PTS"
fi

# Produce stretched video
ffmpeg -y -i "$IN" \
  -vf "$VF" \
  -an \
  -movflags +faststart \
  "$OUT"

echo "Wrote: $OUT"

