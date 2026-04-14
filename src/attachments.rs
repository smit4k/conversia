use poise::serenity_prelude::Attachment;
use std::path::Path;

pub const MAX_ATTACHMENT_BYTES: u64 = 25 * 1024 * 1024;
pub const MAX_TRANSFORMED_BYTES: usize = MAX_ATTACHMENT_BYTES as usize;
pub const MAX_IMAGE_PIXELS: u64 = 16_000_000;
const DEFAULT_FILENAME: &str = "file";
const MAX_FILENAME_LEN: usize = 120;

pub fn validate_attachment_size(file: &Attachment) -> Result<(), String> {
    let size = u64::from(file.size);
    if size > MAX_ATTACHMENT_BYTES {
        return Err(format!(
            "Files larger than {} MiB are rejected to avoid exhausting bot resources.",
            MAX_ATTACHMENT_BYTES / (1024 * 1024)
        ));
    }

    Ok(())
}

pub fn validate_output_size(size: usize, label: &str) -> Result<(), String> {
    if size > MAX_TRANSFORMED_BYTES {
        return Err(format!(
            "{} exceeds the {} MiB output limit.",
            label,
            MAX_TRANSFORMED_BYTES / (1024 * 1024)
        ));
    }

    Ok(())
}

pub fn sanitize_filename(filename: &str) -> String {
    let normalized = filename.replace('\\', "/");
    let candidate = Path::new(&normalized)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_FILENAME);

    let mut sanitized: String = candidate
        .chars()
        .filter(|ch| !ch.is_control())
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect();

    sanitized = sanitized.trim_matches('.').trim().to_string();
    if sanitized.is_empty() {
        sanitized = DEFAULT_FILENAME.to_string();
    }
    if sanitized.len() > MAX_FILENAME_LEN {
        sanitized.truncate(MAX_FILENAME_LEN);
    }

    sanitized
}

pub fn validate_image_dimensions(width: u32, height: u32) -> Result<(), String> {
    let pixel_count = u64::from(width) * u64::from(height);
    if pixel_count > MAX_IMAGE_PIXELS {
        return Err(format!(
            "Images above {} pixels are rejected to avoid excessive memory use.",
            MAX_IMAGE_PIXELS
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_ATTACHMENT_BYTES, sanitize_filename, validate_image_dimensions, validate_output_size,
    };

    #[test]
    fn sanitize_filename_strips_paths_and_controls() {
        assert_eq!(sanitize_filename("../secret.txt"), "secret.txt");
        assert_eq!(sanitize_filename("folder\\\\photo.png"), "photo.png");
        assert_eq!(sanitize_filename("\n"), "file");
    }

    #[test]
    fn output_size_validation_rejects_large_buffers() {
        assert!(validate_output_size(1024, "output").is_ok());
        assert!(validate_output_size(MAX_ATTACHMENT_BYTES as usize + 1, "output").is_err());
    }

    #[test]
    fn image_dimension_validation_rejects_large_images() {
        assert!(validate_image_dimensions(2000, 2000).is_ok());
        assert!(validate_image_dimensions(5000, 5000).is_err());
    }
}
