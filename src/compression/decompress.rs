use poise::{serenity_prelude as serenity};
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use std::fs::File;
use zip::ZipArchive;
use crate::utils::format_file_size;
use crate::{Context, Error};
use std::path::Path;
use tokio::fs;

#[derive(Debug, poise::ChoiceParameter)]
pub enum DecompressionFormat {
    #[name = "zip"]
    Zip,
}

/// Decompress a zipped file
#[poise::command(slash_command)]
pub async fn unzip(
    ctx: Context<'_>,
    #[description = "Zip to decompress"] file: Attachment,
) -> Result<(), Error> {
    ctx.defer().await?;

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

    let (output_filename, temp_output_path) = generate_output_paths(&file.filename);

    let result = extract_zip_from_bytes(&file_data, &temp_output_path).await;

    match result {
        Ok(original_filename) => {
            match fs::read(&temp_output_path).await {
                Ok(decompressed_data) => {
                    let compressed_size = file_data.len() as f64;
                    let decompressed_size = decompressed_data.len() as f64;
                    let ratio = if compressed_size > 0.0 {
                        ((decompressed_size - compressed_size) / compressed_size * 100.0).max(0.0)
                    } else {
                        0.0
                    };

                    let final_filename = original_filename.unwrap_or(output_filename);
                    let attachment = CreateAttachment::bytes(decompressed_data, &final_filename);

                    let embed = CreateEmbed::new()
                        .title("✅ Decompression Complete")
                        .description(format!(
                            "**Compressed:** `{}` ({})\n**Extracted:** `{}` ({})\n**Expansion:** {:.1}%",
                            file.filename,
                            format_file_size(file_data.len() as u64),
                            final_filename,
                            format_file_size(decompressed_size as u64),
                            ratio
                        ))
                        .color(0x27ae60)
                        .footer(serenity::CreateEmbedFooter::new("Format: zip"));

                    ctx.send(poise::CreateReply::default().embed(embed).attachment(attachment)).await?;
                }
                Err(e) => {
                    let embed = CreateEmbed::new()
                        .title("❌ File Read Error")
                        .description(format!("Failed to read decompressed file: {}", e))
                        .color(0xff4444);
                    ctx.send(poise::CreateReply::default().embed(embed)).await?;
                }
            }
            let _ = fs::remove_file(&temp_output_path).await;
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Decompression Failed")
                .description(format!("Failed to decompress file: {}", e))
                .color(0xff4444);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            let _ = fs::remove_file(&temp_output_path).await;
        }
    }
    Ok(())
}

fn generate_output_paths(filename: &str) -> (String, String) {
    let path = Path::new(filename);
    let base_name = path.file_stem().unwrap_or_default().to_string_lossy();
    let output_filename = base_name.to_string();
    let temp_output_path = format!("temp_decompressed_{}", output_filename);
    (output_filename, temp_output_path)
}

async fn extract_zip_from_bytes(
    data: &[u8],
    output_path: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let data = data.to_vec();
    let output_path = output_path.to_string();
    let original_filename = tokio::task::spawn_blocking(move || -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cursor = std::io::Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;
        if archive.len() > 0 {
            let mut file = archive.by_index(0)?;
            let original_name = file.name().to_string();
            let mut output_file = File::create(&output_path)?;
            std::io::copy(&mut file, &mut output_file)?;
            Ok(Some(original_name))
        } else {
            Err("Empty archive".into())
        }
    }).await??;
    Ok(original_filename)
}
