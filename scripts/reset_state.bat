@echo off
echo Deleting Playwright session state...

:: Always run from root
cd /d "%~dp0\.."

if exist "data\state.json" (
    del "data\state.json"
    echo state.json successfully deleted. The next automation run will require a fresh login!
) else (
    echo state.json does not exist. No action taken.
)

pause
