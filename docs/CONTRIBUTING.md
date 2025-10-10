# Contributing to Bot Core

Thank you for your interest in contributing to Bot Core! This document provides guidelines and instructions for contributing to this cryptocurrency trading bot project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Standards](#code-standards)
- [Testing Requirements](#testing-requirements)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Code Review Checklist](#code-review-checklist)
- [Project Structure](#project-structure)
- [Documentation](#documentation)

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other contributors

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- Docker & Docker Compose 2.0+
- Git
- 8GB RAM (minimum)
- 50GB disk space
- Basic knowledge of Rust, Python, or TypeScript/React (depending on your contribution area)

### First-Time Setup

1. **Fork the repository**
   ```bash
   # Fork via GitHub UI, then clone your fork
   git clone https://github.com/YOUR_USERNAME/bot-core.git
   cd bot-core
   ```

2. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/ORIGINAL_OWNER/bot-core.git
   git fetch upstream
   ```

3. **Install development dependencies**
   ```bash
   # Rust
   cd rust-core-engine
   cargo build

   # Python
   cd ../python-ai-service
   pip install -r requirements.dev.txt

   # Frontend
   cd ../nextjs-ui-dashboard
   npm install
   ```

4. **Set up environment**
   ```bash
   cd ..
   cp .env.example .env
   # Edit .env with your test API keys
   ```

5. **Start services in dev mode**
   ```bash
   ./scripts/bot.sh dev
   ```

## Development Setup

### Running Services Locally

For development, you can run services individually without Docker:

#### Rust Core Engine
```bash
cd rust-core-engine
cargo run -- --config config.toml
```

#### Python AI Service
```bash
cd python-ai-service
python main.py
```

#### Frontend Dashboard
```bash
cd nextjs-ui-dashboard
npm run dev
```

### Using Docker for Development

```bash
# Start all services with hot reload
./scripts/bot.sh dev

# Start with specific enterprise features
./scripts/bot.sh dev --with-redis --with-monitoring

# View logs for specific service
./scripts/bot.sh logs --service rust-core-engine
```

## Code Standards

### General Principles

1. **Spec-Driven Development**: Always check `specs/` directory before implementing features
2. **No Hardcoded Secrets**: Use environment variables for all sensitive data
3. **Error Handling**: Always handle errors gracefully, never use unwrap() in production
4. **Documentation**: Document all public APIs and complex logic
5. **Testing**: Write tests for all new features

### Rust Standards

#### Code Style
- Follow Rust official style guide
- Use `rustfmt` for formatting: `cargo fmt`
- Run `clippy` and fix all warnings: `cargo clippy -- -D warnings`
- Maximum line length: 100 characters

#### Error Handling
```rust
// Good - Proper error handling
fn process_trade(order: Order) -> Result<Trade, TradingError> {
    let validated = validate_order(&order)?;
    execute_trade(validated)
}

// Bad - Using unwrap() in production
fn process_trade(order: Order) -> Trade {
    let validated = validate_order(&order).unwrap(); // NEVER DO THIS
    execute_trade(validated).unwrap()
}
```

#### Type Safety
```rust
// Good - Explicit types
pub struct Position {
    pub symbol: String,
    pub quantity: Decimal,
    pub entry_price: Decimal,
}

// Good - Using newtype pattern for domain concepts
pub struct UserId(String);
pub struct OrderId(String);
```

#### Testing Requirements
- Minimum 80% code coverage
- Unit tests for all business logic
- Integration tests for API endpoints
- No unwrap() in tests, use proper assertions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_validation() {
        let position = Position::new("BTCUSDT", 0.001, 45000.0);
        assert!(position.is_ok());

        let invalid = Position::new("INVALID", -1.0, 0.0);
        assert!(invalid.is_err());
    }
}
```

### Python Standards

#### Code Style
- Follow PEP 8
- Use Black formatter with line length 100: `black --line-length 100 .`
- Use type hints for all functions
- Run flake8: `flake8 --max-line-length=100`
- Run mypy: `mypy --strict .`

#### Type Hints
```python
# Good - Proper type hints
from typing import List, Optional
from decimal import Decimal

def calculate_position_size(
    balance: Decimal,
    risk_percent: float,
    stop_loss_distance: Decimal
) -> Decimal:
    """Calculate position size based on risk parameters."""
    return (balance * Decimal(str(risk_percent))) / stop_loss_distance

# Bad - No type hints
def calculate_position_size(balance, risk_percent, stop_loss_distance):
    return (balance * risk_percent) / stop_loss_distance
```

#### Error Handling
```python
# Good - Explicit exception handling
try:
    signal = await generate_ai_signal(symbol)
    return signal
except OpenAIError as e:
    logger.error(f"AI service error: {e}")
    raise AIServiceUnavailable("AI service temporarily unavailable")
except Exception as e:
    logger.exception("Unexpected error in AI analysis")
    raise

# Bad - Bare except
try:
    signal = await generate_ai_signal(symbol)
    return signal
except:  # NEVER DO THIS
    pass
```

#### Testing Requirements
- Minimum 90% code coverage
- Use pytest for testing
- Use pytest-asyncio for async tests
- Mock external API calls

```python
import pytest
from unittest.mock import AsyncMock, patch

@pytest.mark.asyncio
async def test_ai_analysis():
    with patch("services.openai_client.generate") as mock_generate:
        mock_generate.return_value = {"signal": "Long", "confidence": 0.85}

        result = await analyze_market("BTCUSDT")

        assert result.signal == TradingSignal.Long
        assert result.confidence == 0.85
        mock_generate.assert_called_once()
```

### TypeScript/React Standards

#### Code Style
- Use ESLint with strict configuration
- Strict TypeScript mode enabled
- Maximum warnings: 0
- Use Prettier for formatting
- Minimum 85% test coverage

#### Component Structure
```typescript
// Good - Proper typing and structure
interface TradingChartProps {
  symbol: string;
  timeframe: string;
  data: CandleData[];
  onIntervalChange?: (interval: string) => void;
}

export const TradingChart: React.FC<TradingChartProps> = ({
  symbol,
  timeframe,
  data,
  onIntervalChange
}) => {
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    // Effect logic
  }, [symbol, timeframe]);

  return (
    <div className="trading-chart">
      {/* Component JSX */}
    </div>
  );
};

// Bad - No typing
export const TradingChart = ({ symbol, timeframe, data }) => {
  // Implementation
};
```

#### State Management
```typescript
// Good - Typed state
interface AppState {
  user: User | null;
  positions: Position[];
  isAuthenticated: boolean;
}

const [state, setState] = useState<AppState>({
  user: null,
  positions: [],
  isAuthenticated: false
});
```

#### Testing Requirements
```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { TradingChart } from './TradingChart';

describe('TradingChart', () => {
  it('renders chart with data', () => {
    const data = mockCandleData();
    render(<TradingChart symbol="BTCUSDT" timeframe="1h" data={data} />);

    expect(screen.getByTestId('trading-chart')).toBeInTheDocument();
  });

  it('handles interval change', () => {
    const handleChange = jest.fn();
    render(<TradingChart {...props} onIntervalChange={handleChange} />);

    fireEvent.click(screen.getByText('4h'));
    expect(handleChange).toHaveBeenCalledWith('4h');
  });
});
```

## Testing Requirements

### Unit Tests

All services must have comprehensive unit tests:

```bash
# Rust
cd rust-core-engine
cargo test

# Python
cd python-ai-service
pytest --cov=. --cov-report=html

# Frontend
cd nextjs-ui-dashboard
npm test -- --coverage
```

### Integration Tests

Test service-to-service communication:

```bash
# Run integration tests
make test-integration

# Specific integration tests
make test-rust-python
make test-dashboard-rust
```

### End-to-End Tests

```bash
# Run E2E tests
cd tests/e2e
npm run cypress:run

# Interactive mode
npm run cypress:open
```

### Test Coverage Requirements

| Service | Minimum Coverage | Target Coverage |
|---------|-----------------|-----------------|
| Rust Core Engine | 80% | 90% |
| Python AI Service | 90% | 95% |
| Frontend Dashboard | 85% | 90% |

## Commit Guidelines

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting, missing semi-colons, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

#### Examples

```bash
# Feature
feat(rust-core): Add support for trailing stop orders

Implement trailing stop functionality that automatically
adjusts stop loss as price moves favorably.

Closes #123

# Bug fix
fix(python-ai): Handle OpenAI rate limit errors gracefully

Add exponential backoff retry logic for rate limit errors.
Fall back to cached analysis if API is unavailable.

Fixes #456

# Documentation
docs: Update CONTRIBUTING.md with Python standards

Add detailed examples for type hints and error handling.
```

### Commit Best Practices

1. **Atomic Commits**: Each commit should represent a single logical change
2. **Reference Issues**: Include issue numbers (e.g., "Closes #123")
3. **Keep Subject Short**: Maximum 50 characters
4. **Body Details**: Explain what and why, not how
5. **Sign Commits**: Use GPG signing for verified commits

## Pull Request Process

### Before Submitting

1. **Update from main**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all tests**
   ```bash
   make test
   make lint
   ```

3. **Update documentation** if needed

4. **Add tests** for new features

### Creating Pull Request

1. **Use PR template** (auto-populated by GitHub)

2. **Write clear title** following commit conventions
   ```
   feat(rust-core): Add WebSocket reconnection logic
   ```

3. **Describe changes**
   - What does this PR do?
   - Why is this change needed?
   - How has this been tested?
   - Screenshots (if UI changes)

4. **Link related issues**
   ```markdown
   Closes #123
   Related to #456
   ```

5. **Request reviewers**
   - At least 2 reviewers for major changes
   - 1 reviewer for minor changes/docs

### PR Requirements

- [ ] All tests pass
- [ ] Code coverage maintained or improved
- [ ] No linting errors
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for user-facing changes)
- [ ] Follows spec (check `specs/` directory)
- [ ] No hardcoded secrets or API keys

### After Submission

1. **Respond to feedback** promptly
2. **Update PR** based on review comments
3. **Request re-review** after making changes
4. **Squash commits** if requested before merge

## Code Review Checklist

### For Reviewers

#### General
- [ ] Code follows project style guide
- [ ] No hardcoded secrets or credentials
- [ ] Error handling is comprehensive
- [ ] Logging is appropriate (not too verbose, not too sparse)
- [ ] No commented-out code
- [ ] No TODO comments without issue references

#### Functionality
- [ ] Code does what the PR says it does
- [ ] Edge cases are handled
- [ ] Input validation is present
- [ ] Business rules are followed (check `specs/BUSINESS_RULES.md`)
- [ ] API contracts are honored (check `specs/API_SPEC.md`)

#### Testing
- [ ] Tests are present for new functionality
- [ ] Tests cover edge cases
- [ ] Tests are meaningful (not just for coverage)
- [ ] Mocks are used appropriately
- [ ] Test names are descriptive

#### Security
- [ ] No SQL injection vulnerabilities
- [ ] No hardcoded passwords or tokens
- [ ] Sensitive data is encrypted
- [ ] Rate limiting is implemented
- [ ] Authentication is required where needed

#### Performance
- [ ] No obvious performance issues
- [ ] Database queries are optimized
- [ ] Appropriate indexes exist
- [ ] Caching is used where beneficial
- [ ] No N+1 query problems

#### Documentation
- [ ] Public APIs are documented
- [ ] Complex logic has comments
- [ ] README is updated if needed
- [ ] API spec is updated if needed

### Approval Process

- **Minor changes**: 1 approval required
- **Major changes**: 2 approvals required
- **Breaking changes**: 2 approvals + tech lead approval
- **Security changes**: Security team approval required

## Project Structure

```
bot-core/
├── rust-core-engine/       # Rust trading engine
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/           # REST API endpoints
│   │   ├── models/        # Data models
│   │   ├── services/      # Business logic
│   │   └── websocket/     # WebSocket handlers
│   ├── tests/             # Integration tests
│   └── Cargo.toml
│
├── python-ai-service/      # Python AI service
│   ├── models/            # ML models
│   ├── services/          # AI services
│   ├── api/               # FastAPI endpoints
│   ├── tests/             # Tests
│   └── requirements.txt
│
├── nextjs-ui-dashboard/    # React frontend
│   ├── src/
│   │   ├── components/    # React components
│   │   ├── pages/         # Page components
│   │   ├── hooks/         # Custom hooks
│   │   ├── services/      # API clients
│   │   └── utils/         # Utilities
│   └── tests/             # Frontend tests
│
├── specs/                  # Specifications (READ FIRST!)
│   ├── API_SPEC.md
│   ├── DATA_MODELS.md
│   ├── BUSINESS_RULES.md
│   └── INTEGRATION_SPEC.md
│
├── examples/               # API examples
│   ├── rust-api/
│   └── python-api/
│
├── docs/                   # Additional documentation
├── scripts/                # Utility scripts
└── tests/                  # Cross-service tests
```

## Documentation

### When to Update Documentation

- **API changes**: Update `specs/API_SPEC.md`
- **Data model changes**: Update `specs/DATA_MODELS.md`
- **Business logic changes**: Update `specs/BUSINESS_RULES.md`
- **New features**: Update relevant README files
- **Breaking changes**: Update CHANGELOG.md and migration guides

### Documentation Standards

- Use Markdown for all documentation
- Include code examples
- Add diagrams where helpful (Mermaid format preferred)
- Keep documentation close to code
- Update examples/ directory with working examples

## Getting Help

- **Questions**: Open a Discussion on GitHub
- **Bugs**: Open an Issue with bug report template
- **Features**: Open an Issue with feature request template
- **Security**: Email security@example.com (DO NOT open public issue)

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- CHANGELOG.md

Thank you for contributing to Bot Core!
