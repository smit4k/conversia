use dotenv::dotenv;
use poise::{serenity_prelude as serenity};
use serenity::{gateway::ActivityData, model::user::OnlineStatus};

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod commands;
mod conversion;
mod compression;
mod encryption;
mod encoding;
mod utils;

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Bot is connected as {}", data_about_bot.user.name);
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let token = std::env::var("discord_token").expect("Token not found");
    let intents = serenity::GatewayIntents::GUILD_MESSAGES 
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::ping::ping(), 
            commands::about::about(), 
            commands::help::help(),
            commands::hash::hash(),
            commands::resize::resize_image(),
            compression::compress::compress(),
            compression::decompress::decompress(), 
            commands::metadata::audio_meta(),
            conversion::document::convert_document(), 
            conversion::image::convert_image(),
            encryption::encrypt::encrypt(),
            encryption::decrypt::decrypt(),
            encoding::base64::base64_encode(),
            encoding::base64::base64_decode(),
            encoding::hex::hex_encode(),
            encoding::hex::hex_decode(),
        ],

            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("Bot is connected as {}", ready.user.name);
                
                // Set bot presence
                let activity = ActivityData::watching("Your uploads");
                let status = OnlineStatus::Online;
                ctx.set_presence(Some(activity), status);
                
                // Register slash commands globally
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    if let Err(err) = client.unwrap().start().await {
        println!("Client error: {:?}", err);
    }
}