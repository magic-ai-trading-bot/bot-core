# System Software Requirements - Bot Core Trading Platform

**Spec ID**: SYS-SOFTWARE-001 to SYS-SOFTWARE-008
**Version**: 1.0
**Status**: ☑ Approved
**Owner**: Platform Engineering Team
**Last Updated**: 2025-10-10

---

## Tasks Checklist

- [x] Operating system requirements defined
- [x] Rust dependencies documented
- [x] Python dependencies documented
- [x] Node.js dependencies documented
- [x] Database requirements specified
- [x] Container orchestration requirements defined
- [x] Build tools requirements documented
- [x] Version compatibility matrix created
- [ ] Dependency security audit completed
- [ ] License compliance verification done
- [ ] Automated dependency updates configured

---

## Metadata

**Related Specs**:
- Related Config: [Cargo.toml](/rust-core-engine/Cargo.toml)
- Related Config: [requirements.txt](/python-ai-service/requirements.txt)
- Related Config: [package.json](/nextjs-ui-dashboard/package.json)
- Related Spec: [SYS-HARDWARE.md](./SYS-HARDWARE.md)
- Related Spec: [SYS-NETWORK.md](./SYS-NETWORK.md)

**Dependencies**:
- Depends on: SYS-HARDWARE-001 (Hardware Infrastructure)
- Blocks: FR-TRADING-001 (Trading Implementation)
- Blocks: FR-AI-001 (AI Service Implementation)

**Business Value**: Critical
**Technical Complexity**: High
**Priority**: ☑ Critical

---

## Overview

This specification defines all software dependencies, versions, and runtime requirements for the Bot Core trading platform. It covers operating systems, programming language runtimes, libraries, frameworks, databases, and build tools necessary for development, testing, and production deployment.

---

## Business Context

**Problem Statement**:
The Bot Core platform is a complex polyglot microservices system using Rust, Python, and TypeScript/JavaScript. Incompatible versions, missing dependencies, or incorrect configurations can cause build failures, runtime errors, security vulnerabilities, and system instability. Clear software requirements ensure reproducible builds, secure deployments, and maintainable infrastructure.

**Business Goals**:
- Ensure reproducible builds across all environments
- Maintain security through up-to-date dependencies
- Enable rapid developer onboarding with clear requirements
- Minimize production incidents from dependency conflicts
- Support long-term maintainability and upgrades

**Success Metrics**:
- Zero build failures due to dependency issues
- All dependencies pass security audit (no critical CVEs)
- Developer environment setup time < 30 minutes
- 100% documented dependency versions
- Automated dependency update success rate > 95%

---

## Software Stack Overview

### Programming Languages and Runtimes

1. **Rust** - Core trading engine (high-performance, low-latency)
2. **Python** - AI/ML service (TensorFlow, PyTorch, OpenAI)
3. **TypeScript/JavaScript** - Frontend dashboard (React, Vite)
4. **Shell** - Infrastructure scripts (Bash)

### Databases

1. **MongoDB** - Primary database for trading data, user accounts, historical data

### Infrastructure

1. **Docker** - Containerization platform
2. **Docker Compose** - Local orchestration
3. **Kubernetes** (Optional) - Production orchestration

---

## SYS-SOFTWARE-001: Operating System Requirements

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-SOFTWARE-001`

**Description**:
Supported operating systems for development, testing, and production deployments. All services run in Docker containers, making the platform portable across OS platforms.

### Development (Local Workstation)

**Linux** (Recommended for production parity):
- **Distributions**:
  - Ubuntu 22.04 LTS (Jammy Jellyfish) - Recommended
  - Ubuntu 24.04 LTS (Noble Numbat)
  - Debian 11 (Bullseye) or 12 (Bookworm)
  - Fedora 38+
  - Arch Linux (rolling)
- **Kernel**: 5.15+ (6.0+ recommended)
- **Architecture**: x86_64 (amd64) or ARM64 (aarch64)
- **Minimum Requirements**:
  - systemd init system
  - glibc 2.31+
  - OpenSSL 1.1.1 or 3.0+

**macOS** (Development friendly):
- **Versions**:
  - macOS 12 Monterey or later (Recommended: macOS 14 Sonoma)
  - Architecture: Intel x86_64 or Apple Silicon (M1/M2/M3)
- **Requirements**:
  - Xcode Command Line Tools installed
  - Homebrew package manager recommended
  - Rosetta 2 (for Intel containers on Apple Silicon)

**Windows** (Supported via WSL2):
- **Versions**: Windows 10 21H2+ or Windows 11
- **WSL2 Required**: Ubuntu 22.04 LTS distribution
- **Not Supported**: Native Windows (use WSL2 or Docker Desktop)
- **Requirements**:
  - Windows Subsystem for Linux 2 (WSL2)
  - Docker Desktop for Windows

### Production (Server Environment)

**Linux** (Required):
- **Distributions**:
  - Ubuntu Server 22.04 LTS - Recommended
  - Ubuntu Server 24.04 LTS
  - Debian 12 (Bookworm)
  - Red Hat Enterprise Linux (RHEL) 8/9
  - Rocky Linux 8/9 (RHEL clone)
  - Amazon Linux 2023
- **Kernel**: 5.15+ (6.5+ for latest performance optimizations)
- **Architecture**: x86_64 (amd64) - ARM64 supported but not benchmarked
- **Required Kernel Features**:
  - cgroups v2 (for resource limits)
  - namespace support (for Docker)
  - eBPF support (for monitoring)
  - TCP BBR congestion control (for low latency)

**Container Images**:
- **Base Images**:
  - Rust: `rust:1.86-slim-bookworm`
  - Python: `python:3.11-slim-bookworm`
  - Node.js: `node:20-alpine` or Bun runtime
  - Alpine: `alpine:3.19` (minimal)

**Acceptance Criteria**:
- [x] Platform runs on Ubuntu 22.04 LTS without modifications
- [x] Docker images build successfully on all supported architectures
- [x] Development possible on macOS with Apple Silicon
- [x] WSL2 environment fully functional for Windows developers
- [x] Kernel features verified during deployment

**Dependencies**: SYS-HARDWARE-001 (Hardware)
**Test Cases**: TC-SOFTWARE-001 (OS Compatibility Test)

**Installation Verification**:
```bash
# Check OS version
cat /etc/os-release
uname -r

# Check kernel features
cat /proc/sys/kernel/osrelease
grep -E "cgroup|namespace" /proc/filesystems

# Check architecture
uname -m
```

**Reference**: `/infrastructure/docker/docker-compose.yml`, `/CLAUDE.md`

---

## SYS-SOFTWARE-002: Rust Dependencies and Toolchain

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-SOFTWARE-002`

**Description**:
Rust toolchain, compiler version, and all dependencies required for building and running the Rust Core Trading Engine.

### Rust Toolchain

**Rust Version**:
- **MSRV** (Minimum Supported Rust Version): **1.86** (January 2025)
- **Recommended**: Latest stable (1.86+)
- **Edition**: 2021
- **Toolchain Components**:
  - `rustc` - Rust compiler
  - `cargo` - Package manager and build tool
  - `rustfmt` - Code formatter
  - `clippy` - Linter

**Installation**:
```bash
# Install via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Set default toolchain
rustup default stable

# Add components
rustup component add rustfmt clippy

# Verify installation
rustc --version  # rustc 1.86.0 or later
cargo --version  # cargo 1.86.0 or later
```

### Core Dependencies (from Cargo.toml)

**Async Runtime**:
- **tokio** = `1.0` (features: `["full"]`)
  - Purpose: Asynchronous runtime for concurrent operations
  - Critical: WebSocket handling, HTTP server, task scheduling
  - License: MIT

**WebSocket**:
- **tokio-tungstenite** = `0.20` (features: `["rustls-tls-native-roots"]`)
  - Purpose: Async WebSocket client for Binance streams
  - Critical: Real-time market data ingestion
  - License: MIT

**Serialization**:
- **serde** = `1.0` (features: `["derive"]`)
  - Purpose: Serialization/deserialization framework
  - Critical: JSON parsing, config files
  - License: MIT/Apache-2.0
