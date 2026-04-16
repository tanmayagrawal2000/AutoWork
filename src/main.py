from playwright.sync_api import sync_playwright

import config
from auth import authenticate
from screens.jobs_screen import scrape_jobs
from mailer import send_email
import history_manager

import sys

def main():
    try:
        with sync_playwright() as p:
            # Check command line arguments for headless toggle
            is_headless = "--headless" in sys.argv
            if is_headless:
                print("Executing in HEADLESS mode (Invisible Browser)...")
                
            # Launch browser dynamically based on the command line flag
            browser = p.chromium.launch(headless=is_headless)
            page = None
            
            try:
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
            except Exception as inner_e:
                import traceback
                error_msg = traceback.format_exc()
                print(f"\nCRITICAL ERROR ENCOUNTERED:\n{error_msg}")
                
                screenshot_path = None
                
                # Attempt to retrieve a page if it wasn't returned yet (e.g., error in authenticate)
                if not page and browser and len(browser.contexts) > 0 and len(browser.contexts[0].pages) > 0:
                    page = browser.contexts[0].pages[0]
                    
                if page:
                    try:
                        screenshot_path = "error_screenshot.png"
                        page.screenshot(path=screenshot_path)
                        print(f"Screenshot saved to {screenshot_path}")
                        
                        # Check for session expiration / "Something went wrong"
                        if "something went wrong" in page.content().lower():
                            import os
                            state_file = os.path.join(os.path.dirname(__file__), "..", "data", "state.json")
                            if os.path.exists(state_file):
                                print("Detected 'Something went wrong' on the page. Removing stale state.json...")
                                os.remove(state_file)
                                
                            print("Exiting pipeline so the next run triggers a fresh login.")
                            if browser:
                                browser.close()
                            sys.exit(1)
                            
                    except SystemExit:
                        raise
                    except Exception as snap_err:
                        print(f"Failed to capture screenshot or check page state: {snap_err}")
                
                try:
                    from mailer import send_error_email
                    send_error_email(error_msg, config.SENDER_EMAIL, config.RECEIVER_EMAIL, config.EMAIL_PASSWORD, screenshot_path)
                except Exception as mail_err:
                    print(f"Failed to dispatch emergency error email: {mail_err}")
                finally:
                    if browser:
                        browser.close()
    except Exception as outer_e:
        import traceback
        error_msg = traceback.format_exc()
        print(f"\nCRITICAL OUTER ERROR ENCOUNTERED:\n{error_msg}")
        try:
            from mailer import send_error_email
            send_error_email(error_msg, config.SENDER_EMAIL, config.RECEIVER_EMAIL, config.EMAIL_PASSWORD)
        except Exception as mail_err:
            print(f"Failed to dispatch emergency error email: {mail_err}")

if __name__ == "__main__":
    main()
