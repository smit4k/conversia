use dotenv::dotenv;
use poise::{serenity_prelude as serenity};
use serenity::{gateway::ActivityData, model::user::OnlineStatus};
use serenity::builder::CreateEmbed;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Checks the bot's latency
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
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
        .field("Latency:", latency_text, true);

        let reply = poise::CreateReply::default().embed(embed);
        ctx.send(reply).await?;
        
    Ok(())
}

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
            commands: vec![ping()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("c!".into()),
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