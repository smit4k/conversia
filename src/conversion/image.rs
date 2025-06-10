use std::io::Cursor;
use serenity::all::{Attachment, CreateEmbed};
use poise::serenity_prelude::{CreateAttachment};
use image::{load_from_memory, ImageFormat};

use crate::{Context, Error};

#[derive(Debug, poise::ChoiceParameter)]
pub enum OutputFormat {
    #[name = "jpg"]
    Jpg,
    #[name = "png"]
    Png,
    #[name = "webp"]
    Webp,
    #[name = "gif"]
    Gif,
    #[name = "bmp"]
    Bmp,
    #[name = "tiff"]
    Tiff,
}

/// Convert an image
#[poise::command(slash_command)]
pub async fn convert_image(
    ctx: Context<'_>,
    #[description = "Image to convert"] file: Attachment,
    #[description = "Image Format to convert to"] output_format: OutputFormat,
) -> Result<(), Error> {
    ctx.defer().await?;

    let original_extension = file.filename.rsplit('.').next().unwrap_or("tmp");

    // Match user input formats to ImageFormats
    let format = match output_format {
        OutputFormat::Jpg => ImageFormat::Jpeg,
        OutputFormat::Png => ImageFormat::Png,
        OutputFormat::Webp => ImageFormat::WebP,
        OutputFormat::Gif => ImageFormat::Gif,
        OutputFormat::Bmp => ImageFormat::Bmp,
        OutputFormat::Tiff => ImageFormat::Tiff,
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
    let ext = match output_format {
        OutputFormat::Jpg => "jpg",
        OutputFormat::Png => "png",
        OutputFormat::Webp => "webp",
        OutputFormat::Gif => "gif",
        OutputFormat::Bmp => "bmp",
        OutputFormat::Tiff => "tiff",
    };
    
    let filename = format!("converted.{}", ext);
    let attachment = CreateAttachment::bytes(output_bytes, filename);

    let embed = CreateEmbed::default()
        .title("✅ Conversion Complete")
        .description(format!("{} → {}", original_extension, ext))
        .color(0x44ff44);

    let reply = poise::CreateReply::default()
        .embed(embed).
        attachment(attachment);
        
    ctx.send(reply).await?;

    Ok(())
}
