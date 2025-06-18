use std::io::Cursor;
use serenity::all::{Attachment, CreateEmbed};
use poise::serenity_prelude::CreateAttachment;
use image::{load_from_memory, ImageFormat};
use fast_image_resize::Resizer;
use fast_image_resize::images::Image;
use crate::{Context, Error};

/// Resize an image
#[poise::command(slash_command)]
pub async fn resize_image(
    ctx: Context<'_>,
    #[description = "Image to resize"] attachment: Attachment,
    #[description = "New width in pixels"] width: u32,
    #[description = "New height in pixels"] height: u32,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Validate dimensions
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        let embed = CreateEmbed::default()
            .title("❌ Invalid Dimensions")
            .description("Width and height must be between 1 and 8192 pixels")
            .color(0xff4444);
        let reply = poise::CreateReply::default().embed(embed);
        ctx.send(reply).await?;
        return Ok(());
    }

    // Download the attachment
    let bytes = match attachment.download().await {
        Ok(data) => data,
        Err(e) => {
            let embed = CreateEmbed::default()
                .title("❌ Download Failed")
                .description("Failed to download the attached file")
                .color(0xff4444);
            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Err(e.into());
        }
    };

    // Load image
    let src_image = match load_from_memory(&bytes) {
        Ok(img) => img,
        Err(_) => {
            let embed = CreateEmbed::default()
                .title("❌ Could Not Read Image")
                .description("The uploaded file isn't a valid image format.")
                .color(0xff4444);
            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Ok(());
        }
    };

    let original_width = src_image.width();
    let original_height = src_image.height();

    // Convert DynamicImage to RGBA8 for fast_image_resize
    let rgba_image = src_image.to_rgba8();
    
    // Create fast_image_resize Image from RGBA data
    let src_fr_image = Image::from_vec_u8(
        original_width,
        original_height,
        rgba_image.into_raw(),
        fast_image_resize::PixelType::U8x4,
    ).map_err(|e| format!("Failed to create source image: {}", e))?;

    // Create container for destination image
    let mut dst_image = Image::new(
        width,
        height,
        fast_image_resize::PixelType::U8x4,
    );

    // Create Resizer instance and resize source image
    let mut resizer = Resizer::new();
    if let Err(e) = resizer.resize(&src_fr_image, &mut dst_image, None) {
        let embed = CreateEmbed::default()
            .title("❌ Resize Failed")
            .description("Could not resize the image")
            .color(0xff4444);
        let reply = poise::CreateReply::default().embed(embed);
        ctx.send(reply).await?;
        return Err(format!("Resize error: {}", e).into());
    }

    // Determine output format from original filename
    let original_extension = attachment.filename.rsplit('.').next().unwrap_or("png").to_lowercase();
    let format = match original_extension.as_str() {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::WebP,
        "gif" => ImageFormat::Gif,
        "bmp" => ImageFormat::Bmp,
        "tiff" | "tif" => ImageFormat::Tiff,
        _ => ImageFormat::Png,
    };

    // Convert resized data back to RGBA8 image
    let resized_rgba = image::RgbaImage::from_raw(width, height, dst_image.into_vec())
        .ok_or("Failed to create output image")?;
    
    // Convert to appropriate format for encoding
    let final_image = match format {
        ImageFormat::Jpeg => image::DynamicImage::ImageRgb8(
            image::DynamicImage::ImageRgba8(resized_rgba).to_rgb8()
        ),
        _ => image::DynamicImage::ImageRgba8(resized_rgba),
    };

    // Write destination image to buffer
    let mut result_buf = Cursor::new(Vec::new());
    
    if let Err(e) = final_image.write_to(&mut result_buf, format) {
        let embed = CreateEmbed::default()
            .title("❌ Encoding Failed")
            .description("Could not encode the resized image")
            .color(0xff4444);
        let reply = poise::CreateReply::default().embed(embed);
        ctx.send(reply).await?;
        return Err(e.into());
    }

    let output_bytes = result_buf.into_inner();
    
    // Create filename for resized image
    let filename = format!("resized_{}x{}.{}", width, height, original_extension);
    let attachment_out = CreateAttachment::bytes(output_bytes, filename);

    let embed = CreateEmbed::default()
        .title("✅ Resize Complete")
        .description(format!("{}×{} → {}×{}", original_width, original_height, width, height))
        .field("Algorithm", "Lanczos3 (High Quality)", false)
        .color(0x27ae60);

    let reply = poise::CreateReply::default()
        .embed(embed)
        .attachment(attachment_out);

    ctx.send(reply).await?;
    Ok(())
}