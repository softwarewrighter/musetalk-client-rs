# Implementation Plan: MuseTalk CLI

## Overview

This document outlines the implementation plan for the MuseTalk CLI, organized into phases with concrete deliverables.

---

## Phase 1: Project Foundation

### 1.1 Project Setup
- [ ] Initialize Cargo project structure
- [ ] Configure `Cargo.toml` with dependencies
- [ ] Set up module structure (`cli/`, `loader/`, `client/`, `assembler/`)
- [ ] Configure CI/CD (GitHub Actions)
- [ ] Add license file (MIT)
- [ ] Create `.gitignore`

### 1.2 CLI Framework
- [ ] Define argument structure with `clap` derive
- [ ] Implement help text and version info
- [ ] Add input validation (file existence, format checks)
- [ ] Set up logging with `tracing`
- [ ] Create error types with `thiserror`

### Deliverable
Basic CLI that parses arguments and validates inputs, displaying help and errors appropriately.

```bash
$ musetalk-cli --help
$ musetalk-cli -i nonexistent.png -a audio.wav -o out.mp4
Error: Image file not found: nonexistent.png
```

---

## Phase 2: Input Processing

### 2.1 Image Loader
- [ ] Load PNG images with `image` crate
- [ ] Load JPEG images
- [ ] Validate image dimensions (minimum size)
- [ ] Convert to RGB format (strip alpha)
- [ ] Implement face detection/cropping (optional, defer to server)
- [ ] Encode to base64 for API transmission

### 2.2 Audio Loader
- [ ] Load WAV files with `symphonia`
- [ ] Load MP3 files
- [ ] Load FLAC files
- [ ] Validate audio (non-empty, reasonable duration)
- [ ] Resample to target sample rate (16kHz for Whisper)
- [ ] Convert stereo to mono
- [ ] Encode to base64 for API transmission

### Deliverable
CLI loads and validates image/audio files, reporting format and metadata.

```bash
$ musetalk-cli -i avatar.png -a speech.wav -o out.mp4 --verbose
Image: avatar.png (512x512, RGB)
Audio: speech.wav (16000 Hz, mono, 5.2s)
Ready to process...
```

---

## Phase 3: Server Communication

### 3.1 HTTP Client
- [ ] Create async HTTP client with `reqwest`
- [ ] Implement health check endpoint (`GET /health`)
- [ ] Implement inference endpoint (`POST /infer`)
- [ ] Handle request timeouts
- [ ] Implement retry logic with exponential backoff
- [ ] Add connection pooling

### 3.2 Request/Response Types
- [ ] Define `InferenceRequest` struct
- [ ] Define `InferenceResponse` struct
- [ ] Define `Frame` struct for individual frames
- [ ] Implement JSON serialization/deserialization
- [ ] Handle streaming responses (chunked transfer)

### 3.3 Error Handling
- [ ] Connection errors (server unreachable)
- [ ] Timeout errors
- [ ] HTTP error codes (4xx, 5xx)
- [ ] Invalid response handling
- [ ] Server-side inference errors

### Deliverable
CLI can communicate with MuseTalk server, sending requests and receiving responses.

```bash
$ musetalk-cli -i avatar.png -a speech.wav -o out.mp4
Connecting to server at http://localhost:8000...
Server version: MuseTalk 1.5
Sending inference request...
Received 150 frames
```

---

## Phase 4: Video Assembly

