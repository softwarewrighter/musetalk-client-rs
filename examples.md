# MuseTalk CLI Examples

This document covers the full workflow for generating lip-synced avatar videos with transparent backgrounds.

## Quick Start

```bash
# Run the full pipeline with default settings (uses ML background removal)
./generate-lipsync.sh work/magenta-polo.mp4 work/sample4.wav avatar

# First run will download the u2net model (~176MB)
```

## Full Pipeline Script

The `generate-lipsync.sh` script automates the entire workflow:

```bash
./generate-lipsync.sh [reference_video] [audio_file] [output_name]
```

**Steps performed:**
1. Analyze video/audio durations
2. Stretch video with frame interpolation to match audio length
3. Generate lip-sync via MuseTalk backend
4. Extract frames from output
5. Remove background via ML model (rembg)
6. Create transparent WebM (VP9 with alpha)
7. Create 160x160 thumbnail

## Individual Commands

### 1. Check Media Durations

```bash
# Video duration and frame count
ffprobe -v error -show_entries format=duration -of csv=p=0 video.mp4
ffprobe -v error -select_streams v:0 -count_frames \
  -show_entries stream=nb_read_frames -of csv=p=0 video.mp4

# Audio duration
ffprobe -v error -show_entries format=duration -of csv=p=0 audio.wav
```

### 2. Stretch Video with Frame Interpolation

**Important:** Use `minterpolate` to create actual new frames, not just `setpts` which only changes timing.

```bash
# Calculate stretch factor: audio_duration / video_duration
STRETCH=2.03

# WRONG - only changes timing, keeps same frame count:
ffmpeg -i input.mp4 -filter:v "setpts=${STRETCH}*PTS" output.mp4

# CORRECT - interpolates to create new frames:
ffmpeg -i input.mp4 \
  -filter:v "minterpolate=fps=30:mi_mode=dup,setpts=${STRETCH}*PTS" \
  -r 30 -an \
  -c:v libx264 -crf 18 \
  output.mp4
```

### 3. Generate Lip-Sync via MuseTalk

```bash
./target/release/musetalk-cli \
  --server http://hive:3015 \
  -r reference.mp4 \
  -a audio.wav \
  -o output.mp4
```

