use std::io::Cursor;
use serenity::all::Attachment;
use poise::serenity_prelude::{CreateAttachment};
use image::{load_from_memory, ImageFormat};

use crate::{Context, Error};

/// Convert an image
#[poise::command(slash_command)]
pub async fn convert_image(
    ctx: Context<'_>,
    #[description = "Image to convert"] file: Attachment,
    #[description = "Format to convert to (e.g., jpg, png, webp)"] output_format: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Match user input formats to ImageFormats
    let format = match output_format.to_lowercase().as_str() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::WebP,
        "gif" => ImageFormat::Gif,
        "bmp" => ImageFormat::Bmp,
        "tiff" | "tif" => ImageFormat::Tiff,
        _ => {
            ctx.say("Unsupported format! Supported formats: jpg, png, webp, gif, bmp, tiff").await?;
            return Ok(());
        }
    };

    // Download the attachment
    let bytes = file.download().await?;

    // Load image
    let img = load_from_memory(&bytes)?;

    // Convert and save to temp file
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, format)?;
    let output_bytes = buf.into_inner();

    // Upload the file back to Discord
    let filename = format!("converted.{}", output_format.to_lowercase());
    let attachment = CreateAttachment::bytes(output_bytes, filename);
    let reply = poise::CreateReply::default().attachment(attachment);
    ctx.send(reply).await?;

    Ok(())
}
