use crate::{Context, Error};
use poise::serenity_prelude::{Attachment, CreateAttachment};
use secrecy::SecretString;
use serenity::builder::CreateEmbed;
use std::io::Write;
use tempfile::Builder;
use tokio::fs;

fn decrypt_error_embed(message: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title("❌ Decryption Failed")
        .description(message.into())
        .color(0xff4444)
}

/// Decrypt a file using age (ChaCha20-Poly1305)
#[poise::command(slash_command, ephemeral)]
pub async fn decrypt(
    ctx: Context<'_>,
    #[description = "File to decrypt"] file: Attachment,
    #[description = "Password for decryption"] password: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Create temporary directory for file operations
    let temp_dir = Builder::new().prefix("decrypt_").tempdir()?;
    let temp_path = temp_dir.path().to_path_buf();

    // Download the attached file
    let file_data = match file.download().await {
        Ok(data) => data,
        Err(_) => {
            let embed = decrypt_error_embed("Failed to download the attached file.");
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };
    let filename = file.filename.clone();

    // Write file to temp directory
    let input_file_path = temp_path.join(&filename);
    if fs::write(&input_file_path, &file_data).await.is_err() {
        let embed = decrypt_error_embed("Failed to prepare the file for decryption.");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    // Create original filename by removing .age suffix
    let original_filename = filename
        .strip_suffix(".age")
        .unwrap_or(&filename)
        .to_string();
    let original_filename_clone = original_filename.clone();

    // Move heavy lifting to blocking task
    let decrypted_data = match tokio::task::spawn_blocking(
        move || -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
            let output_file_path = temp_path.join(&original_filename_clone);

            let input_file = std::fs::File::open(&input_file_path)?;
            let input_reader = std::io::BufReader::new(input_file);

            let output_file = std::fs::File::create(&output_file_path)?;
            let mut output_writer = std::io::BufWriter::new(output_file);

            let decryptor = age::Decryptor::new(input_reader)?;
            let identity = age::scrypt::Identity::new(SecretString::new(password.clone().into()));
            let mut reader = decryptor.decrypt(std::iter::once(&identity as &dyn age::Identity))?;

            std::io::copy(&mut reader, &mut output_writer)?;
            output_writer.flush()?;
            drop(output_writer);

            let decrypted_data = std::fs::read(&output_file_path)?;
            Ok(decrypted_data)
        },
    )
    .await
    {
        Ok(Ok(data)) => data,
        Ok(Err(_)) => {
            let embed = decrypt_error_embed(
                "Unable to decrypt this file. Check that the file is age-encrypted and that the password is correct.",
            );
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
        Err(_) => {
            let embed = decrypt_error_embed("The decryption task stopped unexpectedly.");
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let attachment = CreateAttachment::bytes(decrypted_data, &original_filename);

    let embed = CreateEmbed::new()
        .title("✅ File Decrypted Successfully")
        .description(format!(
            "Encrypted file: `{}`\nDecrypted file: `{}`",
            file.filename, original_filename
        ))
        .field("Decryption Method", "Age (ChaCha20-Poly1305)", true)
        .color(0x27ae60);

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .attachment(attachment),
    )
    .await?;

    Ok(())
}
