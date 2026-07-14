use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use crate::models::Job;

fn history_file_path() -> PathBuf {
    PathBuf::from("data/seen_jobs.json")
}

pub fn get_seen_job_ids() -> HashSet<String> {
    let path = history_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(ids) = serde_json::from_str::<Vec<String>>(&content) {
                return ids.into_iter().collect();
            }
        }
    }
    HashSet::new()
}

pub fn save_seen_job_ids(job_ids: Vec<String>) {
    let mut existing = get_seen_job_ids();
    existing.extend(job_ids);
    
    let path = history_file_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    let ids_vec: Vec<String> = existing.into_iter().collect();
    if let Ok(json) = serde_json::to_string_pretty(&ids_vec) {
        let _ = fs::write(path, json);
    }
}

pub fn filter_new_jobs(scraped_jobs: Vec<Job>) -> Vec<Job> {
    let seen_ids = get_seen_job_ids();
    scraped_jobs.into_iter().filter(|job| {
        if !job.id.is_empty() && job.id != "Unknown ID" {
            !seen_ids.contains(&job.id)
        } else {
            true // Treat as new if ID is unknown
        }
    }).collect()
}
