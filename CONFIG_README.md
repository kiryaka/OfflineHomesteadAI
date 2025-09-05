# Configuration System: Dev/Prod Split

This document describes the environment-based configuration system that separates development and production settings to prevent regressions and optimize for different use cases.

## 🏗️ Architecture

### Configuration Files
- **`config.dev.toml`** - Development configuration (fast iteration)
- **`config.prod.toml`** - Production configuration (100GB corpus optimized)
- **`config.toml`** - Default/fallback configuration

### Environment Detection
The system automatically detects the environment using:
1. `RUST_ENV` environment variable
2. Defaults to `dev` if not set
3. Supports `dev`, `development`, `prod`, `production`

## 🔧 Development Configuration

**Optimized for fast iteration and testing:**

```toml
[lancedb]
num_partitions = 64         # ~390 vectors per partition (25K vectors)
num_sub_vectors = 96        # 1536/96=16 sub-vectors (SIMD optimized)

[lancedb_search]
nprobes = 4                 # 6% of partitions for good recall
refine_factor = 10          # 10x over-retrieval (vs 40x in prod)
default_limit = 3           # Smaller result set for dev
max_limit = 20              # Smaller max for dev

[dev]
enable_debug_logging = true
fast_indexing = true
skip_expensive_operations = true
test_data_size = 1000
```

**Key Dev Optimizations:**
- **96x fewer partitions** (64 vs 6144) for faster indexing
- **75x fewer search probes** (4 vs 300) for faster queries
- **4x lower refine factor** (10 vs 40) for faster re-ranking
- **Smaller result limits** for faster testing
- **Debug logging enabled** for development insights

## 🏭 Production Configuration

**Optimized for 100GB corpus with maximum performance:**

```toml
[lancedb]
num_partitions = 6144       # ~1,000-4,000 vectors per partition for 10-50M vectors
num_sub_vectors = 96        # 1536/96=16 sub-vectors (SIMD optimized)

[lancedb_search]
nprobes = 300               # 5% of partitions for good recall
refine_factor = 40          # 40x over-retrieval for comprehensive re-ranking
default_limit = 10          # Final results after re-ranking
max_limit = 100             # Maximum results allowed

[prod]
enable_debug_logging = false
fast_indexing = false
skip_expensive_operations = false
monitoring_enabled = true
performance_profiling = true
```

**Key Prod Optimizations:**
- **Large partition count** for optimal vector distribution
- **High search coverage** for maximum recall
- **Comprehensive re-ranking** for accuracy
- **Production monitoring** and profiling
- **Optimized for 25M+ vectors**

## 🧪 Testing Framework

### Automated Test Suite
```bash
# Run all configuration tests
cargo run --release --bin config_test_runner all

# Test specific environments
cargo run --release --bin config_test_runner dev
cargo run --release --bin config_test_runner prod

# Test specific aspects
cargo run --release --bin config_test_runner performance
cargo run --release --bin config_test_runner memory
```

### Test Coverage
- **Environment Validation**: Ensures dev/prod configs meet requirements
- **Parameter Scaling**: Validates dev < prod for performance parameters
- **Performance Regression**: Prevents performance degradation
- **Memory Regression**: Ensures memory usage scales properly
- **Configuration Loading**: Tests environment-based loading

### Validation Rules

**Development Validation:**
- `num_partitions <= 1000` (fast iteration)
- `nprobes <= 20` (fast testing)
- `refine_factor <= 20` (fast testing)
- `default_limit <= 5` (fast testing)
- `max_limit <= 50` (fast testing)

**Production Validation:**
- `num_partitions >= 1000` (production scale)
- `nprobes >= 50` (production recall)
- `refine_factor >= 20` (production accuracy)
- `default_limit >= 5` (production results)
- `max_limit >= 50` (production results)

## 🚀 Development Workflow

### Quick Commands
```bash
# Show current environment and config
./dev_workflow.sh

# Compare dev vs prod configurations
./dev_workflow.sh compare

# Run all tests and validations
./dev_workflow.sh all

# Test specific environment
./dev_workflow.sh dev
./dev_workflow.sh prod
```

### Environment Switching
```bash
# Use development configuration
RUST_ENV=dev cargo run --release --bin lancedb_production_example

# Use production configuration
RUST_ENV=prod cargo run --release --bin lancedb_production_example

# Use default configuration (falls back to dev)
cargo run --release --bin lancedb_production_example
```

## 📊 Performance Comparison

| Parameter | Development | Production | Ratio |
|-----------|-------------|------------|-------|
| **Partitions** | 64 | 6,144 | 96x |
| **Search Probes** | 4 | 300 | 75x |
| **Refine Factor** | 10x | 40x | 4x |
| **Default Limit** | 3 | 10 | 3.3x |
| **Max Limit** | 20 | 100 | 5x |
| **Search Complexity** | 40 | 12,000 | 300x |

## 🔍 Configuration Loading

### Code Usage
```rust
use config::Config;

// Load based on RUST_ENV or default to dev
let config = Config::load()?;

// Load specific environment
let dev_config = Config::load_dev()?;
let prod_config = Config::load_prod()?;

// Load with explicit environment
let config = Config::load_for_env(Some("prod"))?;
```

### Environment Variables
```bash
# Set environment
export RUST_ENV=prod

# Or use inline
RUST_ENV=dev cargo run --bin my_app
```

## 🛡️ Regression Prevention

### Automated Checks
1. **Configuration Validation**: Ensures parameters meet environment requirements
2. **Performance Regression**: Validates dev < prod for performance parameters
3. **Memory Regression**: Ensures memory usage scales properly
4. **Parameter Scaling**: Validates logical relationships between dev and prod

### Manual Checks
1. **Run test suite** before any configuration changes
2. **Compare configurations** using `./dev_workflow.sh compare`
3. **Test both environments** before deployment
4. **Validate performance** with regression tests

## 🎯 Best Practices

### Development
- Use dev config for all development work
- Run tests frequently during development
- Keep dev parameters small for fast iteration
- Enable debug logging for troubleshooting

### Production
- Use prod config for production deployment
- Validate configuration before deployment
- Monitor performance and memory usage
- Keep prod parameters optimized for scale

### Configuration Changes
1. **Update both dev and prod** configs when making changes
2. **Run full test suite** after changes
3. **Validate parameter relationships** between environments
4. **Test in both environments** before deployment

## 🔧 Troubleshooting

### Common Issues
1. **Config not loading**: Check file exists and is valid TOML
2. **Validation errors**: Check parameter values against environment requirements
3. **Performance issues**: Verify environment-specific optimizations
4. **Test failures**: Run individual test components to isolate issues

### Debug Commands
```bash
# Test configuration loading
cargo run --bin config_test_runner dev
cargo run --bin config_test_runner prod

# Show configuration details
./dev_workflow.sh compare

# Validate specific environment
./dev_workflow.sh validate
```

This configuration system ensures that development remains fast and efficient while production maintains optimal performance for large-scale deployments.
