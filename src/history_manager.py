import os
import json

HISTORY_FILE = os.path.join(os.path.dirname(__file__), "..", "data", "seen_jobs.json")

def get_seen_job_ids():
    """Reads the JSON file and returns a set of all seen Job IDs."""
    if os.path.exists(HISTORY_FILE):
        try:
            with open(HISTORY_FILE, 'r') as f:
                return set(json.load(f))
        except Exception as e:
            print(f"Warning: Could not read history file: {e}")
            return set()
    return set()

def save_seen_job_ids(job_ids):
    """Takes a list of new Job IDs, merges them with existing ones, and saves them."""
    existing = get_seen_job_ids()
    existing.update(job_ids)
    
    try:
        with open(HISTORY_FILE, 'w') as f:
            json.dump(list(existing), f, indent=4)
    except Exception as e:
        print(f"Error saving history file: {e}")

def filter_new_jobs(scraped_jobs):
    """Compares scraped Job objects against history, and returns only those not seen before."""
    seen_ids = get_seen_job_ids()
    new_jobs = []
    
    for job in scraped_jobs:
        # Ignore empty ids just in case of weird scrapes
        if job.id and job.id != "Unknown ID":
            if job.id not in seen_ids:
                new_jobs.append(job)
        else:
            # If for some reason we couldn't parse the ID, we'll treat it as new
            new_jobs.append(job)
            
    return new_jobs
