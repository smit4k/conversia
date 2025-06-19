use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Attachment;
use serenity::builder::CreateEmbed;
use serenity::all::CreateEmbedFooter;

use crate::{Context, Error};
use crate::utils::{format_file_size, detect_file_type};
use std::path::Path;
use hex;

/// Encode a file to hex
#[poise::command(slash_command, ephemeral)]
pub async fn hex_encode(
    ctx: Context<'_>,
    #[description = "File to encode"] file: Attachment,
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

    let encoded = tokio::task::spawn_blocking({
        let data = file_data.clone();
        move || hex::encode(&data)
    }).await?;

    let original_size = format_file_size(file_data.len() as u64);
    let encoded_size = format_file_size(encoded.len() as u64);

    if encoded.len() > 1024 {
        let stem = Path::new(&file.filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        let filename = format!("{}_encoded.txt", stem);
        let encoded_bytes = encoded.into_bytes();
        let attachment = serenity::CreateAttachment::bytes(encoded_bytes, filename);

        let embed = CreateEmbed::new()
            .title("✅ Hex Encoded")
            .description(format!(
                "**Original file:** `{}`\n**Size:** {}\n**Encoded size:** {}",
                file.filename, original_size, encoded_size
            ))
            .footer(CreateEmbedFooter::new("Encoded data is attached as a file."))
            .color(0x27ae60);

        ctx.send(
            poise::CreateReply::default()
                .embed(embed)
                .attachment(attachment)
        ).await?;
    } else {
        let embed = CreateEmbed::new()
            .title("✅ Hex Encoded")
            .description(format!(
                "**Original file:** `{}`\n**Size:** {}\n**Encoded size:** {}",
                file.filename,
                original_size,
                encoded_size,
            ))
            .field("Encoded Data", format!("```\n{}\n```", encoded), false)
            .color(0x27ae60);

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
        let filename = file.filename.clone();
        match file.download().await {
            Ok(file_data) => {
                let string_data = String::from_utf8_lossy(&file_data).to_string();
                (string_data, Some(filename))
            },
            Err(e) => {
                let embed = CreateEmbed::new()
                    .title("❌ Download Failed")
                    .description(format!("Failed to download file: {}", e))
                    .color(0xff4444);
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        }
    } else if let Some(s) = hex_string {
        (s, None)
    } else {
        let embed = CreateEmbed::new()
            .title("❌ No Input Provided")
            .description("Please provide either a hex encoded file or string.")
            .color(0xff4444);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    let hex_input_clone = hex_input.clone();
    let decoded_data = match tokio::task::spawn_blocking(move || hex::decode(hex_input_clone)).await? {
        Ok(data) => data,
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Decoding Failed")
                .description(format!("Failed to decode hex: {}", e))
                .color(0xff4444);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let filename = detect_file_type(&decoded_data);
    let attachment = serenity::CreateAttachment::bytes(decoded_data.clone(), filename);

    let embed = CreateEmbed::new()
        .title("✅ Hex Decoded")
        .description(format!(
            "**Original file:** `{}`\n**Encoded size:** {}\n**Decoded size:** {}",
            original_filename.as_deref().unwrap_or("N/A"),
            format_file_size(hex_input.len() as u64),
            format_file_size(decoded_data.len() as u64)
        ))
        .footer(CreateEmbedFooter::new("Decoded data is attached as a file."))
        .color(0x27ae60);

    ctx.send(poise::CreateReply::default().embed(embed).attachment(attachment)).await?;

    Ok(())
}
