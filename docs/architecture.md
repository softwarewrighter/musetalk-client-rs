# Technical Architecture: MuseTalk CLI

## System Overview

```
+-------------------------------------------------------------------------+
|                           User Environment                               |
+-------------------------------------------------------------------------+
|                                                                          |
|  +--------------+     +----------------------------------------------+  |
|  |              |     |           musetalk-cli (Rust)                |  |
|  |  Input Files |---->|                                              |  |
|  |  - image.png |     |  +---------+  +---------+  +-------------+   |  |
|  |  - audio.wav |     |  | Loader  |--| Client  |--| Assembler   |   |  |
|  |              |     |  +---------+  +----+----+  +------+------+   |  |
|  +--------------+     |                    |              |          |  |
|                       +--------------------+--------------+----------+  |
|                                            |              |             |
|  +--------------+                          |              |             |
|  |              |<-------------------------+--------------+             |
|  | Output Video |                          |                            |
|  |  - output.mp4|                          |                            |
|  |              |                          |                            |
|  +--------------+                          |                            |
|                                            |                            |
+--------------------------------------------+----------------------------+
                                             | HTTP/gRPC
                                             v
+-------------------------------------------------------------------------+
|                        MuseTalk Server (Python)                          |
+-------------------------------------------------------------------------+
|                                                                          |
|  +------------------------------------------------------------------+   |
|  |                         API Layer                                 |   |
|  |   POST /infer     POST /health     WebSocket /stream              |   |
|  +----------------------------------+-------------------------------+   |
|                                     |                                    |
|  +----------------------------------v-------------------------------+   |
|  |                      MuseTalk Pipeline                            |   |
|  |                                                                   |   |
|  |  +----------+   +----------+   +----------+   +--------------+   |   |
|  |  | Whisper  |   |   VAE    |   |   UNet   |   | Face Decoder |   |   |
|  |  | (Audio)  |-->| Encoder  |-->| Latent   |-->|  (Output)    |   |   |
|  |  +----------+   +----------+   +----------+   +--------------+   |   |
|  |                                                                   |   |
|  +-------------------------------------------------------------------+   |
|                                     |                                    |
|                              GPU (CUDA)                                  |
|                                                                          |
+--------------------------------------------------------------------------+
```

## Component Architecture

### Rust CLI Components

```
musetalk-cli/
+-- src/
|   +-- main.rs              # Entry point, CLI parsing
|   +-- lib.rs               # Library exports
|   +-- cli/
|   |   +-- mod.rs           # CLI module
|   |   +-- args.rs          # Argument parsing (clap)
|   |   +-- commands.rs      # Command handlers
|   +-- loader/
|   |   +-- mod.rs           # Loader module
|   |   +-- image.rs         # Image loading/preprocessing
|   |   +-- audio.rs         # Audio loading/preprocessing
|   +-- client/
|   |   +-- mod.rs           # Client module
|   |   +-- http.rs          # HTTP client for MuseTalk API
|   |   +-- types.rs         # Request/Response types
|   |   +-- error.rs         # Client error types
|   +-- assembler/
|   |   +-- mod.rs           # Assembler module
|   |   +-- frames.rs        # Frame assembly
|   |   +-- video.rs         # Video encoding
|   +-- config/
|   |   +-- mod.rs           # Configuration module
|   |   +-- settings.rs      # Config file handling
|   +-- error.rs             # Global error types
+-- Cargo.toml
+-- tests/
    +-- integration/
```

### Module Responsibilities

#### CLI Module (`cli/`)
- Parse command-line arguments using `clap`
- Validate input parameters
- Dispatch to appropriate command handlers
- Handle output formatting and progress display

#### Loader Module (`loader/`)
- Load and validate input image files
- Load and validate audio files
- Perform format conversions if needed
- Extract metadata (dimensions, duration, sample rate)

#### Client Module (`client/`)
- Establish connection to MuseTalk server
- Send inference requests
- Handle chunked/streaming responses
- Manage connection lifecycle and retries

#### Assembler Module (`assembler/`)
- Receive generated frames from client
- Assemble frames into video sequence
- Encode final video output
- Handle audio track synchronization

## Data Flow

### Standard Inference Flow

