use std::io::Write;
use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateAttachment};
use serenity::builder::CreateEmbed;
use secrecy::SecretString;
use crate::{utils::format_file_size, Context, Error};

/// Decrypt a file using age (ChaCha20-Poly1305)
#[poise::command(slash_command, ephemeral)]
pub async fn decrypt(
    ctx: Context<'_>,
    #[description = "File to decrypt"] file: Attachment,
    #[description = "Password for decryption"] password: String,
) -> Result<(), Error> {
    // Create temporary directory for file operations
    let temp_dir = Builder::new().prefix("decrypt_").tempdir()?;
    let temp_path = temp_dir.path();
    
    // Download the attached file
    let file_data = file.download().await?;
    let input_file_path = temp_path.join(&file.filename);
    fs::write(&input_file_path, &file_data).await?;
    
    // Create output file path
    let original_filename = file.filename.strip_suffix(".age").unwrap_or(&file.filename);
    let output_file_path = temp_path.join(original_filename);
    
    // Decrypt the file using age
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
    std::io::copy(&mut reader, &mut output_writer)?;
    output_writer.flush()?;
    
    // Read decrypted file
    let decrypted_data = fs::read(&output_file_path).await?;
    
    // Create attachment from decrypted file
    let attachment = CreateAttachment::bytes(decrypted_data.clone(), original_filename);
    
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