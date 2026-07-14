mod models;
mod history;
mod mailer;
mod screens;
mod auth;
mod discord_bot;

use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::env;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--bot".to_string()) {
        println!("Starting Discord Bot...");
        // the error type returned by start_bot is Send+Sync, so we can unwrap or map it
        if let Err(e) = discord_bot::start_bot().await {
            println!("Bot error: {}", e);
        }
        return Ok(());
    }

    println!("Starting AutoWork Scraper (Rust/Obscura Backend)...");
    
    let is_headless = args.contains(&"--headless".to_string());
    let executable_path = env::var("OBSCURA_PATH").unwrap_or_else(|_| "".to_string());
    
    let mut launch_options = LaunchOptionsBuilder::default();
    launch_options.headless(is_headless);
    
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
    let job_titles = screens::scrape_jobs(tab)?;
    
    if !job_titles.is_empty() {
        let new_jobs = history::filter_new_jobs(job_titles);
        if !new_jobs.is_empty() {
            println!("Found {} completely NEW jobs since our last run!", new_jobs.len());
            for job in &new_jobs {
                println!(" - {} (ID: {})", job.name, job.id);
            }
            
            let new_ids: Vec<String> = new_jobs.iter().map(|j| j.id.clone()).collect();
            history::save_seen_job_ids(new_ids);
            
            let sender_email = env::var("SENDER_EMAIL").unwrap_or_default();
            let email_password = env::var("EMAIL_PASSWORD").unwrap_or_default();
            let receivers_str = env::var("JOB_ALERT_EMAILS").unwrap_or_default();
            let receivers: Vec<String> = receivers_str.split(',').map(|s| s.trim().to_string()).collect();
            
            let _ = mailer::send_email(&new_jobs, &sender_email, &receivers, &email_password);
        } else {
            println!("No completely new jobs found since the last run. We are all up to date!");
        }
    } else {
        println!("No job titles were scraped, skipping email.");
    }
    
    Ok(())
}
