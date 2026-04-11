def handle_if_present(page):
    # Check for the "stay signed in" text anywhere on the page (case-insensitive)
    try:
        has_text = page.locator("text=/stay signed in/i").count() > 0
        
        # Look for the standard 'Yes' input / button
        btn = page.locator("#idSIButton9, input[value='Yes'], button:has-text('Yes')").first
        
        if has_text and btn.is_visible():
            print("Automating 'Stay signed in?' prompt...")
            btn.click(timeout=5000)
            print("Successfully clicked 'Yes'!")
            # Give it a couple seconds to process the click and redirect
            page.wait_for_timeout(2000)
            return True
    except Exception:
        pass
        
    return False
