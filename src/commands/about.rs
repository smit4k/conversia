use poise::{serenity_prelude as serenity};
use ::serenity::{all::{CreateActionRow, CreateButton}};
use serenity::builder::CreateEmbed;

use crate::{Context, Error};

/// Shows information about Conversia
#[poise::command(slash_command, prefix_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .title("About Conversia")
        .description("Conversia is a powerful multi purpose file utility bot")
        .footer(
            serenity::builder::CreateEmbedFooter::new("Built in Rust")
                .icon_url("https://cdn.discordapp.com/emojis/1382092921238458448.png")
        );
        
    let mut button = CreateButton::new_link("https://github.com/smit4k/conversia");
    button = button.label("Source Code");
    button = button.emoji(serenity::model::prelude::ReactionType::Custom {
        animated: false,
        id: serenity::model::prelude::EmojiId::new(1382099046654677073),
        name: Some("github_white".to_string()),
    });

    let mut invite_button = CreateButton::new_link("https://discord.com/oauth2/authorize?client_id=1373693356928012328&permissions=51200&integration_type=0&scope=bot+applications.commands");
    invite_button = invite_button.label("Add to server");

    let action_row = CreateActionRow::Buttons(vec![invite_button, button]);

    let reply = poise::CreateReply::default()
        .embed(embed)
        .components(vec![action_row]);

    ctx.send(reply).await?;

    Ok(())
}

