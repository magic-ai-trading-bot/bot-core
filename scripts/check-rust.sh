#!/bin/bash
#
# Rust Code Quality Check Script
# Runs formatting, linting, and tests before commits/CI
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Change to rust-core-engine directory
cd "$(dirname "$0")/../rust-core-engine"

echo -e "${YELLOW}🦀 Running Rust code quality checks...${NC}\n"

# 1. Format check
echo -e "${YELLOW}📝 Checking code formatting with rustfmt...${NC}"
if /Users/dungngo97/.asdf/installs/rust/1.86.0/bin/rustfmt --check --edition 2021 src/**/*.rs tests/**/*.rs 2>/dev/null; then
    echo -e "${GREEN}✅ Code formatting is correct${NC}\n"
else
    echo -e "${RED}❌ Code formatting issues found!${NC}"
    echo -e "${YELLOW}💡 Run 'make format-rust' to fix${NC}\n"
    exit 1
fi

# 2. Clippy linting
echo -e "${YELLOW}🔍 Running Clippy linter...${NC}"
if /Users/dungngo97/.asdf/installs/rust/1.86.0/bin/cargo-clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ No linting issues found${NC}\n"
else
    echo -e "${RED}❌ Clippy found issues!${NC}"
    echo -e "${YELLOW}💡 Fix the issues above before committing${NC}\n"
    exit 1
fi

# 3. Cargo check (fast compile check)
echo -e "${YELLOW}🔨 Running cargo check...${NC}"
if /Users/dungngo97/.asdf/installs/rust/1.86.0/bin/cargo check --all-targets; then
    echo -e "${GREEN}✅ Compilation check passed${NC}\n"
else
    echo -e "${RED}❌ Compilation errors found!${NC}\n"
    exit 1
fi

# 4. Run tests
echo -e "${YELLOW}🧪 Running tests...${NC}"
if /Users/dungngo97/.asdf/installs/rust/1.86.0/bin/cargo test --lib --tests -- --test-threads=1; then
    echo -e "${GREEN}✅ All tests passed${NC}\n"
else
    echo -e "${RED}❌ Some tests failed!${NC}\n"
    exit 1
fi

echo -e "${GREEN}✨ All Rust quality checks passed!${NC}"
echo -e "${GREEN}🚀 Safe to commit and push${NC}\n"
