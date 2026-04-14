use crate::attachments::validate_attachment_size;
use crate::utils::format_file_size;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Attachment;
use serenity::builder::CreateEmbed;
use sha1::Sha1;
use sha2::{Digest as Sha2Digest, Sha256};

#[derive(Debug, poise::ChoiceParameter)]
pub enum HashAlgorithm {
    #[name = "SHA-256"]
    Sha256,
    #[name = "SHA-1 (legacy/insecure)"]
    Sha1,
    #[name = "MD5 (legacy/insecure)"]
    Md5,
    #[name = "BLAKE3"]
    Blake3,
}

/// Compute a hash of the given data using the specified algorithm.
/// Returns the hex-encoded hash string and the algorithm display name.
fn compute_hash(data: &[u8], algorithm: &HashAlgorithm) -> (String, &'static str) {
    match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            (format!("{:x}", hasher.finalize()), "SHA-256")
        }
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(data);
            (format!("{:x}", hasher.finalize()), "SHA-1 (legacy/insecure)")
        }
        HashAlgorithm::Md5 => {
            let mut hasher = md5::Context::new();
            hasher.consume(data);
            (format!("{:x}", hasher.compute()), "MD5 (legacy/insecure)")
        }
        HashAlgorithm::Blake3 => {
            let hash = blake3::hash(data);
            (hash.to_hex().to_string(), "BLAKE3")
        }
    }
}

/// Download an attachment, returning its bytes or sending an error embed on failure.
async fn download_file(ctx: Context<'_>, file: &Attachment) -> Result<Option<Vec<u8>>, Error> {
    if let Err(message) = validate_attachment_size(file) {
        let embed = CreateEmbed::new()
            .title("❌ File Too Large")
            .description(message)
            .color(0xff4444);
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(None);
    }

    match file.download().await {
        Ok(data) => Ok(Some(data)),
        Err(e) => {
            let embed = CreateEmbed::new()
                .title("❌ Download Failed")
                .description(format!("Failed to download file: {}", e))
                .color(0xff4444);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            Ok(None)
        }
    }
}

/// Generate hash for a file
#[poise::command(slash_command)]
pub async fn hash(
    ctx: Context<'_>,
    #[description = "File to hash"] file: Attachment,
    #[description = "Hash algorithm to use"] algorithm: HashAlgorithm,
) -> Result<(), Error> {
    ctx.defer().await?;

    let file_data = match download_file(ctx, &file).await? {
        Some(data) => data,
        None => return Ok(()),
    };

    let (hash_result, algorithm_name) =
        tokio::task::spawn_blocking(move || compute_hash(&file_data, &algorithm)).await?;

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

/// Verify a file's hash against an expected checksum
#[poise::command(slash_command)]
pub async fn verify_hash(
    ctx: Context<'_>,
    #[description = "File to verify"] file: Attachment,
    #[description = "Hash value to compare against"] expected_hash: String,
    #[description = "Hash algorithm"] algorithm: HashAlgorithm,
) -> Result<(), Error> {
    ctx.defer().await?;

    let expected_hash = expected_hash.trim().to_ascii_lowercase();
    let file_data = match download_file(ctx, &file).await? {
        Some(data) => data,
        None => return Ok(()),
    };

    let (actual_hash, algorithm_name) =
        tokio::task::spawn_blocking(move || compute_hash(&file_data, &algorithm)).await?;

    let matches = actual_hash == expected_hash;
    let embed = CreateEmbed::new()
        .title(if matches {
            "✅ Valid Checksum"
        } else {
            "❌ Invalid Checksum"
        })
        .field("Expected Hash", format!("```{}```", expected_hash), true)
        .field("Actual Hash", format!("```{}```", actual_hash), true)
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Algorithm: {}",
            algorithm_name
        )))
        .color(if matches { 0x27ae60 } else { 0xff4444 });

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
