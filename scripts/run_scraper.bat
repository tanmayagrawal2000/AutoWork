@echo off
echo Starting the Workday Automation Tracker...

:: Always run script from the root directory dynamically
cd /d "%~dp0\.."

:: Activate the Python Virtual Environment
call .\.venv\Scripts\activate.bat

:: Execute the scraper and pass any command-line arguments (like --headless)
python src\main.py %*

echo.
echo Job search complete! 
pause
