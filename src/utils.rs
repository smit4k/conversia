use std::path::Path;

/// Format file size in bytes to the most readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["bytes", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= THRESHOLD && unit_index < UNITS.len() - 1 {
        value /= THRESHOLD;
        unit_index += 1;
    }

    if value >= 100.0 {
        format!("{:.0} {}", value, UNITS[unit_index])
    } else if value >= 10.0 {
        format!("{:.1} {}", value, UNITS[unit_index])
    } else {
        format!("{:.2} {}", value, UNITS[unit_index])
    }
}

/// Extract a display-safe file stem while preserving multi-dot names.
pub fn file_stem(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or(filename)
        .to_string()
}

/// Determine whether bytes are safe to preview as text in Discord.
pub fn is_previewable_text(data: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(data) else {
        return false;
    };

    !text.is_empty()
        && text
            .chars()
            .all(|ch| ch == '\n' || ch == '\r' || ch == '\t' || !ch.is_control())
}

fn detect_file_type_name(data: &[u8]) -> &'static str {
    match data {
        [0x89, 0x50, 0x4E, 0x47, ..] => "decoded_image.png",
        [0xFF, 0xD8, 0xFF, ..] => "decoded_image.jpg",
        [0x47, 0x49, 0x46, 0x38, ..] => "decoded_image.gif",
        [0x52, 0x49, 0x46, 0x46, ..] => {
            // Check if it's a WEBP
            if data.len() >= 12 && &data[8..12] == b"WEBP" {
                "decoded_image.webp"
            } else {
                "decoded_audio.wav"
            }
        }
        [0x25, 0x50, 0x44, 0x46, ..] => "decoded_document.pdf",
        [0x50, 0x4B, 0x03, 0x04, ..] => "decoded_archive.zip",
        [0x50, 0x4B, 0x05, 0x06, ..] => "decoded_archive.zip",
        [0x50, 0x4B, 0x07, 0x08, ..] => "decoded_archive.zip",
        _ => {
            // Check if it's plain text
            if data
                .iter()
                .all(|&b| b.is_ascii() && (b.is_ascii_graphic() || b.is_ascii_whitespace()))
            {
                "decoded_text.txt"
            } else {
                "decoded_data.bin"
            }
        }
    }
}

// Helper function to detect file type from magic bytes
pub fn detect_file_type(data: &[u8]) -> String {
    detect_file_type_name(data).to_string()
}

#[cfg(test)]
mod tests {
    use super::{detect_file_type, file_stem, format_file_size, is_previewable_text};

    #[test]
    fn file_stem_preserves_multi_dot_names() {
        assert_eq!(file_stem("example.v1.docx"), "example.v1");
        assert_eq!(file_stem("archive"), "archive");
    }

    #[test]
    fn previewable_text_rejects_binary() {
        assert!(is_previewable_text(b"hello\nworld"));
        assert!(!is_previewable_text(&[0, 159, 146, 150]));
        assert!(!is_previewable_text(&[0, 1, 2, 3]));
    }

    #[test]
    fn file_size_formats_zero_and_kibibytes() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
    }

    #[test]
    fn detect_file_type_handles_common_cases() {
        assert_eq!(detect_file_type(b"%PDF-sample"), "decoded_document.pdf");
        assert_eq!(detect_file_type(b"plain text"), "decoded_text.txt");
        assert_eq!(
            detect_file_type(b"RIFF\x00\x00\x00\x00WAVE"),
            "decoded_audio.wav"
        );
    }
}
