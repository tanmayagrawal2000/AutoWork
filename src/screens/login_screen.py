def check_needs_login(page):
    try:
        page.wait_for_selector("input[type='email']", timeout=5000)
        return True
    except:
        return False

def perform_login(page, email, password):
    print("Logging in...")
    page.fill("input[type='email']", email)
    
    # Click 'Next' after email
    page.click("input[type='submit'], button:has-text('Next')") 
    
    # Small wait to ensure password field is ready
    page.wait_for_selector("input[type='password']", timeout=10000)
    page.fill("input[type='password']", password)
    page.click("input[type='submit'], button[type='submit'], button:has-text('Sign In')")
