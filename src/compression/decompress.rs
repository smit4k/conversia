use poise::{serenity_prelude as serenity};
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use std::fs::File;
use std::io::{Read, Write};
use zip::ZipArchive;
use crate::utils::format_file_size;
use crate::{Context, Error};
use std::path::Path;
use tokio::fs;
use flate2::read::GzDecoder;
use tar::Archive;
use bzip2::read::BzDecoder;
use tokio::task;
use lz4::Decoder;

#[derive(Debug, poise::ChoiceParameter)]
pub enum DecompressionFormat {
    #[name = "zip"]
    Zip,
    #[name = "tar.gz"]
    TarGz,
    #[name = "bz2"]
    Bz2,
    #[name = "zst"]
    Zst,
    #[name = "lz4"]
    Lz4,
    #[name = "auto"]
    Auto,
}

/// Decompress a file
#[poise::command(slash_command)]
pub async fn decompress(
    ctx: Context<'_>,
    #[description = "File to decompress"] file: Attachment,
    #[description = "Decompression format (auto-detect if not specified)"] format: Option<DecompressionFormat>,
) -> Result<(), Error> {

    ctx.defer().await?;

    // Download the file
    let file_data = match file.download().await {
        Ok(data) => data,
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Download failed")
                .description(format!("Failed to download file: {}", e))
                .color(0xff4444);
            
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    // Determine format
    let detected_format = match format {
        Some(DecompressionFormat::Auto) | None => {
            detect_compression_format(&file.filename)
        }
        Some(f) => f,
    };

    // Generate output filename and temp path
    let (output_filename, temp_output_path) = generate_output_paths(&file.filename, &detected_format);

    // Decompress the file
    let result = match detected_format {
        DecompressionFormat::Zip => {
            extract_zip_from_bytes(&file_data, &temp_output_path).await
        }
        DecompressionFormat::TarGz => {
            extract_tar_gz_from_bytes(&file_data, &temp_output_path).await
        }
        DecompressionFormat::Bz2 => {
            extract_bz2_from_bytes(&file_data, &temp_output_path).await
        }
        DecompressionFormat::Zst => {
            extract_zst_from_bytes(&file_data, &temp_output_path).await
        }
        DecompressionFormat::Lz4 => {
            extract_lz4_from_bytes(&file_data, &temp_output_path).await
        }
        DecompressionFormat::Auto => {
            let embed = CreateEmbed::new()
                .title("❌ Format detection failed")
                .description("Could not detect compression format from filename")
                .color(0xff4444);
            
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    match result {
        Ok(original_filename) => {
            // Read the decompressed file
            match fs::read(&temp_output_path).await {
                Ok(decompressed_data) => {
                    // Calculate compression ratio
                    let compressed_size = file_data.len() as f64;
                    let decompressed_size = decompressed_data.len() as f64;
                    let ratio = if compressed_size > 0.0 {
                        ((decompressed_size - compressed_size) / compressed_size * 100.0).max(0.0)
                    } else {
                        0.0
                    };

                    // Use original filename if available, otherwise use generated one
                    let final_filename = original_filename.unwrap_or(output_filename);

                    // Create attachment
                    let attachment = CreateAttachment::bytes(decompressed_data, &final_filename);

                    // Create success embed
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
                        .footer(serenity::CreateEmbedFooter::new(format!(
                            "Format: {}",
                            match detected_format {
                                DecompressionFormat::Zip => "zip",
                                DecompressionFormat::TarGz => "tar.gz",
                                DecompressionFormat::Bz2 => "bz2",
                                DecompressionFormat::Zst => "zst",
                                DecompressionFormat::Lz4 => "lz4",
                                DecompressionFormat::Auto => "auto",
                            }
                        )));

                    ctx.send(
                        poise::CreateReply::default()
                            .embed(embed)
                            .attachment(attachment)
                    ).await?;
                }
                Err(e) => {
                    let embed = CreateEmbed::new()
                        .title("❌ File read error")
                        .description(format!("Failed to read decompressed file: {}", e))
                        .color(0xff4444);
                    
                    ctx.send(poise::CreateReply::default().embed(embed)).await?;
                }
            }

            // Clean up temporary file
            let _ = fs::remove_file(&temp_output_path).await;
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Decompression failed")
                .description(format!("Failed to decompress file: {}", e))
                .color(0xff4444);
            
            ctx.send(poise::CreateReply::default().embed(embed)).await?;

            // Clean up temporary file if it exists
            let _ = fs::remove_file(&temp_output_path).await;
        }
    }

    Ok(())
}

fn detect_compression_format(filename: &str) -> DecompressionFormat {
    let filename_lower = filename.to_lowercase();
    
    if filename_lower.ends_with(".zip") {
        DecompressionFormat::Zip
    } else if filename_lower.ends_with(".tar.gz") || filename_lower.ends_with(".tgz") {
        DecompressionFormat::TarGz
    } else if filename_lower.ends_with(".bz2") {
        DecompressionFormat::Bz2
    } else if filename_lower.ends_with(".zst") {
        DecompressionFormat::Zst
    } else if filename_lower.ends_with(".lz4") {
        DecompressionFormat::Lz4
    } else {
        DecompressionFormat::Auto
    }
}

fn generate_output_paths(filename: &str, format: &DecompressionFormat) -> (String, String) {
    let path = Path::new(filename);
    let base_name = path.file_stem().unwrap_or_default().to_string_lossy();
    
    let output_filename = match format {
        DecompressionFormat::Zip => base_name.to_string(),
        DecompressionFormat::TarGz => {
            // Remove .tar from .tar.gz
            if base_name.ends_with(".tar") {
                base_name[..base_name.len() - 4].to_string()
            } else {
                base_name.to_string()
            }
        }
        DecompressionFormat::Bz2 | DecompressionFormat::Zst | DecompressionFormat::Lz4 => {
            base_name.to_string()
        }
        DecompressionFormat::Auto => "decompressed_file".to_string(),
    };

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
        
        // Extract the first file (assuming single file compression)
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

async fn extract_tar_gz_from_bytes(
    data: &[u8],
    output_path: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let data = data.to_vec();
    let output_path = output_path.to_string();

    let original_filename = tokio::task::spawn_blocking(move || -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cursor = std::io::Cursor::new(data);
        let tar = GzDecoder::new(cursor);
        let mut archive = Archive::new(tar);
        
        // Extract the first file
        let mut entries = archive.entries()?;
        if let Some(entry) = entries.next() {
            let mut file = entry?;
            let original_name = file.header().path()?.to_string_lossy().to_string();
            
            let mut output_file = File::create(&output_path)?;
            std::io::copy(&mut file, &mut output_file)?;
            
            Ok(Some(original_name))
        } else {
            Err("Empty archive".into())
        }
    }).await??;

    Ok(original_filename)
}

async fn extract_bz2_from_bytes(
    data: &[u8],
    output_path: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let data = data.to_vec();
    let output_path = output_path.to_string();

    tokio::task::spawn_blocking(move || -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cursor = std::io::Cursor::new(data);
        let mut decoder = BzDecoder::new(cursor);
        
        let mut output_file = File::create(&output_path)?;
        std::io::copy(&mut decoder, &mut output_file)?;
        
        Ok(None) // bz2 doesn't store original filename
    }).await?
}

async fn extract_zst_from_bytes(
    data: &[u8],
    output_path: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let data = data.to_vec();
    let output_path = output_path.to_string();

    task::spawn_blocking(move || -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cursor = std::io::Cursor::new(data);
        let mut decoder = zstd::stream::Decoder::new(cursor)?;
        
        let mut output_file = File::create(&output_path)?;
        std::io::copy(&mut decoder, &mut output_file)?;
        
        Ok(None) // zst doesn't store original filename
    }).await?
}

async fn extract_lz4_from_bytes(
    data: &[u8],
    output_path: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let data = data.to_vec();
    let output_path = output_path.to_string();

    tokio::task::spawn_blocking(move || -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let cursor = std::io::Cursor::new(data);
        let mut decoder = Decoder::new(cursor)?;
        
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;
        
        let mut output_file = File::create(&output_path)?;
        output_file.write_all(&decompressed_data)?;
        
        Ok(None) // lz4 doesn't store original filename
    }).await?
}