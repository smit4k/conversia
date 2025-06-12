use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use pandoc;

use crate::{Context, Error};

#[derive(Debug, poise::ChoiceParameter)]
pub enum OutputFormat {
    #[name = "Markdown (md)"]
    Markdown,
    #[name = "HTML"]
    Html,
    #[name = "PDF"]
    Pdf,
    #[name = "Word Document (docx)"]
    Docx,
    #[name = "OpenDocument Text (odt)"]
    Odt,
    #[name = "EPUB"]
    Epub,
}

/// Convert a document
#[poise::command(slash_command)]
pub async fn convert_document(
    ctx: Context<'_>,
    #[description = "Document to convert"] file: Attachment,
    #[description = "Document format to convert to"] output_format: OutputFormat,
) -> Result<(), Error> {
    // Map user-friendly format names to pandoc format names
    let pandoc_format = match output_format {
        OutputFormat::Markdown => "markdown",
        OutputFormat::Html => "html",
        OutputFormat::Pdf => "pdf",
        OutputFormat::Docx => "docx",
        OutputFormat::Odt => "odt",
        OutputFormat::Epub => "epub",
    };

    // Save the uploaded file with its original extension
    let original_extension = file.filename.rsplit('.').next().unwrap_or("tmp");
    let input_temp_file = Builder::new()
        .suffix(&format!(".{}", original_extension))
        .tempfile()
        .map_err(|e| {
            Error::from(format!("Failed to create temporary file: {}", e))
        })?;
    let input_path = input_temp_file.path().to_path_buf();

    // Download and save the file
    let file_data = file.download().await.map_err(|e| {
        Error::from(format!("Failed to download file: {}", e))
    })?;

    fs::write(&input_path, file_data).await.map_err(|e| {
        Error::from(format!("Failed to write file: {}", e))
    })?;

    // Create output file
    let output_suffix = if pandoc_format == "markdown" {
        ".md".to_string()
    } else {
        format!(".{}", pandoc_format)
    };

    let output_temp_file = Builder::new()
        .suffix(&output_suffix)
        .tempfile()
        .map_err(|e| {
            Error::from(format!("Failed to create output file: {}", e))
        })?;
    let output_path = output_temp_file.path().to_path_buf();

    // Execute pandoc conversion in blocking task
    let input_path_clone = input_path.clone();
    let output_path_clone = output_path.clone();
    let output_format_clone = output_format;

    let conversion_result = tokio::task::spawn_blocking(move || {
        let mut pandoc = pandoc::new();
        pandoc.add_input(&input_path_clone);
        pandoc.set_output(pandoc::OutputKind::File(output_path_clone));
        
        let pandoc_output_format = match output_format_clone {
            OutputFormat::Markdown => pandoc::OutputFormat::Markdown,
            OutputFormat::Html => pandoc::OutputFormat::Html,
            OutputFormat::Pdf => pandoc::OutputFormat::Pdf,
            OutputFormat::Docx => pandoc::OutputFormat::Docx,
            OutputFormat::Odt => pandoc::OutputFormat::Odt,
            OutputFormat::Epub => pandoc::OutputFormat::Epub,
        };
        
        pandoc.set_output_format(pandoc_output_format, Vec::new());
        pandoc.execute()
    }).await;

    // Handle conversion result
    match conversion_result {
        Ok(Ok(_)) => {
            // Conversion successful, read the output file
            let converted_data = fs::read(&output_path).await.map_err(|e| {
                Error::from(format!("Failed to read converted file: {}", e))
            })?;

            // Create output filename
            let base_filename = file.filename.rsplit('.').nth(1).unwrap_or(&file.filename);
            let output_filename = if pandoc_format == "markdown" {
                format!("{}.md", base_filename)
            } else {
                format!("{}.{}", base_filename, pandoc_format)
            };

            // Create attachment and send response
            let attachment = CreateAttachment::bytes(converted_data, &output_filename);
            let embed = CreateEmbed::default()
                .title("✅ Conversion Complete")
                .description(&format!("{} → {}", 
                                    original_extension, 
                                    pandoc_format))
                .color(0x44ff44);

            let reply = poise::CreateReply::default()
                .embed(embed)
                .attachment(attachment);
            
            ctx.send(reply).await?;
        }
        Ok(Err(e)) => {
            // Pandoc conversion failed
            let embed = CreateEmbed::default()
                .title("❌ Conversion failed")
                .description(&format!("Pandoc conversion failed: {}", e))
                .color(0xff4444);
            
            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Err(Error::from(format!("Pandoc failed: {}", e)));
        }
        Err(e) => {
            // Task execution failed
            let embed = CreateEmbed::default()
                .title("❌ Task failed")
                .description("Failed to execute conversion task.")
                .color(0xff4444);
            
            let reply = poise::CreateReply::default().embed(embed);
            ctx.send(reply).await?;
            return Err(Error::from(format!("Task failed: {}", e)));
        }
    }

    Ok(())
}