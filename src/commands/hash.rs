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

    let (hash_result, algorithm_name) = tokio::task::spawn_blocking(move || {
        match algorithm {
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
        }
    })
    .await
    .expect("Hashing thread panicked");


    // Create success embed with hash result
    let embed = CreateEmbed::new()
        .title("🔐 File Hash Generated")
        .description(format!(
            "**File:** `{}`\n**Size:** {}\n**Algorithm:** {}",
            file.filename,
            format_file_size(file.size.into()),
            algorithm_name,
        ))
        .field("Hash", format!("```{}```", &hash_result), false)
        .color(0x27ae60);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}