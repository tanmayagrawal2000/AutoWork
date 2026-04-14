def check_needs_login(page):
    try:
        # If the Microsoft page asks for either email OR just the password
        page.wait_for_selector("input[type='email'], input[type='password']", timeout=5000)
        return True
    except:
        return False

def perform_login(page, email, password):
    email_field = page.locator("input[type='email']")
    try:
        # Check if the email field is actually visible (Microsoft hasn't remembered us)
        email_field.wait_for(state="visible", timeout=3000)
        print("Entering email...")
        email_field.fill(email)
        page.click("input[type='submit'], button:has-text('Next')") 
    except:
        print("Bypassing email entry (Microsoft remembered us)...")
        
    # Small wait to ensure password field is ready
    print("Entering password...")
    page.wait_for_selector("input[type='password']", timeout=10000)
    page.fill("input[type='password']", password)
    page.click("input[type='submit'], button[type='submit'], button:has-text('Sign In')")
