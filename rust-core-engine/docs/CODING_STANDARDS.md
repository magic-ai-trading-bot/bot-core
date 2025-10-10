# Rust Coding Standards & Quality Checks

## 🎯 Overview

This document outlines the coding standards and quality check processes for the Rust core engine to ensure consistent, maintainable, and high-quality code before pushing to CI/CD.

## ✅ Quick Pre-Push Checklist

Before pushing code, run:
```bash
make check-rust
```

This single command will:
1. ✅ Check code formatting
2. ✅ Run linter (Clippy)
3. ✅ Verify compilation
4. ✅ Run all 655 tests

**Expected output:** All checks should pass with green checkmarks ✅

## 🛠️ Available Commands

### Main Command (Recommended)
```bash
make check-rust          # Run ALL quality checks before committing
```

### Individual Commands
```bash
make format-rust         # Auto-format code with rustfmt
make format-check-rust   # Check formatting without modifying files
make lint-rust          # Run Clippy linter
make test-rust          # Run tests with coverage
```

## 🪝 Pre-Commit Hook

**Automatically installed** - runs on every `git commit`

### Features:
- Detects Rust file changes (*.rs)
- Runs full quality checks automatically
- Prevents commits if checks fail
- Can be bypassed with `--no-verify` (not recommended)

### Setup/Re-install:
```bash
make pre-commit-setup
```

### Location:
- Hook: `.git/hooks/pre-commit`
- Script: `scripts/check-rust.sh`

## 📝 Code Formatting (.rustfmt.toml)

### Standards:
- **Max line width:** 100 characters
- **Tab spaces:** 4
- **Edition:** 2021
- **Import organization:** Enabled
- **Auto-format:** Available

### Commands:
```bash
# Auto-format all Rust files
make format-rust

# Check if formatting is needed (CI-safe)
make format-check-rust
```

## 🔍 Linting (.clippy.toml)

### Rules:
- **Warnings as errors:** Enabled (`-D warnings`)
- **Cognitive complexity:** Max 30
- **Function arguments:** Max 8
- **Function lines:** Max 150
- **All targets:** Enabled (lib + tests)

### Commands:
```bash
# Run Clippy linter
make lint-rust

# Fix auto-fixable issues
cd rust-core-engine && cargo clippy --fix --allow-dirty
```

## 🧪 Testing Standards

### Current Status:
- **Total tests:** 655
- **Pass rate:** 100%
- **Coverage:** 40.96% (tarpaulin), 90%+ functional

### Test Execution:
```bash
# Run all tests
make test-rust

# Run tests single-threaded (stable)
cd rust-core-engine && cargo test -- --test-threads=1

# Run specific test
cd rust-core-engine && cargo test test_name

# Run with coverage
cd rust-core-engine && cargo tarpaulin --timeout 180
```

## 🚫 Common Issues & Solutions

### Issue 1: Formatting Errors
```bash
Error: rustfmt check failed
Solution: make format-rust
```

### Issue 2: Clippy Warnings
```bash
Error: Clippy found issues
Solution: Read warning and fix code, or add #[allow(clippy::lint_name)] if false positive
```

### Issue 3: Test Failures
```bash
Error: Tests failed
Solution: Fix failing tests, ensure all 655 tests pass
```

### Issue 4: Unused Imports (Warnings)
```bash
Warning: unused imports
Solution: Remove unused imports or run `cargo fix --allow-dirty`
```

## 📊 CI/CD Integration

### GitHub Actions Workflow
Location: `rust-core-engine/.github/workflows/rust-ci.yml`

### CI Checks (runs on push/PR):
1. **Code Quality**
   - Formatting check (`rustfmt --check`)
   - Clippy linting (`clippy -- -D warnings`)
   - Compilation check (`cargo check`)

2. **Tests**
   - All unit tests
   - All integration tests
   - Single-threaded execution

3. **Coverage** (optional)
   - Tarpaulin coverage report
   - Upload to Codecov

### Preventing CI Failures:
**Always run `make check-rust` before pushing!**

## 💡 Best Practices

### Before Committing:
1. ✅ Run `make format-rust`
2. ✅ Run `make check-rust`
3. ✅ Fix any issues
4. ✅ Commit (hook runs automatically)

### During Development:
- Use `cargo check` for quick feedback
- Run `make lint-rust` periodically
- Keep tests passing continuously
- Format code frequently

### Code Review:
- All Clippy warnings must be addressed
- Tests must pass
- Code must be formatted
- No TODO comments in production code

## 📁 Configuration Files

```
rust-core-engine/
├── .rustfmt.toml          # Formatting rules
├── .clippy.toml           # Linting rules
├── .github/
│   └── workflows/
│       └── rust-ci.yml    # CI/CD workflow
└── scripts/
    └── check-rust.sh      # Quality check script
```

## 🚀 Quick Start

### First Time Setup:
```bash
# Setup pre-commit hook
make pre-commit-setup

# Run initial checks
make check-rust
```

### Regular Workflow:
```bash
# 1. Make code changes
vim rust-core-engine/src/...

# 2. Format code
make format-rust

# 3. Run checks
make check-rust

# 4. Commit (hook runs automatically)
git add .
git commit -m "feat: add new feature"

# 5. Push (CI will run same checks)
git push
```

## 🔧 Tools & Versions

- **Rust:** 1.86.0 (via asdf)
- **Rustfmt:** 1.8.0-stable
- **Clippy:** 0.1.88
- **Tarpaulin:** Latest (for coverage)

## 📚 Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rustfmt Documentation](https://rust-lang.github.io/rustfmt/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

## ✨ Quality Metrics

Current codebase quality:
- ✅ **655 tests** (100% passing)
- ✅ **Zero Clippy warnings** (strict mode)
- ✅ **Consistent formatting** (rustfmt)
- ✅ **90%+ coverage** (core modules)
- ✅ **Production ready** (all checks passing)

---

**Remember:** Quality checks are automated - just run `make check-rust` before pushing! 🚀
