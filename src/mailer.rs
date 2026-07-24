use lettre::message::{Attachment, MultiPart, SinglePart, header::ContentType};
use lettre::{Message, SmtpTransport, Transport};
use crate::models::Job;

pub fn send_email(job_titles: &[Job], sender_email: &str, receiver_emails: &[String], email_password: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending email...");
    
    let mut text_body = format!("NEW jobs found on Workday ({} total):\n\n", job_titles.len());
    let mut html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        @media screen and (max-width: 600px) {{
            .container {{ padding: 12px !important; }}
            .header {{ padding: 16px 20px !important; }}
            .header h2 {{ font-size: 20px !important; }}
            .header p {{ font-size: 13px !important; }}
            .content-area {{ padding: 20px !important; }}
            .job-card {{ padding: 16px !important; margin-bottom: 24px !important; }}
            .job-title {{ font-size: 19px !important; margin-bottom: 10px !important; }}
            .pill {{ font-size: 12px !important; padding: 4px 8px !important; margin-right: 6px !important; margin-bottom: 6px !important; }}
            .btn {{ font-size: 13px !important; padding: 8px 16px !important; margin-bottom: 16px !important; }}
            .ai-box {{ padding: 16px !important; }}
            .ai-title {{ font-size: 15px !important; margin-bottom: 12px !important; }}
            .ai-list {{ font-size: 14px !important; }}
            .ai-li {{ margin-bottom: 8px !important; }}
        }}
    </style>
</head>
<body class="container" style="font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; background-color: #f4f5f7; color: #1f2937; margin: 0; padding: 20px;">
    <div style="max-width: 650px; margin: 0 auto; background: #ffffff; border-radius: 8px; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);">
        <div class="header" style="background-color: #11325b; border-bottom: 6px solid #306ba8; padding: 24px;">
            <h2 style="color: #ffffff; margin: 0; font-size: 24px; font-weight: 700;">AutoWork Jobs Report</h2>
            <p style="color: #cbd5e1; margin: 8px 0 0 0; font-size: 14px;">Found {} new job{} matching your criteria.</p>
        </div>
        <div class="content-area" style="padding: 32px;">"#,
        job_titles.len(), if job_titles.len() == 1 { "" } else { "s" }
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
            format!("<a href=\"{}\" class=\"btn\" target=\"_blank\" style=\"display: inline-block; background-color: #3182ce; color: #ffffff; text-decoration: none; padding: 10px 20px; border-radius: 6px; font-size: 14px; font-weight: 600; margin-bottom: 20px;\">View Job Posting</a>", job.url)
        };
        
        let desc_html = if let Some(summary) = &job.ai_summary {
            let mut s = String::from("<div class=\"ai-box\" style=\"background-color: #fffaf0; border-left: 4px solid #d69e2e; padding: 20px; border-radius: 4px;\">");
            s.push_str("<h4 class=\"ai-title\" style=\"margin: 0 0 16px 0; color: #b7791f; font-size: 16px; font-weight: 700;\">✨ AI Summary</h4>");
            s.push_str("<ul class=\"ai-list\" style=\"margin: 0; padding-left: 20px; color: #4a5568; font-size: 15px; line-height: 1.6;\">");
            s.push_str(&format!("<li class=\"ai-li\" style=\"margin-bottom: 12px;\"><strong style=\"color: #2d3748;\">Availability:</strong> {}</li>", summary.availability));
            s.push_str(&format!("<li class=\"ai-li\" style=\"margin-bottom: 12px;\"><strong style=\"color: #2d3748;\">Pay Rate:</strong> {}</li>", summary.pay_rate));
            s.push_str(&format!("<li class=\"ai-li\" style=\"margin-bottom: 12px;\"><strong style=\"color: #2d3748;\">Responsibilities:</strong> {}</li>", summary.responsibilities));
            s.push_str(&format!("<li class=\"ai-li\" style=\"margin-bottom: 12px;\"><strong style=\"color: #2d3748;\">Eligibility:</strong> {}</li>", summary.eligibility));
            s.push_str(&format!("<li class=\"ai-li\"><strong style=\"color: #2d3748;\">Note:</strong> {}</li>", summary.note));
            s.push_str("</ul></div>");
            s
        } else if let Some(ref d) = job.description {
            format!("<div style=\"padding: 16px; background-color: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; font-size: 14px; color: #4a5568; line-height: 1.6;\">{}</div>", d.replace("\n", "<br>"))
        } else {
            "".to_string()
        };

        html_body.push_str(&format!(
            r#"<div class="job-card" style="margin-bottom: 32px; padding: 24px; border: 1px solid #e2e8f0; border-radius: 8px;">
                <h3 class="job-title" style="margin: 0 0 12px 0; color: #2d3748; font-size: 22px; font-weight: 700;">{}</h3>
                <div style="margin-bottom: 20px;">
                    <span class="pill" style="display: inline-block; background-color: #f7fafc; color: #718096; font-size: 13px; padding: 6px 12px; border-radius: 6px; border: 1px solid #edf2f7; margin-right: 8px; margin-bottom: 8px;">📍 {}, {}</span>
                    <span class="pill" style="display: inline-block; background-color: #f7fafc; color: #718096; font-size: 13px; padding: 6px 12px; border-radius: 6px; border: 1px solid #edf2f7; margin-right: 8px; margin-bottom: 8px;">📅 Posted: {}</span>
                    <span class="pill" style="display: inline-block; background-color: #f7fafc; color: #718096; font-size: 13px; padding: 6px 12px; border-radius: 6px; border: 1px solid #edf2f7; margin-bottom: 8px;">🆔 {}</span>
                </div>
                {}
                {}
            </div>"#,
            job.name, job.location_city, job.location_country, date_str, job.id, url_html, desc_html
        ));
    }
    html_body.push_str("</div>\n<div style=\"background-color: #f9fafb; padding: 16px; text-align: center; color: #9ca3af; font-size: 12px;\">Sent via AutoWork &bull; Generated Automatically</div>\n</div>\n</body>\n</html>");

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
