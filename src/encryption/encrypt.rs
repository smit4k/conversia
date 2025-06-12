use std::io::Write;
use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use secrecy::SecretString;
use crate::{utils::format_file_size, Context, Error};

/// Encrypt a file using age (ChaCha20-Poly1305)
#[poise::command(slash_command, ephemeral)]
pub async fn encrypt(
    ctx: Context<'_>,
    #[description = "File to encrypt"] file: Attachment,
    #[description = "Password for encryption"] password: String,
) -> Result<(), Error> {
    // Create temporary directory for file operations
    let temp_dir = Builder::new().prefix("encrypt_").tempdir()?;
    let temp_path = temp_dir.path();
    
    // Download the attached file
    let file_data = file.download().await?;
    let input_file_path = temp_path.join(&file.filename);
    fs::write(&input_file_path, &file_data).await?;
    
    // Create output file path
    let output_file_path = temp_path.join(format!("{}.age", file.filename));
    
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
    
    // Read encrypted file
    let encrypted_data = fs::read(&output_file_path).await?;
    let encrypted_filename = format!("{}.age", file.filename);
    
    // Create attachment from encrypted file
    let attachment = CreateAttachment::bytes(encrypted_data.clone(), &encrypted_filename);
    
    // Create  embed response
    let embed = CreateEmbed::new()
        .title("✅ File Encrypted Successfully")
        .description(format!(
            "Original file: `{}`\nEncrypted file: `{}`\n**Save your password!** You'll need it to decrypt the file.", 
            file.filename, encrypted_filename
        ))
        .field("Password", format!("||{}||", password), false)
        .field("Encryption Method", "Age (ChaCha20-Poly1305)", true)
        .field("Original Size", format_file_size(file.size.into()), true)
        .field("Encrypted Size", format_file_size(encrypted_data.len().try_into().unwrap()), true)
        .color(0x27ae60);
    
    // Send response with encrypted file
    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .attachment(attachment)
    ).await?;
    
    Ok(())
}