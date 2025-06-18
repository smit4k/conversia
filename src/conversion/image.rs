use std::io::Cursor;
use serenity::all::{Attachment, CreateEmbed};
use poise::serenity_prelude::{CreateAttachment};

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

pub async fn convert_image_inner(
    file: &Attachment,
    output_format: OutputFormat,
) -> Result<(Vec<u8>, String), Error> {
    let output_extension = match output_format {
        OutputFormat::Jpg => "jpg",
        OutputFormat::Png => "png",
        OutputFormat::Webp => "webp",
        OutputFormat::Bmp => "bmp",
        OutputFormat::Gif => "gif",
        OutputFormat::Tiff => "tiff",
    };

    let file_data = file.download().await.map_err(|e| {
        Error::from(format!("Failed to download image: {}", e))
    })?;

    let img = image::load_from_memory(&file_data).map_err(|e| {
        Error::from(format!("Failed to load image: {}", e))
    })?;

    let mut buf = Cursor::new(Vec::new());

    let image_format = match output_format {
        OutputFormat::Jpg => image::ImageOutputFormat::Jpeg(90),
        OutputFormat::Png => image::ImageOutputFormat::Png,
        OutputFormat::Webp => image::ImageOutputFormat::WebP,
        OutputFormat::Bmp => image::ImageOutputFormat::Bmp,
        OutputFormat::Gif => image::ImageOutputFormat::Gif,
        OutputFormat::Tiff => image::ImageOutputFormat::Tiff,
    };

    img.write_to(&mut buf, image_format)
        .map_err(|e| Error::from(format!("Failed to encode image: {}", e)))?;

    let output_bytes = buf.into_inner();

    // Generate filename based on original, replace extension
    let base_filename = file.filename.rsplit('.').nth(1).unwrap_or(&file.filename);
    let output_filename = format!("{}.{}", base_filename, output_extension);

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

    let original_extension = file.filename.rsplit('.').next().unwrap_or("tmp");

    match convert_image_inner(&file, output_format).await {
        Ok((converted_bytes, output_filename)) => {
            let attachment = CreateAttachment::bytes(converted_bytes, &output_filename);

            let embed = CreateEmbed::default()
                .title("✅ Conversion Complete")
                .description(format!("{} → {}", original_extension, output_filename.rsplit('.').next().unwrap_or("")))
                .color(0x27ae60);

            let reply = poise::CreateReply::default()
                .embed(embed)
                .attachment(attachment);

            ctx.send(reply).await?;
        }
        Err(e) => {
            let embed = CreateEmbed::default()
                .title("❌ Conversion Failed")
                .description(format!("Conversion error: {}", e))
                .color(0xff4444);

            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Err(e);
        }
    }

    Ok(())
}
