use poise::{serenity_prelude as serenity, CreateReply};
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::builder::CreateEmbed::default()
        .title("Conversia Help")
        .description("Here are the commands you can use with Conversia:")
        .field("/convert_document", "Convert documents to formats like PDF, Markdown, HTML, and Word.", false)
        .field("/convert_image", "Convert images between formats (e.g., JPG, PNG, WEBP, etc.).", false)
        .field("/compress", "Compress files into ZIP, TAR.GZ, BZ2, ZST, or LZ4 formats.", false)
        .field("/encrypt", "Encrypt files securely using the Age encryption standard.", false)
        .field("/decrypt", "Decrypt files that were previously encrypted.", false)
        .field("/audio_meta", "Extract metadata from MP3 files (title, artist, album, year, genre).", false)
        .field("/about", "Learn more about the Conversia bot.", false)
        .field("/ping", "Check the bot's latency", false)
        .field("/help", "Shows you this response", false);

        let reply = CreateReply::default().embed(embed);
        ctx.send(reply).await?;

    Ok(())
}