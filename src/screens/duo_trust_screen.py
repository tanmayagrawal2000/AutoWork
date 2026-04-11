def handle_if_present(page):
    # Check for Duo's "Is this your device?" trust prompt
    try:
        trust_btn = page.locator("text=/Yes, this is my device/i").first
        
        if trust_btn.is_visible():
            print("Automating Duo 'Trusted Device' prompt...")
            trust_btn.click(timeout=5000)
            print("Successfully clicked 'Yes, this is my device' on Duo!")
            page.wait_for_timeout(2000)
            return True
    except Exception:
        pass
        
    return False
