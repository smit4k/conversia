use poise::{serenity_prelude as serenity};
use ::serenity::{all::{CreateActionRow, CreateButton}};
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
        .field("/help", "Shows you this response", false)
        .footer(
            serenity::builder::CreateEmbedFooter::new("Need help? Reach out on GitHub!")
        );

    let mut issue_button = CreateButton::new_link("https://github.com/smit4k/conversia/issues");
    issue_button = issue_button.label("Found a bug? Create an Issue");
    issue_button = issue_button.emoji(serenity::model::prelude::ReactionType::Custom {
        animated: false,
        id: serenity::model::prelude::EmojiId::new(1382099046654677073),
        name: Some("github_white".to_string()),
    });

    let action_row = CreateActionRow::Buttons(vec![issue_button]);

    let reply = poise::CreateReply::default()
        .embed(embed)
        .components(vec![action_row]);

    ctx.send(reply).await?;

    Ok(())
}