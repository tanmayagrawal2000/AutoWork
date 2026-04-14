import os
import time

from screens import login_screen, kmsi_screen, remember_device_screen, duo_trust_screen

def authenticate(browser, workday_url, workday_email, workday_password):
    state_file = os.path.join(os.path.dirname(__file__), "..", "data", "state.json")
    
    if os.path.exists(state_file):
        print("Loading previous session to bypass login...")
        context = browser.new_context(storage_state=state_file)
    else:
        context = browser.new_context()

    page = context.new_page()

    print("Navigating to Workday...")
    page.goto(workday_url)

    if login_screen.check_needs_login(page):
        login_screen.perform_login(page, workday_email, workday_password)

        print("Waiting for Duo Authentication... Please approve on your device (you have 90 seconds).")
        
        duo_code_printed = False
        start_time = time.time()
        while time.time() - start_time < 90:
            current_url = page.url.lower()
            
            if "myworkday.com/northeastern/d/home.htmld" in current_url:
                break
                
            # Intercept Duo Security and extract the verification code
            if "duosecurity.com" in current_url and not duo_code_printed:
                try:
                    # Use a powerful JS snippet to find any isolated 3 or 6 digit pin on the screen, skipping hidden logic
                    code = page.evaluate("Array.from(document.querySelectorAll('div, span, p')).map(e => e.innerText ? e.innerText.trim() : '').find(t => /^\\d{3}$|^\\d{6}$/.test(t))")
                    if code:
                        print(f"\n\033[41;97m !!! ACTION REQUIRED: DUO CODE DETECTED !!! \033[0m")
                        print(f"\033[1;93mPlease open your Duo Mobile App and enter: [ {code} ]\033[0m\n")
                        
                        import config
                        from mailer import send_duo_email
                        send_duo_email(code, config.SENDER_EMAIL, config.RECEIVER_EMAIL, config.EMAIL_PASSWORD)
                        
                        duo_code_printed = True
                except Exception:
                    pass
                
            # Attempt to handle any intermediary screens if they pop up
            handled_kmsi = kmsi_screen.handle_if_present(page)
            handled_remember = remember_device_screen.handle_if_present(page)
            handled_duo_trust = duo_trust_screen.handle_if_present(page)
                
            page.wait_for_timeout(1000)
        
        # Wait for the login to complete and dashboard to load
        try:
            page.wait_for_url("**myworkday.com/northeastern/d/home.htmld**", timeout=45000)
            print("Successfully landed on Workday dashboard.")
        except Exception as e:
            print(f"Warning while waiting for Workday dashboard: {e}")
        
        # Save the session state ONLY AFTER we fully reach the Workday page
        print("Saving session...")
        os.makedirs(os.path.dirname(state_file), exist_ok=True)
        context.storage_state(path=state_file)
    else:
        print("Already logged in from saved session!")
        
    return page
