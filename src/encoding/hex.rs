use crate::attachments::{sanitize_filename, validate_attachment_size, validate_output_size};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Attachment;
use serenity::all::CreateEmbedFooter;
use serenity::builder::CreateEmbed;

use crate::utils::{detect_file_type, file_stem, format_file_size, is_previewable_text};
use crate::{Context, Error};
use hex;

const EMBED_ERROR_COLOR: u32 = 0xff4444;
const EMBED_SUCCESS_COLOR: u32 = 0x27ae60;
const INLINE_PREVIEW_LIMIT: usize = 1900;
const INLINE_ENCODE_LIMIT: usize = 1024;

fn error_embed(title: &str, message: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(message.into())
        .color(EMBED_ERROR_COLOR)
}

fn encoded_summary_embed(filename: &str, original_size: &str, encoded_size: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title("✅ Hex Encoded")
        .description(format!(
            "**Original file:** `{}`\n**Size:** {}\n**Encoded size:** {}",
            filename, original_size, encoded_size
        ))
        .color(EMBED_SUCCESS_COLOR)
}

fn decoded_summary_embed(
    original_filename: Option<&str>,
    encoded_len: usize,
    decoded_len: usize,
) -> CreateEmbed {
    CreateEmbed::new()
        .title("✅ Hex Decoded")
        .description(format!(
            "**Original file:** `{}`\n**Encoded size:** {}\n**Decoded size:** {}",
            original_filename.unwrap_or("N/A"),
            format_file_size(encoded_len as u64),
            format_file_size(decoded_len as u64)
        ))
        .color(EMBED_SUCCESS_COLOR)
}

async fn send_decoded_response(
    ctx: Context<'_>,
    original_filename: Option<&str>,
    encoded_len: usize,
    decoded_data: Vec<u8>,
) -> Result<(), Error> {
    if is_previewable_text(&decoded_data) {
        let decoded_string = String::from_utf8(decoded_data.clone())
            .map_err(|e| Error::from(format!("Failed to prepare decoded text: {}", e)))?;

        if decoded_string.len() <= INLINE_PREVIEW_LIMIT {
            let embed = decoded_summary_embed(original_filename, encoded_len, decoded_data.len())
                .field(
                    "Decoded Data",
                    format!("```\n{}\n```", decoded_string),
                    false,
                );

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    }

    validate_output_size(decoded_data.len(), "Decoded data").map_err(Error::from)?;
    let attachment =
        serenity::CreateAttachment::bytes(decoded_data.clone(), detect_file_type(&decoded_data));
    let embed = decoded_summary_embed(original_filename, encoded_len, decoded_data.len()).footer(
        CreateEmbedFooter::new("Decoded data is attached as a file."),
    );

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .attachment(attachment),
    )
    .await?;

    Ok(())
}

/// Encode a file to hex
#[poise::command(slash_command, ephemeral)]
pub async fn hex_encode(
    ctx: Context<'_>,
    #[description = "File to encode"] file: Attachment,
) -> Result<(), Error> {
    ctx.defer().await?;

    if let Err(message) = validate_attachment_size(&file) {
        let embed = error_embed("❌ File Too Large", message);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let file_data = match file.download().await {
        Ok(data) => data,
        Err(e) => {
            let embed = error_embed(
                "❌ Download Failed",
                format!("Failed to download file: {}", e),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let encoded = tokio::task::spawn_blocking({
        let data = file_data.clone();
        move || hex::encode(&data)
    })
    .await?;

    let original_size = format_file_size(file_data.len() as u64);
    let encoded_size = format_file_size(encoded.len() as u64);
    let embed = encoded_summary_embed(&file.filename, &original_size, &encoded_size);

    if encoded.len() > INLINE_ENCODE_LIMIT {
        if let Err(message) = validate_output_size(encoded.len(), "Encoded data") {
            let embed = error_embed("❌ Encoding Failed", message);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }

        let filename = format!("{}_encoded.txt", sanitize_filename(&file_stem(&file.filename)));
        let encoded_bytes = encoded.into_bytes();
        let attachment = serenity::CreateAttachment::bytes(encoded_bytes, filename);
        let embed = embed.footer(CreateEmbedFooter::new(
            "Encoded data is attached as a file.",
        ));

        ctx.send(
            poise::CreateReply::default()
                .embed(embed)
                .attachment(attachment),
        )
        .await?;
    } else {
        let embed = embed.field("Encoded Data", format!("```\n{}\n```", encoded), false);

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}

/// Decode a hex encoded file or string
#[poise::command(slash_command, ephemeral)]
pub async fn hex_decode(
    ctx: Context<'_>,
    #[description = "Hex encoded file"] file: Option<Attachment>,
    #[description = "Hex encoded string"] hex_string: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let (hex_input, original_filename) = if let Some(file) = file {
        if let Err(message) = validate_attachment_size(&file) {
            let embed = error_embed("❌ File Too Large", message);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }

        let filename = file.filename.clone();
        match file.download().await {
            Ok(file_data) => {
                let string_data = String::from_utf8_lossy(&file_data).to_string();
                (string_data, Some(filename))
            }
            Err(e) => {
                let embed = error_embed(
                    "❌ Download Failed",
                    format!("Failed to download file: {}", e),
                );
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        }
    } else if let Some(s) = hex_string {
        let trimmed = s.trim();
        validate_output_size(trimmed.len(), "Encoded input").map_err(Error::from)?;
        (trimmed.to_string(), None)
    } else {
        let embed = error_embed(
            "❌ No Input Provided",
            "Please provide either a hex encoded file or string.",
        );
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    let hex_input_clone = hex_input.clone();
    let decoded_data = match tokio::task::spawn_blocking(move || hex::decode(hex_input_clone))
        .await?
    {
        Ok(data) => data,
        Err(e) => {
            let embed = error_embed("❌ Decoding Failed", format!("Failed to decode hex: {}", e));
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    send_decoded_response(
        ctx,
        original_filename.as_deref(),
        hex_input.len(),
        decoded_data,
    )
    .await?;

    Ok(())
}
