use std::io::Write;
use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use secrecy::SecretString;
use crate::{Context, Error};

/// Encrypt a file using age (ChaCha20-Poly1305)
#[poise::command(slash_command, ephemeral)]
pub async fn encrypt(
    ctx: Context<'_>,
    #[description = "File to encrypt"] file: Attachment,
    #[description = "Password for encryption"] password: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let password_clone = password.clone();
    
    // Create temporary directory for file operations
    let temp_dir = Builder::new().prefix("encrypt_").tempdir()?;
    let temp_path = temp_dir.path().to_path_buf();
    
    // Download the attached file
    let file_data = file.download().await?;
    let filename = file.filename.clone();
    
    // Write file to temp directory
    let input_file_path = temp_path.join(&filename);
    fs::write(&input_file_path, &file_data).await?;
    
    // Move heavy lifting to blocking task
    let encrypted_data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
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
    }).await??; // First ? for JoinError, second ? for the inner Result
    
    let encrypted_filename = format!("{}.age", file.filename);
    
    // Create attachment from encrypted file
    let attachment = CreateAttachment::bytes(encrypted_data, &encrypted_filename);
    
    // Create embed response
    let embed = CreateEmbed::new()
        .title("✅ File Encrypted Successfully")
        .description(format!(
            "Original file: `{}`\nEncrypted file: `{}`\n**Save your password!** You'll need it to decrypt the file.",
            file.filename, encrypted_filename
        ))
        .field("Password", format!("||{}||", password_clone), false)
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
    let file_data = file.download().await?;
    let filename = file.filename.clone();
    
    // Write file to temp directory
    let input_file_path = temp_path.join(&filename);
    fs::write(&input_file_path, &file_data).await?;
    
    // Create original filename by removing .age suffix
    let original_filename = filename.strip_suffix(".age").unwrap_or(&filename).to_string();
    let original_filename_clone = original_filename.clone();
    
    // Move heavy lifting to blocking task
    let decrypted_data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Create output file path
        let output_file_path = temp_path.join(&original_filename_clone);
        
        // Read input file
        let input_file = std::fs::File::open(&input_file_path)?;
        let input_reader = std::io::BufReader::new(input_file);
        
        // Create output file
        let output_file = std::fs::File::create(&output_file_path)?;
        let mut output_writer = std::io::BufWriter::new(output_file);
        
        // Create decryptor from input
        let decryptor = age::Decryptor::new(input_reader)?;
        
        // Create identity from passphrase
        let identity = age::scrypt::Identity::new(SecretString::new(password.clone().into()));
        let mut reader = decryptor.decrypt(std::iter::once(&identity as &dyn age::Identity))?;
        
        // Perform decryption
        std::io::copy(&mut reader, &mut output_writer)?;
        output_writer.flush()?;
        drop(output_writer); // Ensure file is closed before reading
        
        // Read decrypted file
        let decrypted_data = std::fs::read(&output_file_path)?;
        
        Ok(decrypted_data)
    }).await??; // First ? for JoinError, second ? for the inner Result
    
    // Create attachment from decrypted file
    let attachment = CreateAttachment::bytes(decrypted_data, &original_filename);
    
    // Create embed response
    let embed = CreateEmbed::new()
        .title("✅ File Decrypted Successfully")
        .description(format!(
            "Encrypted file: `{}`\nDecrypted file: `{}`",
            file.filename, original_filename
        ))
        .field("Decryption Method", "Age (ChaCha20-Poly1305)", true)
        .color(0x27ae60);
    
    // Send response with decrypted file
    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .attachment(attachment)
    ).await?;
    
    Ok(())
}