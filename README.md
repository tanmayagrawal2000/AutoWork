# AutoWork: 🎓 Intelligent Workday Automation

AutoWork is a fully autonomous, headless-capable web scraper that securely logs into Northeastern Workday, completely bypasses secondary authentication loops (Duo Mobile, Microsoft KMSI), deeply scrapes dynamic job listings, and emails you a beautiful HTML payload containing *only the completely new jobs* since its last run.

---

## 🚀 Setup & Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/tanmayagrawal2000/AutoWork.git
   cd AutoWork
   ```

2. **Initialize a Python Virtual Environment:**
   Run the following to initialize space for Playwright:
   ```bash
   python -m venv .venv
   
   # For Windows:
   .\.venv\Scripts\activate
   
   # For Ubuntu/Linux:
   source .venv/bin/activate
   ```

3. **Install Dependencies:**
   ```bash
   pip install playwright
   playwright install chromium
   ```

---

## ⚙️ Configuration Setup

Since GitHub is public, this repository protects your passwords by purposefully ignoring your configuration file. You must create one locally:

1. Duplicate the `src/config.example.py` template and rename the new file strictly to `src/config.py`.
2. Open `src/config.py` and replace the placeholder variables with your genuine credentials:
   - **Workday Credentials**: Your university email and password.
   - **Gmail Setup**: The script requires a **Gmail App Password** to act as a bot to send you emails. *(Do not use your normal Google profile password. Go to your Google Account -> Security -> 2-Step Verification -> App Passwords to generate a unique 16-character string).*

---

## 🛠 Operation & Execution

You do NOT need to run the Python scripts manually. Simply use the provided launchers located in the `scripts/` directory based on your Operating System:

### For Windows Users (Desktop Testing / Task Scheduler)
* **`scripts\run_scraper.bat`**: The main execution engine. Double-click this to safely activate your environment, launch the browser, and scrape the portal.
* **`scripts\reset_state.bat`**: A hot-reset tool. Double-click this to purposefully delete your saved Playwright cookies if you want to test the raw Duo authentication login flow. 

### For Ubuntu / Linux Users (Cloud Server Automation)
> [!WARNING] 
> To run this in an Ubuntu terminal flawlessly, you **must** open `src/main.py` and change `headless=False` to `headless=True`! Ubuntu servers often do not have desktop graphical interfaces, so Playwright will crash if it tries to spawn a physical browser window!

* **`./scripts/run_scraper.sh`**: The main execution engine. Schedule this via `cron` (e.g. `0 9 * * * /absolute/path/to/AutoWork/scripts/run_scraper.sh`) to run it every morning automatically!
* **`./scripts/reset_state.sh`**: Execute this to instantly destroy any saved session cookies and force a raw login.

*(Don't forget to give your Linux scripts execution permissions before attempting to run them!)*
```bash
chmod +x scripts/run_scraper.sh scripts/reset_state.sh
```

---

## 📱 The "Duo" Emergency Loop
If the script attempts to login but your standard saved-session cookies have expired over time, the Workday proxy will force a Duo Universal Prompt lock. 

AutoWork is programmed to recognize this intercept natively. It will pause the automation loop, dynamically extract the required Duo passcode straight from the rendered browser DOM, and instantly email you a massive red **"Auth Required"** dashboard. 

You will have exactly 90 seconds to type the passcode into your Duo mobile app, at which point AutoWork will instantly press "Yes, this is my device", detect the unlock, and resume its web scraping without skipping a beat!
