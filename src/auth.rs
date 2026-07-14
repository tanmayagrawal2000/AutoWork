use headless_chrome::Browser;
use headless_chrome::Tab;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::fs;

pub fn authenticate(
    browser: &Browser,
    workday_url: &str,
    email: &str,
    password: &str,
) -> Result<Arc<Tab>, Box<dyn std::error::Error>> {
    let tab = browser.new_tab()?;
    
    let tab_clone = Arc::clone(&tab);
    thread::spawn(move || {
        loop {
            let debug_enabled = if let Ok(content) = fs::read_to_string("data/state.json") {
                if let Ok(state) = serde_json::from_str::<crate::models::AppState>(&content) {
                    state.mode == "Debug"
                } else {
                    false
                }
            } else {
                false
            };
            
            if debug_enabled {
                if let Ok(png_data) = tab_clone.capture_screenshot(headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png, None, None, true) {
                    let sender_email = std::env::var("SENDER_EMAIL").unwrap_or_default();
                    let receiver_email = std::env::var("RECEIVER_EMAIL").unwrap_or_default();
                    let email_password = std::env::var("EMAIL_PASSWORD").unwrap_or_default();
                    let _ = crate::mailer::send_debug_screenshot_email(png_data, &sender_email, &receiver_email, &email_password);
                }
            }
            thread::sleep(Duration::from_secs(10));
        }
    });

    println!("Navigating to Workday...");
    tab.navigate_to(workday_url)?;
    tab.wait_until_navigated()?;
    
    println!("Authenticating...");
    let start_time = Instant::now();
    let mut duo_code_printed = false;
    
    while start_time.elapsed().as_secs() < 120 {
        let url = tab.get_url();
        
        if url.contains("myworkday.com/northeastern/d/home.htmld") {
            println!("Successfully landed on Workday dashboard.");
            return Ok(tab);
        }
        
        // Handle Email Prompt
        if let Ok(email_elem) = tab.find_element("input[type='email']") {
            let val = tab.evaluate("document.querySelector('input[type=\"email\"]').value", false)
                         .ok().and_then(|r| r.value).and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
            
            if val != email {
                println!("Email prompt detected. Entering email...");
                let _ = email_elem.click();
                let _ = email_elem.type_into(&format!("{}\n", email));
            } else {
                let _ = email_elem.click();
                let _ = email_elem.type_into("\n");
            }
            thread::sleep(Duration::from_secs(4));
            continue;
        }
        
        // Handle Password Prompt
        if let Ok(pass_elem) = tab.find_element("input[type='password']") {
            let val = tab.evaluate("document.querySelector('input[type=\"password\"]').value", false)
                         .ok().and_then(|r| r.value).and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
                         
            if val != password {
                println!("Password prompt detected. Entering password...");
                let _ = pass_elem.click();
                let _ = pass_elem.type_into(&format!("{}\n", password));
            } else {
                let _ = pass_elem.click();
                let _ = pass_elem.type_into("\n");
            }
            thread::sleep(Duration::from_secs(4));
            continue;
        }
        
        // Handle Duo Code Extract
        if url.contains("duosecurity.com") && !duo_code_printed {
            let js = r#"
                Array.from(document.querySelectorAll('div, span, p'))
                    .map(e => e.innerText ? e.innerText.trim() : '')
                    .find(t => /^\d{3}$|^\d{6}$/.test(t))
            "#;
            if let Ok(remote_obj) = tab.evaluate(js, false) {
                if let Some(val) = remote_obj.value {
                    if let Some(code) = val.as_str() {
                        println!("\n!!! ACTION REQUIRED: DUO CODE DETECTED !!!");
                        println!("Please open your Duo Mobile App and enter: [{}]\n", code);
                        
                        let sender_email = std::env::var("SENDER_EMAIL").unwrap_or_default();
                        let receiver_email = std::env::var("RECEIVER_EMAIL").unwrap_or_default();
                        let email_password = std::env::var("EMAIL_PASSWORD").unwrap_or_default();
                        let _ = crate::mailer::send_duo_email(code, &sender_email, &receiver_email, &email_password);
                        
                        duo_code_printed = true;
                    }
                }
            }
        }
        
        // KMSI Check bypass
        if let Ok(_) = tab.find_element("input[name='DontShowAgain']") {
            println!("Stay signed in prompt detected...");
            
            // Try checking the box
            let _ = tab.evaluate(r#"
                let kmsiCheckbox = document.querySelector("input[name='DontShowAgain']");
                if (kmsiCheckbox && !kmsiCheckbox.checked) {
                    kmsiCheckbox.click();
                }
            "#, false);
            
            // Wait briefly as requested before clicking Yes
            thread::sleep(Duration::from_millis(1500));
            
            if let Ok(yes_btn) = tab.find_element("input#idSIButton9") {
                let _ = yes_btn.click();
            }
            thread::sleep(Duration::from_secs(4));
            continue;
        }
        
        // Trust Device Bypass (Duo Prompt)
        if let Ok(elem) = tab.find_element("button.c--primary") {
            if let Ok(text) = elem.get_inner_text() {
                if text.contains("Yes, this is my device") {
                    let _ = elem.click();
                }
            }
        }
        
        // Remember Device Bypass (Duo React Prompt)
        let _ = tab.evaluate(r#"
            let cb = document.querySelector('input[type="checkbox"]');
            if (cb && !cb.checked) {
                cb.click();
            }
            let submitBtn = document.querySelector('button[data-testid="btn-trust"]');
            if (submitBtn && !submitBtn.disabled) {
                submitBtn.click();
            } else {
                let skipBtn = document.querySelector('button[data-testid="btn-skip"]');
                if (skipBtn) skipBtn.click();
            }
        "#, false);
        
        thread::sleep(Duration::from_secs(1));
    }
    
    Ok(tab)
}
