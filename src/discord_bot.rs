use poise::serenity_prelude as serenity;
use std::env;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("▶️ Scraper is ACTIVE (Rust/Obscura Backend).").await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn run_scraper(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🚀 Starting scraper manually...").await?;
    Ok(())
}

pub async fn start_bot() -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| "your_bot_token_here".to_string());
    
    if token == "your_bot_token_here" {
        println!("Please configure your DISCORD_TOKEN in .env");
        return Ok(());
    }

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![status(), run_scraper()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("Discord Bot is running and connected!");
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;
    
    Ok(())
}
