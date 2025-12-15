#!/bin/bash
# Composite lip-synced avatar onto a background video
# Usage: ./composite-video.sh <background_video> <avatar_raw_video> <avatar_audio_source> [output]
#
# Example: ./composite-video.sh ~/Movies/screencast.mp4 work/magenta-ml-raw.mp4 work/magenta-ml-transparent.webm work/final.mp4
#
# Uses chromakey on magenta background for clean edges (no dark halos)

set -e

BACKGROUND="${1:?Usage: $0 <background_video> <avatar_raw_video> <avatar_audio_source> [output]}"
AVATAR_RAW="${2:?Missing avatar raw video (with magenta background)}"
AUDIO_SOURCE="${3:?Missing audio source (webm with audio track)}"
OUTPUT="${4:-work/composite-output.mp4}"

# Magenta background color and chromakey settings
MAGENTA="0xc94591"
SIMILARITY="0.08"
BLEND="0.05"

# Avatar size and position (left edge, vertically centered)
AVATAR_SIZE="160"
X_POS="0"
# Y_POS calculated as (H-160)/2 = 460 for 1080p

echo "=============================================="
echo "  Video Compositor"
echo "=============================================="
echo ""
echo "Configuration:"
echo "  Background:  $BACKGROUND"
echo "  Avatar:      $AVATAR_RAW"
echo "  Audio from:  $AUDIO_SOURCE"
echo "  Output:      $OUTPUT"
echo "  Method:      Chromakey (magenta: $MAGENTA)"
echo ""

# Get background video duration
BG_DURATION=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$BACKGROUND")
echo "Background duration: ${BG_DURATION}s"

# Get background height for centering calculation
BG_HEIGHT=$(ffprobe -v error -select_streams v:0 -show_entries stream=height -of csv=p=0 "$BACKGROUND")
Y_POS=$(( (BG_HEIGHT - AVATAR_SIZE) / 2 ))
echo "Avatar position: ${X_POS},${Y_POS} (${AVATAR_SIZE}x${AVATAR_SIZE})"
echo ""

echo "Compositing..."
ffmpeg -y \
  -i "$BACKGROUND" \
  -i "$AVATAR_RAW" \
  -i "$AUDIO_SOURCE" \
  -filter_complex "[1:v]scale=${AVATAR_SIZE}:${AVATAR_SIZE},chromakey=${MAGENTA}:${SIMILARITY}:${BLEND}[fg];[0:v][fg]overlay=${X_POS}:${Y_POS}:shortest=1[out]" \
  -map "[out]" \
  -map 2:a \
  -c:v libx264 -crf 23 \
  -c:a aac -b:a 128k \
  -t "$BG_DURATION" \
  "$OUTPUT" 2>/dev/null

OUTPUT_SIZE=$(ls -lh "$OUTPUT" | awk '{print $5}')
OUTPUT_DURATION=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$OUTPUT")

echo ""
echo "=============================================="
echo "  Complete!"
echo "=============================================="
echo ""
echo "Output: $OUTPUT"
echo "Size:   $OUTPUT_SIZE"
echo "Duration: ${OUTPUT_DURATION}s"
echo ""
echo "Play: open $OUTPUT"
