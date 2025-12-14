# Design Decisions: MuseTalk CLI

## Overview

This document captures key design decisions made for the MuseTalk CLI, including rationale and alternatives considered.

---

## Decision 1: Rust as Implementation Language

### Decision
Implement the CLI in Rust rather than Python.

### Rationale
1. **Performance**: Native compilation provides fast startup and efficient memory usage
2. **Distribution**: Single binary with no runtime dependencies simplifies installation
3. **Type Safety**: Catch errors at compile time, improving reliability
4. **Ecosystem**: Strong libraries for CLI (`clap`), HTTP (`reqwest`), and media processing
5. **Cross-platform**: Easy cross-compilation for Linux, macOS, Windows

### Alternatives Considered
- **Python**: Easier integration with MuseTalk ecosystem, but requires Python runtime
- **Go**: Good for CLI tools, but less mature media processing libraries
- **Node.js**: Fast development, but heavier runtime and packaging complexity

### Trade-offs
- Steeper learning curve for contributors familiar with Python
- MuseTalk's Python codebase means some protocol translation needed
- FFmpeg binding complexity in Rust

---

## Decision 2: Client-Server Architecture

### Decision
The CLI acts as a client to a separate MuseTalk inference server, rather than embedding the model.

### Rationale
1. **Separation of Concerns**: CLI handles I/O; server handles ML inference
2. **Resource Management**: GPU resources managed by server, CLI stays lightweight
3. **Flexibility**: Server can run locally, on a remote machine, or in cloud
4. **Reusability**: Same server can serve multiple clients, web UIs, etc.
5. **Maintenance**: Update model/server independently of CLI

### Alternatives Considered
- **Embedded Model**: Bundle MuseTalk into CLI using PyO3 or ONNX
  - Rejected: Too complex, large binary, hard to maintain
- **Local Process Spawning**: CLI spawns Python process for each job
  - Rejected: Slow startup, complex process management

### Trade-offs
- Requires running a separate server process
- Network overhead for local processing
- More complex deployment (two components)

---

## Decision 3: HTTP API for Client-Server Communication

### Decision
Use HTTP/REST API as primary communication protocol.

### Rationale
1. **Simplicity**: Well-understood, easy to debug with standard tools (curl)
2. **Compatibility**: Works across networks, proxies, firewalls
3. **Tooling**: Rich ecosystem of HTTP clients in Rust (`reqwest`)
4. **Streaming**: HTTP supports chunked transfer encoding for streaming frames

### Alternatives Considered
- **gRPC**: Better performance, strong typing, but more complex setup
- **WebSocket**: Good for bidirectional streaming, but overkill for request-response
- **Unix Socket**: Fast for local, but not portable and no remote support

### Trade-offs
- Slightly higher latency than binary protocols
- Base64 encoding overhead for binary data
- Need to implement streaming manually with chunked encoding

---

## Decision 4: Frame-by-Frame Streaming

### Decision
Server streams individual frames; client assembles into video.

### Rationale
1. **Progress Feedback**: Show progress as frames arrive
2. **Memory Efficiency**: Don't buffer entire video on server
3. **Flexibility**: Client controls final encoding parameters
4. **Resumability**: Could potentially resume from last received frame

### Alternatives Considered
- **Server-Side Video Encoding**: Server returns complete MP4
  - Rejected: Less flexible, larger responses, no progress visibility
- **HLS/DASH Streaming**: Standard video streaming
  - Rejected: Overkill for batch processing use case

### Trade-offs
- More complex client logic
- Requires video encoding in CLI
- Higher bandwidth (uncompressed frames vs compressed video)

---

## Decision 5: FFmpeg for Video Encoding

### Decision
Use FFmpeg (via `ffmpeg-next` crate) for video encoding rather than pure Rust.

### Rationale
1. **Quality**: FFmpeg is the gold standard for video encoding
2. **Codec Support**: H.264, H.265, VP9, AV1, etc. out of the box
3. **Performance**: Highly optimized, hardware acceleration support
4. **Flexibility**: Extensive options for quality, size, compatibility

### Alternatives Considered
- **Pure Rust encoders**: `rav1e` (AV1), `x264` bindings
  - Rejected: Less mature, fewer codecs, more dependencies
- **System FFmpeg**: Call `ffmpeg` binary via subprocess
  - Rejected: Requires FFmpeg installed, process overhead

### Trade-offs
- Complex FFmpeg linking (static vs dynamic)
- Platform-specific build considerations
- Large binary size with static linking

---

## Decision 6: Clap for CLI Parsing

### Decision
Use `clap` crate with derive macros for argument parsing.

### Rationale
1. **Ergonomics**: Derive macros make CLI definition declarative
2. **Features**: Built-in help, completions, validation
3. **Community**: Most popular Rust CLI library, well-maintained
4. **Type Safety**: Arguments parsed directly into typed structs

