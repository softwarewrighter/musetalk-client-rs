# MuseTalk CLI

A Rust-based command-line interface for generating lip-synced avatar videos using [MuseTalk](https://github.com/TMElyralab/MuseTalk).

## Overview

MuseTalk CLI takes a static avatar image and an audio file, then produces an animated video of the avatar speaking with realistic lip movements, facial expressions, and eye animations. It's designed for content creators who want to generate animated avatar overlays for educational videos, tutorials, and live-coding content.

## Features

- **Simple CLI**: Single command to transform image + audio into video
- **Multiple Formats**: Supports PNG/JPEG images and WAV/MP3/FLAC audio
- **High Quality**: Leverages MuseTalk 1.5 for realistic lip-sync
- **Configurable**: Adjust resolution, frame rate, and output format
- **Progress Feedback**: Visual progress bars during processing
- **Scriptable**: Easy to integrate into video production pipelines

## Quick Start

```bash
# Full pipeline: generate lip-synced avatar with transparent background
./generate-lipsync.sh work/avatar-magenta.mp4 work/narration.wav myavatar

# Composite avatar onto a screencast video
./composite-video.sh ~/Movies/screencast.mp4 work/myavatar-raw.mp4 work/myavatar-transparent.webm work/final.mp4
```

### Basic CLI Usage

```bash
# Basic usage with image
musetalk-cli -r avatar.png -a narration.wav -o output.mp4

# With video reference (for longer content)
musetalk-cli -r reference.mp4 -a narration.wav -o output.mp4

# With custom server
musetalk-cli -r avatar.png -a narration.wav -o output.mp4 --server http://gpu-server:3015
```

## Installation

### Prerequisites

- MuseTalk server running (see [Server Setup](#server-setup))
- FFmpeg libraries installed

### From Source

```bash
git clone https://github.com/yourusername/musetalk-client-rs
cd musetalk-client-rs
cargo build --release
```

### Binary Releases

Pre-built binaries will be available for:
- Linux (x86_64, aarch64)
- macOS (Intel, Apple Silicon)
- Windows

## Usage

```
musetalk-cli [OPTIONS] --reference <REFERENCE> --audio <AUDIO> --output <OUTPUT>

Options:
  -r, --reference <REFERENCE>      Path to reference image (PNG/JPEG) or video (MP4)
  -a, --audio <AUDIO>              Path to audio file (WAV/MP3/FLAC)
  -o, --output <OUTPUT>            Path for output video (MP4)
  -s, --server <SERVER>            MuseTalk server URL [default: http://localhost:3015]
      --resolution <RESOLUTION>    Output resolution (WxH) [default: 512x512]
  -f, --fps <FPS>                  Frame rate [default: 30]
      --face-center <FACE_CENTER>  Manual face center coordinates (X,Y)
  -v, --verbose                    Enable verbose output
  -q, --quiet                      Suppress all output except errors
  -n, --dry-run                    Dry run - validate inputs without processing
  -h, --help                       Print help
  -V, --version                    Print version
```

### Examples

```bash
# Generate lip-synced video from avatar image and narration
musetalk-cli -r my-avatar.png -a podcast-intro.wav -o intro.mp4

# Use a video reference for longer content
musetalk-cli -r talking-head.mp4 -a speech.wav -o result.mp4

# Use a remote GPU server for processing
musetalk-cli -r avatar.png -a speech.mp3 -o result.mp4 \
  --server http://192.168.1.100:3015

# Validate inputs without processing (dry run)
musetalk-cli -r avatar.png -a narration.wav -o output.mp4 --dry-run

# Higher quality output
musetalk-cli -r avatar.png -a narration.flac -o hd-output.mp4 \
  --resolution 1024x1024 --fps 60
```

## Complete Workflow

The full workflow for creating an animated avatar overlay involves several steps. Scripts are provided to automate this process.

### Scripts

| Script | Purpose |
|--------|---------|
| `generate-lipsync.sh` | Full pipeline: stretch video, lip-sync, background removal, create WebM |
| `composite-video.sh` | Overlay transparent avatar onto background video using chromakey |
| `stretch.sh` | Stretch video to match audio duration with frame interpolation |

### Workflow Steps

#### 1. Prepare Reference Video

Create a short video (2-10 seconds) of your avatar with a **magenta background** (#c94591). Magenta works best because it doesn't overlap with skin tones.

#### 2. Adjust Video Length to Match Audio

The reference video must match your audio duration. Use frame interpolation (not just speed change) to preserve quality:

```bash
# Calculate stretch factor: audio_duration / video_duration
./stretch.sh reference.mp4 2.5 stretched.mp4

# Or use minterpolate directly
ffmpeg -i reference.mp4 \
  -filter:v "minterpolate=fps=30:mi_mode=dup,setpts=2.5*PTS" \
  -r 30 -an stretched.mp4
```

#### 3. Generate Lip-Synced Video

Send the stretched video and audio to the MuseTalk server:

```bash
./target/release/musetalk-cli \
  --server http://your-server:3015 \
  -r stretched.mp4 \
  -a narration.wav \
  -o lipsynced.mp4
```

#### 4. Remove Background

Two methods available depending on your use case:

**For web playback (HTML video tag):** Use ML-based removal (rembg)
```bash
# Requires: uv pip install "rembg[cli]" onnxruntime
.venv/bin/rembg p -m u2net frames/ frames_alpha/
```

**For compositing onto other videos:** Use chromakey (cleaner edges)
```bash
ffmpeg -i lipsynced.mp4 \
  -vf "chromakey=0xc94591:0.08:0.05" \
  -c:v libvpx-vp9 -pix_fmt yuva420p \
  transparent.webm
```

#### 5. Create Thumbnail (Optional)

Scale down for overlay use:

```bash
ffmpeg -i transparent.webm \
  -vf "scale=160:160" \
  -c:v libvpx-vp9 -pix_fmt yuva420p -b:v 200k \
  -c:a libopus -b:a 32k \
  thumb.webm
```

#### 6. Composite onto Screencast

Overlay the avatar onto your background video:

```bash
./composite-video.sh ~/Movies/screencast.mp4 \
  work/avatar-raw.mp4 \
  work/avatar-transparent.webm \
  work/final-video.mp4
```

The script uses chromakey on the original magenta background for clean edges (no dark halos).

### One-Command Pipeline

For convenience, `generate-lipsync.sh` runs steps 2-5 automatically:

```bash
./generate-lipsync.sh work/avatar-magenta.mp4 work/narration.wav myavatar

# Outputs:
#   work/myavatar-raw.mp4          - Lip-synced with magenta background
#   work/myavatar-transparent.webm - Transparent WebM (for web)
#   work/myavatar-thumb.webm       - 160x160 thumbnail
```

See [examples.md](examples.md) for detailed command reference and troubleshooting.

## Backend Server

The MuseTalk inference server runs the ML models. See the [backend/](backend/) directory or clone separately:

```bash
# Backend setup (requires CUDA GPU)
git clone https://github.com/yourusername/musetalk-backend
cd musetalk-backend
./setup.sh
./start-server.sh --port 3015
```

Backend documentation: [backend/README.md](backend/README.md)

## Server Setup

The CLI requires a MuseTalk inference server. See the [MuseTalk repository](https://github.com/TMElyralab/MuseTalk) for model setup.

### Quick Server Start

```bash
# Clone MuseTalk
git clone https://github.com/TMElyralab/MuseTalk
cd MuseTalk

# Set up environment
conda create -n musetalk python=3.10
conda activate musetalk
pip install -r requirements.txt

# Download models (see MuseTalk README)

# Start server (wrapper implementation needed)
python server.py --port 8000
```

## Configuration

Configuration can be set via:
1. Command-line arguments (highest priority)
2. Environment variables
3. Config file (`~/.config/musetalk/config.toml`)

### Config File Example

```toml
[server]
url = "http://localhost:3015"
timeout_secs = 300

[output]
default_fps = 30
default_format = "mp4"

[processing]
chunk_duration_secs = 30
```

### Environment Variables

```bash
export MUSETALK_SERVER_URL=http://gpu-server:3015
export MUSETALK_LOG_LEVEL=debug
```

## Documentation

| Document | Description |
|----------|-------------|
| [Product Requirements](docs/prd.md) | Features, requirements, and use cases |
| [Architecture](docs/architecture.md) | System design and component structure |
| [Design Decisions](docs/design.md) | Technical decisions and rationale |
| [Implementation Plan](docs/plan.md) | Development phases and milestones |
| [Project Status](docs/status.md) | Current progress and blockers |

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- -r test.png -a test.wav -o out.mp4
```

### Project Structure

```
musetalk-client-rs/
+-- src/
|   +-- main.rs          # Entry point
|   +-- lib.rs           # Library exports
|   +-- cli/             # CLI argument parsing
|   +-- loader/          # Image/audio loading
|   +-- client/          # MuseTalk server client
|   +-- assembler/       # Video assembly
|   +-- config/          # Configuration handling
+-- docs/                # Documentation
+-- tests/               # Integration tests
+-- Cargo.toml
```

## How It Works

1. **Input Processing**: Load and validate avatar image and audio file
2. **Server Communication**: Send image and audio to MuseTalk server
3. **Inference**: MuseTalk generates lip-synced frames using:
   - Whisper for audio encoding
   - VAE for image encoding
   - UNet for latent space inpainting
4. **Video Assembly**: Receive frames, add audio track, encode to video

## Requirements

- Rust 1.70+
- FFmpeg development libraries
- MuseTalk server with GPU (CUDA recommended)

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- [MuseTalk](https://github.com/TMElyralab/MuseTalk) by TMElyralab for the lip-sync model
- The Rust community for excellent CLI and media processing libraries

## Contributing

Contributions welcome! Please read the documentation in `docs/` before submitting PRs.
