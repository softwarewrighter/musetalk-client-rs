//! Input loading modules for images, audio, and video.

pub mod audio;
pub mod image;
pub mod video;

pub use audio::{AudioData, load_audio};
pub use image::{ImageData, load_image};
pub use video::{VideoData, load_video};