**Options:**
- `-r, --reference` - Reference image (PNG/JPEG) or video (MP4)
- `-a, --audio` - Audio file (WAV/MP3/FLAC)
- `-o, --output` - Output video path
- `-s, --server` - MuseTalk server URL (default: http://localhost:3015)
- `-f, --fps` - Frame rate (default: 30)
- `-v, --verbose` - Enable debug output
- `-n, --dry-run` - Validate inputs without processing

### 4. Extract Frames

```bash
mkdir -p frames
ffmpeg -i video.mp4 frames/frame_%04d.png
```

### 5. Remove Background (ML-based with rembg)

Using rembg (ML-based background removal) for reliable results regardless of background color:

```bash
# Install rembg in a virtual environment
uv venv .venv
source .venv/bin/activate
uv pip install "rembg[cli]" onnxruntime

# Process a folder of frames
.venv/bin/rembg p -m u2net frames/ frames_alpha/
```

**Advantages over color-based keying:**
- Works with any background color
- No color bleeding into subject
- Handles hair and fine details better
- No need to tune fuzz/tolerance parameters

**Alternative models:**
- `u2net` - Default, good balance of speed/quality
- `u2net_human_seg` - Optimized for human subjects
- `isnet-general-use` - Higher quality, slower

### 5b. Remove Background (Flood Fill - Legacy)

For solid color backgrounds, ImageMagick flood fill can work but may have artifacts:

```bash
# Get frame dimensions
DIM=$(magick identify -format '%wx%h' frame.png)
W=$(echo $DIM | cut -dx -f1)
H=$(echo $DIM | cut -dx -f2)

# Apply flood fill from all four corners
magick frame.png \
  -fuzz 20% \
  -fill none \
  -draw "color 0,0 floodfill" \
  -draw "color 0,$((H-1)) floodfill" \
  -draw "color $((W-1)),0 floodfill" \
  -draw "color $((W-1)),$((H-1)) floodfill" \
  frame_alpha.png
```

**Note:** This method can cause artifacts (face turning black, hair blinking) if colors overlap with the subject. Use ML-based removal for best results.

### 6. Batch Process All Frames (Legacy Flood Fill)

```bash
mkdir -p frames_alpha
for f in frames/frame_*.png; do
  magick "$f" \
    -fuzz 20% \
    -fill none \
    -draw "color 0,0 floodfill" \
    -draw "color 0,959 floodfill" \
    -draw "color 959,0 floodfill" \
    -draw "color 959,959 floodfill" \
    "frames_alpha/$(basename $f)"
done
```

### 7. Create Transparent WebM

```bash
ffmpeg -framerate 30 -i frames_alpha/frame_%04d.png \
  -i audio.wav \
  -c:v libvpx-vp9 -pix_fmt yuva420p -b:v 2M \
  -c:a libopus \
  -shortest \
  output.webm
```

**Key settings:**
- `-pix_fmt yuva420p` - Enables alpha channel
- `-c:v libvpx-vp9` - VP9 codec supports transparency
- `-b:v 2M` - Video bitrate

### 8. Create Thumbnail

```bash
ffmpeg -i input.webm \
  -vf "scale=160:160" \
  -c:v libvpx-vp9 -pix_fmt yuva420p -b:v 200k \
  -c:a libopus -b:a 32k \
  thumb.webm
```

### 9. Composite Over Another Video

There are two methods for compositing. Use chromakey for cleaner results.

#### Method A: Chromakey on Original (Recommended for Compositing)

Use the **raw video with magenta background** and chromakey filter. This produces cleaner edges than using the rembg-processed transparent video.

```bash
# Use composite-video.sh script
./composite-video.sh ~/Movies/screencast.mp4 work/magenta-ml-raw.mp4 work/magenta-ml-transparent.webm work/final.mp4

# Or manually:
ffmpeg -i background.mp4 -i avatar-raw.mp4 -i avatar-transparent.webm \
  -filter_complex "[1:v]scale=160:160,chromakey=0xc94591:0.08:0.05[fg];[0:v][fg]overlay=0:460:shortest=1[out]" \
  -map "[out]" -map 2:a \
  -c:v libx264 -crf 23 -c:a aac \
  output.mp4
```

**Chromakey parameters for magenta (#c94591):**
- `0xc94591` - The magenta color to key out
- `0.08` - Similarity (lower = more precise, less ghosting)
- `0.05` - Blend (edge softness)

#### Method B: Using Transparent WebM (Has Dark Halo Issue)

Using the rembg-processed transparent WebM directly can cause dark halos around the subject due to edge artifacts from ML background removal.

```bash
# Requires -c:v libvpx-vp9 on INPUT to decode alpha properly
ffmpeg -i background.mp4 -c:v libvpx-vp9 -i avatar-transparent.webm \
  -filter_complex "[0:v]format=rgba[bg];[1:v]format=rgba[fg];[bg][fg]overlay=0:460:format=auto:shortest=1,format=yuv420p[out]" \
  -map "[out]" -map 1:a \
  -c:v libx264 -crf 23 \
  output.mp4
```

**Note:** The transparent WebM works perfectly in web browsers (they handle VP9 alpha natively), but ffmpeg compositing often shows dark edge artifacts.

## Detecting Background Color

```bash
# Extract color from corner pixel
magick video_frame.png -crop 1x1+5+5 -format '%[hex:p{0,0}]' info:
# Output: 71B56C (greenscreen green)
```

## FFmpeg Chromakey for Compositing

Chromakey works well for **compositing onto background videos** when you have a solid color background (like magenta). Use tight tolerance to avoid ghosting.

```bash
# Good settings for magenta background compositing
ffmpeg -i background.mp4 -i avatar-magenta.mp4 \
  -filter_complex "[1:v]chromakey=0xc94591:0.08:0.05[fg];[0:v][fg]overlay=0:460[out]" \
  -map "[out]" \
  output.mp4

# For greenscreen (less reliable due to skin tone overlap)
ffmpeg -i input.mp4 \
  -vf "chromakey=0x71B56C:0.1:0.0" \
  -c:v libvpx-vp9 -pix_fmt yuva420p \
  output.webm
```

**Tip:** Magenta (#c94591) works better than green for human subjects because it has less overlap with skin tones.

## Video Formats with Alpha Support

| Format | Codec | Notes |
|--------|-------|-------|
| WebM | VP9 | Best for web, good compression |
| MOV | ProRes 4444 | Best quality, large files |
| PNG sequence | N/A | Lossless, very large |

**Note:** MP4 (H.264) and JPG do NOT support transparency.

## Background Removal Methods

### Choosing the Right Method

| Use Case | Recommended Method | Why |
|----------|-------------------|-----|
| Web playback (HTML video) | rembg (ML) | Browsers handle VP9 alpha perfectly |
| Compositing onto video | Chromakey | Cleaner edges, no dark halos |
| Any background color | rembg (ML) | Works regardless of background |
| Solid magenta background | Chromakey | Fastest, cleanest for compositing |

### ML-Based Removal (rembg) - Best for Web Playback

The `generate-lipsync.sh` script uses **rembg** for ML-based background removal. This approach:
- Works with **any background color** (solid, patterned, or complex)
- Produces clean edges around hair and fine details
- No color bleeding or artifacts on face/hair
- Processes at ~2.9 frames/second on CPU
- **Best for:** Displaying in web browsers via HTML `<video>` tag

**Limitation:** When compositing onto other videos via ffmpeg, rembg output can show dark halos around edges. Use chromakey method instead for video compositing.

### Color-Based Keying (Legacy)

If using color-based methods (flood fill, chromakey), background color matters:

| Color | Hex | Notes |
|-------|-----|-------|
| Magenta | #BC427B | Best for most skin/hair tones |
| Greenscreen Green | #71B56C | Standard, but can affect dark hair |
| Blue | #0000FF | Good alternative, may affect blue clothing |

**Known issues with color-based keying:**
- Face/hair can turn black intermittently
- Color bleeding into subject edges
- Requires tuning fuzz/tolerance per video

## Troubleshooting

### Lip-sync is too fast/slow
- Ensure video is stretched with `minterpolate`, not just `setpts`
- Check that MuseTalk received enough frames (should match audio duration * fps)

### Blotches on face after background removal
- Use flood fill method instead of chromakey
- Reduce fuzz percentage (try 10-15%)
- Ensure background color is uniform (use greenscreen)

### Black video output
- FFmpeg chromakey tolerance too high
- Use flood fill method instead

## Preview Results

```bash
python3 -m http.server 8080
# Open http://localhost:8080
```
