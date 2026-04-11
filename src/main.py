from playwright.sync_api import sync_playwright

import config
from auth import authenticate
from screens.jobs_screen import scrape_jobs
from mailer import send_email
import history_manager

import sys

def main():
    with sync_playwright() as p:
        # Check command line arguments for headless toggle
        is_headless = "--headless" in sys.argv
        if is_headless:
            print("Executing in HEADLESS mode (Invisible Browser)...")
            
        # Launch browser dynamically based on the command line flag
        browser = p.chromium.launch(headless=is_headless)
        
        # 1. Authenticate (handles login, Duo, KMSI, and session saving)
        page = authenticate(
            browser, 
            config.WORKDAY_URL, 
            config.WORKDAY_EMAIL, 
            config.WORKDAY_PASSWORD
        )

        # 2. Navigate and Scrape Job Titles
        job_titles = scrape_jobs(page)

        browser.close()
        
        # 3. Handle Results (Delta tracking & Notifications)
        if job_titles:
            # Filter the raw scraped jobs against our local history JSON
            new_jobs = history_manager.filter_new_jobs(job_titles)
            
            if new_jobs:
                print(f"Found {len(new_jobs)} completely NEW jobs since our last run!")
                for job in new_jobs:
                    date_str = job.posting_date.strftime("%b %d, %Y") if job.posting_date else "Unknown Date"
                    print(f"\033[94m - {job.name}\033[0m (ID: {job.id} | Posted: {date_str} | Location: {job.location_city}, {job.location_country})")
                
                # Save these new unique job IDs to our history state forever
                new_ids = [job.id for job in new_jobs if job.id and job.id != "Unknown ID"]
                if new_ids:
                    history_manager.save_seen_job_ids(new_ids)
                
                # Actually send the email containing ONLY the new jobs!
                send_email(new_jobs, config.SENDER_EMAIL, config.RECEIVER_EMAIL, config.EMAIL_PASSWORD)
            else:
                print("No completely new jobs found since the last run. We are all up to date!")
        else:
            print("No job titles were scraped, skipping email.")

if __name__ == "__main__":
    main()
