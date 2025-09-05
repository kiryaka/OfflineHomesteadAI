# Contributing to Tantivy + LanceDB Hybrid Search

Thank you for your interest in contributing to this project! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites
- Rust 1.70 or later
- Git
- Basic understanding of search systems and vector databases

### Development Setup
1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/tantivy-lancedb-hybrid-search.git`
3. Navigate to the project: `cd tantivy-lancedb-hybrid-search`
4. Build the project: `cargo build`
5. Run tests: `./dev_workflow.sh all`

## 🧪 Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --lib                    # Unit tests
cargo test --test config_integration_tests  # Integration tests
cargo test --test config_test_runner        # Test runner

# Run development workflow
./dev_workflow.sh all
```

### Test Requirements
- All tests must pass before submitting PR
- New features must include tests
- Configuration changes must include validation tests
- Performance changes must include regression tests

## 📝 Development Guidelines

### Code Style
- Follow Rust conventions and idioms
- Use `cargo fmt` to format code
- Use `cargo clippy` to check for issues
- Write clear, self-documenting code
- Add comments for complex logic

### Configuration Changes
- Update both `config.dev.toml` and `config.prod.toml`
- Ensure dev parameters < prod parameters for performance
- Add validation rules if needed
- Update documentation

### Commit Messages
Use conventional commit format:
```
feat: add new search algorithm
fix: resolve memory leak in vector indexing
docs: update README with performance metrics
test: add regression tests for configuration
```

## 🔧 Configuration Development

### Environment-Specific Changes
When modifying configurations:

1. **Update both environments**:
   - `config.dev.toml` - Development optimizations
   - `config.prod.toml` - Production optimizations

2. **Validate relationships**:
   - Dev parameters should be smaller/faster than prod
   - Maintain logical relationships between parameters
   - Ensure validation rules are appropriate

3. **Test thoroughly**:
   - Run `./dev_workflow.sh all`
   - Test both environments
   - Verify performance characteristics

### Adding New Parameters
1. Add to `Config` struct in `src/config.rs`
2. Add to both config files
3. Add validation rules if needed
4. Update tests
5. Update documentation

## 🐛 Bug Reports

### Before Reporting
1. Check existing issues
2. Ensure you're using the latest version
3. Try to reproduce the issue
4. Check logs and error messages

### Bug Report Template
```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
A clear description of what you expected to happen.

**Environment**
- OS: [e.g. Ubuntu 20.04]
- Rust version: [e.g. 1.70.0]
- Configuration: [dev/prod]

**Additional context**
Add any other context about the problem here.
```

## ✨ Feature Requests

### Before Requesting
1. Check existing issues and discussions
2. Consider if it fits the project's scope
3. Think about implementation complexity
4. Consider impact on existing functionality

### Feature Request Template
```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
A clear description of any alternative solutions.

**Additional context**
Add any other context or screenshots about the feature request.
```

## 🔄 Pull Request Process

### Before Submitting
1. Ensure all tests pass: `./dev_workflow.sh all`
2. Format code: `cargo fmt`
3. Check with clippy: `cargo clippy`
4. Update documentation if needed
5. Add tests for new features

### PR Template
```markdown
## Description
Brief description of changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Performance improvement

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] Performance tests pass (if applicable)

## Configuration Changes
- [ ] Updated dev config
- [ ] Updated prod config
- [ ] Added validation rules
- [ ] Updated documentation

## Checklist
- [ ] Code follows project style
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] No breaking changes (or documented)
```

## 📚 Documentation

### Code Documentation
- Use rustdoc comments for public APIs
- Include examples in documentation
- Document configuration parameters
- Explain complex algorithms

### User Documentation
- Update README.md for user-facing changes
- Update CONFIG_README.md for configuration changes
- Add examples for new features
- Keep performance metrics current

## 🏗️ Architecture Guidelines

### Adding New Components
1. Follow existing patterns
2. Add to appropriate module
3. Include comprehensive tests
4. Update configuration if needed
5. Document public APIs

### Performance Considerations
- Consider impact on both dev and prod
- Add performance tests for critical paths
- Document performance characteristics
- Consider memory usage

## 🤝 Community Guidelines

### Code of Conduct
- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the golden rule

### Communication
- Use clear, concise language
- Provide context for issues
- Be patient with questions
- Share knowledge and experience

## 📞 Getting Help

- **Issues**: [GitHub Issues](https://github.com/yourusername/tantivy-lancedb-hybrid-search/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/tantivy-lancedb-hybrid-search/discussions)
- **Documentation**: [Project Wiki](https://github.com/yourusername/tantivy-lancedb-hybrid-search/wiki)

## 🎉 Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Project documentation

Thank you for contributing to this project! 🚀
