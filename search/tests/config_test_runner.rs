// Integration test runner for configuration system
// Provides comprehensive testing and validation

use anyhow::Result;
use std::env;
use tantivy_demo::tests::common::ConfigTestUtils;

/// Configuration test runner
/// Runs comprehensive tests to ensure no regressions between dev and prod
fn main() -> Result<()> {
    println!("🚀 Configuration Test Runner");
    println!("=============================");
    println!();

    // Check if we should run specific tests
    let args: Vec<String> = env::args().collect();
    let test_type = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    match test_type {
        "all" => {
            ConfigTestUtils::run_all_tests()?;
            ConfigTestUtils::test_performance_regression()?;
            ConfigTestUtils::test_memory_regression()?;
        }
        "basic" => {
            ConfigTestUtils::run_all_tests()?;
        }
        "performance" => {
            ConfigTestUtils::test_performance_regression()?;
        }
        "memory" => {
            ConfigTestUtils::test_memory_regression()?;
        }
        "dev" => {
            println!("🔧 Testing Dev Configuration Only...");
            ConfigTestUtils::test_dev_config()?;
        }
        "prod" => {
            println!("🏭 Testing Prod Configuration Only...");
            ConfigTestUtils::test_prod_config()?;
        }
        _ => {
            println!("Usage: cargo test --test config_test_runner [test_type]");
            println!("Test types: all, basic, performance, memory, dev, prod");
            return Ok(());
        }
    }

    println!();
    println!("🎉 All tests completed successfully!");
    println!();
    println!("💡 Tips:");
    println!("  • Run 'cargo test --test config_test_runner dev' to test dev config only");
    println!("  • Run 'cargo test --test config_test_runner prod' to test prod config only");
    println!("  • Run 'cargo test --test config_test_runner performance' for performance tests");
    println!("  • Set RUST_ENV=dev or RUST_ENV=prod to test environment loading");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_suite() {
        ConfigTestUtils::run_all_tests().unwrap();
    }

    #[test]
    fn test_performance_regression() {
        ConfigTestUtils::test_performance_regression().unwrap();
    }

    #[test]
    fn test_memory_regression() {
        ConfigTestUtils::test_memory_regression().unwrap();
    }
}
