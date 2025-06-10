use poise::{serenity_prelude as serenity};
use serenity::builder::CreateEmbed;

use crate::{Context, Error};

/// Checks the bot's latency
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let shard_manager = ctx.framework().shard_manager();

    let runners  = shard_manager.runners.lock().await;
    
    let latency = if let Some((_, runner)) = runners.iter().next() {
        runner.latency
    } else {
        None
    };

    let latency_text = match latency {
        Some(duration) => format!("{}ms", duration.as_millis()),
        None => "Unknown".to_string(),
    };
    
    let embed = CreateEmbed::default()
        .title("Pong!")
        .field("Latency:", latency_text.clone(), true)
        .color(if latency_text == "Unknown" { 0xff4444 } else { 0x44ff44 });

        let reply = poise::CreateReply::default().embed(embed);
        ctx.send(reply).await?;
        
    Ok(())
}