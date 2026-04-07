use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use std::path::Path;
use tempfile::Builder;
use zip::ZipArchive;
use crate::utils::format_file_size;
use crate::{Context, Error};

/// Extract the first file from a ZIP archive stored in memory.
/// Returns the original filename from inside the archive (if any).
async fn extract_first_zip_entry(
    data: &[u8],
    output_path: std::path::PathBuf,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let data = data.to_vec();
    tokio::task::spawn_blocking(move || -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cursor = std::io::Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;
        if archive.len() == 0 {
            return Err("Empty archive".into());
        }
        if archive.len() > 1 {
            return Err("This ZIP archive contains multiple files. Conversia currently extracts only single-file ZIP archives.".into());
        }
        let mut file = archive.by_index(0)?;
        let original_name = file.name().to_string();
        let mut output_file = std::fs::File::create(&output_path)?;
        std::io::copy(&mut file, &mut output_file)?;
        Ok(Some(original_name))
    }).await?
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

    let base_name = Path::new(&file.filename)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Use tempfile for safe, auto-cleaned temporary storage
    let temp_file = Builder::new()
        .prefix("conversia_unzip_")
        .tempfile()?;
    let temp_path = temp_file.path().to_path_buf();

    match extract_first_zip_entry(&file_data, temp_path.clone()).await {
        Ok(original_filename) => {
            match tokio::fs::read(&temp_path).await {
                Ok(decompressed_data) => {
                    let compressed_size = file_data.len() as f64;
                    let decompressed_size = decompressed_data.len() as f64;
                    let ratio = if compressed_size > 0.0 {
                        ((decompressed_size - compressed_size) / compressed_size * 100.0).max(0.0)
                    } else {
                        0.0
                    };

                    let final_filename = original_filename.unwrap_or(base_name);
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

                    ctx.send(
                        poise::CreateReply::default()
                            .embed(embed)
                            .attachment(attachment)
                    ).await?;
                }
                Err(e) => {
                    let embed = CreateEmbed::new()
                        .title("❌ File Read Error")
                        .description(format!("Failed to read decompressed file: {}", e))
                        .color(0xff4444);
                    ctx.send(poise::CreateReply::default().embed(embed)).await?;
                }
            }
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Decompression Failed")
                .description(format!("Failed to decompress file: {}", e))
                .color(0xff4444);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }
    // temp_file is dropped here, automatically cleaning up

    Ok(())
}
