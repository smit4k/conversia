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
    #[description = "Image format to convert to"] output_format: OutputFormat,
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
    let bytes = match file.download().await {
        Ok(data) => data,
        Err(e) => {
            let embed = CreateEmbed::default()
                .title("❌ Download failed")
                .description("Failed to download the attached file")
                .color(0xff4444);

                let reply = poise::CreateReply::default().embed(embed);
                ctx.send(reply).await?;
                return Err(e.into());
        }
    };

    // Load image
    let img = match load_from_memory(&bytes) {
        Ok(i) => i,
        _ => {
            let embed = CreateEmbed::default()
                .title("❌ Could not read image")
                .description("The uploaded file isn't a valid image format.")
                .field("Supported formats", "jpg, png, webp, gif, bmp, tiff", false)
                .color(0xff4444);

            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Ok(());
        }
    };

    // Convert and save to temp file
    let mut buf = Cursor::new(Vec::new());

    // Write image to chosen output format
    if let Err(e) = img.write_to(&mut buf, format) {
        let embed = CreateEmbed::default()
            .title("❌ Conversion failed")
            .description("Could not convert the image to the selected format.")
            .color(0xff4444);

        let reply = poise::CreateReply::default().embed(embed);
        ctx.send(reply).await?;

        return Err(e.into());
    }

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
        .color(0x27ae60);

    let reply = poise::CreateReply::default()
        .embed(embed)
        .attachment(attachment);
        
    ctx.send(reply).await?;

    Ok(())
}
