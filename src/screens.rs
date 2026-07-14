use headless_chrome::Tab;
use std::sync::Arc;
use crate::models::Job;
use std::thread;
use std::time::Duration;
use chrono::NaiveDate;

pub fn scrape_jobs(tab: Arc<Tab>) -> Result<Vec<Job>, Box<dyn std::error::Error>> {
    println!("Waiting for Workday dashboard to load...");
    
    // 1. Click Menu (Using the exact data-automation-id from the screenshot)
    let mut menu_clicked = false;
    for _ in 0..15 {
        if let Ok(elem) = tab.find_element("[data-automation-id='globalNavButton']") {
            let _ = elem.click();
            menu_clicked = true;
            break;
        }
        thread::sleep(Duration::from_millis(2000));
    }
    
    let send_err = |element_name: &str, tab_ref: &Arc<Tab>| -> Box<dyn std::error::Error> {
        let msg = format!("Failed to find or click UI element: '{}'. Workday layout might have changed or page load timed out.", element_name);
        println!("CRITICAL ERROR: {}", msg);
        let sender = std::env::var("SENDER_EMAIL").unwrap_or_default();
        let receiver = std::env::var("RECEIVER_EMAIL").unwrap_or_default();
        let pass = std::env::var("EMAIL_PASSWORD").unwrap_or_default();
        
        // Capture screenshot of the error state to attach to the email
        let screenshot = tab_ref.capture_screenshot(headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png, None, None, true).ok();
        
        let _ = crate::mailer::send_error_email(&msg, screenshot, &sender, &receiver, &pass);
        msg.into()
    };

    if !menu_clicked {
        return Err(send_err("globalNavButton (Main Menu)", &tab));
    }
    thread::sleep(Duration::from_millis(3000));
    
    // 2. Click Jobs and Career Hub
    // Helper closure to click by aria-label or XPath text content (much simpler, no Shadow DOM piercing needed)
    let click_text = |text: &str, tab: &Arc<Tab>| {
        for _ in 0..15 {
            let res = tab.evaluate(&format!(r#"
                (() => {{
                    // 1. Try native aria-label (matching the user's inspector screenshot)
                    let a = document.querySelector(`[aria-label="{}"]`);
                    if (a) {{ a.click(); return true; }}
                    
                    // 2. Try XPath text matching
                    let xpath = `//*[contains(text(), "{}")]`;
                    let el = document.evaluate(xpath, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                    if (el) {{ el.click(); return true; }}
                    
                    return false;
                }})()
            "#, text, text), false);
            
            if let Ok(obj) = res {
                if obj.value.unwrap_or(serde_json::Value::Bool(false)).as_bool().unwrap_or(false) {
                    return true;
                }
            }
            thread::sleep(Duration::from_millis(1000));
        }
        false
    };

    println!("Clicking 'Jobs and Career Hub'...");
    if !click_text("Jobs and Career Hub", &tab) {
        return Err(send_err("Jobs and Career Hub", &tab));
    }
    thread::sleep(Duration::from_millis(2000));
    
    println!("Clicking 'Student Employment'...");
    if !click_text("Student Employment", &tab) {
        return Err(send_err("Student Employment", &tab));
    }
    thread::sleep(Duration::from_millis(2000));

    println!("Clicking 'NU Find Student Jobs'...");
    if !click_text("NU Find Student Jobs", &tab) {
        return Err(send_err("NU Find Student Jobs", &tab));
    }
    
    println!("Reached the jobs portal! (Waiting for listings to load...)");
    thread::sleep(Duration::from_millis(5000)); 
    
    println!("Scraping job titles and metadata...");
    let mut job_titles: Vec<Job> = Vec::new();
    
    let js_script = r#"
        (() => {
            let jobs = [];
            let containers = document.querySelectorAll("[data-automation-id='compositeContainer']");
            if (containers.length > 0) {
                containers.forEach(container => {
                    let titleElem = container.querySelector("[data-automation-id='compositeHeader']");
                    let title = titleElem ? titleElem.innerText.trim() : "";
                    
                    let subheaderElem = container.querySelector("[data-automation-id='compositeSubHeaderOne']");
                    let subheader = subheaderElem ? subheaderElem.innerText.trim() : "";
                    
                    let parts = subheader.split("|").map(p => p.trim());
                    let id = parts.length > 0 ? parts[0] : "Unknown ID";
                    
                    let postingDateStr = "";
                    for (let p of parts) {
                        if (p.includes("Posting Date:")) {
                            postingDateStr = p.replace("Posting Date:", "").trim();
                            break;
                        }
                    }
                    
                    let locationRaw = parts.length > 2 ? parts[2] : "Unknown Location";
                    let city = locationRaw;
                    let country = "";
                    if (locationRaw.includes(",")) {
                        let locParts = locationRaw.split(",");
                        city = locParts[0].trim();
                        country = locParts[1].trim();
                    }
                    
                    if (title && title.toLowerCase() !== "home" && title.toLowerCase() !== "jobs and career hub") {
                        jobs.push({
                            name: title,
                            id: id,
                            posting_date_str: postingDateStr,
                            location_city: city,
                            location_country: country
                        });
                    }
                });
            }
            return JSON.stringify(jobs);
        })()
    "#;
    
    let remote_object = tab.evaluate(js_script, false)?;
    if let Some(val) = remote_object.value {
        if let Some(json_str) = val.as_str() {
            #[derive(serde::Deserialize)]
            struct RawJob {
                name: String,
                id: String,
                posting_date_str: String,
                location_city: String,
                location_country: String,
            }
            
            if let Ok(raw_jobs) = serde_json::from_str::<Vec<RawJob>>(json_str) {
                for rj in raw_jobs {
                    let mut p_date = None;
                    if !rj.posting_date_str.is_empty() {
                        if let Ok(parsed) = NaiveDate::parse_from_str(&rj.posting_date_str, "%m/%d/%Y") {
                            p_date = Some(parsed);
                        }
                    }
                    job_titles.push(Job {
                        name: rj.name,
                        id: rj.id,
                        posting_date: p_date,
                        location_city: rj.location_city,
                        location_country: rj.location_country,
                    });
                }
            }
        }
    }
    
    println!("Found {} jobs.", job_titles.len());
    Ok(job_titles)
}
