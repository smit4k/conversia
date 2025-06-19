use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Attachment;
use ::serenity::all::CreateEmbedFooter;
use serenity::builder::CreateEmbed;
use crate::utils::format_file_size;
use crate::utils::detect_file_type;

use base64::{engine::general_purpose, Engine};
use crate::{Context, Error};


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
            let embed = CreateEmbed::new()
                .title("❌ Download Failed")
                .description(format!("Failed to download file: {}", e))
                .color(0xff4444);
            
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let file_data_clone = file_data.clone();
    let encoded = tokio::task::spawn_blocking(move || {
        general_purpose::STANDARD.encode(&file_data_clone)
    }).await?;

    if encoded.len() > 1024 {  // Leave room for embed formatting
        // Send as file attachment instead
        let encoded_bytes = encoded.as_bytes();
        let attachment = serenity::CreateAttachment::bytes(
            encoded_bytes, 
            format!("{}_encoded.txt", file.filename.trim_end_matches(|c| c == '.' || char::is_alphanumeric(c)))
        );

        let embed = CreateEmbed::new()
            .title("✅ Base64 Encoded")
            .description(format!(
                "**Original file:** `{}`\n**Size:** {}\n**Encoded size:** {}",
                file.filename,
                format_file_size(file_data.len() as u64),
                format_file_size(encoded.len() as u64)
            ))
            .footer(CreateEmbedFooter::new("Encoded data is attached as a file."))
            .color(0x27ae60);

        ctx.send(
            poise::CreateReply::default()
                .embed(embed)
                .attachment(attachment)
        ).await?;
    } else {
        // Send encoded string in embed
        let embed = CreateEmbed::new()
            .title("✅ Base64 Encoded")
            .description(format!(
                "**Original file:** `{}`\n**Size:** {}\n**Encoded size:** {}",
                file.filename,
                format_file_size(file_data.len() as u64),
                format_file_size(encoded.len() as u64)
            ))
            .field("Encoded Data", format!("```\n{}\n```", encoded), false)
            .color(0x27ae60);

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
                let embed = CreateEmbed::new()
                    .title("❌ Download Failed")
                    .description(format!("Failed to download file: {}", e))
                    .color(0xff4444);
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        }
    } else if let Some(string) = string {
        string.into_bytes()
    } else {
        let embed = CreateEmbed::new()
            .title("❌ No Input Provided")
            .description("Please provide either a txt file or a base64 encoded string.")
            .color(0xff4444);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    let decoded_result = tokio::task::spawn_blocking(move || {
        general_purpose::STANDARD.decode(&data_to_decode)
    }).await?;

    match decoded_result {
        Ok(decoded) => {
            // Check if decoded data is too large for Discord message
            let decoded_string = String::from_utf8_lossy(&decoded);
            
            if decoded_string.len() > 1900 {
                // Send as file attachment
                let filename = detect_file_type(&decoded);
                let attachment = serenity::CreateAttachment::bytes(
                    decoded.clone(),
                    filename
                );

                let embed = CreateEmbed::new()
                    .title("✅ Base64 Decoded")
                    .description(format!(
                        "**Decoded size:** {}",
                        format_file_size(decoded.len() as u64)
                    ))
                    .footer(CreateEmbedFooter::new("Decoded data is attached as a file."))
                    .color(0x27ae60);

                ctx.send(
                    poise::CreateReply::default()
                        .embed(embed)
                        .attachment(attachment)
                ).await?;
            } else {
                let embed = CreateEmbed::new()
                    .title("✅ Base64 Decoded")
                    .description(format!("**Decoded size:** {}", format_file_size(decoded.len() as u64)))
                    .field("Decoded Data", format!("```\n{}\n```", decoded_string), false)
                    .color(0x27ae60);

                ctx.send(poise::CreateReply::default().embed(embed)).await?;
            }
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Decode Failed")
                .description(format!("Failed to decode base64 data: {}", e))
                .color(0xff4444);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}
