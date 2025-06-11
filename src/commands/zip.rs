use poise::{serenity_prelude as serenity};
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use std::fs::File;
use std::io::Write;
use zip::{ZipWriter, CompressionMethod};
use crate::{Context, Error};
use std::path::Path;
use tokio::fs;
use flate2::write::GzEncoder;
use flate2::Compression as Flate2Compression;
use tar::Builder;
use bzip2::write::BzEncoder;
use bzip2::Compression as Bzip2Compression;
use tokio::task;

#[derive(Debug, poise::ChoiceParameter)]
pub enum CompressionFormat {
    #[name = "zip"]
    Zip,
    #[name = "tar.gz"]
    TarGz,
    #[name = "tar.bz2"]
    TarBz2,
    #[name = "zst"]
    Zst
}

/// Compress a file
#[poise::command(slash_command)]
pub async fn zip(
    ctx: Context<'_>,
    #[description = "File to compress"] file: Attachment,
    #[description = "Compression format"] output_format: CompressionFormat,
) -> Result<(), Error> {

    // Make sure user upload isn't too large
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

    // Generate output filenames
    let original_name = Path::new(&file.filename)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    let (output_filename, temp_output_path) = match output_format {
        CompressionFormat::Zip => (
            format!("{}.zip", original_name),
            format!("temp_compressed_{}.zip", original_name),
        ),
        CompressionFormat::TarGz => (
            format!("{}.tar.gz", original_name),
            format!("temp_compressed_{}.tar.gz", original_name),
        ),
        CompressionFormat::TarBz2 => (
            format!("{}.tar.bz2", original_name),
            format!("temp_compressed_{}.tar.bz2", original_name),
        ),
        CompressionFormat::Zst => (
            format!("{}.zst", original_name),
            format!("temp_compressed_{}.zst", original_name),   
        ),
    };

    // Compress the file
    let result = match output_format {
        CompressionFormat::Zip => {
            create_zip_from_bytes(&file.filename, &file_data, &temp_output_path).await
        }
        CompressionFormat::TarGz => {
            create_tar_from_bytes(&file.filename, &file_data, &temp_output_path).await
        },
        CompressionFormat::TarBz2 => {
            create_tar_bz2_from_bytes(&file.filename, &file_data, &temp_output_path).await
        },
        CompressionFormat::Zst => {
            create_zst_from_bytes(&file.filename, &file_data, &temp_output_path).await
        },
    };

    match result {
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
                        .footer(serenity::CreateEmbedFooter::new(format!(
                            "Format: {}",
                            match output_format {
                                CompressionFormat::Zip => "zip",
                                CompressionFormat::TarGz => "tar.gz",
                                CompressionFormat::TarBz2 => "tar.bz2",
                                CompressionFormat::Zst => "zst",
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

            // Clean up temporary file if it exists
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

pub async fn create_tar_from_bytes(
    filename: &str,
    data: &[u8],
    tar_gz_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filename = filename.to_string();
    let data = data.to_vec();
    let tar_gz_path = tar_gz_path.to_string();

    tokio::task::spawn_blocking(move || {
        let tar_gz = File::create(&tar_gz_path)?;
        let enc = GzEncoder::new(tar_gz, Flate2Compression::default());
        let mut tar = Builder::new(enc);

        tar.append_data(&mut tar::Header::new_gnu(), filename.as_str(), data.as_slice())?;
        tar.finish()?;

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    }).await?
}

pub async fn create_tar_bz2_from_bytes(
    filename: &str,
    data: &[u8],
    tar_bz2_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filename = filename.to_string();
    let data = data.to_vec();
    let tar_bz2_path = tar_bz2_path.to_string();

    tokio::task::spawn_blocking(move || {
        let tar_bz2 = File::create(&tar_bz2_path)?;
        let enc = BzEncoder::new(tar_bz2, Bzip2Compression::best());
        let mut tar = Builder::new(enc);

        tar.append_data(&mut tar::Header::new_gnu(), filename.as_str(), data.as_slice())?;
        tar.finish()?;

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    }).await?
}

pub async fn create_zst_from_bytes(
    filename: &str,
    data: &[u8],
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filename = filename.to_string();
    let data = data.to_vec();
    let output_path = output_path.to_string();

    // Run the blocking compression in a spawn_blocking to avoid blocking the async runtime
    task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut output_file = File::create(output_path)?;
        let mut encoder = zstd::stream::Encoder::new(&mut output_file, 0)?; // 0 = default compression level
        encoder.write_all(&data)?;
        encoder.finish()?;
        Ok(())
    })
    .await?
}