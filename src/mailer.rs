use lettre::message::{Attachment, MultiPart, SinglePart, header::ContentType};
use lettre::{Message, SmtpTransport, Transport};
use crate::models::Job;

pub fn send_email(job_titles: &[Job], sender_email: &str, receiver_emails: &[String], email_password: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending email...");
    
    let mut text_body = format!("NEW jobs found on Workday ({} total):\n\n", job_titles.len());
    let mut html_body = format!(
        "<html><body style=\"font-family: sans-serif;\"><h2>AutoWork Jobs Report</h2><p>Found <b>{}</b> new jobs.</p><div>",
        job_titles.len()
    );

    for job in job_titles {
        let date_str = match &job.posting_date {
            Some(d) => d.format("%b %d, %Y").to_string(),
            None => "Unknown Date".to_string(),
        };

        let url_text = if job.url.is_empty() {
            "".to_string()
        } else {
            format!("\n   URL: {}", job.url)
        };
        
        let desc_text = if let Some(summary) = &job.ai_summary {
            let mut s = String::from("\n   AI Summary:\n");
            s.push_str(&format!("   - Availability: {}\n", summary.availability));
            s.push_str(&format!("   - Pay Rate: {}\n", summary.pay_rate));
            s.push_str(&format!("   - Responsibilities: {}\n", summary.responsibilities));
            s.push_str(&format!("   - Eligibility: {}\n", summary.eligibility));
            s.push_str(&format!("   - Note: {}\n", summary.note));
            s
        } else if let Some(ref d) = job.description {
            format!("\n   Description:\n   {}\n", d.replace("\n", "\n   "))
        } else {
            "".to_string()
        };

        text_body.push_str(&format!("{}\n   ID: {} | Posted: {}\n   Location: {}, {}{}{}\n\n", 
            job.name, job.id, date_str, job.location_city, job.location_country, url_text, desc_text));
            
        let url_html = if job.url.is_empty() {
            "".to_string()
        } else {
            format!("<br><a href=\"{}\" target=\"_blank\">View Job Posting</a>", job.url)
        };
        
        let desc_html = if let Some(summary) = &job.ai_summary {
            let mut s = String::from("<div style=\"margin-top:12px;color:#333;font-size:14px;background:#f9f9f9;padding:10px;border-radius:5px;\"><strong>✨ AI Summary:</strong><ul style=\"margin-top:8px;margin-bottom:0;padding-left:20px;\">");
            s.push_str(&format!("<li><strong>Availability</strong>: {}</li>", summary.availability));
            s.push_str(&format!("<li><strong>Pay Rate</strong>: {}</li>", summary.pay_rate));
            s.push_str(&format!("<li><strong>Responsibilities</strong>: {}</li>", summary.responsibilities));
            s.push_str(&format!("<li><strong>Eligibility</strong>: {}</li>", summary.eligibility));
            s.push_str(&format!("<li><strong>Note</strong>: {}</li>", summary.note));
            s.push_str("</ul></div>");
            s
        } else if let Some(ref d) = job.description {
            format!("<div style=\"margin-top:12px;color:#555;font-size:14px;\">{}</div>", d.replace("\n", "<br>"))
        } else {
            "".to_string()
        };

        html_body.push_str(&format!(
            "<div style=\"margin-bottom:16px;padding:18px;border:1px solid #e1e4e8;\"><h3>{}</h3><p>ID: {} | Posted: {}<br>Location: {}, {}{}</p>{}</div>",
            job.name, job.id, date_str, job.location_city, job.location_country, url_html, desc_html
        ));
    }
    html_body.push_str("</div></body></html>");

    let to_addresses = receiver_emails.join(", ");

    let email = Message::builder()
        .from(sender_email.parse()?)
        .to(to_addresses.parse()?)
        .subject(format!("AutoWork: Found {} Jobs", job_titles.len()))
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(text_body))
                .singlepart(SinglePart::html(html_body))
        )?;

    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            sender_email.to_string(),
            email_password.to_string(),
        ))
        .build();

    mailer.send(&email)?;
    println!("Email sent successfully!");
    Ok(())
}

pub fn send_duo_email(code: &str, sender_email: &str, receiver_email: &str, email_password: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Emailing Duo verification code...");
    
    let text_body = format!("Your Workday automation requires Duo authentication. Code: {}", code);
    let html_body = format!("<html><body><h2>🚨 Auth Required</h2><h1>{}</h1><p>Enter in your Duo app immediately.</p></body></html>", code);

    let email = Message::builder()
        .from(sender_email.parse()?)
        .to(receiver_email.parse()?)
        .subject(format!("URGENT: Duo Code {}", code))
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(text_body))
                .singlepart(SinglePart::html(html_body))
        )?;

    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            sender_email.to_string(),
            email_password.to_string(),
        ))
        .build();

    mailer.send(&email)?;
    println!("Duo email sent successfully!");
    Ok(())
}

pub fn send_error_email(error_message: &str, screenshot: Option<Vec<u8>>, sender_email: &str, receiver_email: &str, email_password: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Emailing error notification...");
    
    let text_body = format!("Error Encountered:\n\n{}", error_message);
    let html_body = format!("<html><body><h2>⚠️ Automation Error</h2><pre>{}</pre></body></html>", error_message);

    let multipart = MultiPart::alternative()
        .singlepart(SinglePart::plain(text_body))
        .singlepart(SinglePart::html(html_body));

    let email_builder = Message::builder()
        .from(sender_email.parse()?)
        .to(receiver_email.parse()?)
        .subject("⚠️ AutoWork: Error Encountered");

    let email = if let Some(png_data) = screenshot {
        let attachment = Attachment::new(String::from("error_screenshot.png"))
            .body(png_data, ContentType::parse("image/png").unwrap());
        email_builder.multipart(MultiPart::mixed().multipart(multipart).singlepart(attachment))?
    } else {
        email_builder.multipart(multipart)?
    };

    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            sender_email.to_string(),
            email_password.to_string(),
        ))
        .build();

    mailer.send(&email)?;
    println!("Error email sent successfully!");
    Ok(())
}

pub fn send_debug_screenshot_email(png_data: Vec<u8>, sender_email: &str, receiver_email: &str, email_password: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Emailing debug screenshot...");
    
    let text_body = "Debug Screenshot Attached.".to_string();
    let html_body = "<html><body><h2>🐛 Debug Screenshot</h2><p>Here is the latest screenshot from the headless browser.</p></body></html>".to_string();

    let multipart = MultiPart::alternative()
        .singlepart(SinglePart::plain(text_body))
        .singlepart(SinglePart::html(html_body));

    let attachment = Attachment::new(String::from("debug_screenshot.png"))
        .body(png_data, ContentType::parse("image/png").unwrap());

    let email = Message::builder()
        .from(sender_email.parse()?)
        .to(receiver_email.parse()?)
        .subject("🐛 AutoWork Debug Screenshot")
        .multipart(MultiPart::mixed().multipart(multipart).singlepart(attachment))?;

    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            sender_email.to_string(),
            email_password.to_string(),
        ))
        .build();

    mailer.send(&email)?;
    println!("Debug screenshot emailed successfully!");
    Ok(())
}