### 4.1 Frame Processing
- [ ] Decode base64 frames from server response
- [ ] Validate frame dimensions and format
- [ ] Handle frame ordering (ensure sequence)
- [ ] Buffer frames efficiently (don't hold all in memory)

### 4.2 Video Encoding
- [ ] Initialize FFmpeg encoder via `ffmpeg-next`
- [ ] Configure H.264 codec with sensible defaults
- [ ] Set frame rate from CLI option
- [ ] Set output resolution
- [ ] Write frames to video stream
- [ ] Add audio track to output
- [ ] Finalize and close output file

### 4.3 Output Formats
- [ ] MP4 container with H.264 (primary)
- [ ] WebM container with VP9 (secondary)
- [ ] Configurable quality/bitrate

### Deliverable
CLI produces playable video files with synchronized audio.

```bash
$ musetalk-cli -i avatar.png -a speech.wav -o out.mp4
...
Output written: out.mp4 (15.2 MB, 5.2s, 30fps)
```

---

## Phase 5: User Experience

### 5.1 Progress Reporting
- [ ] Add progress bar for file loading
- [ ] Add progress bar for server upload
- [ ] Add progress bar for frame generation
- [ ] Add progress bar for video encoding
- [ ] Handle non-TTY output (no progress bars)
- [ ] Implement `--quiet` mode
- [ ] Implement `--verbose` mode

### 5.2 Configuration
- [ ] Implement config file loading (`~/.config/musetalk/config.toml`)
- [ ] Support environment variable overrides
- [ ] Merge configuration sources (CLI > env > file > defaults)
- [ ] Add `config` subcommand to show/edit config

### 5.3 Output Options
- [ ] `--resolution` flag for output size
- [ ] `--fps` flag for frame rate
- [ ] `--format` flag for container format
- [ ] `--quality` flag for encoding quality

### Deliverable
Polished CLI with progress feedback and flexible configuration.

```bash
$ musetalk-cli -i avatar.png -a speech.wav -o out.mp4
Loading image...          [################] 100%
Loading audio...          [################] 100%
Generating lip-sync...    [########--------] 53% (79/150 frames)
```

---

## Phase 6: Advanced Features

### 6.1 Batch Processing
- [ ] `batch` subcommand for multiple files
- [ ] Directory scanning mode
- [ ] Manifest file support (JSON/TOML)
- [ ] Parallel processing option
- [ ] Summary report at completion

### 6.2 Face Configuration
- [ ] `--face-center` flag for manual face position
- [ ] Face detection integration (local or server-side)
- [ ] Preview mode (show detected face region)

### 6.3 Additional Formats
- [ ] Support more image formats (WebP, BMP, TIFF)
- [ ] Support more audio formats (OGG, AAC, M4A)
- [ ] GIF output for short clips

### Deliverable
Feature-complete CLI with batch processing and advanced options.

```bash
$ musetalk-cli batch -d ./inputs -o ./outputs
Processing 5 jobs...
[1/5] avatar1.png + speech1.wav -> output1.mp4 [OK]
[2/5] avatar2.png + speech2.wav -> output2.mp4 [OK]
...
Completed: 5/5 successful
```

---

## Phase 7: Polish & Release

### 7.1 Testing
- [ ] Unit tests for loader module
- [ ] Unit tests for client module
- [ ] Integration tests with mock server
- [ ] End-to-end tests with real MuseTalk server
- [ ] Test on Linux, macOS, Windows

### 7.2 Documentation
- [ ] Complete README with examples
- [ ] Man page generation
- [ ] Shell completion scripts (bash, zsh, fish)
- [ ] Troubleshooting guide

### 7.3 Distribution
- [ ] Binary releases for Linux (x86_64, aarch64)
- [ ] Binary releases for macOS (Intel, Apple Silicon)
- [ ] Binary releases for Windows
- [ ] Homebrew formula
- [ ] Cargo publish

### Deliverable
Production-ready release with documentation and easy installation.

---

## Milestones

| Milestone | Phases | Description |
|-----------|--------|-------------|
| M1: MVP | 1-4 | Basic functionality: image+audio -> video |
| M2: Usable | 5 | Progress, config, quality of life |
| M3: Complete | 6 | Batch processing, advanced features |
| M4: Release | 7 | Tested, documented, distributed |

---

## Dependencies & Prerequisites

### Development Environment
- Rust 1.70+ (stable)
- FFmpeg development libraries
- MuseTalk server for integration testing

### MuseTalk Server Setup
The CLI requires a running MuseTalk server. Setup steps:

1. Clone MuseTalk repository
2. Create Python environment (Python 3.10)
3. Install dependencies
4. Download model weights
5. Start inference server (to be implemented or adapted from existing code)

### Server API Implementation
Note: MuseTalk may not have a ready-to-use HTTP API. Implementation options:

1. **Wrap existing CLI**: Create HTTP server that calls MuseTalk Python scripts
2. **FastAPI wrapper**: Implement FastAPI server exposing MuseTalk inference
3. **Gradio API**: Use Gradio's built-in API endpoint
4. **Custom server**: Implement dedicated inference server

Recommended: FastAPI wrapper for clean API and good performance.

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| MuseTalk API changes | High | Pin to specific version, abstract API layer |
| FFmpeg linking issues | Medium | Provide static builds, document dependencies |
| Server unavailable | Medium | Clear error messages, retry logic |
| Long processing times | Low | Progress feedback, chunked processing |
| Memory issues with long audio | Medium | Streaming/chunked processing |

---

## Success Criteria

### MVP (M1)
- [ ] Accepts PNG image and WAV audio
- [ ] Connects to MuseTalk server
- [ ] Produces MP4 video with lip-synced avatar
- [ ] Video plays correctly in standard players
- [ ] Audio is synchronized with video

### Production Ready (M4)
- [ ] All input formats supported (PNG/JPEG, WAV/MP3/FLAC)
- [ ] Progress reporting works correctly
- [ ] Configuration file support
- [ ] Batch processing
- [ ] Cross-platform binaries available
- [ ] Documentation complete
- [ ] Tests pass on CI
