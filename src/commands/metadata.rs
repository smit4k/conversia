use tempfile::Builder;
use tokio::fs;
use poise::serenity_prelude::{Attachment, CreateEmbed};
use id3::{Tag as Id3Tag, TagLike};
use metaflac::Tag as FlacTag;

use crate::{Context, Error};

/// View the metadata of an mp3 or flac file
#[poise::command(slash_command)]
pub async fn audio_meta(
    ctx: Context<'_>,
    #[description = "Audio file (.mp3 or .flac) to extract metadata from"] file: Attachment,
) -> Result<(), Error> {
    // Check file extension
    let is_mp3 = file.filename.ends_with(".mp3");
    let is_flac = file.filename.ends_with(".flac");

    if !is_mp3 && !is_flac {
        let embed = CreateEmbed::default()
            .title("❌ Invalid File Format")
            .description("Please upload a valid `.mp3` or `.flac` file.")
            .color(0xff4444);
        let reply = poise::CreateReply::default().embed(embed);
        ctx.send(reply).await?;
        return Ok(());
    }

    // Create a temporary file path with appropriate suffix
    let temp_file_path = Builder::new()
        .suffix(if is_mp3 { ".mp3" } else { ".flac" })
        .tempfile()?
        .into_temp_path();
    let path = temp_file_path.to_path_buf();

    // Download and save file
    let bytes = file.download().await?;
    fs::write(&path, &bytes).await?;

    // Extract metadata
    let (title, artist, album, year, genre) = tokio::task::spawn_blocking(move || {
        if is_mp3 {
            match Id3Tag::read_from_path(&path) {
                Ok(tag) => Ok((
                    tag.title().unwrap_or("Unknown").to_string(),
                    tag.artist().unwrap_or("Unknown").to_string(),
                    tag.album().unwrap_or("Unknown").to_string(),
                    tag.year().map_or("Unknown".to_string(), |y| y.to_string()),
                    tag.genre().unwrap_or("Unknown").to_string(),
                )),
                Err(err) => Err(format!("ID3 error: {}", err)),
            }
        } else {
            match FlacTag::read_from_path(&path) {
                Ok(tag) => {
                    let get = |k: &str| {
                        tag.vorbis_comments()
                            .and_then(|c| c.get(k).and_then(|v| v.first().cloned()))
                            .unwrap_or_else(|| "Unknown".to_string())
                    };
                    Ok((
                        get("TITLE"),
                        get("ARTIST"),
                        get("ALBUM"),
                        get("DATE"),
                        get("GENRE"),
                    ))
                }
                Err(err) => Err(format!("FLAC error: {}", err)),
            }
        }
    }).await.unwrap_or_else(|_| Err("Metadata reading task panicked".to_string()))?;

    // Create response embed
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
