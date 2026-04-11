#!/bin/bash
echo "Deleting Playwright session state..."

# Always run from root
cd "$(dirname "$0")/.."

if [ -f "data/state.json" ]; then
    rm data/state.json
    echo "state.json successfully deleted. The next automation run will require a fresh login!"
else
    echo "state.json does not exist. No action taken."
fi
