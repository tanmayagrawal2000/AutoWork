def handle_if_present(page):
    # Check for Workday's "Should we remember this device?" prompt
    try:
        has_remember_text = page.locator("text=/should we remember this device/i").count() > 0
        submit_btn = page.locator("button:has-text('Submit'), button:has-text('Yes'), input[value='Submit'], input[value='Yes']").first
        
        if has_remember_text and submit_btn.is_visible():
            print("Automating 'Should we remember this device?' prompt...")
            try:
                # Find and check the 'Remember this device' checkbox
                checkbox = page.locator("input[type='checkbox']").first
                if checkbox.is_visible():
                    checkbox.check(timeout=3000)
            except:
                pass
                
            submit_btn.click(timeout=5000)
            print("Successfully clicked Submit on 'Remember this device'!")
            page.wait_for_timeout(2000)
            return True
    except Exception:
        pass
        
    return False
