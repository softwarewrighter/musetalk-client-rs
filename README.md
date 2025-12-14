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
# Basic usage
musetalk-cli -i avatar.png -a narration.wav -o output.mp4

# With custom server
musetalk-cli -i avatar.png -a narration.wav -o output.mp4 --server http://gpu-server:8000

# With options
musetalk-cli -i avatar.png -a narration.wav -o output.mp4 --resolution 1024x1024 --fps 30
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
musetalk-cli [OPTIONS] --image <IMAGE> --audio <AUDIO> --output <OUTPUT>

Options:
  -i, --image <IMAGE>       Path to avatar image (PNG/JPEG)
  -a, --audio <AUDIO>       Path to audio file (WAV/MP3/FLAC)
  -o, --output <OUTPUT>     Path for output video (MP4)
  -s, --server <URL>        MuseTalk server URL [default: http://localhost:8000]
  -r, --resolution <WxH>    Output resolution [default: 512x512]
  -f, --fps <FPS>           Frame rate [default: 30]
      --face-center <X,Y>   Manual face center coordinates
      --format <FORMAT>     Output format [default: mp4]
  -v, --verbose             Enable verbose output
  -q, --quiet               Suppress all output except errors
  -h, --help                Print help
  -V, --version             Print version
```

### Examples

```bash
# Generate lip-synced video from avatar and narration
musetalk-cli -i my-avatar.png -a podcast-intro.wav -o intro.mp4

# Use a remote GPU server for processing
musetalk-cli -i avatar.png -a speech.mp3 -o result.mp4 \
  --server http://192.168.1.100:8000

# Higher quality output
musetalk-cli -i avatar.png -a narration.flac -o hd-output.mp4 \
  --resolution 1024x1024 --fps 60

# Batch process multiple files (coming soon)
musetalk-cli batch -d ./inputs -o ./outputs
```

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
url = "http://localhost:8000"
timeout_secs = 300

[output]
default_fps = 30
default_format = "mp4"

[processing]
chunk_duration_secs = 30
```

### Environment Variables

```bash
export MUSETALK_SERVER_URL=http://gpu-server:8000
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
RUST_LOG=debug cargo run -- -i test.png -a test.wav -o out.mp4
```

### Project Structure

```
musetalk-client-rs/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library exports
│   ├── cli/             # CLI argument parsing
│   ├── loader/          # Image/audio loading
│   ├── client/          # MuseTalk server client
│   ├── assembler/       # Video assembly
│   └── config/          # Configuration handling
├── docs/                # Documentation
├── tests/               # Integration tests
└── Cargo.toml
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