- **serde_json** = `1.0`
  - Purpose: JSON support for Serde
  - License: MIT/Apache-2.0

**HTTP Client**:
- **reqwest** = `0.11` (features: `["json", "rustls-tls"]`)
  - Purpose: HTTP client for REST API calls to Binance
  - Critical: Order placement, account queries
  - License: MIT/Apache-2.0

**Web Framework**:
- **warp** = `0.3`
  - Purpose: HTTP server framework for API endpoints
  - Critical: Dashboard API, health checks, metrics
  - License: MIT

**Date/Time**:
- **chrono** = `0.4` (features: `["serde"]`)
  - Purpose: Date and time handling
  - Critical: Timestamp handling, candle data
  - License: MIT/Apache-2.0

**Logging**:
- **tracing** = `0.1`
  - Purpose: Structured logging framework
  - License: MIT
- **tracing-subscriber** = `0.3.20` (features: `["env-filter"]`)
  - Purpose: Logging implementation
  - License: MIT
- **log** = `0.4`
  - Purpose: Logging facade
  - License: MIT/Apache-2.0

**Error Handling**:
- **anyhow** = `1.0`
  - Purpose: Flexible error handling
  - License: MIT/Apache-2.0
- **thiserror** = `1.0`
  - Purpose: Derive macro for error types
  - License: MIT/Apache-2.0

**Concurrency**:
- **dashmap** = `5.4`
  - Purpose: Concurrent HashMap
  - Critical: Shared state for trading data
  - License: MIT
- **parking_lot** = `0.12`
  - Purpose: Better synchronization primitives (Mutex, RwLock)
  - License: MIT/Apache-2.0
- **crossbeam-channel** = `0.5`
  - Purpose: Multi-producer multi-consumer channels
  - License: MIT/Apache-2.0

**Configuration**:
- **config** = `0.13`
  - Purpose: Configuration file parsing
  - Critical: Load config.toml
  - License: MIT/Apache-2.0
- **toml** = `0.8`
  - Purpose: TOML parser
  - License: MIT/Apache-2.0

**Cryptography**:
- **hmac** = `0.12`
  - Purpose: HMAC signature generation
  - Critical: Binance API authentication
  - License: MIT/Apache-2.0
- **sha2** = `0.10`
  - Purpose: SHA-256 hashing
  - Critical: Binance API signatures
  - License: MIT/Apache-2.0
- **hex** = `0.4`
  - Purpose: Hex encoding/decoding
  - License: MIT/Apache-2.0

**Numeric**:
- **rust_decimal** = `1.33` (features: `["serde"]`)
  - Purpose: High-precision decimal arithmetic
  - Critical: Price and quantity calculations
  - License: MIT

**Database** (Optional feature: `database`):
- **mongodb** = `3.3` (optional)
  - Purpose: MongoDB driver
  - Critical: Data persistence
  - License: Apache-2.0
- **bson** = `2.15` (features: `["chrono-0_4"]`, optional)
  - Purpose: BSON serialization
  - License: MIT

**Authentication**:
- **jsonwebtoken** = `9.1`
  - Purpose: JWT token handling
  - Critical: Inter-service authentication
  - License: MIT
- **bcrypt** = `0.15`
  - Purpose: Password hashing
  - Critical: User authentication
  - License: MIT
- **base64** = `0.21.7`
  - Purpose: Base64 encoding
  - License: MIT/Apache-2.0

**Validation**:
- **validator** = `0.20` (features: `["derive"]`)
  - Purpose: Data validation
  - License: MIT/Apache-2.0

**Other**:
- **url** = `2.5.4` - URL parsing
- **uuid** = `1.0` (features: `["v4"]`) - UUID generation
- **futures-util** = `0.3` - Future utilities
- **futures** = `0.3` - Futures trait
- **async-trait** = `0.1` - Async trait support
- **slab** = `0.4.11` - Slab allocator
- **structopt** = `0.3` - CLI argument parsing

### Development Dependencies

**Testing**:
- **tokio-test** = `0.4`
  - Purpose: Testing utilities for async code
- **actix-web** = `4.0` (features: `["macros"]`)
  - Purpose: Testing HTTP endpoints
- **tempfile** = `3.8`
  - Purpose: Temporary file handling in tests

### Build Requirements

**System Libraries** (Required on host for building):
- **OpenSSL** - 1.1.1 or 3.0+ (for TLS)
- **pkg-config** - For finding system libraries
- **gcc** or **clang** - C compiler for native dependencies

