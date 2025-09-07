#!/bin/bash
# Run ETL pipeline on test data

echo "🧪 Running ETL pipeline on test data..."
echo ""

# Activate virtual environment
cd /Users/kirillbutin/work/universe
source .venv/bin/activate

# Run ETL pipeline with test environment
cd etl
python load.py --env test

echo ""
echo "✅ Test ETL pipeline completed!"
echo "📁 Check test_data/txt/ for processed files"
