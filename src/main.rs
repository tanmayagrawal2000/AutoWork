mod models;
mod history;
mod mailer;
mod screens;
mod auth;
mod discord_bot;

use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::env;
use std::fs;
use std::sync::Arc;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let args: Vec<String> = env::args().collect();
    
    let debug_mode_env = env::var("DEBUG_MODE").unwrap_or_else(|_| "false".to_string()) == "true" || args.contains(&"--debug".to_string());
    
    // Initialize the state.json file if a debug flag is explicitly provided
    if debug_mode_env {
        let _ = fs::create_dir_all("data");
        let state = crate::models::AppState {
            mode: "Debug".to_string(),
            pause_time: None,
            dashboard_msg_id: None,
            consecutive_errors: 0,
        };
        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = fs::write("data/state.json", json);
        }
    }
    
    if args.contains(&"--bot".to_string()) {
        println!("Starting Discord Bot...");
        if let Err(e) = discord_bot::start_bot().await {
            println!("Bot error: {}", e);
        }
        return Ok(());
    }

    let is_headless = args.contains(&"--headless".to_string());
    run_scraper_logic(is_headless).await
}

pub async fn run_scraper_logic(is_headless: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting AutoWork Scraper (Rust/Obscura Backend)...");
    
    if let Ok(content) = std::fs::read_to_string("data/state.json") {
        if let Ok(mut state) = serde_json::from_str::<crate::models::AppState>(&content) {
            if let Some(pause_time) = state.pause_time {
                if chrono::Utc::now() < pause_time {
                    println!("Scraper is paused until {}. Skipping run.", pause_time);
                    return Ok(());
                } else {
                    println!("Pause time has passed. Resuming...");
                    state.pause_time = None;
                    if let Ok(json) = serde_json::to_string_pretty(&state) {
                        let _ = std::fs::write("data/state.json", json);
                    }
                }
            }
        }
    }
    
    let executable_path = env::var("OBSCURA_PATH").unwrap_or_else(|_| "".to_string());
    
    let mut launch_options = LaunchOptionsBuilder::default();
    launch_options.headless(is_headless);
    launch_options.window_size(Some((1920, 1080)));
    
    // Save cookies and session state across runs
    let profile_dir = std::env::current_dir().unwrap().join("data").join("browser_profile");
    let _ = std::fs::create_dir_all(&profile_dir);
    launch_options.user_data_dir(Some(profile_dir));
    
    if !executable_path.is_empty() {
        println!("Using Obscura executable at: {}", executable_path);
        launch_options.path(Some(std::path::PathBuf::from(executable_path)));
    }
    
    let browser = match Browser::new(launch_options.build().unwrap()) {
        Ok(b) => b,
        Err(e) => {
            println!("Failed to launch browser: {}", e);
            return Err(e.into());
        }
    };
    
    let workday_url = env::var("WORKDAY_URL").expect("WORKDAY_URL not set in .env");
    let workday_email = env::var("WORKDAY_EMAIL").expect("WORKDAY_EMAIL not set in .env");
    let workday_password = env::var("WORKDAY_PASSWORD").expect("WORKDAY_PASSWORD not set in .env");
    
    let tab = auth::authenticate(&browser, &workday_url, &workday_email, &workday_password)?;
    let job_titles = screens::scrape_jobs(Arc::clone(&tab))?;
    
    if !job_titles.is_empty() {
        let mut new_jobs = history::filter_new_jobs(job_titles);
        if !new_jobs.is_empty() {
            println!("Found {} completely NEW jobs since our last run!", new_jobs.len());
            
            for job in &mut new_jobs {
                match screens::scrape_job_details_by_click(Arc::clone(&tab), &job.name) {
                    Ok((url, desc)) => {
                        job.url = url;
                        job.description = Some(desc);
                    },
                    Err(e) => println!("Error fetching description for {}: {}", job.name, e),
                }
                println!(" - {} (ID: {})", job.name, job.id);
            }
            
            println!("Hitting the AI API to generate summaries...");
            let client = reqwest::Client::new();
            for job in &mut new_jobs {
                if let Some(desc) = &job.description {
                    let default_prompt = "Analyze the following job description and extract the most critical information needed to decide whether to apply. Output the result strictly as a valid JSON array.\n\nDo not include any conversational text, markdown formatting (like ```json), or explanations.\n\nThe JSON must be an array of objects, where each object contains exactly two keys: \"title\" (the dynamic category name) and \"content\" (a concise, 1-2 sentence summary).\n\nExample format:\n[\n{\"title\": \"Role & Pay\", \"content\": \"CTL Monitor, $17.00 - $19.00/hr, 6-20 hours/week.\"},\n{\"title\": \"Mandatory Requirements\", \"content\": \"Must attend training Sept 2-3 and be in 2nd year or later.\"}\n]".to_string();
                    let ai_prompt_template = env::var("AI_PROMPT").unwrap_or(default_prompt);
                    
                    let prompt = format!("{}\n\nJob Description:\n{}", ai_prompt_template.replace("\\n", "\n"), desc);
                    
                    let req_body = serde_json::json!({ "prompt": prompt });
                    let ai_endpoint = env::var("AI_API_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080/ask".to_string());
                    match client.post(&ai_endpoint).json(&req_body).send().await {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                if let Ok(json_resp) = resp.json::<serde_json::Value>().await {
                                    if let Some(answer) = json_resp.get("answer").and_then(|a| a.as_str()) {
                                        match serde_json::from_str::<crate::models::JobAiSummary>(answer) {
                                            Ok(summary) => {
                                                job.ai_summary = Some(summary);
                                                println!(" - Successfully summarized {}", job.name);
                                            },
                                            Err(e) => println!(" - Failed to parse AI summary for {}: {}", job.name, e),
                                        }
                                    }
                                }
                            } else {
                                println!(" - API returned error status {} for {}", resp.status(), job.name);
                            }
                        },
                        Err(e) => println!(" - Failed to call AI API for {}: {}", job.name, e),
                    }
                }
            }
            
            let new_ids: Vec<String> = new_jobs.iter().map(|j| j.id.clone()).collect();
            history::save_seen_job_ids(new_ids);
            
            let sender_email = env::var("SENDER_EMAIL").unwrap_or_default();
            let email_password = env::var("EMAIL_PASSWORD").unwrap_or_default();
            let receivers_str = env::var("JOB_ALERT_EMAILS").unwrap_or_default();
            let receivers: Vec<String> = receivers_str.split(',').map(|s| s.trim().to_string()).collect();
            
            if let Err(e) = mailer::send_email(&new_jobs, &sender_email, &receivers, &email_password) {
                eprintln!("Failed to send email: {:?}", e);
            }
        } else {
            println!("No completely new jobs found since the last run. We are all up to date!");
        }
    } else {
        println!("No job titles were scraped, skipping email.");
    }
    
    // Reset consecutive errors on successful run
    if let Ok(content) = std::fs::read_to_string("data/state.json") {
        if let Ok(mut state) = serde_json::from_str::<crate::models::AppState>(&content) {
            if state.consecutive_errors > 0 {
                state.consecutive_errors = 0;
                if let Ok(json) = serde_json::to_string_pretty(&state) {
                    let _ = std::fs::write("data/state.json", json);
                }
            }
        }
    }
    
    Ok(())
}