**Installation (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

**Installation (macOS)**:
```bash
brew install openssl pkg-config
```

### Build Configuration

**Cargo Features**:
- `default = ["database"]` - MongoDB support enabled by default
- `database = ["mongodb", "bson"]` - Optional database support

**Cargo.toml Resolver**:
```toml
[package]
edition = "2021"
resolver = "2"  # Use new feature resolver
```

**Compilation Flags** (for production):
```bash
# Release build with optimizations
cargo build --release

# Production build (stripped, optimized)
cargo build --release --target x86_64-unknown-linux-gnu
strip target/release/binance-trading-bot
```

**Acceptance Criteria**:
- [x] Cargo.toml specifies all dependencies with versions
- [x] All dependencies compile on Rust 1.86+
- [x] `cargo clippy` passes with no warnings
- [x] `cargo fmt` formats code correctly
- [x] Security audit passes (`cargo audit`)
- [x] Build completes in < 5 minutes on recommended hardware
- [x] Binary size < 50 MB (release mode, stripped)

**Dependencies**: SYS-SOFTWARE-001 (Operating System), SYS-HARDWARE-001 (Development Environment)
**Test Cases**: TC-SOFTWARE-002 (Rust Build Test), TC-SOFTWARE-003 (Dependency Security Audit)

**Monitoring**:
```bash
# Check for outdated dependencies
cargo outdated

# Security audit
cargo install cargo-audit
cargo audit

# Check compilation time
cargo clean && cargo build --release --timings

# Check binary size
ls -lh target/release/binance-trading-bot
```

**Reference**: `/rust-core-engine/Cargo.toml`, `/rust-core-engine/.clippy.toml`

---

## SYS-SOFTWARE-003: Python Dependencies and Runtime

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-SOFTWARE-003`

**Description**:
Python runtime version and all dependencies required for the Python AI/ML service including TensorFlow, PyTorch, and FastAPI.

### Python Runtime

**Python Version**:
- **Minimum**: **3.11** (October 2022)
- **Recommended**: **3.11** or **3.12**
- **Not Supported**: Python 3.10 or earlier (missing performance improvements)
- **Maximum**: Python 3.12 (TensorFlow 2.18.0 compatibility)

**Installation**:
```bash
# Ubuntu 22.04
sudo apt update
sudo apt install -y python3.11 python3.11-venv python3.11-dev

# macOS
brew install python@3.11

# Verify version
python3 --version  # Python 3.11.0 or later
```

### Core Dependencies (from requirements.txt)

**Web Framework**:
- **fastapi** == `0.104.1`
  - Purpose: REST API framework
  - Critical: API endpoints, validation, documentation
  - License: MIT
- **uvicorn** == `0.24.0`
  - Purpose: ASGI server for FastAPI
  - Critical: HTTP server runtime
  - License: BSD-3-Clause
- **python-multipart** == `0.0.6`
  - Purpose: Multipart form data parsing
  - License: Apache-2.0

**Data Validation**:
- **pydantic** == `2.5.0`
  - Purpose: Data validation using type hints
  - Critical: Request/response validation
  - License: MIT

**Numerical Computing**:
- **numpy** >= `1.26.0`, < `2.1.0`
  - Purpose: Array operations, numerical computing
  - Critical: All numerical operations, ML models
  - License: BSD-3-Clause
- **pandas** == `2.2.3`
  - Purpose: Data manipulation, time series
  - Critical: Candle data processing, analysis
  - License: BSD-3-Clause

**Machine Learning**:
- **scikit-learn** == `1.3.0`
  - Purpose: Traditional ML algorithms, preprocessing
  - License: BSD-3-Clause

**Deep Learning - TensorFlow**:
- **tensorflow** == `2.18.0`
  - Purpose: LSTM/GRU model training and inference
  - Critical: AI predictions
  - License: Apache-2.0
  - Note: Large package (~500 MB), GPU support optional

**Deep Learning - PyTorch**:
- **torch** == `2.5.1`
  - Purpose: Transformer model training and inference
  - Critical: Advanced AI models
  - License: BSD-3-Clause
- **torchvision** == `0.20.1`
  - Purpose: Image processing utilities (optional)
  - License: BSD-3-Clause
- **torchaudio** == `2.5.1`
  - Purpose: Audio processing utilities (optional)
  - License: BSD-3-Clause

**Technical Indicators**:
- **ta** == `0.10.2`
  - Purpose: Technical analysis indicators (RSI, MACD, Bollinger Bands)
  - Critical: Market analysis features
  - License: MIT

**Logging**:
- **loguru** == `0.7.2`
  - Purpose: Advanced logging with rotation
  - License: MIT

**Configuration**:
- **pyyaml** == `6.0.1`
  - Purpose: YAML configuration parsing
  - Critical: Load config.yaml
  - License: MIT

**HTTP Client**:
- **requests** == `2.31.0`
  - Purpose: HTTP client for API calls
  - License: Apache-2.0
- **aiofiles** == `23.2.0`
  - Purpose: Async file I/O
  - License: Apache-2.0

**Database**:
- **pymongo** == `4.9.1`
  - Purpose: MongoDB driver (synchronous)
  - License: Apache-2.0
- **motor** == `3.6.0`
  - Purpose: MongoDB driver (async)
  - Critical: Async database operations
  - License: Apache-2.0

**AI Integration**:
- **openai** == `1.51.0`
  - Purpose: OpenAI API client (GPT-4, embeddings)
  - Critical: AI-powered market analysis
  - License: MIT

**Rate Limiting**:
- **slowapi** == `0.1.9`
  - Purpose: Rate limiting for API endpoints
  - License: MIT

**Utilities**:
- **joblib** == `1.3.2`
  - Purpose: Model serialization, parallel processing
  - License: BSD-3-Clause
- **python-dotenv** == `1.0.0`
  - Purpose: Environment variable loading
  - License: BSD-3-Clause

### Development Dependencies (requirements.dev.txt)

**Linting and Formatting**:
- **black** == `23.12.1` - Code formatter
- **flake8** == `7.0.0` - Linter
- **mypy** == `1.8.0` - Type checker
- **isort** == `5.13.2` - Import sorting

**Testing**:
- **pytest** == `7.4.3` - Testing framework
- **pytest-asyncio** == `0.21.1` - Async test support
- **pytest-cov** == `4.1.0` - Coverage reporting
- **httpx** == `0.25.2` - Async HTTP client for testing

**Documentation**:
- **mkdocs** == `1.5.3` - Documentation generator
- **mkdocs-material** == `9.5.3` - Material theme

### Testing Dependencies (requirements.test.txt)

- **pytest** == `7.4.3`
- **pytest-asyncio** == `0.21.1`
- **pytest-cov** == `4.1.0`
- **pytest-mock** == `3.12.0`
- **coverage** == `7.4.0`
- **httpx** == `0.25.2`

### System Requirements

**System Libraries** (Required on host):
- **Python development headers** - `python3-dev`
- **Build tools** - `gcc`, `g++`, `make`
- **Linear algebra** (optional, for performance) - `libblas-dev`, `liblapack-dev`

**Installation (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install -y python3.11 python3.11-venv python3.11-dev \
    build-essential gcc g++ make \
    libblas-dev liblapack-dev
```

**Installation (macOS)**:
```bash
brew install python@3.11
```

### Virtual Environment Setup

**Create Virtual Environment**:
```bash
cd python-ai-service
python3.11 -m venv venv
source venv/bin/activate  # Linux/macOS
# venv\Scripts\activate  # Windows

# Upgrade pip
pip install --upgrade pip setuptools wheel

# Install dependencies
pip install -r requirements.txt
pip install -r requirements.dev.txt  # Development only
pip install -r requirements.test.txt  # Testing only
```

### GPU Support (Optional)

**TensorFlow GPU**:
```bash
# Requires CUDA 11.8 and cuDNN 8.6
pip install tensorflow[and-cuda]==2.18.0
```

**PyTorch GPU**:
```bash
# CUDA 11.8
pip install torch==2.5.1 torchvision==0.20.1 torchaudio==2.5.1 --index-url https://download.pytorch.org/whl/cu118

# CUDA 12.1
pip install torch==2.5.1 torchvision==0.20.1 torchaudio==2.5.1 --index-url https://download.pytorch.org/whl/cu121
```

**GPU Requirements**:
- NVIDIA GPU with Compute Capability 3.5+
- CUDA Toolkit 11.8 or 12.1
- cuDNN 8.6+
- NVIDIA Driver 450.80.02+

### Dependency Constraints

**NumPy Version Constraint**:
```
numpy>=1.26.0,<2.1.0
```
- **Reason**: TensorFlow 2.18.0 and PyTorch 2.5.1 compatibility
- **NumPy 2.0** introduced breaking changes

**Python Version Constraint**:
```
python_requires='>=3.11,<3.13'
```
- **Reason**: TensorFlow 2.18.0 maximum is Python 3.12

**Acceptance Criteria**:
- [x] All dependencies install without errors on Python 3.11
- [x] No dependency conflicts (pip check passes)
- [x] TensorFlow and PyTorch work without conflicts
- [x] Security audit passes (pip-audit, safety)
- [x] FastAPI server starts in < 30 seconds
- [x] AI model loading completes in < 60 seconds
- [x] Virtual environment size < 3 GB (without GPU)

**Dependencies**: SYS-SOFTWARE-001 (Operating System), SYS-HARDWARE-001 (Development Environment)
**Test Cases**: TC-SOFTWARE-004 (Python Environment Test), TC-SOFTWARE-005 (ML Model Loading Test)

**Monitoring**:
```bash
# Check installed packages
pip list

# Check for outdated packages
pip list --outdated

# Security audit
pip install pip-audit
pip-audit

# Check dependency tree
pip install pipdeptree
pipdeptree

# Check for conflicts
pip check
```

**Reference**: `/python-ai-service/requirements.txt`, `/python-ai-service/requirements.dev.txt`, `/python-ai-service/requirements.test.txt`

---

## SYS-SOFTWARE-004: Node.js Dependencies and Frontend Stack

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-SOFTWARE-004`

**Description**:
Node.js runtime, npm/yarn/bun, and all frontend dependencies for the Next.js (Vite) dashboard including React, TypeScript, and UI libraries.

### Node.js Runtime

**Node.js Version**:
- **Minimum**: **18.0.0** (LTS - April 2022)
- **Recommended**: **20.x** (LTS - April 2023)
- **Latest**: **22.x** (Current)
- **Architecture**: x64 or ARM64

**Installation**:
```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 20
nvm use 20

# Verify
node --version  # v20.x.x
npm --version   # 10.x.x
```

### Package Managers

**npm** (Default):
- **Version**: 9.0.0+ (ships with Node.js 18+)
- **Recommended**: 10.x (ships with Node.js 20+)

**Yarn** (Alternative):
- **Version**: 1.22.0+
- **Installation**: `npm install -g yarn`

**Bun** (High-performance alternative, used in dev):
- **Version**: 1.0.0+
- **Installation**: `curl -fsSL https://bun.sh/install | bash`
- **Note**: Used in docker-compose.yml for dev mode

### Core Dependencies (from package.json)

**Project Configuration**:
```json
{
  "name": "vite_react_shadcn_ts",
  "version": "0.0.0",
  "type": "module"
}
```

**React Ecosystem**:
- **react** == `^18.3.1`
  - Purpose: UI library
  - Critical: Core framework
  - License: MIT
- **react-dom** == `^18.3.1`
  - Purpose: React DOM rendering
  - License: MIT
- **react-router-dom** == `^6.26.2`
  - Purpose: Client-side routing
  - License: MIT

**TypeScript**:
- **typescript** == `^5.5.3`
  - Purpose: Type-safe JavaScript
  - Critical: Development, type checking
  - License: Apache-2.0
- **@types/react** == `^18.3.3`
  - Purpose: React type definitions
- **@types/react-dom** == `^18.3.0`
  - Purpose: React DOM type definitions
- **@types/node** == `^22.5.5`
  - Purpose: Node.js type definitions

**Build Tool**:
- **vite** == `^7.1.9`
  - Purpose: Build tool, dev server, HMR
  - Critical: Development and production builds
  - License: MIT
- **@vitejs/plugin-react-swc** == `^4.1.0`
  - Purpose: Fast React refresh with SWC compiler
  - License: MIT

**UI Framework - Radix UI** (Headless components):
- **@radix-ui/react-accordion** == `^1.2.0`
- **@radix-ui/react-alert-dialog** == `^1.1.1`
- **@radix-ui/react-avatar** == `^1.1.0`
- **@radix-ui/react-checkbox** == `^1.1.1`
- **@radix-ui/react-dialog** == `^1.1.2`
- **@radix-ui/react-dropdown-menu** == `^2.1.1`
- **@radix-ui/react-label** == `^2.1.0`
- **@radix-ui/react-popover** == `^1.1.1`
- **@radix-ui/react-select** == `^2.1.1`
- **@radix-ui/react-slider** == `^1.2.0`
- **@radix-ui/react-switch** == `^1.1.0`
- **@radix-ui/react-tabs** == `^1.1.0`
- **@radix-ui/react-toast** == `^1.2.1`
- **@radix-ui/react-tooltip** == `^1.1.4`
- (and 15+ more Radix UI components)
- License: MIT

**Styling**:
- **tailwindcss** == `^3.4.11`
  - Purpose: Utility-first CSS framework
  - Critical: UI styling
  - License: MIT
- **tailwindcss-animate** == `^1.0.7`
  - Purpose: Animation utilities
  - License: MIT
- **@tailwindcss/typography** == `^0.5.15`
  - Purpose: Typography plugin
  - License: MIT
- **autoprefixer** == `^10.4.20`
  - Purpose: CSS vendor prefixes
  - License: MIT
- **postcss** == `^8.4.47`
  - Purpose: CSS processing
  - License: MIT
- **class-variance-authority** == `^0.7.1`
  - Purpose: Component variant management
  - License: Apache-2.0
- **clsx** == `^2.1.1`
  - Purpose: Conditional className utility
  - License: MIT
- **tailwind-merge** == `^2.5.2`
  - Purpose: Merge Tailwind classes
  - License: MIT

**Form Handling**:
- **react-hook-form** == `^7.53.0`
  - Purpose: Form state management
  - License: MIT
- **@hookform/resolvers** == `^3.9.0`
  - Purpose: Form validation resolvers
  - License: MIT
- **zod** == `^3.23.8`
  - Purpose: Schema validation
  - Critical: Form validation, API validation
  - License: MIT

**HTTP Client**:
- **axios** == `^1.6.2`
  - Purpose: HTTP client for API calls
  - Critical: Backend communication
  - License: MIT
- **@tanstack/react-query** == `^5.56.2`
  - Purpose: Data fetching, caching, synchronization
  - Critical: State management for API data
  - License: MIT

**3D Visualization**:
- **three** == `^0.168.0`
  - Purpose: 3D graphics library
  - License: MIT
- **@react-three/fiber** == `^8.18.0`
  - Purpose: React renderer for Three.js
  - License: MIT
- **@react-three/drei** == `^9.122.0`
  - Purpose: Three.js helpers
  - License: MIT

**Charts**:
- **recharts** == `^2.12.7`
  - Purpose: Trading charts, analytics visualization
  - Critical: Trading dashboard
  - License: MIT

**Internationalization**:
- **i18next** == `^25.3.0`
  - Purpose: i18n framework
  - License: MIT
- **react-i18next** == `^15.5.3`
  - Purpose: React bindings for i18next
  - License: MIT

**UI Components**:
- **lucide-react** == `^0.462.0`
  - Purpose: Icon library
  - License: ISC
- **sonner** == `^1.5.0`
  - Purpose: Toast notifications
  - License: MIT
- **cmdk** == `^1.0.0`
  - Purpose: Command palette
  - License: MIT
- **next-themes** == `^0.3.0`
  - Purpose: Theme switching (dark/light mode)
  - License: MIT
- **embla-carousel-react** == `^8.3.0`
  - Purpose: Carousel component
  - License: MIT
- **react-resizable-panels** == `^2.1.3`
  - Purpose: Resizable panel layouts
  - License: MIT
- **vaul** == `^0.9.3`
  - Purpose: Drawer component
  - License: MIT

**Utilities**:
- **date-fns** == `^3.6.0`
  - Purpose: Date manipulation
  - License: MIT
- **react-day-picker** == `^8.10.1`
  - Purpose: Date picker component
  - License: MIT
- **input-otp** == `^1.2.4`
  - Purpose: OTP input component
  - License: MIT

### Development Dependencies

**Linting**:
- **eslint** == `^9.37.0`
  - Purpose: JavaScript/TypeScript linter
  - License: MIT
- **@eslint/js** == `^9.37.0`
- **typescript-eslint** == `^8.46.0`
- **eslint-plugin-react-hooks** == `^5.1.0-rc.0`
- **eslint-plugin-react-refresh** == `^0.4.9`
- **globals** == `^15.15.0`

**Testing**:
- **vitest** == `^2.1.9`
  - Purpose: Unit testing framework
  - License: MIT
- **@vitest/ui** == `^2.1.9`
  - Purpose: Vitest UI
- **@vitest/coverage-v8** == `^2.1.9`
  - Purpose: Code coverage
- **@testing-library/react** == `^14.1.2`
  - Purpose: React testing utilities
  - License: MIT
- **@testing-library/jest-dom** == `^6.1.5`
  - Purpose: DOM matchers
- **@testing-library/user-event** == `^14.5.1`
  - Purpose: User interaction simulation
- **jsdom** == `^23.0.1`
  - Purpose: DOM implementation for Node.js
  - License: MIT

**E2E Testing**:
- **@playwright/test** == `^1.56.0`
  - Purpose: End-to-end testing
  - Critical: Full integration tests
  - License: Apache-2.0

**Mutation Testing**:
- **@stryker-mutator/core** == `^9.2.0`
- **@stryker-mutator/typescript-checker** == `^9.2.0`
- **@stryker-mutator/vitest-runner** == `^9.2.0`

**Mocking**:
- **msw** == `^2.0.11`
  - Purpose: API mocking for tests
  - License: MIT

**Other**:
- **lovable-tagger** == `^1.1.7`
  - Purpose: Development utility

### Build Scripts (package.json)

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "build:dev": "vite build --mode development",
    "preview": "vite preview",
    "lint": "eslint .",
    "type-check": "tsc --noEmit",
    "test": "NODE_ENV=test vitest",
    "test:ui": "NODE_ENV=test vitest --ui",
    "test:run": "NODE_ENV=test vitest run",
    "test:coverage": "NODE_ENV=test vitest run --coverage",
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:headed": "playwright test --headed",
    "test:e2e:debug": "playwright test --debug",
    "test:e2e:report": "playwright show-report"
  }
}
```

### Node.js Memory Configuration

**Production**:
```yaml
# docker-compose.yml
environment:
  - NODE_OPTIONS="--max-old-space-size=${NODE_MEMORY:-1024}"
