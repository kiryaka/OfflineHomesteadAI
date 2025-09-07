#!/bin/bash
# Activate virtual environment for the universe project

cd /Users/kirillbutin/work/universe
source .venv/bin/activate
echo "✅ Virtual environment activated"
echo "📁 Working directory: $(pwd)"
echo "🐍 Python: $(which python)"
echo ""
echo "Ready to run ETL pipeline:"
echo "  cd etl && python load.py --env dev"
