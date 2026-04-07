use crate::utils::{detect_file_type, file_stem, format_file_size, is_previewable_text};
use ::serenity::all::CreateEmbedFooter;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Attachment;
use serenity::builder::CreateEmbed;

use crate::{Context, Error};
use base64::{Engine, engine::general_purpose};

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

fn encoded_summary_embed(filename: &str, original_len: usize, encoded_len: usize) -> CreateEmbed {
    CreateEmbed::new()
        .title("✅ Base64 Encoded")
        .description(format!(
            "**Original file:** `{}`\n**Size:** {}\n**Encoded size:** {}",
            filename,
            format_file_size(original_len as u64),
            format_file_size(encoded_len as u64)
        ))
        .color(EMBED_SUCCESS_COLOR)
}

fn decoded_summary_embed(decoded_len: usize) -> CreateEmbed {
    CreateEmbed::new()
        .title("✅ Base64 Decoded")
        .description(format!(
            "**Decoded size:** {}",
            format_file_size(decoded_len as u64)
        ))
        .color(EMBED_SUCCESS_COLOR)
}

async fn send_decoded_response(ctx: Context<'_>, decoded: Vec<u8>) -> Result<(), Error> {
    if is_previewable_text(&decoded) {
        let decoded_string = String::from_utf8(decoded.clone())
            .map_err(|e| Error::from(format!("Failed to prepare decoded text: {}", e)))?;

        if decoded_string.len() <= INLINE_PREVIEW_LIMIT {
            let embed = decoded_summary_embed(decoded.len()).field(
                "Decoded Data",
                format!("```\n{}\n```", decoded_string),
                false,
            );

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    }

    let attachment = serenity::CreateAttachment::bytes(decoded.clone(), detect_file_type(&decoded));
    let embed = decoded_summary_embed(decoded.len()).footer(CreateEmbedFooter::new(
        "Decoded data is attached as a file.",
    ));

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .attachment(attachment),
    )
    .await?;

    Ok(())
}

/// Encode a file to base64
#[poise::command(slash_command, ephemeral)]
pub async fn base64_encode(
    ctx: Context<'_>,
    #[description = "File to encode"] file: Attachment,
) -> Result<(), Error> {
    ctx.defer().await?;

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

    let file_data_clone = file_data.clone();
    let encoded =
        tokio::task::spawn_blocking(move || general_purpose::STANDARD.encode(&file_data_clone))
            .await?;

    let embed = encoded_summary_embed(&file.filename, file_data.len(), encoded.len());

    if encoded.len() > INLINE_ENCODE_LIMIT {
        // Send as file attachment instead
        let encoded_name = file_stem(&file.filename);
        let attachment = serenity::CreateAttachment::bytes(
            encoded.as_bytes(),
            format!("{}_encoded.txt", encoded_name),
        );

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
        let embed = embed
            .field("Encoded Data", format!("```\n{}\n```", encoded), false)
            .color(EMBED_SUCCESS_COLOR);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}

/// Decode a base64 encoded file or string
#[poise::command(slash_command, ephemeral)]
pub async fn base64_decode(
    ctx: Context<'_>,
    #[description = "Base64 encoded file"] file: Option<Attachment>,
    #[description = "Base64 encoded string"] string: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let data_to_decode = if let Some(file) = file {
        match file.download().await {
            Ok(file_data) => file_data,
            Err(e) => {
                let embed = error_embed(
                    "❌ Download Failed",
                    format!("Failed to download file: {}", e),
                );
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        }
    } else if let Some(string) = string {
        string.trim().as_bytes().to_vec()
    } else {
        let embed = error_embed(
            "❌ No Input Provided",
            "Please provide either a txt file or a base64 encoded string.",
        );
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    let decoded_result =
        tokio::task::spawn_blocking(move || general_purpose::STANDARD.decode(&data_to_decode))
            .await?;

    match decoded_result {
        Ok(decoded) => send_decoded_response(ctx, decoded).await?,
        Err(e) => {
            let embed = error_embed(
                "❌ Decode Failed",
                format!("Failed to decode base64 data: {}", e),
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}
