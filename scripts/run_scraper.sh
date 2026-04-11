#!/bin/bash
echo "Starting the Workday Automation Tracker..."

# Always run script from the root directory dynamically
cd "$(dirname "$0")/.."

# Activate the Python Virtual Environment (Linux path format)
source .venv/bin/activate

# Execute the scraper
python3 src/main.py

echo ""
echo "Job search complete!"
