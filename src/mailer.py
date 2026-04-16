import smtplib
from email.message import EmailMessage

def send_email(job_titles, sender_email, receiver_email, email_password):
    print("Sending email...")
    msg = EmailMessage()
    msg['Subject'] = f'AutoWork: Found {len(job_titles)} Jobs'
    msg['From'] = f"autowork <{sender_email}>"
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

def send_duo_email(code, sender_email, receiver_email, email_password):
    print("Emailing Duo verification code...")
    msg = EmailMessage()
    msg['Subject'] = f'URGENT: Duo Code {code}'
    msg['From'] = f"autowork <{sender_email}>"
    msg['To'] = receiver_email

    # Plain text fallback
    text_body = f"Your Workday automation requires Duo authentication.\n\nPlease open your Duo Mobile App and enter: {code}\n\nYou have less than 90 seconds before the login times out."
    
    # Rich HTML formatted version
    html_body = f"""
    <html>
      <body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color: #333; line-height: 1.6; padding: 20px; background-color: #f7f7f7;">
        <div style="max-width: 380px; margin: 0 auto; border: 1px solid #e1e4e8; border-radius: 12px; background-color: #ffffff; padding: 35px 25px; text-align: center; box-shadow: 0 4px 6px rgba(0,0,0,0.05);">
            <h2 style="color: #d93025; margin-top: 0; font-size: 22px;">🚨 Auth Required</h2>
            <p style="font-size: 15px; color: #555; margin-bottom: 25px;">Your AutoWork tracker requires Duo mobile verification to continue logging in.</p>
            
            <div style="margin: 0; padding: 20px; background-color: #f8f9fa; border-radius: 8px; border: 1px solid #e9ecef;">
                <p style="margin: 0; font-size: 13px; text-transform: uppercase; color: #6c757d; letter-spacing: 1.5px; font-weight: bold;">Verification Code</p>
                <h1 style="margin: 10px 0 0 0; font-size: 54px; letter-spacing: 6px; color: #212529;">{code}</h1>
            </div>
            
            <p style="font-size: 13px; color: #777; margin-top: 25px; margin-bottom: 0;">⏳ Please enter this in your Duo app immediately. The browser will time out.</p>
        </div>
      </body>
    </html>
    """
    
    msg.set_content(text_body)
    msg.add_alternative(html_body, subtype='html')

    try:
        with smtplib.SMTP_SSL('smtp.gmail.com', 465) as smtp:
            smtp.login(sender_email, email_password)
            smtp.send_message(msg)
        print("Duo email sent successfully!")
    except Exception as e:
        print(f"Failed to send Duo email: {e}")

def send_error_email(error_message, sender_email, receiver_email, email_password, screenshot_path=None):
    print("Emailing error notification...")
    msg = EmailMessage()
    msg['Subject'] = '⚠️ AutoWork: Error Encountered'
    msg['From'] = f"autowork <{sender_email}>"
    msg['To'] = receiver_email

    # Plain text fallback
    text_body = f"Your Workday automation encountered an error:\n\n{error_message}\n\nPlease check the server logs."
    
    # Rich HTML formatted version
    html_body = f"""
    <html>
      <body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; color: #333; line-height: 1.6; padding: 20px; background-color: #f7f7f7;">
        <div style="max-width: 500px; margin: 0 auto; border: 1px solid #e1e4e8; border-radius: 12px; background-color: #ffffff; padding: 35px 25px; box-shadow: 0 4px 6px rgba(0,0,0,0.05);">
            <h2 style="color: #d93025; margin-top: 0; font-size: 22px;">⚠️ Automation Error</h2>
            <p style="font-size: 15px; color: #555; margin-bottom: 25px;">Your AutoWork tracker ran into an issue and crashed.</p>
            
            <div style="margin: 0; padding: 15px; background-color: #fff3f3; border-radius: 8px; border: 1px solid #ffcdd2;">
                <p style="margin: 0; font-size: 13px; text-transform: uppercase; color: #d32f2f; letter-spacing: 1.5px; font-weight: bold;">Error Traceback</p>
                <pre style="margin: 10px 0 0 0; font-size: 13px; color: #333; white-space: pre-wrap; word-wrap: break-word;">{error_message}</pre>
            </div>
            
            <p style="font-size: 13px; color: #777; margin-top: 25px; margin-bottom: 0;">Please VPN into your Ubuntu server and check the terminal logs.</p>
        </div>
      </body>
    </html>
    """
    
    msg.set_content(text_body)
    msg.add_alternative(html_body, subtype='html')

    if screenshot_path:
        try:
            with open(screenshot_path, 'rb') as f:
                img_data = f.read()
            msg.add_attachment(img_data, maintype='image', subtype='png', filename='error_screenshot.png')
        except Exception as e:
            print(f"Could not attach screenshot: {e}")

    try:
        with smtplib.SMTP_SSL('smtp.gmail.com', 465) as smtp:
            smtp.login(sender_email, email_password)
            smtp.send_message(msg)
        print("Error email sent successfully!")
    except Exception as e:
        print(f"Failed to send Error email: {e}")
