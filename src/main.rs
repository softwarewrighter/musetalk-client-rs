//! MuseTalk CLI entry point.

use anyhow::{Context, Result};
use musetalk_cli::assembler::{VideoAssembler, check_ffmpeg};
use musetalk_cli::client::{MuseTalkClient, ReferenceInput};
use musetalk_cli::loader::{load_audio, load_image, load_video};
use musetalk_cli::{Args, ReferenceType, validate_inputs};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse_args();

    // Initialize logging based on verbosity
    let filter = if args.verbose {
        EnvFilter::new("debug")
    } else if args.quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::debug!("Parsed arguments: {args:?}");

    // Validate inputs and determine reference type
    let ref_type = validate_inputs(&args.reference, &args.audio, &args.output)
        .context("Input validation failed")?;

    // Check FFmpeg availability
    check_ffmpeg().context("FFmpeg check failed")?;

    // Dry run mode - exit after validation
    if args.dry_run {
        println!("Dry run: inputs validated successfully");
        println!(
            "  Reference: {} ({})",
            args.reference.display(),
            match ref_type {
                ReferenceType::Image => "image",
                ReferenceType::Video => "video",
            }
        );
        println!("  Audio: {}", args.audio.display());
        println!("  Output: {}", args.output.display());
        println!("  Server: {}", args.server);
        println!("  Resolution: {}", args.resolution);
        println!("  FPS: {}", args.fps);
        println!("  FFmpeg: available");
        return Ok(());
    }

    // Load reference and audio
    let audio_data = load_audio(&args.audio).context("Failed to load audio")?;
    println!(
        "Loaded audio: {:.2}s, {} Hz from {}",
        audio_data.duration_secs,
        audio_data.sample_rate,
        args.audio.display()
    );

    // Load reference based on type
    let image_data;
    let video_data;
    let reference_input = match ref_type {
        ReferenceType::Image => {
            image_data = load_image(&args.reference).context("Failed to load image")?;
            println!(
                "Loaded image: {}x{} from {}",
                image_data.width,
                image_data.height,
                args.reference.display()
            );
            ReferenceInput::Image(&image_data)
        }
        ReferenceType::Video => {
            video_data = load_video(&args.reference).context("Failed to load video")?;
            println!(
                "Loaded video: {} bytes from {}",
                video_data.file_size,
                args.reference.display()
            );
            ReferenceInput::Video(&video_data)
        }
    };

    // Try to connect to MuseTalk server
    let client = MuseTalkClient::new(&args.server);
    let server_available = match client.health_check().await {
        Ok(health) => {
            println!(
                "Connected to MuseTalk server: {} (version: {})",
                health.status,
                health.version.unwrap_or_else(|| "unknown".to_string())
            );
            true
        }
        Err(e) => {
            tracing::warn!("Server not available: {e}");
            println!("MuseTalk server not available at {}", args.server);
            println!("Falling back to static video mode (no lip-sync)");
            false
        }
    };

    // Create video assembler
    let assembler = VideoAssembler::new(args.fps).context("Failed to create video assembler")?;

    if server_available {
        // Request inference from server
        println!("Requesting lip-sync inference...");
        let response = client
            .infer(reference_input, &audio_data, args.fps)
            .await
            .context("Inference request failed")?;

        println!(
            "Received {} frames, assembling video...",
            response.total_frames
        );

        // Extract frame data
        let frames: Vec<String> = response.frames.into_iter().map(|f| f.data).collect();

        // Assemble video from frames
        assembler
            .assemble_from_frames(&frames, &args.audio, &args.output)
            .context("Failed to assemble video")?;
    } else {
        // Fallback: create static video with image + audio (only works for image reference)
        match ref_type {
            ReferenceType::Image => {
                let image_data = load_image(&args.reference).context("Failed to load image")?;
                println!("Creating static video...");
                assembler
                    .assemble_static(
                        &image_data,
                        &audio_data,
                        &args.reference,
                        &args.audio,
                        &args.output,
                    )
                    .context("Failed to create static video")?;
            }
            ReferenceType::Video => {
                println!("Warning: Video reference requires server connection.");
                println!("Cannot create fallback video from video reference.");
                return Err(anyhow::anyhow!(
                    "Server unavailable and video reference cannot be used for static fallback"
                ));
            }
        }
    }

    // Report success
    let output_size = std::fs::metadata(&args.output)
        .map(|m| m.len())
        .unwrap_or(0);
    println!();
    println!("Output video created successfully!");
    println!("  File: {}", args.output.display());
    println!("  Size: {:.2} MB", output_size as f64 / 1_000_000.0);
    println!("  Duration: {:.2}s", audio_data.duration_secs);
    println!("  FPS: {}", args.fps);

    if !server_available {
        println!();
        println!("Note: This is a static video (no lip-sync).");
        println!(
            "Start a MuseTalk server at {} for lip-sync generation.",
            args.server
        );
    }

    Ok(())
}
