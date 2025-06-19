use std::io::Cursor;
use serenity::all::{Attachment, CreateEmbed, CreateEmbedFooter};
use poise::serenity_prelude::CreateAttachment;
use image::{load_from_memory, ImageFormat, RgbaImage, DynamicImage};
use resize::{Pixel::RGBA8, Type, Resizer};
use rgb::RGBA;
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
    
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        let embed = CreateEmbed::default()
            .title("❌ Invalid Dimensions")
            .description("Width and height must be between 1 and 16384 pixels")
            .footer(CreateEmbedFooter::new("Dimension limits prevent resource exhaustion."))
            .color(0xff4444);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }
    
    let bytes = attachment.download().await.map_err(|e| {
        let embed = CreateEmbed::default()
            .title("❌ Download Failed")
            .description("Failed to download the attached file.")
            .color(0xff4444);
        let _ = ctx.send(poise::CreateReply::default().embed(embed));
        e
    })?;
    
    let (output_bytes, original_width, original_height, original_extension) = tokio::task::spawn_blocking(move || {
        // Load image
        let src_image = load_from_memory(&bytes).map_err(|_| "Invalid image format")?;
        let original_width = src_image.width();
        let original_height = src_image.height();
        let rgba = src_image.to_rgba8();
        
        // Convert raw bytes to RGBA pixels - using RGBA<u8> from rgb crate
        let src_pixels: Vec<RGBA<u8>> = rgba.as_raw()
            .chunks_exact(4)
            .map(|chunk| RGBA { r: chunk[0], g: chunk[1], b: chunk[2], a: chunk[3] })
            .collect();
        
        // Create destination buffer with proper RGBA pixel type
        let mut dst_pixels = vec![RGBA { r: 0u8, g: 0u8, b: 0u8, a: 0u8 }; (width * height) as usize];
        
        // Choose optimal resize algorithm based on scaling direction (as stated by resize crate docs)
        let resize_type = if (width as f32 * height as f32) < (original_width as f32 * original_height as f32) {
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
        ).map_err(|_| "Failed to create resizer")?;
        
        resizer.resize(&src_pixels[..], &mut dst_pixels[..]).map_err(|_| "Resize failed")?;
        
        // Convert back to raw bytes
        let dst_bytes: Vec<u8> = dst_pixels
            .iter()
            .flat_map(|pixel| vec![pixel.r, pixel.g, pixel.b, pixel.a])
            .collect();
        
        let resized = RgbaImage::from_raw(width, height, dst_bytes).ok_or("Failed to build image")?;
        
        let ext = attachment
            .filename
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
            ImageFormat::Jpeg => DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(resized).to_rgb8()),
            _ => DynamicImage::ImageRgba8(resized),
        };
        
        let mut buffer = Cursor::new(Vec::new());
        dyn_img.write_to(&mut buffer, format).map_err(|_| "Encoding failed")?;
        
        Ok::<_, &str>((buffer.into_inner(), original_width, original_height, ext))
    }).await.unwrap_or_else(|_| Err("Resize task panicked"))?;
    
    // Prepare response
    let filename = format!("resized_{}x{}.{}", width, height, original_extension);
    let embed = CreateEmbed::default()
        .title("✅ Resize Complete")
        .description(format!("{}×{} → {}×{}", original_width, original_height, width, height))
        .color(0x27ae60);
    
    let reply = poise::CreateReply::default()
        .embed(embed)
        .attachment(CreateAttachment::bytes(output_bytes, filename));
    
    ctx.send(reply).await?;
    Ok(())
}