# AutoWork: 🎓 Intelligent Workday Automation (Rust Edition)

AutoWork is a fully autonomous, high-performance web scraper rewritten in Rust. It securely logs into Northeastern Workday, completely bypasses secondary authentication loops (Duo Mobile, Microsoft KMSI), deeply scrapes dynamic job listings, and emails you a beautiful HTML payload containing *only the completely new jobs* since its last run. It will also capture screenshots and notify you of any UI errors!

---

## 🚀 Setup & Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/tanmayagrawal2000/AutoWork.git
   cd AutoWork
   ```

2. **Install Rust:**
   If you don't have Rust installed, download it from [rustup.rs](https://rustup.rs/).

3. **Install Dependencies:**
   Since this project uses Rust's Cargo package manager, dependencies are automatically fetched when you run the project. However, the scraper uses the `headless_chrome` crate, which requires a Chromium-based browser (Google Chrome or Microsoft Edge) to be installed on your system.

---

## ⚙️ Configuration Setup

Since GitHub is public, this repository protects your passwords by purposefully ignoring your configuration file. You must create one locally:

1. Duplicate the `.env.example` template and rename the new file strictly to `.env`.
2. Open `.env` and replace the placeholder variables with your genuine credentials:
   - **Workday Credentials**: Your university email and password.
   - **Gmail Setup**: The script requires a **Gmail App Password** to act as a bot to send you emails. *(Go to your Google Account -> Security -> 2-Step Verification -> App Passwords)*.
   - **Discord Bot** (Optional but Recommended): Add your `DISCORD_TOKEN` and a `DISCORD_CHANNEL_ID` to run the interactive dashboard.
   - **Error Handling**: Set `ERROR_THRESHOLD` to define how many consecutive UI failures must occur before an emergency screenshot email is sent (default is 3).

---

## 🛠 Operation & Execution

### 1. Interactive Discord Bot (Recommended)
You can spawn a 24/7 background Discord Bot that automatically posts a persistent **Control Panel** to your server! From this dashboard, you can click buttons to instantly trigger the scraper headlessly, switch to Debug mode, reset your browser cookies, or pause the automation for hours.

```bash
cargo run --release -- --bot
```

### 2. Manual Terminal Run
If you just want to run the scraper once through the terminal:

```bash
cargo run --release
```
*(Add the `--headless` flag to the command to run the browser invisibly in the background! e.g., `cargo run --release -- --headless`)*

### Resetting State
To force a fresh login (to test the Duo authentication flow), you can either click the **Reset State** button on the Discord Dashboard, or manually delete the `data/browser_profile` folder. This destroys saved cookies.

---

## 📱 The "Duo" Emergency Loop
If the script attempts to login but your standard saved-session cookies have expired over time, the Workday proxy will force a Duo Universal Prompt lock. 

AutoWork is programmed to recognize this intercept natively. It will pause the automation loop, dynamically extract the required Duo passcode straight from the rendered browser DOM, and instantly email you a massive red **"Auth Required"** dashboard. 

You will have exactly 90 seconds to type the passcode into your Duo mobile app, at which point AutoWork will instantly press "Yes, this is my device", detect the unlock, and resume its web scraping without skipping a beat!

## ⚠️ Advanced Error Handling
Workday frequently updates their UI dashboard layouts. If AutoWork gets stuck and cannot find a navigation button, it will increment an error counter. If the failure happens consecutively across multiple runs (configured by `ERROR_THRESHOLD` in your `.env`, defaulting to 3), it will abort the scrape, command `headless_chrome` to capture a full native PNG screenshot of the browser window, and email you an emergency alert with the screenshot attached so you can diagnose the problem immediately! If a run is successful, the error counter safely resets to 0.
