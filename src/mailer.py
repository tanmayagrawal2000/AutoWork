import smtplib
from email.message import EmailMessage

def send_email(job_titles, sender_email, receiver_email, email_password):
    print("Sending email...")
    msg = EmailMessage()
    msg['Subject'] = f'AutoWork: Found {len(job_titles)} Jobs'
    msg['From'] = sender_email
    msg['To'] = receiver_email

    # Plain text fallback
    text_body = f"NEW jobs found on Workday ({len(job_titles)} total):\n\n"
    
    # Rich HTML formatted version
    html_body = f"""
    <html>
      <body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color: #333; line-height: 1.6;">
        <h2 style="color: #004d99; border-bottom: 2px solid #f0f0f0; padding-bottom: 10px;"> AutoWork Jobs Report</h2>
        <p style="font-size: 15px;">We found <b>{len(job_titles)}</b> completely new job postings on Workday.</p>
        <div style="margin-top: 20px;">
    """
    
    for idx, job in enumerate(job_titles, 1):
        date_str = job.posting_date.strftime("%b %d, %Y") if job.posting_date else "Unknown Date"
        
        # Build plain text variant
        text_body += f"{job.name}\n   ID: {job.id} | Posted: {date_str}\n   Location: {job.location_city}, {job.location_country}\n\n"
        
        # Build HTML card variant
        html_body += f"""
          <div style="margin-bottom: 16px; padding: 18px; border: 1px solid #e1e4e8; border-radius: 8px; background-color: #fcfcfc;">
              <h3 style="margin: 0 0 8px 0; color: #111; font-size: 17px;">{job.name}</h3>
              <div style="font-size: 14px; color: #555;">
                  <span style="color: #777;">ID:</span> {job.id} &nbsp;|&nbsp; 
                  <span style="color: #777;">Posted:</span> {date_str}<br>
                  <span style="color: #777;">Location:</span> {job.location_city}, {job.location_country}
              </div>
          </div>
        """
        
    html_body += """
        </div>
      </body>
    </html>
    """
    
    # Package into the EmailMessage
    msg.set_content(text_body)
    msg.add_alternative(html_body, subtype='html')

    # Example uses Gmail's SMTP server. 
    # If using Outlook/Office365, use smtp.office365.com and port 587
    try:
        with smtplib.SMTP_SSL('smtp.gmail.com', 465) as smtp:
            smtp.login(sender_email, email_password)
            smtp.send_message(msg)
        print("Email sent successfully!")
    except Exception as e:
        print(f"Failed to send email: {e}")
