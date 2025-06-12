use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateEmbed};
use id3::{Tag, TagLike};

use crate::{Context, Error};


/// View the metadata of an mp3 file
#[poise::command(slash_command)]
pub async fn audio_meta(
    ctx: Context<'_>,
    #[description = "Audio file (MP3) to extract metadata from"] file: Attachment,
) -> Result<(), Error> {
    // Ensure it's an mp3 file
    if !file.filename.ends_with(".mp3") {
        let embed = CreateEmbed::default()
                .title("❌ Invalid file format")
                .description("Please upload a valid .mp3 file")
                .color(0xff4444);

                let reply = poise::CreateReply::default().embed(embed);
                ctx.send(reply).await?;
        return Ok(());
    }

    // Create a temporary file path
    let temp_file_path = Builder::new()
        .suffix(".mp3")
        .tempfile()?
        .into_temp_path();
    let path = temp_file_path.to_path_buf();

    // Download file to bytes
    let bytes = file.download().await?;
    fs::write(&path, &bytes).await?;

    // Read metadata
    let tag = match Tag::read_from_path(&path) {
        Ok(tag) => tag,
        Err(err) => {
            let embed = CreateEmbed::default()
                .title("❌ Failed to read metadata")
                .description(format!("Failed to read ID3 metadata: {}", err))
                .color(0xff4444);

                let reply = poise::CreateReply::default().embed(embed);
                ctx.send(reply).await?;
            
            return Ok(());
        }
    };

    // Metadata variables
    let artist = tag.artist().unwrap_or("Unknown");
    let title = tag.title().unwrap_or("Unknown");
    let album = tag.album().unwrap_or("Unknown");
    let year = tag.year().map_or("Unknown".to_string(), |y| y.to_string());
    let genre = tag.genre().unwrap_or("Unknown");

    let embed = CreateEmbed::default()
        .title(title)
        .field("Artist", artist, false)
        .field("Album", album, false)
        .field("Year", year, false)
        .field("Genre", genre, false)
        .color(0x27ae60);
        
    let reply = poise::CreateReply::default().embed(embed);
    ctx.send(reply).await?;

    Ok(())
}