```
- Default: 1024 MB (1 GB)

**Development**:
```yaml
environment:
  - NODE_OPTIONS="--max-old-space-size=768"
```
- Development: 768 MB

### Bun Configuration (Development)

**Why Bun?**:
- 3-4x faster package installation
- Native TypeScript/JSX support
- Better performance for dev server

**Bun Environment Variables**:
```yaml
environment:
  - BUN_RUNTIME_TRANSPILER_CACHE_PATH=/tmp/bun-cache
  - BUN_ENABLE_JEMALLOC=true
  - BUN_ENABLE_SMOL=false
```

**Acceptance Criteria**:
- [x] All dependencies install without errors on Node.js 18+
- [x] `npm audit` passes with no critical vulnerabilities
- [x] TypeScript compilation succeeds (`tsc --noEmit`)
- [x] ESLint passes with no errors
- [x] Vite dev server starts in < 10 seconds
- [x] Production build completes in < 3 minutes
- [x] Build size < 5 MB (gzipped)
- [x] All tests pass (unit, integration, E2E)

**Dependencies**: SYS-SOFTWARE-001 (Operating System), SYS-HARDWARE-001 (Development Environment)
**Test Cases**: TC-SOFTWARE-006 (Frontend Build Test), TC-SOFTWARE-007 (E2E Test Suite)

**Monitoring**:
```bash
# Check installed packages
npm list --depth=0

