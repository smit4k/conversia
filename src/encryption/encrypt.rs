use std::io::Write;
use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use secrecy::SecretString;
use crate::{Context, Error};

fn encrypt_error_embed(message: impl Into<String>) -> CreateEmbed {
    CreateEmbed::new()
        .title("❌ Encryption Failed")
        .description(message.into())
        .color(0xff4444)
}

/// Encrypt a file using age (ChaCha20-Poly1305)
#[poise::command(slash_command, ephemeral)]
pub async fn encrypt(
    ctx: Context<'_>,
    #[description = "File to encrypt"] file: Attachment,
    #[description = "Password for encryption"] password: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Create temporary directory for file operations
    let temp_dir = Builder::new().prefix("encrypt_").tempdir()?;
    let temp_path = temp_dir.path().to_path_buf();
    
    // Download the attached file
    let file_data = match file.download().await {
        Ok(data) => data,
        Err(_) => {
            let embed = encrypt_error_embed("Failed to download the attached file.");
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };
    let filename = file.filename.clone();
    
    // Write file to temp directory
    let input_file_path = temp_path.join(&filename);
    if fs::write(&input_file_path, &file_data).await.is_err() {
        let embed = encrypt_error_embed("Failed to prepare the file for encryption.");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }
    
    // Move heavy lifting to blocking task
    let encrypted_data = match tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Create output file path
        let output_file_path = temp_path.join(format!("{}.age", filename));
        
        // Encrypt the file using age
        let passphrase = SecretString::new(password.clone().into());
        let encryptor = age::Encryptor::with_user_passphrase(passphrase);
        
        // Read input file
        let input_file = std::fs::File::open(&input_file_path)?;
        let mut input_reader = std::io::BufReader::new(input_file);
        
        // Create output file
        let output_file = std::fs::File::create(&output_file_path)?;
        let mut output_writer = std::io::BufWriter::new(output_file);
        
        // Perform encryption
        let mut writer = encryptor.wrap_output(&mut output_writer)?;
        std::io::copy(&mut input_reader, &mut writer)?;
        writer.finish()?;
        output_writer.flush()?;
        drop(output_writer); // Ensure file is closed before reading
        
        // Read encrypted file
        let encrypted_data = std::fs::read(&output_file_path)?;

        Ok(encrypted_data)
    }).await {
        Ok(Ok(data)) => data,
        Ok(Err(_)) => {
            let embed = encrypt_error_embed("Unable to encrypt this file. Please verify the input and try again.");
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
        Err(_) => {
            let embed = encrypt_error_embed("The encryption task stopped unexpectedly.");
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };
    
    let encrypted_filename = format!("{}.age", file.filename);
    
    // Create attachment from encrypted file
    let attachment = CreateAttachment::bytes(encrypted_data, &encrypted_filename);
    
    // Create embed response
    let embed = CreateEmbed::new()
        .title("✅ File Encrypted Successfully")
        .description(format!(
            "Original file: `{}`\nEncrypted file: `{}`\nKeep your password safe. It is required to decrypt the file later.",
            file.filename, encrypted_filename
        ))
        .field("Encryption Method", "Age (ChaCha20-Poly1305)", true)
        .color(0x27ae60);
    
    // Send response with encrypted file
    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .attachment(attachment)
    ).await?;
    
    Ok(())
}
