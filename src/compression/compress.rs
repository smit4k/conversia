use poise::{serenity_prelude as serenity};
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use std::fs::File;
use std::io::Write;
use zip::{ZipWriter, CompressionMethod};
use crate::utils::format_file_size;
use crate::{Context, Error};
use std::path::Path;
use tokio::fs;

#[derive(Debug, poise::ChoiceParameter)]
pub enum CompressionFormat {
    #[name = "zip"]
    Zip,
}

/// Strip all extensions from a file
fn strip_all_extensions(filename: &str) -> String {
    let mut stem = filename.to_string();
    loop {
        let new_stem = Path::new(&stem).file_stem();
        match new_stem {
            Some(s) => {
                let s_str = s.to_string_lossy().to_string();
                if s_str == stem {
                    break;
                }
                stem = s_str;
            }
            None => break,
        }
    }
    stem
}

/// Compress a file into a zip archive
#[poise::command(slash_command)]
pub async fn zip(
    ctx: Context<'_>,
    #[description = "File to compress"] file: Attachment,
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

    let original_name = Path::new(&file.filename)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    let output_filename = format!("{}.zip", original_name);
    let temp_output_path = format!("temp_compressed_{}.zip", original_name);

    // Clean internal filename: strip all extensions so inside zip archive file is clean
    let internal_filename = strip_all_extensions(&file.filename);

    let result = create_zip_from_bytes(&internal_filename, &file_data, &temp_output_path).await;

    match result {
        Ok(()) => {
            match fs::read(&temp_output_path).await {
                Ok(compressed_data) => {
                    let original_size = file_data.len() as f64;
                    let compressed_size = compressed_data.len() as f64;
                    let ratio = ((original_size - compressed_size) / original_size * 100.0).max(0.0);

                    let attachment = CreateAttachment::bytes(compressed_data, &output_filename);

                    let embed = CreateEmbed::new()
                        .title("✅ Compression Complete")
                        .description(format!(
                            "**Original:** `{}` ({})\n**Compressed:** `{}` ({})\n**Saved:** {:.1}%",
                            file.filename,
                            format_file_size(file_data.len() as u64),
                            output_filename,
                            format_file_size(compressed_size as u64),
                            ratio
                        ))
                        .color(0x27ae60)
                        .footer(serenity::CreateEmbedFooter::new("Format: zip"));

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
            let _ = fs::remove_file(&temp_output_path).await;
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Compression Failed")
                .description(format!("Failed to compress file: {}", e))
                .color(0xff4444);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
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
