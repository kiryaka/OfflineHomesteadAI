#!/bin/bash

# Development Workflow Script
# Ensures proper dev/prod configuration and prevents regressions

set -e

echo "🚀 Development Workflow"
echo "======================"
echo

# Function to run tests
run_tests() {
    echo "🧪 Running Configuration Tests..."
    cargo test --test config_integration_tests
    cargo test --test config_test_runner
    echo "✅ Tests passed!"
    echo
}

# Function to show current environment
show_env() {
    echo "🌍 Current Environment:"
    echo "  RUST_ENV: ${RUST_ENV:-dev (default)}"
    echo "  Config file: $(if [ "$RUST_ENV" = "prod" ]; then echo "config.prod.toml"; else echo "config.dev.toml"; fi)"
    echo
}

# Function to show dev vs prod comparison
show_comparison() {
    echo "📊 Dev vs Prod Configuration Comparison:"
    echo "========================================"
    echo
    echo "🔧 Development Configuration:"
    RUST_ENV=dev cargo run --release --bin lancedb_production_example | grep -E "(partitions|probes|refine_factor|limit)"
    echo
    echo "🏭 Production Configuration:"
    RUST_ENV=prod cargo run --release --bin lancedb_production_example | grep -E "(partitions|probes|refine_factor|limit)"
    echo
}

# Function to validate configuration
validate_config() {
    echo "🔍 Validating Configuration..."
    
    # Test dev config
    echo "  Testing dev config..."
    cargo test --test config_integration_tests test_dev_config_loading
    
    # Test prod config
    echo "  Testing prod config..."
    cargo test --test config_integration_tests test_prod_config_loading
    
    echo "✅ Configuration validation passed!"
    echo
}

# Function to run performance tests
run_performance_tests() {
    echo "⚡ Running Performance Tests..."
    cargo test --test config_test_runner test_performance_regression
    echo "✅ Performance tests passed!"
    echo
}

# Function to run memory tests
run_memory_tests() {
    echo "💾 Running Memory Tests..."
    cargo test --test config_test_runner test_memory_regression
    echo "✅ Memory tests passed!"
    echo
}

# Function to show help
show_help() {
    echo "Usage: $0 [command]"
    echo
    echo "Commands:"
    echo "  test        - Run all configuration tests"
    echo "  dev         - Show dev configuration"
    echo "  prod        - Show prod configuration"
    echo "  compare     - Compare dev vs prod configurations"
    echo "  validate    - Validate both dev and prod configurations"
    echo "  performance - Run performance regression tests"
    echo "  memory      - Run memory regression tests"
    echo "  all         - Run all tests and validations"
    echo "  help        - Show this help message"
    echo
    echo "Environment Variables:"
    echo "  RUST_ENV=dev   - Use development configuration"
    echo "  RUST_ENV=prod  - Use production configuration"
    echo
}

# Main command handling
case "${1:-help}" in
    "test")
        run_tests
        ;;
    "dev")
        show_env
        echo "🔧 Development Configuration:"
        RUST_ENV=dev cargo run --release --bin lancedb_production_example
        ;;
    "prod")
        show_env
        echo "🏭 Production Configuration:"
        RUST_ENV=prod cargo run --release --bin lancedb_production_example
        ;;
    "compare")
        show_env
        show_comparison
        ;;
    "validate")
        show_env
        validate_config
        ;;
    "performance")
        run_performance_tests
        ;;
    "memory")
        run_memory_tests
        ;;
    "all")
        show_env
        run_tests
        validate_config
        run_performance_tests
        run_memory_tests
        show_comparison
        echo "🎉 All checks completed successfully!"
        ;;
    "help"|*)
        show_help
        ;;
esac
