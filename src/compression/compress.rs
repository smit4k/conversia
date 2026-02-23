use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::Builder;
use zip::{ZipWriter, CompressionMethod};
use crate::utils::format_file_size;
use crate::{Context, Error};

/// Strip all extensions from a filename, returning only the stem.
fn strip_all_extensions(filename: &str) -> String {
    let mut stem = filename.to_string();
    loop {
        match Path::new(&stem).file_stem() {
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

/// Create a ZIP archive containing the given data as a single file.
fn create_zip_archive(
    internal_filename: &str,
    data: &[u8],
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);
    zip.start_file(internal_filename, options)?;
    zip.write_all(data)?;
    zip.finish()?;
    Ok(())
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
        .to_string_lossy()
        .to_string();
    let output_filename = format!("{}.zip", original_name);
    let internal_filename = strip_all_extensions(&file.filename);

    // Use tempfile for safe, auto-cleaned temporary storage
    let temp_file = Builder::new()
        .prefix("conversia_zip_")
        .suffix(".zip")
        .tempfile()?;
    let temp_path = temp_file.path().to_path_buf();

    match create_zip_archive(&internal_filename, &file_data, &temp_path) {
        Ok(()) => {
            match tokio::fs::read(&temp_path).await {
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
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Compression Failed")
                .description(format!("Failed to compress file: {}", e))
                .color(0xff4444);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }
    // temp_file is dropped here, automatically cleaning up

    Ok(())
}
