use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Attachment;
use serenity::builder::CreateEmbed;
use sha2::{Sha256, Digest as Sha2Digest};
use sha1::Sha1;
use blake3;
use crate::utils::format_file_size;
use crate::{Context, Error};

#[derive(Debug, poise::ChoiceParameter)]
pub enum HashAlgorithm {
    #[name = "SHA-256"]
    Sha256,
    #[name = "SHA-1"]
    Sha1,
    #[name = "MD5"]
    Md5,
    #[name = "BLAKE3"]
    Blake3,
}

/// Generate hash for a file
#[poise::command(slash_command)]
pub async fn hash(
    ctx: Context<'_>,
    #[description = "File to hash"] file: Attachment,
    #[description = "Hash algorithm to use"] algorithm: HashAlgorithm,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Check file size limit (25MB)
    if file.size > 25 * 1024 * 1024 {
        let embed = CreateEmbed::new()
            .title("❌ File Too Large")
            .description("File size must be under 25MB for hashing")
            .color(0xff4444);

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // Download the file
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

    // Calculate hash based on selected algorithm
    let (hash_result, algorithm_name) = match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(&file_data);
            let result = hasher.finalize();
            (format!("{:x}", result), "SHA-256")
        }
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(&file_data);
            let result = hasher.finalize();
            (format!("{:x}", result), "SHA-1")
        }
        HashAlgorithm::Md5 => {
            let mut hasher = md5::Context::new();
            hasher.consume(&file_data);
            let result = hasher.compute();
            (format!("{:x}", result), "MD5")
        }
        HashAlgorithm::Blake3 => {
            let hash = blake3::hash(&file_data);
            (hash.to_hex().to_string(), "BLAKE3")
        }
    };

    // Create success embed with hash result
    let embed = CreateEmbed::new()
        .title("🔐 File Hash Generated")
        .description(format!(
            "**File:** `{}`\n**Size:** {}\n**Algorithm:** {}\n\n**Hash:**\n```\n{}\n```",
            file.filename,
            format_file_size(file.size.into()),
            algorithm_name,
            hash_result
        ))
        .color(0x27ae60);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}