use std::io::Cursor;
use serenity::all::{Attachment, CreateEmbed};
use poise::serenity_prelude::CreateAttachment;
use image::{DynamicImage, ImageOutputFormat};
use crate::{Context, Error};

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum OutputFormat {
    #[name = "JPEG"]
    Jpg,
    #[name = "PNG"]
    Png,
    #[name = "WebP"]
    Webp,
    #[name = "GIF"]
    Gif,
    #[name = "BMP"]
    Bmp,
    #[name = "TIFF"]
    Tiff,
}

impl OutputFormat {
    /// Get the file extension for this format
    const fn extension(self) -> &'static str {
        match self {
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
        }
    }

    /// Convert to image output format with optimized settings
    const fn to_image_format(self) -> ImageOutputFormat {
        match self {
            Self::Jpg => ImageOutputFormat::Jpeg(85),
            Self::Png => ImageOutputFormat::Png,
            Self::Webp => ImageOutputFormat::WebP,
            Self::Gif => ImageOutputFormat::Gif,
            Self::Bmp => ImageOutputFormat::Bmp,
            Self::Tiff => ImageOutputFormat::Tiff,
        }
    }
}

/// Extract file extension from filename
fn get_extension(filename: &str) -> &str {
    filename.rsplit('.').next().unwrap_or("tmp")
}

/// Generate output filename from input filename and format
fn generate_output_filename(input_filename: &str, format: OutputFormat) -> String {
    let base = input_filename
        .rsplit_once('.')
        .map_or(input_filename, |(base, _)| base);
    format!("{}.{}", base, format.extension())
}

/// Optimize image based on output format
fn optimize_image_for_format(img: DynamicImage, format: OutputFormat) -> DynamicImage {
    match format {
        // Convert to RGB for formats that don't support transparency
        OutputFormat::Jpg | OutputFormat::Bmp => {
            if img.color().has_alpha() {
                DynamicImage::ImageRgb8(img.to_rgb8())
            } else {
                img
            }
        }
        // Keep original for formats that support transparency
        _ => img,
    }
}

/// Estimate output buffer size to reduce allocations
fn estimate_output_size(img: &DynamicImage, format: OutputFormat) -> usize {
    let pixel_count = (img.width() * img.height()) as usize;
    match format {
        OutputFormat::Jpg => pixel_count / 4,      // ~25% of raw size for JPEG
        OutputFormat::Png => pixel_count * 2,      // ~200% for PNG (conservative)
        OutputFormat::Webp => pixel_count / 3,     // ~33% for WebP
        OutputFormat::Bmp => pixel_count * 3,      // ~300% for BMP (uncompressed)
        OutputFormat::Gif => pixel_count,          // ~100% for GIF
        OutputFormat::Tiff => pixel_count * 2,     // ~200% for TIFF
    }
}

/// Create success embed for conversion
fn create_success_embed(original_filename: &str, output_filename: &str) -> CreateEmbed {
    let original_ext = get_extension(original_filename);
    let target_ext = get_extension(output_filename);
    
    CreateEmbed::default()
        .title("✅ Image Conversion Complete")
        .description(format!("{} → {}", original_ext, target_ext))
        .color(0x27ae60)
}

/// Create error embed for conversion failure
fn create_error_embed(error: &Error) -> CreateEmbed {
    CreateEmbed::default()
        .title("❌ Image Conversion Failed")
        .description(format!("Error: {}", error))
        .color(0xff4444)
}

/// Helper function that performs the actual image conversion
pub async fn convert_image_inner(
    file: &Attachment,
    output_format: OutputFormat,
) -> Result<(Vec<u8>, String), Error> {
    // Download file data
    let file_data = file
        .download()
        .await
        .map_err(|e| Error::from(format!("Failed to download image: {}", e)))?;

    // Load image in blocking task to avoid blocking the async runtime
    let img = tokio::task::spawn_blocking(move || {
        image::load_from_memory(&file_data)
            .map_err(|e| Error::from(format!("Failed to load image: {}", e)))
    })
    .await
    .map_err(|e| Error::from(format!("Image loading task failed: {}", e)))??;

    // Optimize image for target format
    let optimized_img = optimize_image_for_format(img, output_format);

    // Perform encoding in blocking task
    let output_bytes = tokio::task::spawn_blocking(move || {
        // Pre-allocate buffer with estimated size
        let estimated_size = estimate_output_size(&optimized_img, output_format);
        let mut buf = Cursor::new(Vec::with_capacity(estimated_size));
        
        let image_format = output_format.to_image_format();
        optimized_img
            .write_to(&mut buf, image_format)
            .map_err(|e| Error::from(format!("Failed to encode image: {}", e)))?;
        
        Ok::<Vec<u8>, Error>(buf.into_inner())
    })
    .await
    .map_err(|e| Error::from(format!("Image encoding task failed: {}", e)))??;

    // Generate output filename
    let output_filename = generate_output_filename(&file.filename, output_format);

    Ok((output_bytes, output_filename))
}

/// Convert an image
#[poise::command(slash_command)]
pub async fn convert_image(
    ctx: Context<'_>,
    #[description = "Image to convert"] file: Attachment,
    #[description = "Image format to convert to"] output_format: OutputFormat,
) -> Result<(), Error> {
    ctx.defer().await?;

    match convert_image_inner(&file, output_format).await {
        Ok((converted_bytes, output_filename)) => {
            let attachment = CreateAttachment::bytes(converted_bytes, &output_filename);
            let embed = create_success_embed(&file.filename, &output_filename);

            let reply = poise::CreateReply::default()
                .embed(embed)
                .attachment(attachment);

            ctx.send(reply).await?;
        }
        Err(e) => {
            let embed = create_error_embed(&e);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Err(e);
        }
    }

    Ok(())
}