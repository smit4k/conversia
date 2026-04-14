use crate::attachments::{sanitize_filename, validate_attachment_size, validate_output_size};
use pandoc;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use tempfile::Builder;
use tokio::fs;

use crate::utils::file_stem;
use crate::{Context, Error};

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
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

impl OutputFormat {
    const fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Pdf => "pdf",
            Self::Latex => "tex",
            Self::Docx => "docx",
            Self::Rtf => "rtf",
            Self::Odt => "odt",
            Self::Epub => "epub",
        }
    }

    const fn pandoc_output_format(self) -> pandoc::OutputFormat {
        match self {
            Self::Markdown => pandoc::OutputFormat::Markdown,
            Self::Html => pandoc::OutputFormat::Html,
            Self::Pdf => pandoc::OutputFormat::Pdf,
            Self::Latex => pandoc::OutputFormat::Latex,
            Self::Docx => pandoc::OutputFormat::Docx,
            Self::Rtf => pandoc::OutputFormat::Rtf,
            Self::Odt => pandoc::OutputFormat::Odt,
            Self::Epub => pandoc::OutputFormat::Epub,
        }
    }
}

fn output_filename(input_filename: &str, output_format: OutputFormat) -> String {
    let base = file_stem(input_filename);
    format!("{}.{}", sanitize_filename(&base), output_format.extension())
}

fn document_error_message(error: &str, output_format: OutputFormat) -> String {
    let normalized = error.to_lowercase();

    if normalized.contains("pdflatex")
        || normalized.contains("xelatex")
        || normalized.contains("lualatex")
        || normalized.contains("latex")
    {
        return "PDF conversion requires a working LaTeX engine such as pdfTeX, XeLaTeX, or LuaLaTeX.".to_string();
    }

    if normalized.contains("pandoc") && normalized.contains("not found") {
        return "Pandoc is not installed or is not available on the bot host.".to_string();
    }

    if normalized.contains("unknown reader")
        || normalized.contains("could not find reader")
        || normalized.contains("unknown input format")
    {
        return "Pandoc could not determine how to read this document. Try a different source format.".to_string();
    }

    format!(
        "Unable to convert this file to {}. Please verify the input file and server dependencies.",
        output_format.extension()
    )
}

#[cfg(test)]
mod tests {
    use super::{OutputFormat, document_error_message, output_filename};

    #[test]
    fn output_filename_preserves_multi_dot_stem() {
        assert_eq!(
            output_filename("draft.v2.docx", OutputFormat::Pdf),
            "draft.v2.pdf"
        );
        assert_eq!(output_filename("notes", OutputFormat::Markdown), "notes.md");
    }

    #[test]
    fn document_error_message_maps_latex_failures() {
        let message = document_error_message("xelatex not found", OutputFormat::Pdf);
        assert!(message.contains("LaTeX engine"));
    }
}

/// Helper function that does the actual document conversion work.
///
/// Returns the converted file bytes and the output filename.
pub async fn convert_document_inner(
    file: &Attachment,
    output_format: OutputFormat,
) -> Result<(Vec<u8>, String), Error> {
    validate_attachment_size(file).map_err(Error::from)?;

    let original_extension = file.filename.rsplit('.').next().unwrap_or("tmp");
    let input_temp_file = Builder::new()
        .suffix(&format!(".{}", original_extension))
        .tempfile()
        .map_err(|e| Error::from(format!("Failed to create temporary file: {}", e)))?;
    let input_path = input_temp_file.path().to_path_buf();

    let file_data = file
        .download()
        .await
        .map_err(|e| Error::from(format!("Failed to download file: {}", e)))?;
    fs::write(&input_path, file_data)
        .await
        .map_err(|e| Error::from(format!("Failed to write file: {}", e)))?;

    let output_temp_file = Builder::new()
        .suffix(&format!(".{}", output_format.extension()))
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

        pandoc.set_output_format(output_format_clone.pandoc_output_format(), Vec::new());
        pandoc.execute()
    })
    .await
    .map_err(|e| Error::from(format!("Task failed: {}", e)))??;

    let converted_data = fs::read(&output_path)
        .await
        .map_err(|e| Error::from(format!("Failed to read converted file: {}", e)))?;
    validate_output_size(converted_data.len(), "Converted document").map_err(Error::from)?;
    let output_filename = output_filename(&file.filename, output_format);

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
    ctx.defer().await?;

    match convert_document_inner(&file, output_format).await {
        Ok((converted_data, output_filename)) => {
            let attachment = CreateAttachment::bytes(converted_data, &output_filename);

            let reply = poise::CreateReply::default().attachment(attachment);

            ctx.send(reply).await?;
        }
        Err(e) => {
            let message = document_error_message(&e.to_string(), output_format);
            let embed = CreateEmbed::default()
                .title("❌ Conversion Failed")
                .description(message)
                .color(0xff4444);

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    }

    Ok(())
}
