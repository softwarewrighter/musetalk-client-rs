//! Command-line interface argument parsing.

use clap::Parser;
use std::path::PathBuf;

/// MuseTalk CLI - Generate lip-synced avatar videos.
///
/// Takes a reference (static image or video) and an audio file, produces
/// an animated video of the avatar speaking with realistic lip movements.
#[derive(Parser, Debug)]
#[command(name = "musetalk-cli")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to reference image (PNG/JPEG) or video (MP4)
    #[arg(short = 'r', long)]
    pub reference: PathBuf,

    /// Path to audio file (WAV/MP3/FLAC)
    #[arg(short, long)]
    pub audio: PathBuf,

    /// Path for output video (MP4)
    #[arg(short, long)]
    pub output: PathBuf,

    /// MuseTalk server URL
    #[arg(short, long, default_value = "http://localhost:3015")]
    pub server: String,

    /// Output resolution (WxH)
    #[arg(long, default_value = "512x512")]
    pub resolution: String,

    /// Frame rate
    #[arg(short, long, default_value_t = 30)]
    pub fps: u32,

    /// Manual face center coordinates (X,Y)
    #[arg(long)]
    pub face_center: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long)]
    pub quiet: bool,

    /// Dry run - validate inputs without processing
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

impl Args {
    /// Parse arguments from command line.
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Parse arguments from an iterator (for testing).
    pub fn try_parse_from_args<I, T>(iter: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::try_parse_from(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_args() {
        let args = Args::try_parse_from_args([
            "musetalk-cli",
            "-r",
            "avatar.png",
            "-a",
            "audio.wav",
            "-o",
            "output.mp4",
        ])
        .unwrap();

        assert_eq!(args.reference, PathBuf::from("avatar.png"));
        assert_eq!(args.audio, PathBuf::from("audio.wav"));
        assert_eq!(args.output, PathBuf::from("output.mp4"));
        assert_eq!(args.server, "http://localhost:3015");
        assert_eq!(args.fps, 30);
        assert!(!args.verbose);
        assert!(!args.quiet);
    }

    #[test]
    fn test_parse_all_args() {
        let args = Args::try_parse_from_args([
            "musetalk-cli",
            "-r",
            "avatar.png",
            "-a",
            "audio.wav",
            "-o",
            "output.mp4",
            "-s",
            "http://gpu:8000",
            "--resolution",
            "1024x1024",
            "-f",
            "60",
            "--face-center",
            "256,300",
            "-v",
            "-n",
        ])
        .unwrap();

        assert_eq!(args.server, "http://gpu:8000");
        assert_eq!(args.resolution, "1024x1024");
        assert_eq!(args.fps, 60);
        assert_eq!(args.face_center, Some("256,300".to_string()));
        assert!(args.verbose);
        assert!(args.dry_run);
    }

    #[test]
    fn test_parse_video_reference() {
        let args = Args::try_parse_from_args([
            "musetalk-cli",
            "--reference",
            "avatar.mp4",
            "-a",
            "audio.wav",
            "-o",
            "output.mp4",
        ])
        .unwrap();

        assert_eq!(args.reference, PathBuf::from("avatar.mp4"));
    }

    #[test]
    fn test_dry_run_flag() {
        let args = Args::try_parse_from_args([
            "musetalk-cli",
            "-r",
            "avatar.png",
            "-a",
            "audio.wav",
            "-o",
            "output.mp4",
            "--dry-run",
        ])
        .unwrap();

        assert!(args.dry_run);
    }

    #[test]
    fn test_missing_required_args() {
        let result = Args::try_parse_from_args(["musetalk-cli", "-r", "avatar.png"]);
        assert!(result.is_err());
    }
}
