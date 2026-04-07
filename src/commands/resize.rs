use crate::{Context, Error};
use image::{DynamicImage, ImageFormat, RgbaImage, load_from_memory};
use poise::serenity_prelude::CreateAttachment;
use resize::{Pixel::RGBA8, Resizer, Type};
use rgb::RGBA;
use serenity::all::{Attachment, CreateEmbed, CreateEmbedFooter};
use std::io::Cursor;

fn resize_error_embed(title: &str, message: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(message)
        .color(0xff4444)
}

/// Resize an image
#[poise::command(slash_command)]
pub async fn resize_image(
    ctx: Context<'_>,
    #[description = "Image to resize"] attachment: Attachment,
    #[description = "New width in pixels"] width: u32,
    #[description = "New height in pixels"] height: u32,
) -> Result<(), Error> {
    ctx.defer().await?;

    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        let embed = CreateEmbed::default()
            .title("❌ Invalid Dimensions")
            .description("Width and height must be between 1 and 16384 pixels")
            .footer(CreateEmbedFooter::new(
                "Dimension limits prevent resource exhaustion.",
            ))
            .color(0xff4444);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let bytes = match attachment.download().await {
        Ok(bytes) => bytes,
        Err(_) => {
            let embed = resize_error_embed(
                "❌ Download Failed",
                "Failed to download the attached file.",
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let original_filename = attachment.filename.clone();
    let (output_bytes, original_width, original_height, original_extension) =
        match tokio::task::spawn_blocking(move || -> Result<(Vec<u8>, u32, u32, String), Error> {
            // Load image
            let src_image = load_from_memory(&bytes).map_err(|_| {
                Error::from("Invalid image format. Please upload a supported image file.")
            })?;
            let original_width = src_image.width();
            let original_height = src_image.height();
            let rgba = src_image.to_rgba8();

            // Convert raw bytes to RGBA pixels - using RGBA<u8> from rgb crate
            let src_pixels: Vec<RGBA<u8>> = rgba
                .as_raw()
                .chunks_exact(4)
                .map(|chunk| RGBA {
                    r: chunk[0],
                    g: chunk[1],
                    b: chunk[2],
                    a: chunk[3],
                })
                .collect();

            // Create destination buffer with proper RGBA pixel type
            let mut dst_pixels = vec![
                RGBA {
                    r: 0u8,
                    g: 0u8,
                    b: 0u8,
                    a: 0u8
                };
                (width * height) as usize
            ];

            // Choose optimal resize algorithm based on scaling direction (as stated by resize crate docs)
            let resize_type = if (width as f32 * height as f32)
                < (original_width as f32 * original_height as f32)
            {
                Type::Lanczos3
            } else {
                Type::Mitchell
            };

            let mut resizer = Resizer::new(
                original_width as usize,
                original_height as usize,
                width as usize,
                height as usize,
                RGBA8,
                resize_type,
            )
            .map_err(|_| Error::from("Failed to initialize the image resizer."))?;

            resizer
                .resize(&src_pixels[..], &mut dst_pixels[..])
                .map_err(|_| Error::from("Image resizing failed."))?;

            // Convert back to raw bytes
            let dst_bytes: Vec<u8> = dst_pixels
                .iter()
                .flat_map(|pixel| vec![pixel.r, pixel.g, pixel.b, pixel.a])
                .collect();

            let resized = RgbaImage::from_raw(width, height, dst_bytes)
                .ok_or_else(|| Error::from("Failed to rebuild the resized image."))?;

            let ext = original_filename
                .rsplit('.')
                .next()
                .unwrap_or("png")
                .to_lowercase();

            let format = match ext.as_str() {
                "jpg" | "jpeg" => ImageFormat::Jpeg,
                "png" => ImageFormat::Png,
                "webp" => ImageFormat::WebP,
                "gif" => ImageFormat::Gif,
                "bmp" => ImageFormat::Bmp,
                "tif" | "tiff" => ImageFormat::Tiff,
                _ => ImageFormat::Png,
            };

            let dyn_img = match format {
                ImageFormat::Jpeg => {
                    DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(resized).to_rgb8())
                }
                _ => DynamicImage::ImageRgba8(resized),
            };

            let mut buffer = Cursor::new(Vec::new());
            dyn_img
                .write_to(&mut buffer, format)
                .map_err(|_| Error::from("Failed to encode the resized image."))?;

            Ok((buffer.into_inner(), original_width, original_height, ext))
        })
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(err)) => {
                let embed = resize_error_embed("❌ Resize Failed", &err.to_string());
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
            Err(_) => {
                let embed =
                    resize_error_embed("❌ Resize Failed", "The resize task stopped unexpectedly.");
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        };

    // Prepare response
    let filename = format!("resized_{}x{}.{}", width, height, original_extension);
    let embed = CreateEmbed::default()
        .title("✅ Resize Complete")
        .description(format!(
            "{}×{} → {}×{}",
            original_width, original_height, width, height
        ))
        .color(0x27ae60);

    let reply = poise::CreateReply::default()
        .embed(embed)
        .attachment(CreateAttachment::bytes(output_bytes, filename));

    ctx.send(reply).await?;
    Ok(())
}
