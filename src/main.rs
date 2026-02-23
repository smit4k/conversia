use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use serenity::{gateway::ActivityData, model::user::OnlineStatus};

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod commands;
mod compression;
mod conversion;
mod encoding;
mod encryption;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("discord_token").expect("Missing 'discord_token' environment variable");
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                // General
                commands::ping::ping(),
                commands::about::about(),
                commands::help::help(),
                // Hashing
                commands::hash::hash(),
                commands::hash::verify_hash(),
                // Image tools
                commands::resize::resize_image(),
                conversion::image::convert_image(),
                // Document conversion
                conversion::document::convert_document(),
                // Compression
                compression::compress::zip(),
                compression::decompress::unzip(),
                // Encryption
                encryption::encrypt::encrypt(),
                encryption::decrypt::decrypt(),
                // Encoding
                encoding::base64::base64_encode(),
                encoding::base64::base64_decode(),
                encoding::hex::hex_encode(),
                encoding::hex::hex_decode(),
                // Audio
                commands::metadata::audio_meta(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("Bot is connected as {}", ready.user.name);

                let activity = ActivityData::watching("Your uploads");
                ctx.set_presence(Some(activity), OnlineStatus::Online);

                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    match client {
        Ok(mut client) => {
            if let Err(err) = client.start().await {
                eprintln!("Client runtime error: {:?}", err);
            }
        }
        Err(err) => {
            eprintln!("Failed to create client: {:?}", err);
        }
    }
}
