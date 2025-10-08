# Rust Code Quality Checks

This document explains the code quality checks for the Rust core engine and how to use them.

## 🛠️ Available Commands

### Quick Check (Recommended before commit)
```bash
make check-rust
```
Runs all quality checks:
- ✅ Code formatting (rustfmt)
- ✅ Linting (clippy)
- ✅ Compilation check (cargo check)
- ✅ Tests (cargo test)

### Individual Commands

#### Format Code
```bash
make format-rust
```
Automatically formats all Rust code according to `.rustfmt.toml` configuration.

#### Check Formatting (without modifying)
```bash
make format-check-rust
```
Checks if code is properly formatted. Fails if formatting is needed.

#### Run Linter
```bash
make lint-rust
```
Runs Clippy with strict warnings. Configured via `.clippy.toml`.

#### Run Tests
```bash
make test-rust
```
Runs all Rust tests with coverage.

## 🪝 Pre-Commit Hook

A pre-commit hook is automatically installed that runs all quality checks before allowing commits.

### Setup/Re-install Hook
```bash
make pre-commit-setup
```

### Skip Hook (Not Recommended)
```bash
git commit --no-verify
```

## 📋 Configuration Files

### `.rustfmt.toml`
Controls code formatting rules:
- Line width: 100 characters
- Tab spaces: 4
- Edition: 2021
- Import organization
- Comment formatting

### `.clippy.toml`
Controls linting rules:
- Cognitive complexity threshold: 30
- Type complexity threshold: 250
- Max arguments: 8
- Max lines per function: 150

## 🚀 CI/CD Integration

The `.github/workflows/rust-ci.yml` workflow runs on every push/PR:

1. **Code Quality Checks**
   - Formatting check
   - Clippy linting
   - Compilation check

2. **Tests**
   - All unit and integration tests
   - Single-threaded execution for stability

3. **Coverage** (optional)
   - Code coverage with tarpaulin
   - Upload to Codecov

## 💡 Best Practices

### Before Committing
1. Run `make format-rust` to auto-format code
2. Run `make check-rust` to ensure all checks pass
3. Fix any issues reported
4. Commit (hook will run automatically)

### During Development
- Use `cargo check` for quick compilation feedback
- Run `make lint-rust` periodically to catch issues early
- Keep tests passing with `make test-rust`

### Common Issues

#### Formatting Errors
```bash
# Fix automatically
make format-rust
```

#### Clippy Warnings
Read the warning message and fix the code. Common fixes:
- Add `#[allow(clippy::lint_name)]` for false positives
- Refactor code to follow Rust best practices
- Update `.clippy.toml` if threshold needs adjustment

#### Test Failures
```bash
# Run tests to see failures
make test-rust

# Run specific test
cd rust-core-engine && cargo test test_name
```

## 📊 Code Quality Metrics

Current status:
- ✅ **655 tests** (100% passing)
- ✅ **90%+ functional coverage** for core modules
- ✅ **Zero clippy warnings** with strict settings
- ✅ **Consistent formatting** across codebase

## 🔧 Troubleshooting

### Hook Not Running
```bash
# Re-install hook
make pre-commit-setup

# Check hook permissions
ls -la .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Clippy/Rustfmt Not Found
Ensure Rust toolchain is properly installed:
```bash
# Check installation
asdf current rust

# Reinstall if needed
asdf install rust 1.86.0
```

### Slow Pre-Commit Hook
The hook runs all tests. For faster commits during development:
```bash
# Skip tests temporarily (not recommended for final commit)
git commit --no-verify
```

Or comment out the test section in `scripts/check-rust.sh`.

## 📚 Resources

- [Rustfmt Documentation](https://rust-lang.github.io/rustfmt/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
