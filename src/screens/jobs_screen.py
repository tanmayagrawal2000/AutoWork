def scrape_jobs(page):
    print("Navigating to jobs...")
    # Stabilize page before clicking
    page.wait_for_timeout(3000)
    
    # 1. Click "Menu" hamburger button
    print("Clicking 'Menu'...")
    page.locator("text=MENU >> visible=true").first.click(timeout=15000, force=True)
    
    # 2. Click "Jobs and Career Hub"
    print("Clicking 'Jobs and Career Hub'...")
    page.locator("text=/Jobs and Career Hub/i >> visible=true").first.click(timeout=15000, force=True)
    
    # 3. Click "Student Employment"
    print("Clicking 'Student Employment'...")
    page.locator("text=/Student Employment/i >> visible=true").first.click(timeout=15000, force=True)
    
    # 4. Click "NU Find Student Jobs"
    print("Clicking 'NU Find Student Jobs'...")
    page.locator("text=/NU Find Student Jobs/i >> visible=true").first.click(timeout=15000, force=True)
    
    print("Reached the jobs portal! (Waiting for listings to load...)")
    
    # Wait for the job listings to appear
    try:
        # Wait for "Posting Date" text to guarantee the actual job cards have rendered into the DOM
        page.locator("text=/Posting Date/i").first.wait_for(state="visible", timeout=20000)
    except Exception:
        print("Note: Timed out waiting for job results to load.")

    # Small extra buffer to let Workday's React components finish injecting
    page.wait_for_timeout(2000)

    from models.job import Job
    
    print("Scraping job titles and metadata...")
    job_titles = []
    
    try:
        # First, try to grab the rich GWT container cards directly to get full metadata 
        containers = page.locator("[data-automation-id='compositeContainer']").all()
        
        if containers:
            for container in containers:
                try:
                    # Get Title
                    title = container.locator("[data-automation-id='compositeHeader']").first.inner_text().strip()
                    
                    # Get Subheader Metadata string
                    subheader_elem = container.locator("[data-automation-id='compositeSubHeaderOne']").first
                    subheader = subheader_elem.inner_text().strip() if subheader_elem.is_visible() else ""
                    
                    parts = [p.strip() for p in subheader.split("|")]
                    
                    # 1. ID
                    job_id = parts[0] if len(parts) > 0 else "Unknown ID"
                    
                    # 2. Posting Date
                    posting_date_str = ""
                    for p in parts:
                        if "Posting Date:" in p:
                            posting_date_str = p.replace("Posting Date:", "").strip()
                            break
                    
                    from datetime import datetime
                    posting_date = None
                    if posting_date_str:
                        try:
                            # Workday formats it as MM/DD/YYYY
                            posting_date = datetime.strptime(posting_date_str, "%m/%d/%Y").date()
                        except ValueError:
                            pass
                            
                    # 3. Location
                    # Based on standard Workday layout, location usually ends up in the 2nd index after posting date segment 
                    location_raw = parts[2] if len(parts) > 2 else "Unknown Location"
                    city = location_raw
                    country = ""
                    if "," in location_raw:
                        loc_parts = location_raw.split(",", 1)
                        city = loc_parts[0].strip()
                        country = loc_parts[1].strip()
                        
                    if title and title.lower() not in ["home", "jobs and career hub"]:
                        job_titles.append(Job(
                            name=title,
                            id=job_id,
                            posting_date=posting_date,
                            location_city=city,
                            location_country=country
                        ))
                except Exception as container_err:
                    print(f"Skipping malformed job card: {container_err}")
                    
        else:
            # Fallback legacy approach (Just raw titles if no rich cards exist)
            selectors = [
                "a[data-automation-id='jobTitle']",
                "a[data-automation-id='job-title']",
                "h3 a",
                "h3",
                "[role='heading'] a"
            ]
            for selector in selectors:
                elements = page.locator(selector).all()
                if elements:
                    for element in elements:
                        title = element.inner_text().strip()
                        if title and title.lower() not in ["home", "jobs and career hub"]:
                            job_titles.append(Job(name=title, id="N/A", posting_date="N/A", location_city="N/A", location_country="N/A"))
                    if job_titles:
                        break
                        
    except Exception as e:
        print(f"Error while parsing titles: {e}")
            
    print(f"Found {len(job_titles)} jobs.")
    return job_titles
