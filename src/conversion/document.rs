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
    #[name = "LaTeX"]
    Latex,
    #[name = "Word Document (docx)"]
    Docx,
    #[name = "Rich Text Format (rtf)"]
    Rtf,
    #[name = "OpenDocument Text (odt)"]
    Odt,
    #[name = "EPUB"]
    Epub,
}

/// Helper function that does the actual document conversion work.
///
/// Returns the converted file bytes and the output filename.
pub async fn convert_document_inner(
    file: &Attachment,
    output_format: OutputFormat,
) -> Result<(Vec<u8>, String), Error> {
    let pandoc_format = match output_format {
        OutputFormat::Markdown => "markdown",
        OutputFormat::Html => "html",
        OutputFormat::Pdf => "pdf",
        OutputFormat::Latex => "tex",
        OutputFormat::Docx => "docx",
        OutputFormat::Rtf => "rtf",
        OutputFormat::Odt => "odt",
        OutputFormat::Epub => "epub",
    };

    let original_extension = file.filename.rsplit('.').next().unwrap_or("tmp");
    let input_temp_file = Builder::new()
        .suffix(&format!(".{}", original_extension))
        .tempfile()
        .map_err(|e| Error::from(format!("Failed to create temporary file: {}", e)))?;
    let input_path = input_temp_file.path().to_path_buf();

    let file_data = file.download().await.map_err(|e| Error::from(format!("Failed to download file: {}", e)))?;
    fs::write(&input_path, file_data).await.map_err(|e| Error::from(format!("Failed to write file: {}", e)))?;

    let output_suffix = if pandoc_format == "markdown" {
        ".md".to_string()
    } else {
        format!(".{}", pandoc_format)
    };

    let output_temp_file = Builder::new()
        .suffix(&output_suffix)
        .tempfile()
        .map_err(|e| Error::from(format!("Failed to create output file: {}", e)))?;
    let output_path = output_temp_file.path().to_path_buf();

    let input_path_clone = input_path.clone();
    let output_path_clone = output_path.clone();
    let output_format_clone = output_format;

    tokio::task::spawn_blocking(move || {
        let mut pandoc = pandoc::new();
        pandoc.add_input(&input_path_clone);
        pandoc.set_output(pandoc::OutputKind::File(output_path_clone));

        let pandoc_output_format = match output_format_clone {
            OutputFormat::Markdown => pandoc::OutputFormat::Markdown,
            OutputFormat::Html => pandoc::OutputFormat::Html,
            OutputFormat::Pdf => pandoc::OutputFormat::Pdf,
            OutputFormat::Latex => pandoc::OutputFormat::Latex,
            OutputFormat::Docx => pandoc::OutputFormat::Docx,
            OutputFormat::Rtf => pandoc::OutputFormat::Rtf,
            OutputFormat::Odt => pandoc::OutputFormat::Odt,
            OutputFormat::Epub => pandoc::OutputFormat::Epub,
        };

        pandoc.set_output_format(pandoc_output_format, Vec::new());
        pandoc.execute()
    }).await.map_err(|e| Error::from(format!("Task failed: {}", e)))??;

    let converted_data = fs::read(&output_path).await.map_err(|e| Error::from(format!("Failed to read converted file: {}", e)))?;

    let base_filename = file.filename.rsplit('.').nth(1).unwrap_or(&file.filename);
    let output_filename = if pandoc_format == "markdown" {
        format!("{}.md", base_filename)
    } else {
        format!("{}.{}", base_filename, pandoc_format)
    };

    // Return converted bytes and output filename
    Ok((converted_data, output_filename))
}

/// Convert a document
#[poise::command(slash_command)]
pub async fn convert_document(
    ctx: Context<'_>,
    #[description = "Document to convert"] file: Attachment,
    #[description = "Document format to convert to"] output_format: OutputFormat,
) -> Result<(), Error> {
    match convert_document_inner(&file, output_format).await {
        Ok((converted_data, output_filename)) => {
            let attachment = CreateAttachment::bytes(converted_data, &output_filename);

            let reply = poise::CreateReply::default()
                .attachment(attachment);

            ctx.send(reply).await?;
        }
        Err(e) => {
            let embed = CreateEmbed::default()
                .title("❌ Conversion Failed")
                .description(format!("{e}"))
                .color(0xff4444);

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Err(e);
        }
    }

    Ok(())
}
