# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust CLI client for MuseTalk that generates lip-synced avatar videos. Takes a static image + audio file and produces animated video via a remote MuseTalk inference server.

## Build Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # Run all tests
cargo test test_name           # Run specific test
cargo test -- --nocapture      # Run tests with output
cargo clippy --all-targets --all-features -- -D warnings  # Lint (zero warnings required)
cargo fmt --all                # Format code
cargo doc --open               # Generate and view docs
```

## Pre-Commit Process (Mandatory)

Before every commit, run in order:
1. `cargo test` - all tests must pass
2. `cargo clippy --all-targets --all-features -- -D warnings` - zero warnings
3. `cargo fmt --all` - format code
4. `markdown-checker -f "**/*.md"` - validate markdown (ASCII-only)
5. `sw-checklist` - project compliance check

## Architecture

**Client-Server Model**: The Rust CLI is a thin client; ML inference runs on a separate MuseTalk Python server.

**Planned Module Structure**:
- `cli/` - Argument parsing with clap, command dispatch
- `loader/` - Image (PNG/JPEG) and audio (WAV/MP3/FLAC) loading
- `client/` - HTTP client for MuseTalk server API
- `assembler/` - Frame assembly and video encoding via FFmpeg
- `config/` - Configuration file and environment variable handling

**Data Flow**: Image + Audio -> Loader -> HTTP to Server -> Receive Frames -> Assembler -> MP4 Output

**Key Dependencies** (planned):
- clap (CLI), tokio (async), reqwest (HTTP), serde (JSON)
- image (loading), symphonia (audio), ffmpeg-next (video encoding)
- indicatif (progress), tracing (logging), anyhow/thiserror (errors)

## Code Quality Standards

- Rust 2024 edition
- Zero clippy warnings (enforced with `-D warnings`)
- Files under 500 lines (prefer 200-300)
- Functions under 50 lines (prefer 10-30)
- Max 3 TODO comments per file; never commit FIXMEs
- Use inline format args: `format!("{name}")` not `format!("{}", name)`
- Module docs with `//!`, item docs with `///`

## Development Workflow

This project follows TDD (Red/Green/Refactor):
1. Write failing test
2. Write minimal code to pass
3. Refactor while keeping tests green

## Documentation

- `docs/prd.md` - Product requirements
- `docs/architecture.md` - System design and API contracts
- `docs/design.md` - Technical decisions and rationale
- `docs/plan.md` - Implementation phases
- `docs/process.md` - Development workflow details
- `docs/ai_agent_instructions.md` - Extended AI agent guidelines
