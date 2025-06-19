use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Attachment;
use ::serenity::all::CreateEmbedFooter;
use serenity::builder::CreateEmbed;
use crate::utils::format_file_size;
use crate::utils::detect_file_type;

use hex;
use crate::{Context, Error};



/// Encode a file to hex
#[poise::command(slash_command)]
pub async fn hex_encode(
    ctx: Context<'_>,
    #[description = "File to encode"] file: Attachment,
) -> Result<(), Error> {
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

    let encoded = hex::encode(&file_data);
    if encoded.len() > 1024 { // Leave room for embed formatting
        let encoded_bytes = encoded.as_bytes();
        let attachment = serenity::CreateAttachment::bytes(
            encoded_bytes, 
            format!("{}_encoded.txt", file.filename.trim_end_matches(|c| c == '.' || char::is_alphanumeric(c)))
        );

        let embed = CreateEmbed::new()
            .title("✅ Hex Encoded")
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


/// Decode a hex encoded file or string
#[poise::command(slash_command)]
pub async fn hex_decode(
    ctx: Context<'_>,
    #[description = "Hex encoded file"] file: Option<Attachment>,
    #[description = "Hex encoded string"] hex_string: Option<String>,
) -> Result<(), Error> {
    let (data_to_decode, original_filename) = if let Some(file) = file {
        let filename = file.filename.clone();
        match file.download().await {
            Ok(file_data) => (file_data, Some(filename)),
            Err(e) => {
                let embed = CreateEmbed::new()
                    .title("❌ Download Failed")
                    .description(format!("Failed to download file: {}", e))
                    .color(0xff4444);
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
                return Ok(());
            }
        }
    } else if let Some(string) = hex_string {
        (string.into_bytes(), None)
    } else {
        let embed = CreateEmbed::new()
            .title("❌ No Input Provided")
            .description("Please provide either a txt file or a hex encoded string.")
            .color(0xff4444);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    match hex::decode(&data_to_decode) {
        Ok(decoded_data) => {
            let filename = detect_file_type(&decoded_data);
            let attachment = serenity::CreateAttachment::bytes(
                decoded_data.clone(),
                filename
            );

            let embed = CreateEmbed::new()
                .title("✅ Hex Decoded")
                .description(format!(
                    "**Original file:** `{}`\n**Size:** {}\n**Decoded size:** {}",
                    original_filename.as_deref().unwrap_or("N/A"),
                    format_file_size(data_to_decode.len() as u64),
                    format_file_size(decoded_data.len() as u64)
                ))
                .footer(CreateEmbedFooter::new("Decoded data is attached as a file."))
                .color(0x27ae60);

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
                    .attachment(attachment)
            ).await?;
        }
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Decoding Failed")
                .description(format!("Failed to decode hex string: {}", e))
                .color(0xff4444);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}