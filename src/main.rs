//! MuseTalk CLI entry point.

use anyhow::{Context, Result};
use musetalk_cli::{Args, validate_inputs};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
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

    // Validate inputs
    validate_inputs(&args.image, &args.audio, &args.output).context("Input validation failed")?;

    tracing::info!("Image: {}", args.image.display());
    tracing::info!("Audio: {}", args.audio.display());
    tracing::info!("Output: {}", args.output.display());
    tracing::info!("Server: {}", args.server);
    tracing::info!("Resolution: {}", args.resolution);
    tracing::info!("FPS: {}", args.fps);

    if let Some(ref face_center) = args.face_center {
        tracing::info!("Face center: {face_center}");
    }

    // Dry run mode - exit after validation
    if args.dry_run {
        println!("Dry run: inputs validated successfully");
        println!("  Image: {}", args.image.display());
        println!("  Audio: {}", args.audio.display());
        println!("  Output: {}", args.output.display());
        println!("  Server: {}", args.server);
        println!("  Resolution: {}", args.resolution);
        println!("  FPS: {}", args.fps);
        return Ok(());
    }

    // TODO: Implement processing pipeline
    println!(
        "MuseTalk CLI v{} - Inputs validated, processing not yet implemented",
        env!("CARGO_PKG_VERSION")
    );

    Ok(())
}