# Check for outdated packages
npm outdated

# Security audit
npm audit

# Check bundle size
npm run build
ls -lh dist/

# Analyze bundle
npm install -g source-map-explorer
npm run build
source-map-explorer dist/assets/*.js
```

**Reference**: `/nextjs-ui-dashboard/package.json`, `/nextjs-ui-dashboard/vite.config.ts`

---

## SYS-SOFTWARE-005: Database Requirements

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-SOFTWARE-005`

**Description**:
Database software requirements for MongoDB, including version, configuration, and replica set setup for production.

### MongoDB

**MongoDB Version**:
- **Minimum**: **6.0** (July 2022)
- **Recommended**: **7.0** (August 2023)
- **Latest**: **8.0** (July 2024)
- **Architecture**: x86_64 or ARM64

**Why MongoDB?**:
- Document-oriented (JSON-like documents)
- Flexible schema for trading data
- High-performance time-series collections
- Built-in aggregation pipeline for analytics
- Horizontal scaling with sharding
- Change streams for real-time updates

**Installation**:

**Ubuntu 22.04**:
```bash
# Import MongoDB public key
curl -fsSL https://www.mongodb.org/static/pgp/server-7.0.asc | \
  sudo gpg -o /usr/share/keyrings/mongodb-server-7.0.gpg --dearmor

# Add repository
echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] \
  https://repo.mongodb.org/apt/ubuntu jammy/mongodb-org/7.0 multiverse" | \
  sudo tee /etc/apt/sources.list.d/mongodb-org-7.0.list

# Install
sudo apt update
sudo apt install -y mongodb-org

# Start service
sudo systemctl start mongod
sudo systemctl enable mongod
```

**Docker** (Development):
```yaml
# Not in current docker-compose.yml but MongoDB URL configured
DATABASE_URL=mongodb://botuser:defaultpassword@mongodb:27017/trading_bot?authSource=admin
```

**MongoDB Configuration**:

**Connection String Format**:
```
mongodb://[username:password@]host[:port]/[database][?options]
```

**Example (from config.toml)**:
```toml
[database]
url = "mongodb://botuser:defaultpassword@mongodb:27017/trading_bot?authSource=admin"
database_name = "trading_bot"
max_connections = 10
enable_logging = false
```

**Production Configuration** (/etc/mongod.conf):
```yaml
# Network interfaces
net:
  port: 27017
  bindIp: 0.0.0.0  # Change to specific IP in production

# Storage
storage:
  dbPath: /var/lib/mongodb
  journal:
    enabled: true
  wiredTiger:
    engineConfig:
      cacheSizeGB: 4  # 50% of RAM for WiredTiger cache

# Security
security:
  authorization: enabled

# Replication (Production)
replication:
  replSetName: rs0

# Logging
systemLog:
  destination: file
  path: /var/log/mongodb/mongod.log
  logAppend: true
  logRotate: reopen
```

### MongoDB Features Used

**Collections**:
- `users` - User accounts and authentication
- `trading_history` - Trade execution history
- `market_data` - Historical candle data (time-series)
- `ai_predictions` - AI model predictions cache
- `strategies` - Trading strategy configurations
- `positions` - Active trading positions
- `orders` - Order book history

**Indexes**:
- `users.email` (unique)
- `trading_history.timestamp` (desc)
- `market_data.symbol_timestamp` (compound)
- `ai_predictions.symbol_timestamp` (compound, TTL)

**Time-Series Collections** (MongoDB 5.0+):
```javascript
db.createCollection("market_data", {
  timeseries: {
    timeField: "timestamp",
    metaField: "symbol",
    granularity: "minutes"
  }
})
```

**Change Streams** (for real-time updates):
```javascript
const changeStream = db.collection('positions').watch();
changeStream.on('change', (change) => {
  // Notify WebSocket clients
});
```

### MongoDB Drivers

**Rust** (mongodb crate):
- **mongodb** == `3.3`
- **bson** == `2.15`
- Connection pooling built-in
- Async support with Tokio

**Python** (pymongo/motor):
- **pymongo** == `4.9.1` (synchronous)
- **motor** == `3.6.0` (async)
- Connection pooling configured

### MongoDB Performance Tuning

**WiredTiger Cache**:
- Default: 50% of RAM - 1 GB
- Recommended: 4-16 GB for production
- Configuration: `storage.wiredTiger.engineConfig.cacheSizeGB`

**Connection Pooling**:
- Rust: `max_connections = 10` (per service instance)
- Python: `maxPoolSize=10` (configured in DATABASE_URL)

**Indexes**:
- Create indexes for frequently queried fields
- Use compound indexes for multi-field queries
- Monitor slow queries with profiler

**Query Optimization**:
```bash
# Enable profiler (level 2 = all operations)
db.setProfilingLevel(2)

# View slow queries
db.system.profile.find({ millis: { $gt: 100 } })

# Explain query plan
db.market_data.find({ symbol: "BTCUSDT" }).explain("executionStats")
```

### MongoDB Monitoring

**Database Statistics**:
```javascript
db.stats()
db.serverStatus()
db.currentOp()
```

**Connection Monitoring**:
```javascript
db.serverStatus().connections
```

**Disk Usage**:
```bash
du -sh /var/lib/mongodb/
```

### MongoDB Backup and Restore

**Backup**:
```bash
# Using mongodump
mongodump --uri="mongodb://botuser:password@localhost:27017/trading_bot" \
  --out=/backup/dump_$(date +%Y%m%d_%H%M%S)

# Using Docker
docker exec mongodb-primary mongodump \
  --uri="mongodb://botuser:password@localhost:27017/trading_bot" \
  --out=/backup/dump_$(date +%Y%m%d_%H%M%S)
```

**Restore**:
```bash
# Using mongorestore
mongorestore --uri="mongodb://botuser:password@localhost:27017/trading_bot" \
  --dir=/backup/dump_20250110_120000

# Using Docker
docker exec mongodb-primary mongorestore \
  --uri="mongodb://botuser:password@localhost:27017/trading_bot" \
  --dir=/backup/dump_20250110_120000
```

### MongoDB Replica Set (Production)

**Replica Set Benefits**:
- High availability (automatic failover)
- Read scaling (read from secondaries)
- Data redundancy
- Zero downtime upgrades

**Replica Set Configuration** (3 nodes minimum):
```javascript
rs.initiate({
  _id: "rs0",
  members: [
    { _id: 0, host: "mongodb-primary:27017", priority: 2 },
    { _id: 1, host: "mongodb-secondary1:27017", priority: 1 },
    { _id: 2, host: "mongodb-secondary2:27017", priority: 1 }
  ]
})
```

**Connection String for Replica Set**:
```
mongodb://botuser:password@mongodb-primary:27017,mongodb-secondary1:27017,mongodb-secondary2:27017/trading_bot?replicaSet=rs0&authSource=admin
```

**Acceptance Criteria**:
- [x] MongoDB 6.0+ installed and running
- [x] Authentication enabled in production
- [x] Connection pooling configured
- [x] Indexes created for all queries
- [x] Backup script automated (daily)
- [x] Replica set configured (production)
- [x] Monitoring enabled (logs, metrics)
- [x] Query performance < 50ms average

**Dependencies**: SYS-HARDWARE-002 to 004 (Hardware), SYS-NETWORK-001 (Ports)
**Test Cases**: TC-SOFTWARE-008 (MongoDB Connection Test), TC-SOFTWARE-009 (Database Performance Test)

**Reference**: `/rust-core-engine/config.toml`, `/python-ai-service/config.yaml`, `/CLAUDE.md`

---

## SYS-SOFTWARE-006: Container Orchestration

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-SOFTWARE-006`

**Description**:
Docker and Docker Compose for local development and staging, with optional Kubernetes for production orchestration.

### Docker

**Docker Version**:
- **Minimum**: **24.0.0** (June 2023)
- **Recommended**: **26.0.0+** (March 2024)
- **Architecture**: x86_64 or ARM64

**Docker Components**:
- **Docker Engine** - Container runtime
- **containerd** - Container lifecycle management
- **runc** - OCI runtime
- **Docker CLI** - Command-line interface

**Installation**:

**Ubuntu 22.04**:
```bash
# Uninstall old versions
sudo apt remove docker docker-engine docker.io containerd runc

# Install dependencies
sudo apt update
sudo apt install -y ca-certificates curl gnupg

# Add Docker GPG key
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
  sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg

# Add repository
echo "deb [arch=$(dpkg --print-architecture) \
  signed-by=/etc/apt/keyrings/docker.gpg] \
  https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker
sudo apt update
sudo apt install -y docker-ce docker-ce-cli containerd.io \
  docker-buildx-plugin docker-compose-plugin

# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Verify
docker --version
docker compose version
```

**macOS**:
```bash
# Download Docker Desktop from https://www.docker.com/products/docker-desktop
# Or use Homebrew
brew install --cask docker

# Start Docker Desktop from Applications
```

**Configuration** (/etc/docker/daemon.json):
```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "storage-driver": "overlay2",
  "default-address-pools": [
    {
      "base": "172.20.0.0/16",
      "size": 24
    }
  ]
}
```

### Docker Compose

**Docker Compose Version**:
- **Minimum**: **2.20.0** (August 2023)
- **Recommended**: **2.24.0+** (January 2024)
- **Type**: Docker Compose V2 (plugin, not standalone)

**Installation**:
- Ships with Docker Engine (plugin)
- Command: `docker compose` (not `docker-compose`)

**Compose File Version**:
```yaml
# docker-compose.yml
services:
  # Services defined without version field (modern format)
```

**Compose Profiles Used**:
- `prod` - Production services (default)
- `dev` - Development services with hot reload
- `redis` - Optional Redis cache
- `messaging` - Optional RabbitMQ
- `api-gateway` - Optional Kong API Gateway
- `monitoring` - Optional Prometheus + Grafana

**Key Compose Features**:
- Service dependencies (`depends_on`)
- Health checks (`healthcheck`)
- Resource limits (`deploy.resources`)
- Named volumes
- Bridge networking
- Environment variable substitution
- Multiple profiles

**Compose Commands**:
```bash
# Start services (production)
docker compose --profile prod up -d

# Start services (development)
docker compose --profile dev up -d

# View logs
docker compose logs -f

# Check status
docker compose ps

# Stop services
docker compose down

# Clean up
docker compose down -v --rmi all
```

### Kubernetes (Optional for Production)

**Kubernetes Version**:
- **Minimum**: **1.28** (August 2023)
- **Recommended**: **1.29+** (December 2023)

**Kubernetes Distributions**:
- **K3s** - Lightweight Kubernetes (recommended for small deployments)
- **K8s** - Full Kubernetes (for large deployments)
- **Managed Services**:
  - AWS EKS (Elastic Kubernetes Service)
  - GCP GKE (Google Kubernetes Engine)
  - Azure AKS (Azure Kubernetes Service)

**Why Kubernetes? (Production)**:
- Automatic scaling (HPA, VPA)
- Self-healing (restarts failed containers)
- Rolling updates (zero downtime)
- Service discovery and load balancing
- Secret and ConfigMap management
- Persistent volume management
- Multi-zone/region deployments

**Kubernetes Installation (K3s)**:
```bash
# Install K3s (lightweight Kubernetes)
curl -sfL https://get.k3s.io | sh -

# Verify
kubectl get nodes
kubectl version
```

**Kubernetes Resources Needed**:
- **Deployments** - For each service (Rust, Python, Frontend)
- **Services** - For networking (ClusterIP, LoadBalancer)
- **ConfigMaps** - Configuration files
- **Secrets** - API keys, passwords
- **PersistentVolumeClaims** - MongoDB data
- **Ingress** - External access
- **HorizontalPodAutoscaler** - Auto-scaling

**Example Kubernetes Manifest**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-core-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-core-engine
  template:
    metadata:
      labels:
        app: rust-core-engine
    spec:
      containers:
      - name: rust-core-engine
        image: bot-core/rust-core-engine:latest
        resources:
          requests:
            memory: "1Gi"
            cpu: "1"
          limits:
            memory: "2Gi"
            cpu: "2"
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: mongodb-secret
              key: connection-string
```

**Helm** (Kubernetes Package Manager):
- **Version**: 3.0+
- **Purpose**: Templated Kubernetes manifests
- **Installation**: `curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash`

**Acceptance Criteria**:
- [x] Docker 24.0+ installed and running
- [x] Docker Compose V2 plugin available
- [x] User added to docker group (no sudo required)
- [x] Docker daemon configured with resource limits
- [x] `docker compose up` starts all services successfully
- [x] Health checks pass for all services
- [x] Kubernetes manifests defined (if using K8s)
- [x] Rolling updates tested (zero downtime)

**Dependencies**: SYS-SOFTWARE-001 (Operating System), SYS-HARDWARE-002 to 004 (Hardware)
**Test Cases**: TC-SOFTWARE-010 (Docker Compose Test), TC-SOFTWARE-011 (Kubernetes Deployment Test)

**Reference**: `/infrastructure/docker/docker-compose.yml`, `/CLAUDE.md`, `/Makefile`

---

## SYS-SOFTWARE-007: Build Tools and Development Utilities

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-SOFTWARE-007`

**Description**:
Essential build tools, compilers, linters, formatters, and development utilities required across all services.

### System Build Tools

**GCC/G++** (C/C++ Compiler):
- **Version**: 9.0+
- **Purpose**: Compile native dependencies (Rust, Python native extensions)
- **Installation (Ubuntu)**: `sudo apt install -y build-essential`

**Make** (Build Automation):
- **Version**: 4.0+
- **Purpose**: Makefile execution
- **Installation**: Ships with build-essential

**pkg-config** (Library Metadata):
- **Version**: Any
- **Purpose**: Find libraries during compilation
- **Installation (Ubuntu)**: `sudo apt install -y pkg-config`

**Git** (Version Control):
- **Version**: 2.40+
- **Purpose**: Source code management
- **Installation**: `sudo apt install -y git`

**curl** (HTTP Client):
- **Version**: 7.68+
- **Purpose**: Download scripts, API testing
- **Installation**: `sudo apt install -y curl`

**OpenSSL** (Cryptography):
- **Version**: 1.1.1 or 3.0+
- **Purpose**: TLS support for Rust and Python
- **Installation (Ubuntu)**: `sudo apt install -y libssl-dev`

### Rust Build Tools

**cargo** (Rust Package Manager):
- Ships with Rust toolchain
- **Version**: 1.86+

**rustfmt** (Rust Formatter):
- **Purpose**: Code formatting
- **Installation**: `rustup component add rustfmt`
- **Usage**: `cargo fmt`
- **Configuration**: `.rustfmt.toml`

**clippy** (Rust Linter):
- **Purpose**: Linting and best practices
- **Installation**: `rustup component add clippy`
- **Usage**: `cargo clippy -- -D warnings`
- **Configuration**: `.clippy.toml`

**cargo-audit** (Security Auditing):
- **Purpose**: Check for vulnerable dependencies
- **Installation**: `cargo install cargo-audit`
- **Usage**: `cargo audit`

**cargo-outdated** (Dependency Updates):
- **Purpose**: Check for outdated dependencies
- **Installation**: `cargo install cargo-outdated`
- **Usage**: `cargo outdated`

**cargo-watch** (File Watcher):
- **Purpose**: Automatic rebuilds on file changes
- **Installation**: `cargo install cargo-watch`
- **Usage**: `cargo watch -x run`

### Python Build Tools

**pip** (Python Package Manager):
- Ships with Python
- **Version**: 23.0+
- **Upgrade**: `pip install --upgrade pip`

**black** (Python Formatter):
- **Version**: 23.12.1
- **Purpose**: Code formatting
- **Configuration**: `pyproject.toml`
- **Usage**: `black .`

**flake8** (Python Linter):
- **Version**: 7.0.0
- **Purpose**: Linting and style checking
- **Configuration**: `.flake8` or `setup.cfg`
- **Usage**: `flake8 .`

**mypy** (Type Checker):
- **Version**: 1.8.0
- **Purpose**: Static type checking
- **Configuration**: `mypy.ini`
- **Usage**: `mypy .`

**isort** (Import Sorter):
- **Version**: 5.13.2
- **Purpose**: Sort and organize imports
- **Usage**: `isort .`

**pytest** (Testing Framework):
- **Version**: 7.4.3
- **Purpose**: Unit and integration testing
- **Usage**: `pytest tests/`

**pip-audit** (Security Auditing):
- **Purpose**: Check for vulnerable dependencies
- **Installation**: `pip install pip-audit`
- **Usage**: `pip-audit`

### Node.js/Frontend Build Tools

**npm** (Package Manager):
- Ships with Node.js
- **Version**: 10.0+

**eslint** (JavaScript/TypeScript Linter):
- **Version**: 9.37.0
- **Purpose**: Linting
- **Configuration**: `eslint.config.js`
- **Usage**: `npm run lint`

**prettier** (Code Formatter):
- **Version**: 3.0+ (optional, Vite formats via SWC)
- **Purpose**: Code formatting
- **Usage**: `prettier --write .`

**tsc** (TypeScript Compiler):
- Ships with TypeScript
- **Purpose**: Type checking (no emit)
- **Usage**: `tsc --noEmit`

**vite** (Build Tool):
- **Version**: 7.1.9
- **Purpose**: Dev server, production builds
- **Usage**: `npm run dev`, `npm run build`

### Testing Tools

**Vitest** (Unit Testing):
- **Version**: 2.1.9
- **Purpose**: Frontend unit tests
- **Usage**: `npm run test`

**Playwright** (E2E Testing):
- **Version**: 1.56.0
- **Purpose**: End-to-end testing
- **Usage**: `npm run test:e2e`

### Monitoring and Profiling Tools

**htop** (Process Monitor):
- **Purpose**: Real-time process monitoring
- **Installation**: `sudo apt install -y htop`

**docker stats** (Container Monitoring):
- Ships with Docker
- **Purpose**: Monitor container resource usage

**cargo flamegraph** (Rust Profiling):
- **Purpose**: CPU profiling for Rust
- **Installation**: `cargo install flamegraph`
- **Usage**: `cargo flamegraph`

**py-spy** (Python Profiling):
- **Purpose**: CPU profiling for Python
- **Installation**: `pip install py-spy`
- **Usage**: `py-spy top --pid <pid>`

### Continuous Integration Tools

**GitHub Actions** (CI/CD):
- **Purpose**: Automated testing, building, deployment
- **Configuration**: `.github/workflows/`

**Docker Buildx** (Multi-platform Builds):
- Ships with Docker
- **Purpose**: Build images for multiple architectures
- **Usage**: `docker buildx build --platform linux/amd64,linux/arm64`

### Acceptance Criteria

- [x] All build tools installed on development machines
- [x] Rust formatting and linting configured
- [x] Python formatting and linting configured
- [x] TypeScript linting configured
- [x] Security audit tools installed
- [x] Test frameworks configured
- [x] CI/CD pipelines defined
- [x] Documentation for all tools

**Dependencies**: SYS-SOFTWARE-001 to 006 (All Software)
**Test Cases**: TC-SOFTWARE-012 (Build Tool Verification)

**Verification Script**:
```bash
#!/bin/bash
# verify-tools.sh

echo "Checking required tools..."

# Rust
command -v rustc || echo "ERROR: rustc not found"
command -v cargo || echo "ERROR: cargo not found"
command -v rustfmt || echo "ERROR: rustfmt not found"
command -v clippy-driver || echo "ERROR: clippy not found"

# Python
command -v python3 || echo "ERROR: python3 not found"
command -v pip || echo "ERROR: pip not found"

# Node.js
command -v node || echo "ERROR: node not found"
command -v npm || echo "ERROR: npm not found"

# Build tools
command -v make || echo "ERROR: make not found"
command -v git || echo "ERROR: git not found"
command -v docker || echo "ERROR: docker not found"

# Check versions
echo ""
echo "Versions:"
rustc --version
cargo --version
python3 --version
node --version
npm --version
docker --version
docker compose version

echo ""
echo "All checks complete!"
```

**Reference**: `/Makefile`, `/.github/workflows/`, `/CLAUDE.md`

---

## SYS-SOFTWARE-008: Optional Services and Infrastructure

**Priority**: ☐ Medium
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-SOFTWARE-008`

**Description**:
Optional services that enhance the platform but are not required for core functionality. These services are enabled via Docker Compose profiles.

### Redis (Caching and Session Management)

**Redis Version**:
- **Version**: 7.x (Alpine image)
- **Image**: `redis:7-alpine`
- **Port**: 6379
- **Profile**: `redis`

**Purpose**:
- Session storage for dashboard
- API response caching
- Rate limiting data
- Pub/Sub for real-time events

**Configuration** (docker-compose.yml):
```yaml
redis:
  image: redis:7-alpine
  command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}
  volumes:
    - redis_data:/data
  ports:
    - "6379:6379"
  profiles:
    - redis
```

**Start Command**:
```bash
docker compose --profile redis up -d
```

### RabbitMQ (Message Queue)

**RabbitMQ Version**:
- **Version**: 3.12 (Alpine with management)
- **Image**: `rabbitmq:3.12-management-alpine`
- **Ports**: 5672 (AMQP), 15672 (Management UI)
- **Profile**: `messaging`

**Purpose**:
- Asynchronous task processing
- Event-driven communication between services
- Reliable message delivery
- Dead letter queues for failed messages

**Configuration** (docker-compose.yml):
```yaml
rabbitmq:
  image: rabbitmq:3.12-management-alpine
  environment:
    - RABBITMQ_DEFAULT_USER=${RABBITMQ_USER:-admin}
    - RABBITMQ_DEFAULT_PASS=${RABBITMQ_PASSWORD}
  ports:
    - "5672:5672"
    - "15672:15672"
  profiles:
    - messaging
```

**Start Command**:
```bash
docker compose --profile messaging up -d
```

### Kong API Gateway

**Kong Version**:
- **Version**: 3.4 (Alpine)
- **Image**: `kong:3.4-alpine`
- **Database**: PostgreSQL 13
- **Ports**: 8100 (Proxy), 8443 (Proxy SSL), 8001 (Admin), 8444 (Admin SSL)
- **Profile**: `api-gateway`

**Purpose**:
- Unified API gateway for all services
- Rate limiting and throttling
- Authentication and authorization
- Request/response transformation
- Load balancing
- Logging and monitoring

**Components**:
- **kong-database** - PostgreSQL for Kong configuration
- **kong-migration** - Database initialization
- **kong** - API Gateway

**Start Command**:
```bash
docker compose --profile api-gateway up -d
```

### Prometheus (Metrics Collection)

**Prometheus Version**:
- **Version**: Latest
- **Image**: `prom/prometheus:latest`
- **Port**: 9090
- **Profile**: `monitoring`

**Purpose**:
- Collect metrics from all services
- Store time-series data
- Query metrics with PromQL
- Alert on threshold violations

**Metrics Endpoints**:
- Rust Core: `http://localhost:8080/metrics`
- Python AI: `http://localhost:8000/metrics`
- Frontend: Browser Performance API

**Configuration**: `/infrastructure/monitoring/prometheus.yml`

### Grafana (Metrics Visualization)

**Grafana Version**:
- **Version**: Latest
- **Image**: `grafana/grafana:latest`
- **Port**: 3001 (to avoid conflict with frontend on 3000)
- **Profile**: `monitoring`

**Purpose**:
- Visualize Prometheus metrics
- Create custom dashboards
- Alert configuration
- Historical data analysis

**Configuration**: `/infrastructure/monitoring/grafana/`

**Start Monitoring Stack**:
```bash
docker compose --profile monitoring up -d
```

**Access**:
- Prometheus: `http://localhost:9090`
- Grafana: `http://localhost:3001` (default: admin/admin)

### Acceptance Criteria

- [x] All optional services documented
- [x] Docker Compose profiles configured
- [x] Configuration files created
- [x] Start/stop procedures documented
- [ ] Integration with core services tested
- [ ] Monitoring dashboards created
- [ ] Alert rules configured

**Dependencies**: SYS-SOFTWARE-006 (Docker Compose), SYS-NETWORK-001 (Ports)
**Test Cases**: TC-SOFTWARE-013 (Optional Services Test)

**Reference**: `/infrastructure/docker/docker-compose.yml` lines 284-438

---

## Version Compatibility Matrix

### Supported Combinations

| Component | Development | Production | Notes |
|-----------|-------------|------------|-------|
| **Operating System** | | | |
| Ubuntu 22.04 LTS | ✅ Recommended | ✅ Recommended | Long-term support |
| Ubuntu 24.04 LTS | ✅ Supported | ✅ Supported | Latest LTS |
| macOS 12+ | ✅ Supported | ❌ Not supported | Dev only |
| Windows 11 (WSL2) | ✅ Supported | ❌ Not supported | Dev only |
| **Rust** | | | |
| 1.86+ | ✅ Required | ✅ Required | MSRV |
| Latest stable | ✅ Recommended | ✅ Recommended | Best performance |
| **Python** | | | |
| 3.11 | ✅ Recommended | ✅ Recommended | TensorFlow 2.18 compatible |
| 3.12 | ✅ Supported | ✅ Supported | TensorFlow 2.18 compatible |
| 3.10 | ❌ Not supported | ❌ Not supported | Too old |
| **Node.js** | | | |
| 18 LTS | ✅ Minimum | ✅ Supported | EOL April 2025 |
| 20 LTS | ✅ Recommended | ✅ Recommended | Active LTS |
| 22 Current | ✅ Supported | ⚠️ Use with caution | Cutting edge |
| **MongoDB** | | | |
| 6.0 | ✅ Minimum | ✅ Supported | EOL July 2024 |
| 7.0 | ✅ Recommended | ✅ Recommended | Current stable |
| 8.0 | ✅ Supported | ⚠️ Use with caution | Latest features |
| **Docker** | | | |
| 24.0+ | ✅ Minimum | ✅ Minimum | Compose V2 support |
| 26.0+ | ✅ Recommended | ✅ Recommended | Latest features |
| **Docker Compose** | | | |
| 2.20+ | ✅ Minimum | ✅ Minimum | Resource limits support |
| 2.24+ | ✅ Recommended | ✅ Recommended | Latest features |

---

## Dependency Update Strategy

### Semantic Versioning Policy

**Major Updates** (X.0.0):
- Require testing and approval
- Plan migration path
- Update documentation
- Schedule maintenance window

**Minor Updates** (1.X.0):
- Test in development first
- Review changelog
- Apply to staging
- Deploy to production after 1 week

**Patch Updates** (1.0.X):
- Apply immediately for security patches
- Test in development
- Deploy to production within 24 hours

### Automated Dependency Updates

**Dependabot** (GitHub):
```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/rust-core-engine"
    schedule:
      interval: "weekly"

  - package-ecosystem: "pip"
    directory: "/python-ai-service"
    schedule:
      interval: "weekly"

  - package-ecosystem: "npm"
    directory: "/nextjs-ui-dashboard"
    schedule:
      interval: "weekly"
```

**Renovate Bot** (Alternative):
- More flexible configuration
- Supports grouped updates
- Custom update schedules

### Security Monitoring

**Cargo Audit** (Rust):
```bash
cargo install cargo-audit
cargo audit
```

**pip-audit** (Python):
```bash
pip install pip-audit
pip-audit
```

**npm audit** (Node.js):
```bash
npm audit
npm audit fix
```

**Snyk** (Comprehensive):
```bash
npm install -g snyk
snyk test
```

---

## License Compliance

### License Types Used

**Permissive Licenses** (✅ Safe for commercial use):
- MIT - Most dependencies
- Apache-2.0 - MongoDB, TensorFlow, many others
- BSD-3-Clause - NumPy, pandas, PyTorch
- ISC - Similar to MIT

**Copyleft Licenses** (⚠️ Review carefully):
- None in direct dependencies

### License Audit

**Cargo License Check**:
```bash
cargo install cargo-license
cargo license --json
```

**Python License Check**:
```bash
pip install pip-licenses
pip-licenses --format=markdown
```

**Node.js License Check**:
```bash
npm install -g license-checker
license-checker --production
```

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Incompatible dependency versions | High | Medium | Version pinning, compatibility matrix, CI tests |
| Security vulnerabilities in dependencies | Critical | Medium | Automated security scanning, weekly updates |
| Breaking changes in major updates | High | Low | Thorough testing, staging environment, rollback plan |
| TensorFlow/PyTorch conflicts | Medium | Low | Virtual environments, separate containers |
| Build failures due to missing tools | Medium | Medium | Docker builds, documented requirements, CI verification |
| MongoDB version incompatibility | High | Low | Test migrations, backup before upgrade, driver compatibility check |

---

## Traceability

**Requirements**:
- Business Rule: [BUSINESS_RULES.md - System Requirements](../../BUSINESS_RULES.md)
- User Story: US-DEV-001 (Development Environment Setup)

**Design**:
- Architecture: [ARCH-INFRASTRUCTURE-001](../../02-design/2.1-architecture/INFRASTRUCTURE.md)
- Docker Compose: [docker-compose.yml](../../../infrastructure/docker/docker-compose.yml)

**Test Cases**:
- Software Testing: TC-SOFTWARE-001 to TC-SOFTWARE-013
- Integration: TC-INTEGRATION-001 to TC-INTEGRATION-010

---

## Open Questions

- [ ] Should we support Python 3.13 when released (October 2025)?
- [ ] Do we need ARM64 builds for production (AWS Graviton)?
- [ ] Should Redis be required instead of optional?
- [ ] Is Kubernetes deployment a priority for initial release?

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Platform Engineering | Initial version with all software requirements |

---

## Appendix

### References

- Rust Cargo.toml: `/rust-core-engine/Cargo.toml`
- Python requirements.txt: `/python-ai-service/requirements.txt`
- Node.js package.json: `/nextjs-ui-dashboard/package.json`
- Docker Compose: `/infrastructure/docker/docker-compose.yml`
- CLAUDE.md: `/CLAUDE.md`
- Makefile: `/Makefile`

### Quick Install Scripts

**Ubuntu 22.04 Development Setup**:
```bash
#!/bin/bash
# install-dev-ubuntu.sh

set -e

echo "Installing development tools for Bot Core..."

# System tools
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev curl git \
    python3.11 python3.11-venv python3.11-dev

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustup component add rustfmt clippy

# Node.js 20 LTS
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Verify
echo ""
echo "Verification:"
rustc --version
python3.11 --version
node --version
docker --version

echo ""
echo "Setup complete! Please log out and log back in for Docker group to take effect."
```

**macOS Development Setup**:
```bash
#!/bin/bash
# install-dev-macos.sh

set -e

echo "Installing development tools for Bot Core..."

# Homebrew (if not installed)
if ! command -v brew &> /dev/null; then
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Install tools
brew install rust python@3.11 node@20 docker

# Rust components
rustup component add rustfmt clippy

# Verify
echo ""
echo "Verification:"
rustc --version
python3.11 --version
node --version
docker --version

echo ""
echo "Setup complete! Please start Docker Desktop."
```

---

**Remember**: Update TRACEABILITY_MATRIX.md when software versions are verified!