### Alternatives Considered
- **structopt**: Merged into clap, so use clap directly
- **argh**: Simpler, but fewer features
- **Manual parsing**: Full control, but tedious and error-prone

### Trade-offs
- Compile time cost of derive macros
- Learning curve for advanced features

---

## Decision 7: Async Runtime with Tokio

### Decision
Use Tokio as the async runtime for network I/O.

### Rationale
1. **Performance**: Efficient handling of concurrent I/O
2. **Ecosystem**: `reqwest` and other crates integrate with Tokio
3. **Maturity**: Battle-tested in production systems
4. **Features**: Timeouts, cancellation, channels for pipeline

### Alternatives Considered
- **async-std**: Similar capabilities, smaller community
- **Sync I/O**: Simpler, but blocks on network calls

### Trade-offs
- Binary size increase
- Complexity of async code
- Need to manage runtime lifecycle

---

## Decision 8: Configuration Hierarchy

### Decision
Support configuration from (in priority order):
1. Command-line arguments
2. Environment variables
3. Config file (`~/.config/musetalk/config.toml`)
4. Built-in defaults

### Rationale
1. **Flexibility**: Different mechanisms for different use cases
2. **Convention**: Follows Unix/12-factor app patterns
3. **Scripting**: Environment variables easy in shell scripts
4. **Persistence**: Config file for frequently used settings

### Trade-offs
- More code to merge configuration sources
- Potential confusion about which setting applies
- Need to document precedence clearly

---

## Decision 9: Error Handling with anyhow/thiserror

### Decision
Use `anyhow` for application errors and `thiserror` for library error types.

### Rationale
1. **Ergonomics**: `anyhow` simplifies error propagation
2. **Context**: Easy to add context to errors
3. **Type Safety**: `thiserror` for well-defined error variants
4. **User Experience**: Clear error messages for CLI users

### Pattern
```rust
// Library code - specific error types
#[derive(thiserror::Error, Debug)]
pub enum LoaderError {
    #[error("Failed to load image: {0}")]
    ImageLoad(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

// Application code - anyhow for convenience
fn main() -> anyhow::Result<()> {
    let image = load_image(&path)
        .context("Failed to load avatar image")?;
    Ok(())
}
```

---

## Decision 10: Progress Reporting with indicatif

### Decision
Use `indicatif` crate for progress bars and status display.

### Rationale
1. **User Experience**: Visual feedback during long operations
2. **Features**: Progress bars, spinners, multi-bar support
3. **Terminal Handling**: Handles terminal width, non-TTY gracefully

### Design
```
Loading image...          [################] 100%
Sending to server...      [########--------] 45%
Generating frames...      [##--------------] Frame 23/150
Encoding video...         [################] 100%

Output: output.mp4 (15.2 MB, 5.0s duration)
```

---

## Decision 11: Image Format Support

### Decision
Support PNG and JPEG input formats initially, with extensibility.

### Rationale
1. **Common Formats**: Cover 95%+ of use cases
2. **PNG**: Preferred for avatars (lossless, transparency)
3. **JPEG**: Widely used, efficient for photos
4. **Extensibility**: `image` crate supports many formats if needed

### Processing
- Load image with `image` crate
- Normalize to RGB (strip alpha if present)
- Detect face region for MuseTalk
- Encode as PNG for server transmission (lossless)

---

## Decision 12: Audio Format Support

### Decision
Support WAV, MP3, and FLAC with automatic resampling.

### Rationale
1. **WAV**: Native format, no quality loss
2. **MP3**: Most common compressed format
3. **FLAC**: Lossless compression for quality-conscious users
4. **Resampling**: MuseTalk expects specific sample rate

### Processing
- Decode audio with `symphonia`
- Resample to MuseTalk's expected rate (likely 16kHz for Whisper)
- Convert to mono if stereo
- Chunk long audio for processing

---

## Open Questions

### Q1: Server Discovery
How should the CLI find the MuseTalk server?
- Options: Config file, environment variable, mDNS, default localhost
- Current: Config + env var + CLI flag, defaulting to localhost:8000

### Q2: Authentication
Should the CLI support authentication for remote servers?
- Options: API key, OAuth, mutual TLS
- Current: Defer to v2.0; assume trusted local network

### Q3: Batch Processing Format
How should batch jobs be specified?
- Options: Directory scan, manifest file, glob patterns
- Current: Start with directory scan, add manifest in v2.0

### Q4: Output Format Options
What video containers/codecs to support?
- Options: MP4/H.264 only, or also WebM/VP9, MOV, GIF
- Current: MP4/H.264 default, WebM as secondary, extensible

---

## Design Principles

1. **Fail Fast**: Validate inputs before expensive operations
2. **Informative Errors**: Tell users what went wrong and how to fix it
3. **Sensible Defaults**: Work out of the box with minimal configuration
4. **Progressive Disclosure**: Simple for basic use, configurable for power users
5. **Unix Philosophy**: Do one thing well, compose with other tools
