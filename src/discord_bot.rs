use poise::serenity_prelude as serenity;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Data {
    pub debug_mode: Arc<AtomicBool>,
} // User data, which is stored and accessible in all command invocations

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let debug = ctx.data().debug_mode.load(Ordering::SeqCst);
    ctx.say(format!("▶️ Scraper is ACTIVE (Rust/Obscura Backend). Debug Mode: {}", debug)).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn toggle_debug(ctx: Context<'_>) -> Result<(), Error> {
    let debug = ctx.data().debug_mode.load(Ordering::SeqCst);
    ctx.data().debug_mode.store(!debug, Ordering::SeqCst);
    ctx.say(format!("🔧 Debug mode is now {}", if !debug { "ON" } else { "OFF" })).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn run_scraper(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🚀 Starting scraper in the background...").await?;
    
    let debug_mode = Arc::clone(&ctx.data().debug_mode);
    let is_headless = env::args().collect::<Vec<String>>().contains(&"--headless".to_string());
    
    tokio::spawn(async move {
        if let Err(e) = crate::run_scraper_logic(debug_mode, is_headless).await {
            println!("Background scraper failed: {}", e);
        }
    });
    
    Ok(())
}

pub async fn start_bot(debug_mode: Arc<AtomicBool>) -> Result<(), Error> {
    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| "your_bot_token_here".to_string());
    
    if token == "your_bot_token_here" {
        println!("Please configure your DISCORD_TOKEN in .env");
        return Ok(());
    }

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![status(), run_scraper(), toggle_debug()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("Discord Bot is running and connected!");
                Ok(Data { debug_mode })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    client.start().await?;
    
    Ok(())
}
