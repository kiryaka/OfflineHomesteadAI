#!/bin/bash
# Test runner script for text cleaning validation

echo "Running Text Cleaning Validation Tests..."
echo "========================================"

# Activate virtual environment
source ../../.venv/bin/activate

# Run tests with pytest
echo "Running with pytest..."
python -m pytest test_text_cleaning.py -v -s

echo ""
echo "Running individual test runner..."
python run_text_cleaning_tests.py
