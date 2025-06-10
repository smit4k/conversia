use poise::{serenity_prelude as serenity};
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use std::fs::File;
use std::io::Write;
use zip::{ZipWriter, CompressionMethod};
use crate::{Context, Error};
use std::path::Path;
use tokio::fs;


/// Compress a file to ZIP
#[poise::command(slash_command)]
pub async fn compress(
    ctx: Context<'_>,
    #[description = "File to compress"] file: Attachment,
) -> Result<(), Error> {
    const MAX_FILE_SIZE: u32 = 25 * 1024 * 1024; // 25MB
    if file.size > MAX_FILE_SIZE {
        let embed = CreateEmbed::new()
            .title("❌ File Too Large")
            .description("File must be smaller than 25MB")
            .color(0xff4444);
        
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // Download the file
    let file_data = match file.download().await {
        Ok(data) => data,
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Download Failed")
                .description(format!("Failed to download file: {}", e))
                .color(0xff4444);
            
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    // Generate output filename
    let original_name = Path::new(&file.filename)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let output_filename = format!("{}.zip", original_name);
    let temp_output_path = format!("temp_compressed_{}", output_filename);

    // Compress the file
    match create_zip_from_bytes(&file.filename, &file_data, &temp_output_path).await {
        Ok(()) => {
            // Read the compressed file
            match fs::read(&temp_output_path).await {
                Ok(compressed_data) => {
                    // Calculate compression ratio
                    let original_size = file_data.len() as f64;
                    let compressed_size = compressed_data.len() as f64;
                    let ratio = ((original_size - compressed_size) / original_size * 100.0).max(0.0);

                    // Create attachment
                    let attachment = CreateAttachment::bytes(compressed_data, &output_filename);

                    // Create success embed
                    let embed = CreateEmbed::new()
                        .title("✅ Compression Complete")
                        .description(format!(
                            "**Original:** `{}` ({} bytes)\n**Compressed:** `{}` ({} bytes)\n**Saved:** {:.1}%",
                            file.filename,
                            file_data.len(),
                            output_filename,
                            compressed_size as usize,
                            ratio
                        ))
                        .color(0x44ff44)
                        .footer(serenity::CreateEmbedFooter::new("Format: ZIP"));

                    ctx.send(
                        poise::CreateReply::default()
                            .embed(embed)
                            .attachment(attachment)
                    ).await?;
                }
                Err(e) => {
                    let embed = CreateEmbed::new()
                        .title("❌ File Read Error")
                        .description(format!("Failed to read compressed file: {}", e))
                        .color(0xff4444);
                    
                    ctx.send(poise::CreateReply::default().embed(embed)).await?;
                }
            }

            // Clean up temporary file
            let _ = fs::remove_file(&temp_output_path).await;
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Compression Failed")
                .description(format!("Failed to compress file: {}", e))
                .color(0xff4444);
            
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            
            // Clean up temporary file
            let _ = fs::remove_file(&temp_output_path).await;
        }
    }

    Ok(())
}

async fn create_zip_from_bytes(
    filename: &str,
    data: &[u8],
    zip_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);
    
    zip.start_file(filename, options)?;
    zip.write_all(data)?;
    zip.finish()?;
    
    Ok(())
}