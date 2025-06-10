use std::process::Command;
use std::path::Path;
use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;

use crate::{Context, Error};

/// Convert a document
#[poise::command(slash_command)]
pub async fn convert_document(
    ctx: Context<'_>,
    #[description = "File to convert"] file: Attachment,
    #[description = "Format to convert to (e.g., pdf, html, markdown, docx)"] output_format: String,
) -> Result<(), Error> {
    // Map user-friendly format names to pandoc format names
    let pandoc_format = match output_format.as_str() {
        "md" | "markdown" => "markdown",
        "html" => "html",
        "pdf" => "pdf",
        "docx" => "docx",
        _ => {
            let embed = CreateEmbed::new()
                .title("❌ Unsupported output format")
                .description("Supported formats: md, markdown, html, pdf, docx")
                .color(0xff4444);

                let reply = poise::CreateReply::default().embed(embed);
                ctx.send(reply).await?;

            return Ok(());
        }
    };

    // Save the uploaded file with its original extension
    let original_extension = file.filename.rsplit('.').next().unwrap_or("tmp");
    let input_temp_file = match Builder::new()
        .suffix(&format!(".{}", original_extension))
        .tempfile() {
        Ok(f) => f,
        Err(e) => {
            let embed = CreateEmbed::default()
                .title("❌ Conversia ran into an error")
                .description("Failed to create a temporary file for the input.")
                .color(0xff4444);
        
            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Err(e.into());
        }
    };
    let input_path = input_temp_file.path().to_path_buf();

    let file_data = match file.download().await {
        Ok(data) => data,
        Err(e) => {
            let embed = CreateEmbed::default()
            .title("❌ Conversia ran into an error")
            .description("Failed to download the attached file.")
            .color(0xff4444);
        
            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Err(e.into());
        }
    };

    if let Err(e) = fs::write(&input_path, file_data).await {
        let embed = CreateEmbed::default()
            .title("❌ Conversia ran into an error")
            .description("Failed to save the attached file to a temporary location.")
            .color(0xff4444);
        
        let reply = poise::CreateReply::default().embed(embed);
        ctx.send(reply).await?;
        return Err(e.into());
    }

    let output_suffix = if pandoc_format == "markdown" {
        format!(".{}", "md")
    } else {
        format!(".{}", pandoc_format)
    };

    let output_temp_file = match Builder::new()
        .suffix(&output_suffix)
        .tempfile() {
        Ok(f) => f,
        Err(e) => {
            let embed = CreateEmbed::default()
                .title("❌ Conversia ran into an error")
                .description("Failed to create a temporary file for the output")
                .color(0xff4444);
            
            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Err(e.into());
        }
    };
    let output_path = output_temp_file.path().to_path_buf();

    // Run the Pandoc command
    let output = Command::new("pandoc")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg(format!("--to={}", pandoc_format)) // Specify the output format
        .arg(format!("--from={}", original_extension)) // Specify the input format
        .output();

    match output {
        Ok(output) if output.status.success() => {
            // Read the converted file into memory
            let converted_file_data = match fs::read(&output_path).await {
                Ok(data) => data,
                Err(e) => {
                    let embed = CreateEmbed::default()
                        .title("❌ Conversia ran into an error")
                        .description("Failed to read the converted file.")
                        .color(0xff4444);

                    let reply = poise::CreateReply::default().embed(embed);
                    ctx.send(reply).await?;
                    return Err(e.into())
                }
            };

            let base_name = Path::new(&file.filename)
                .file_stem()
                .unwrap_or_else(|| std::ffi::OsStr::new("converted"))
                .to_string_lossy()
                .to_string();

            // Create an attachment from the converted file
            let attachment = CreateAttachment::bytes(
                converted_file_data,
                format!("{}.{}", base_name, if pandoc_format == "markdown" { "md" } else { pandoc_format }),
            );

            let embed = CreateEmbed::default()
                .title("✅ Conversion Complete")
                .description(format!("{} → {}", original_extension, pandoc_format))
                .color(0x44ff44);

            let reply = poise::CreateReply::default()
                .embed(embed)
                .attachment(attachment);

            ctx.send(reply).await?;
        }
        Ok(output) => {
            let error_message = String::from_utf8_lossy(&output.stderr);

            let embed = CreateEmbed::default()
                .title("❌ Conversion failed")
                .description(error_message)
                .color(0xff4444);

            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
        }
        Err(error) => {
            ctx.say(format!("Failed to execute Pandoc: {}", error))
                .await?;
        }
    }

    Ok(())
}