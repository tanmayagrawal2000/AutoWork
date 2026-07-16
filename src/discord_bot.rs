use poise::serenity_prelude as serenity;
use std::env;
use std::fs;
use chrono::{Duration, Utc};
use crate::models::AppState;

pub struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;

fn get_state() -> AppState {
    if let Ok(content) = fs::read_to_string("data/state.json") {
        if let Ok(state) = serde_json::from_str::<AppState>(&content) {
            return state;
        }
    }
    AppState {
        mode: "Default".to_string(),
        pause_time: None,
        dashboard_msg_id: None,
        consecutive_errors: 0,
    }
}

fn set_state(state: &AppState) {
    let _ = fs::create_dir_all("data");
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write("data/state.json", json);
    }
}

fn get_dashboard_ui(state: &AppState) -> (String, Vec<serenity::CreateActionRow>) {
    let pause_status = match state.pause_time {
        Some(pt) if pt > Utc::now() => {
            let diff = pt - Utc::now();
            format!("⏸️ Paused for {} more hours (until {} UTC)", diff.num_hours(), pt.format("%H:%M"))
        },
        _ => "▶️ Active".to_string(),
    };

    let content = format!("**AutoWork Control Panel**\n**Current Mode:** {}\n**Status:** {}", state.mode, pause_status);
    let components = vec![
        serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new(
            "mode_select",
            serenity::CreateSelectMenuKind::String {
                options: vec![
                    serenity::CreateSelectMenuOption::new("Default Mode", "Default"),
                    serenity::CreateSelectMenuOption::new("Debug Mode (Screenshots)", "Debug"),
                ],
            },
        ).placeholder("Change Mode...")),
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("run_scraper")
                .label("Run Scraper")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("pause_scraper")
                .label("Pause")
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new("resume_scraper")
                .label("Resume")
                .style(serenity::ButtonStyle::Success),
            serenity::CreateButton::new("reset_state")
                .label("Reset State")
                .style(serenity::ButtonStyle::Danger),
        ])
    ];
    
    (content, components)
}

async fn send_dashboard(ctx: &serenity::Context, channel_id: serenity::ChannelId) -> Result<(), Error> {
    let mut state = get_state();
    
    if let Some(old_msg_id) = state.dashboard_msg_id {
        let _ = channel_id.delete_message(ctx, serenity::MessageId::new(old_msg_id)).await;
    }
    
    let (content, components) = get_dashboard_ui(&state);
    let builder = serenity::CreateMessage::new().content(content).components(components);

    let msg = channel_id.send_message(ctx, builder).await?;
    
    state.dashboard_msg_id = Some(msg.id.get());
    set_state(&state);
    
    Ok(())
}

async fn handle_event(
    ctx: &serenity::Context,
    event: &poise::serenity_prelude::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        poise::serenity_prelude::FullEvent::InteractionCreate { interaction } => {
            match interaction {
                serenity::Interaction::Component(component) => {
                    let custom_id = component.data.custom_id.as_str();
                    
                    match custom_id {
                        "run_scraper" => {
                            let (mut content, components) = get_dashboard_ui(&get_state());
                            content = format!("{}\n*(🚀 Starting headless scraper...)*", content);
                            let _ = component.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new().content(content).components(components)
                            )).await;
                            
                            tokio::spawn(async move {
                                if let Err(e) = crate::run_scraper_logic(true).await {
                                    println!("Background scraper failed: {}", e);
                                }
                            });
                        },
                        "reset_state" => {
                            let _ = fs::remove_dir_all("data/browser_profile");
                            let (mut content, components) = get_dashboard_ui(&get_state());
                            content = format!("{}\n*(🗑️ Browser profile deleted. Will re-authenticate on next run.)*", content);
                            let _ = component.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new().content(content).components(components)
                            )).await;
                        },
                        "resume_scraper" => {
                            let mut state = get_state();
                            state.pause_time = None;
                            set_state(&state);
                            
                            let (content, components) = get_dashboard_ui(&state);
                            let _ = component.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new().content(content).components(components)
                            )).await;
                        },
                        "pause_scraper" => {
                            let state = get_state();
                            if let Some(pt) = state.pause_time {
                                if pt > Utc::now() {
                                    let (content, components) = get_dashboard_ui(&state);
                                    let _ = component.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                                        serenity::CreateInteractionResponseMessage::new().content(content).components(components)
                                    )).await;
                                    return Ok(());
                                }
                            }
                            
                            let modal = serenity::CreateModal::new("pause_modal", "Pause Scraper")
                                .components(vec![serenity::CreateActionRow::InputText(
                                    serenity::CreateInputText::new(serenity::InputTextStyle::Short, "Pause for how many hours?", "hours_input")
                                        .placeholder("e.g. 2")
                                        .required(true)
                                )]);
                            
                            let _ = component.create_response(ctx, serenity::CreateInteractionResponse::Modal(modal)).await;
                        },
                        "mode_select" => {
                            if let serenity::ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
                                if let Some(selected) = values.first() {
                                    let mut state = get_state();
                                    state.mode = selected.clone();
                                    set_state(&state);
                                    
                                    let (content, components) = get_dashboard_ui(&state);
                                    let _ = component.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                                        serenity::CreateInteractionResponseMessage::new().content(content).components(components)
                                    )).await;
                                }
                            }
                        },
                        _ => {}
                    }
                },
                serenity::Interaction::Modal(modal) => {
                    if modal.data.custom_id == "pause_modal" {
                        if let Some(serenity::ActionRowComponent::InputText(input)) = modal.data.components.first().and_then(|r| r.components.first()) {
                            if let Ok(hours) = input.value.as_ref().unwrap_or(&"".to_string()).parse::<i64>() {
                                let mut state = get_state();
                                state.pause_time = Some(Utc::now() + Duration::hours(hours));
                                set_state(&state);
                                
                                let (content, components) = get_dashboard_ui(&state);
                                let _ = modal.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                                    serenity::CreateInteractionResponseMessage::new().content(content).components(components)
                                )).await;
                            } else {
                                let (mut content, components) = get_dashboard_ui(&get_state());
                                content = format!("{}\n*(❌ Invalid number of hours entered!)*", content);
                                let _ = modal.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                                    serenity::CreateInteractionResponseMessage::new().content(content).components(components)
                                )).await;
                            }
                        }
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }
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
            event_handler: |ctx, event, framework, data| {
                Box::pin(handle_event(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("Discord Bot is running and connected!");
                
                if let Ok(channel_str) = env::var("DISCORD_CHANNEL_ID") {
                    if let Ok(channel_id_u64) = channel_str.parse::<u64>() {
                        let channel_id = serenity::ChannelId::new(channel_id_u64);
                        if let Err(e) = send_dashboard(ctx, channel_id).await {
                            println!("Failed to send dashboard on startup: {}", e);
                        } else {
                            println!("Dashboard successfully posted to channel {}!", channel_id_u64);
                        }
                    } else {
                        println!("DISCORD_CHANNEL_ID is not a valid u64 number.");
                    }
                } else {
                    println!("DISCORD_CHANNEL_ID not set. The bot will not auto-post the dashboard.");
                }
                
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