```
1. Input Parsing
   +-----------------------------------------------------------+
   | CLI receives: --image avatar.png --audio speech.wav       |
   | Validates file existence and formats                       |
   +-----------------------------------------------------------+
                              |
                              v
2. Preprocessing
   +-----------------------------------------------------------+
   | Image: Load PNG/JPEG, detect face region, normalize       |
   | Audio: Load WAV, resample if needed, chunk if long        |
   +-----------------------------------------------------------+
                              |
                              v
3. API Request
   +-----------------------------------------------------------+
   | POST /infer                                                |
   | Body: { image: base64, audio: base64, options: {...} }    |
   +-----------------------------------------------------------+
                              |
                              v
4. Server Processing (MuseTalk)
   +-----------------------------------------------------------+
   | a. Whisper encodes audio to embeddings                    |
   | b. VAE encodes image to latent space                      |
   | c. UNet performs latent inpainting                        |
   | d. VAE decodes latent to output frames                    |
   +-----------------------------------------------------------+
                              |
                              v
5. Response Handling
   +-----------------------------------------------------------+
   | Receive frames (streamed or batched)                      |
   | Decode base64 frame data                                  |
   +-----------------------------------------------------------+
                              |
                              v
6. Video Assembly
   +-----------------------------------------------------------+
   | Assemble frames in order                                  |
   | Add audio track                                           |
   | Encode to MP4 (H.264)                                     |
   | Write to output file                                      |
   +-----------------------------------------------------------+
```

## API Contract

### MuseTalk Server API (Expected)

#### Health Check
```
GET /health
Response: { "status": "ok", "version": "1.5" }
```

#### Inference Request
```
POST /infer
Content-Type: application/json

Request:
{
  "image": "<base64-encoded-image>",
  "audio": "<base64-encoded-audio>",
  "options": {
    "fps": 30,
    "face_center": [256, 256],  // optional
    "output_format": "frames"   // "frames" | "video"
  }
}

Response (streaming):
{
  "status": "processing",
  "total_frames": 150,
  "frames": [
    { "index": 0, "data": "<base64-frame>" },
    { "index": 1, "data": "<base64-frame>" },
    ...
  ]
}
```

## Technology Stack

### Rust Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `clap` | CLI argument parsing | 4.x |
| `tokio` | Async runtime | 1.x |
| `reqwest` | HTTP client | 0.11+ |
| `serde` | Serialization | 1.x |
| `image` | Image loading/processing | 0.24+ |
| `symphonia` | Audio decoding | 0.5+ |
| `ffmpeg-next` | Video encoding | 6.x |
| `indicatif` | Progress bars | 0.17+ |
| `tracing` | Logging/diagnostics | 0.1+ |
| `anyhow` | Error handling | 1.x |
| `thiserror` | Error definitions | 1.x |

### External Dependencies

- **MuseTalk Server**: Python-based inference server
- **FFmpeg**: Video encoding (linked via `ffmpeg-next`)
- **CUDA**: GPU acceleration (on server side)

## Error Handling Strategy

```rust
// Hierarchical error types
pub enum CliError {
    Io(std::io::Error),
    ImageLoad(ImageError),
    AudioLoad(AudioError),
    Server(ServerError),
    Video(VideoError),
    Config(ConfigError),
}

pub enum ServerError {
    ConnectionFailed(String),
    Timeout,
    InvalidResponse(String),
    InferenceFailed(String),
}
```

## Configuration

### Config File (`~/.config/musetalk/config.toml`)

```toml
[server]
url = "http://localhost:8000"
timeout_secs = 300

[output]
default_fps = 30
default_format = "mp4"
codec = "h264"

[processing]
chunk_duration_secs = 30
max_retries = 3
```

### Environment Variables

```
MUSETALK_SERVER_URL   - Override server URL
MUSETALK_CONFIG_PATH  - Custom config file path
MUSETALK_LOG_LEVEL    - Logging verbosity (debug, info, warn, error)
```

## Performance Considerations

1. **Streaming**: Use chunked transfer for large audio files
2. **Memory**: Process frames incrementally, don't hold all in memory
3. **Parallelism**: Pipeline frame receiving and video encoding
4. **Caching**: Cache face detection results for batch processing

## Security Considerations

1. **Input Validation**: Validate image/audio before sending to server
2. **TLS**: Use HTTPS for remote server connections
3. **File Permissions**: Respect filesystem permissions for output
4. **No Secrets**: Don't log sensitive data (API keys if added later)
