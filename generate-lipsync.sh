#!/bin/bash
# Generate lip-synced video with transparent background
# Usage: ./generate-lipsync.sh [reference_video] [audio_file] [output_name]
#
# Example: ./generate-lipsync.sh work/greenscreen-polo.mp4 work/sample4.wav greenscreen

set -e

# Configuration
SERVER="http://hive:3015"
REF_VIDEO="${1:-work/magenta-polo.mp4}"
AUDIO="${2:-work/sample4.wav}"
OUTPUT_NAME="${3:-lipsync}"

# Output files
STRETCHED_VIDEO="work/${OUTPUT_NAME}-stretched.mp4"
RAW_OUTPUT="work/${OUTPUT_NAME}-raw.mp4"
TRANSPARENT_WEBM="work/${OUTPUT_NAME}-transparent.webm"
THUMB_WEBM="work/${OUTPUT_NAME}-thumb.webm"
FRAMES_DIR="work/${OUTPUT_NAME}_frames"
ALPHA_DIR="work/${OUTPUT_NAME}_alpha"

echo "=============================================="
echo "  Lip-Sync Video Generator with Transparency"
echo "=============================================="
echo ""
echo "Configuration:"
echo "  Server:      $SERVER"
echo "  Reference:   $REF_VIDEO"
echo "  Audio:       $AUDIO"
echo "  Output:      $OUTPUT_NAME"
echo "  BG Removal:  ML-based (rembg)"
echo ""

# Step 1: Analyze durations
echo "[1/7] Analyzing media durations..."
VIDEO_DURATION=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$REF_VIDEO")
AUDIO_DURATION=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$AUDIO")
VIDEO_FRAMES=$(ffprobe -v error -select_streams v:0 -count_frames -show_entries stream=nb_read_frames -of csv=p=0 "$REF_VIDEO")

echo "  Video: ${VIDEO_DURATION}s, ${VIDEO_FRAMES} frames"
echo "  Audio: ${AUDIO_DURATION}s"

# Calculate stretch factor
STRETCH_FACTOR=$(echo "scale=6; $AUDIO_DURATION / $VIDEO_DURATION" | bc)
TARGET_FRAMES=$(echo "scale=0; $AUDIO_DURATION * 30 / 1" | bc)
echo "  Stretch factor: ${STRETCH_FACTOR}x"
echo "  Target frames (30fps): $TARGET_FRAMES"
echo ""

# Step 2: Stretch video with frame interpolation
echo "[2/7] Stretching video with frame interpolation..."
ffmpeg -y -i "$REF_VIDEO" \
  -filter:v "minterpolate=fps=30:mi_mode=dup,setpts=${STRETCH_FACTOR}*PTS" \
  -r 30 \
  -an \
  -c:v libx264 -crf 18 \
  "$STRETCHED_VIDEO" 2>/dev/null

STRETCHED_FRAMES=$(ffprobe -v error -select_streams v:0 -count_frames -show_entries stream=nb_read_frames -of csv=p=0 "$STRETCHED_VIDEO")
STRETCHED_DURATION=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$STRETCHED_VIDEO")
echo "  Created: $STRETCHED_VIDEO"
echo "  Frames: $STRETCHED_FRAMES, Duration: ${STRETCHED_DURATION}s"
echo ""

# Step 3: Generate lip-sync via MuseTalk
echo "[3/7] Generating lip-sync via MuseTalk (this may take several minutes)..."
./target/release/musetalk-cli \
  --server "$SERVER" \
  -r "$STRETCHED_VIDEO" \
  -a "$AUDIO" \
  -o "$RAW_OUTPUT"

RAW_FRAMES=$(ffprobe -v error -select_streams v:0 -count_frames -show_entries stream=nb_read_frames -of csv=p=0 "$RAW_OUTPUT")
echo "  Output: $RAW_OUTPUT ($RAW_FRAMES frames)"
echo ""

# Step 4: Extract frames
echo "[4/7] Extracting frames..."
rm -rf "$FRAMES_DIR" "$ALPHA_DIR"
mkdir -p "$FRAMES_DIR" "$ALPHA_DIR"

ffmpeg -y -i "$RAW_OUTPUT" "${FRAMES_DIR}/frame_%04d.png" 2>/dev/null
FRAME_COUNT=$(ls "$FRAMES_DIR"/*.png | wc -l | tr -d ' ')
echo "  Extracted $FRAME_COUNT frames"
echo ""

# Step 5: Remove background using ML (rembg)
echo "[5/7] Removing background with ML model (rembg)..."
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REMBG="${SCRIPT_DIR}/.venv/bin/rembg"

if [ ! -f "$REMBG" ]; then
  echo "  ERROR: rembg not found. Install with:"
  echo "    uv venv .venv && source .venv/bin/activate && uv pip install 'rembg[cli]' onnxruntime"
  exit 1
fi

echo "  Processing $FRAME_COUNT frames (this may take a while on first run)..."
"$REMBG" p -m u2net "$FRAMES_DIR" "$ALPHA_DIR"
echo "  Done processing frames"
echo ""

# Step 6: Create transparent WebM
echo "[6/7] Creating transparent WebM..."
ffmpeg -y -framerate 30 -i "${ALPHA_DIR}/frame_%04d.png" \
  -i "$AUDIO" \
  -c:v libvpx-vp9 -pix_fmt yuva420p -b:v 2M \
  -c:a libopus \
  -shortest \
  "$TRANSPARENT_WEBM" 2>/dev/null

WEBM_SIZE=$(ls -lh "$TRANSPARENT_WEBM" | awk '{print $5}')
WEBM_DURATION=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$TRANSPARENT_WEBM")
echo "  Created: $TRANSPARENT_WEBM"
echo "  Size: $WEBM_SIZE, Duration: ${WEBM_DURATION}s"
echo ""

# Step 7: Create thumbnail
echo "[7/7] Creating 160x160 thumbnail..."
ffmpeg -y -i "$TRANSPARENT_WEBM" \
  -vf "scale=160:160" \
  -c:v libvpx-vp9 -pix_fmt yuva420p -b:v 200k \
  -c:a libopus -b:a 32k \
  "$THUMB_WEBM" 2>/dev/null

THUMB_SIZE=$(ls -lh "$THUMB_WEBM" | awk '{print $5}')
echo "  Created: $THUMB_WEBM ($THUMB_SIZE)"
echo ""

# Summary
echo "=============================================="
echo "  Complete!"
echo "=============================================="
echo ""
echo "Output files:"
echo "  Raw MP4:          $RAW_OUTPUT"
echo "  Transparent WebM: $TRANSPARENT_WEBM"
echo "  Thumbnail WebM:   $THUMB_WEBM"
echo ""
echo "Preview: python3 -m http.server 8080"
echo "         Open http://localhost:8080"
