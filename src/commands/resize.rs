use crate::attachments::{validate_attachment_size, validate_image_dimensions, validate_output_size};
use crate::{Context, Error};
use image::{DynamicImage, ImageFormat, RgbaImage, load_from_memory};
use poise::serenity_prelude::CreateAttachment;
use resize::{Pixel::RGBA8, Resizer, Type};
use rgb::RGBA;
use serenity::all::{Attachment, CreateEmbed, CreateEmbedFooter};
use std::io::Cursor;

const MAX_DIMENSION: u32 = 16_384;

fn resize_error_embed(title: &str, message: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(message)
        .color(0xff4444)
}

fn invalid_dimension_embed() -> CreateEmbed {
    CreateEmbed::default()
        .title("❌ Invalid Dimensions")
        .description("Width and height must be between 1 and 16384 pixels")
        .footer(CreateEmbedFooter::new(
            "Dimension limits prevent resource exhaustion.",
        ))
        .color(0xff4444)
}

fn dimensions_are_valid(width: u32, height: u32) -> bool {
    (1..=MAX_DIMENSION).contains(&width) && (1..=MAX_DIMENSION).contains(&height)
}

fn normalized_extension(filename: &str) -> String {
    filename
        .rsplit_once('.')
        .map_or_else(|| String::from("png"), |(_, ext)| ext.to_ascii_lowercase())
}

fn image_format_for_extension(extension: &str) -> ImageFormat {
    match extension {
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "png" => ImageFormat::Png,
        "webp" => ImageFormat::WebP,
        "gif" => ImageFormat::Gif,
        "bmp" => ImageFormat::Bmp,
        "tif" | "tiff" => ImageFormat::Tiff,
        _ => ImageFormat::Png,
    }
}

fn should_use_lanczos(
    source_width: u32,
    source_height: u32,
    target_width: u32,
    target_height: u32,
) -> bool {
    u64::from(target_width) * u64::from(target_height)
        < u64::from(source_width) * u64::from(source_height)
}

fn rgba_image_to_pixels(image: &image::RgbaImage) -> Vec<RGBA<u8>> {
    let mut pixels = Vec::with_capacity((image.width() as usize) * (image.height() as usize));

    for chunk in image.as_raw().chunks_exact(4) {
        pixels.push(RGBA {
            r: chunk[0],
            g: chunk[1],
            b: chunk[2],
            a: chunk[3],
        });
    }

    pixels
}

fn rgba_pixels_to_bytes(pixels: &[RGBA<u8>]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(pixels.len() * 4);

    for pixel in pixels {
        bytes.push(pixel.r);
        bytes.push(pixel.g);
        bytes.push(pixel.b);
        bytes.push(pixel.a);
    }

    bytes
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

    if !dimensions_are_valid(width, height) {
        ctx.send(poise::CreateReply::default().embed(invalid_dimension_embed()))
            .await?;
        return Ok(());
    }

    if let Err(message) = validate_image_dimensions(width, height) {
        let embed = resize_error_embed("❌ Invalid Dimensions", &message);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    if let Err(message) = validate_attachment_size(&attachment) {
        let embed = resize_error_embed("❌ File Too Large", &message);
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

    let normalized_extension = normalized_extension(&attachment.filename);
    let image_format = image_format_for_extension(&normalized_extension);
    let (output_bytes, original_width, original_height, original_extension) =
        match tokio::task::spawn_blocking(move || -> Result<(Vec<u8>, u32, u32, String), Error> {
            // Load image
            let src_image = load_from_memory(&bytes).map_err(|_| {
                Error::from("Invalid image format. Please upload a supported image file.")
            })?;
            let original_width = src_image.width();
            let original_height = src_image.height();
            validate_image_dimensions(original_width, original_height).map_err(Error::from)?;
            let rgba = src_image.to_rgba8();

            let src_pixels = rgba_image_to_pixels(&rgba);

            // Create destination buffer with the requested size.
            let mut dst_pixels = vec![
                RGBA {
                    r: 0u8,
                    g: 0u8,
                    b: 0u8,
                    a: 0u8
                };
                (width * height) as usize
            ];

            // Choose the resize filter based on whether this is a downscale or upscale.
            let resize_type = if should_use_lanczos(original_width, original_height, width, height)
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

            let dst_bytes = rgba_pixels_to_bytes(&dst_pixels);

            let resized = RgbaImage::from_raw(width, height, dst_bytes)
                .ok_or_else(|| Error::from("Failed to rebuild the resized image."))?;

            let ext = normalized_extension;

            let dyn_img = match image_format {
                ImageFormat::Jpeg => {
                    DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(resized).to_rgb8())
                }
                _ => DynamicImage::ImageRgba8(resized),
            };

            let mut buffer = Cursor::new(Vec::new());
            dyn_img
                .write_to(&mut buffer, image_format)
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
    if let Err(message) = validate_output_size(output_bytes.len(), "Resized image") {
        let embed = resize_error_embed("❌ Resize Failed", &message);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

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
