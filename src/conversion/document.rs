use std::process::Command;
use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateAttachment};

use crate::{Context, Error};

/// Convert a document
#[poise::command(slash_command, prefix_command)]
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
            ctx.say("Unsupported output format. Please use one of: md, markdown, html, pdf, docx.")
                .await?;
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
            ctx.say("Failed to create a temporary file for the input.").await?;
            return Err(e.into());
        }
    };
    let input_path = input_temp_file.path().to_path_buf();

    let file_data = match file.download().await {
        Ok(data) => data,
        Err(e) => {
            ctx.say("Failed to download the attached file.").await?;
            return Err(e.into());
        }
    };

    if let Err(e) = fs::write(&input_path, file_data).await {
        ctx.say("Failed to save the attached file to a temporary location.")
            .await?;
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
            ctx.say("Failed to create a temporary file for the output.").await?;
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
                    ctx.say("Failed to read the converted file.").await?;
                    return Err(e.into());
                }
            };

            // Create an attachment from the converted file
            let attachment = CreateAttachment::bytes(
                converted_file_data,
                format!("converted.{}", pandoc_format),
            );

            // Send the converted file back to the user
            let reply = poise::CreateReply::default().attachment(attachment);
            ctx.send(reply).await?;
        }
        Ok(output) => {
            let error_message = String::from_utf8_lossy(&output.stderr);
            ctx.say(format!("Conversion failed: {}", error_message))
                .await?;
        }
        Err(error) => {
            ctx.say(format!("Failed to execute Pandoc: {}", error))
                .await?;
        }
    }

    Ok(())
}