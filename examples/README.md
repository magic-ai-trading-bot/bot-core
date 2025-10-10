# Bot Core API Examples

This directory contains comprehensive examples for all APIs, configurations, and integrations in the Bot Core trading platform.

## Directory Structure

```
examples/
├── api/                      # API request/response examples
│   ├── rust-core/           # Rust Core Engine API examples
│   ├── python-ai/           # Python AI Service API examples
│   └── common/              # Common patterns and utilities
├── config/                  # Configuration file examples
├── strategies/             # Trading strategy examples
├── integration/            # Integration examples
└── README.md              # This file
```

## Quick Start

### 1. Authentication Example

```bash
# Login to get JWT token
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d @examples/api/rust-core/auth-login-request.json

# Response will contain JWT token
# Store token for subsequent requests
export BOT_TOKEN="eyJhbGci..."
```

### 2. Execute a Trade

```bash
# Place a limit buy order
curl -X POST http://localhost:8080/api/trades/execute \
  -H "Authorization: Bearer $BOT_TOKEN" \
  -H "Content-Type: application/json" \
  -d @examples/api/rust-core/trade-execute-limit-buy-request.json
```

### 3. Get AI Analysis

```bash
# Request AI market analysis
curl -X POST http://localhost:8000/ai/analyze \
  -H "Content-Type: application/json" \
  -d @examples/api/python-ai/ai-analyze-request.json
```

## API Examples Index

### Rust Core Engine API

#### Authentication
- [Login Request](./api/rust-core/auth-login-request.json) / [Response](./api/rust-core/auth-login-response.json)
- [Refresh Token Request](./api/rust-core/auth-refresh-request.json) / [Response](./api/rust-core/auth-refresh-response.json)
- [Logout Request](./api/rust-core/auth-logout-request.json)

#### Trading
- [Execute Limit Buy](./api/rust-core/trade-execute-limit-buy-request.json) / [Response](./api/rust-core/trade-execute-response.json)
- [Execute Market Sell](./api/rust-core/trade-execute-market-sell-request.json)
- [Place Stop Loss](./api/rust-core/trade-stop-loss-request.json)
- [Place Take Profit](./api/rust-core/trade-take-profit-request.json)
- [Cancel Order](./api/rust-core/trade-cancel-request.json)

#### Positions & Account
- [Get Open Positions](./api/rust-core/positions-get-response.json)
- [Close Position](./api/rust-core/positions-close-request.json)
- [Get Account Info](./api/rust-core/account-info-response.json)
- [Get Trade History](./api/rust-core/trade-history-response.json)

#### Paper Trading
- [Start Paper Trading](./api/rust-core/paper-trading-start-request.json)
- [Paper Trade Execution](./api/rust-core/paper-trade-execute-request.json)
- [Get Paper Trading Stats](./api/rust-core/paper-trading-stats-response.json)

### Python AI Service API

#### AI Analysis
- [Market Analysis Request](./api/python-ai/ai-analyze-request.json) / [Response](./api/python-ai/ai-analyze-response.json)
- [Batch Analysis Request](./api/python-ai/ai-batch-analyze-request.json)
- [Strategy Recommendations](./api/python-ai/ai-strategy-recommendations-request.json)

#### ML Predictions
- [Price Prediction Request](./api/python-ai/ml-predict-price-request.json)
- [Trend Prediction](./api/python-ai/ml-predict-trend-request.json)
- [Market Sentiment](./api/python-ai/ml-market-sentiment-response.json)

### WebSocket Examples

#### Rust WebSocket
- [Connection Example](./api/rust-core/websocket-connection.js)
- [Subscribe to Markets](./api/rust-core/websocket-subscribe.json)
- [Price Update Message](./api/rust-core/websocket-price-update.json)
- [Order Update Message](./api/rust-core/websocket-order-update.json)

#### Python WebSocket
- [AI Signals Stream](./api/python-ai/websocket-ai-signals.js)
- [Real-time Predictions](./api/python-ai/websocket-predictions.json)

### Error Examples

All error responses follow a consistent format:
- [Rate Limit Error](./api/common/error-rate-limit.json)
- [Authentication Error](./api/common/error-unauthorized.json)
- [Validation Error](./api/common/error-validation.json)
- [Insufficient Balance](./api/common/error-insufficient-balance.json)
- [Service Unavailable](./api/common/error-service-unavailable.json)

## Configuration Examples

### Environment Configuration
- [Development .env](./config/env-development.example)
- [Production .env](./config/env-production.example)
- [Testing .env](./config/env-testing.example)

### Service Configuration
- [Rust config.toml](./config/rust-config.toml)
- [Python config.yaml](./config/python-config.yaml)
- [Frontend .env](./config/frontend-config.env)

### Docker Configuration
- [Docker Compose Development](./config/docker-compose-dev.yml)
- [Docker Compose Production](./config/docker-compose-prod.yml)

## Strategy Examples

### Built-in Strategies
- [RSI Strategy](./strategies/rsi-strategy.json)
- [MACD Strategy](./strategies/macd-strategy.json)
- [Bollinger Bands](./strategies/bollinger-bands-strategy.json)
- [Volume Strategy](./strategies/volume-strategy.json)

### AI-Enhanced Strategies
- [AI Ensemble Strategy](./strategies/ai-ensemble-strategy.json)
- [Sentiment-Based Strategy](./strategies/sentiment-strategy.json)

### Custom Strategy Template
- [Custom Strategy Template](./strategies/custom-strategy-template.json)

## Integration Examples

### Node.js/TypeScript
- [Trading Bot Client](./integration/nodejs-trading-bot.ts)
- [WebSocket Client](./integration/nodejs-websocket-client.ts)
- [AI Analysis Client](./integration/nodejs-ai-client.ts)

### Python
- [Trading Bot Client](./integration/python-trading-bot.py)
- [Data Analysis Script](./integration/python-data-analysis.py)

### Shell Scripts
- [Automated Trading Script](./integration/automated-trading.sh)
- [Monitoring Script](./integration/monitoring.sh)

## Testing Examples

### cURL Scripts
All examples can be tested with cURL. See individual `.md` files for complete cURL commands.

### Postman Collection
Import `postman/Bot-Core-API.postman_collection.json` into Postman for interactive testing.

### Integration Tests
See `tests/integration/` for automated integration test examples.

## Common Patterns

### Authentication Pattern
```bash
# 1. Login
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}' \
  | jq -r '.token')

# 2. Use token in requests
curl -X GET http://localhost:8080/api/account \
  -H "Authorization: Bearer $TOKEN"
```

### Error Handling Pattern
```javascript
async function executeTradeWithRetry(order, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch('http://localhost:8080/api/trades/execute', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(order)
      });

      if (!response.ok) {
        const error = await response.json();

        // Handle specific errors
        if (error.code === 'RATE_LIMITED') {
          await sleep(error.details.retry_after * 1000);
          continue;
        }

        if (error.code === 'INSUFFICIENT_BALANCE') {
          throw new Error('Insufficient balance');
        }

        throw new Error(error.message);
      }

      return await response.json();
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await sleep(1000 * Math.pow(2, i)); // Exponential backoff
    }
  }
}
```

### Pagination Pattern
```bash
# Get paginated trade history
PAGE=1
LIMIT=50

while true; do
  RESPONSE=$(curl -s "http://localhost:8080/api/trades/history?page=$PAGE&limit=$LIMIT" \
    -H "Authorization: Bearer $TOKEN")

  # Process trades
  echo "$RESPONSE" | jq '.trades[]'

  # Check if there are more pages
  TOTAL_PAGES=$(echo "$RESPONSE" | jq '.pagination.pages')
  if [ $PAGE -ge $TOTAL_PAGES ]; then
    break
  fi

  PAGE=$((PAGE + 1))
done
```

## Environment Variables

All examples assume the following default endpoints:

```bash
export RUST_API_URL="http://localhost:8080"
export PYTHON_API_URL="http://localhost:8000"
export WS_URL="ws://localhost:8080/ws"
```

For production, use HTTPS and WSS:

```bash
export RUST_API_URL="https://api.botcore.com"
export PYTHON_API_URL="https://ai.botcore.com"
export WS_URL="wss://api.botcore.com/ws"
```

## Rate Limits

Be aware of rate limits when testing:

| Service | Endpoint | Limit |
|---------|----------|-------|
| Rust Core | Authentication | 5 req/15 min |
| Rust Core | Trading | 10 req/sec |
| Rust Core | Other APIs | 100 req/min |
| Python AI | Analysis | 10 req/min |
| Python AI | Other APIs | 60 req/min |

## Best Practices

1. **Always handle errors gracefully**
   - Check response status codes
   - Parse error messages
   - Implement retry logic for transient failures

2. **Use environment variables**
   - Never hardcode API keys
   - Use different configs for dev/staging/prod

3. **Implement request signing** (for production)
   - Use JWT tokens
   - Refresh tokens before expiry
   - Implement token rotation

4. **Log all trading activity**
   - Log requests and responses
   - Include timestamps and request IDs
   - Store audit trail

5. **Test with paper trading first**
   - Always test new strategies in paper trading mode
   - Validate order logic before live trading
   - Monitor performance metrics

## Troubleshooting

### Common Issues

**401 Unauthorized**
```bash
# Token may have expired, re-authenticate
curl -X POST http://localhost:8080/api/auth/login ...
```

**429 Rate Limited**
```bash
# Check rate limit headers
curl -I http://localhost:8080/api/health
# Wait for X-RateLimit-Reset timestamp
```

**500 Internal Server Error**
```bash
# Check service health
curl http://localhost:8080/api/health
curl http://localhost:8000/health
```

**Connection Refused**
```bash
# Ensure services are running
./scripts/bot.sh status

# Start services if needed
./scripts/bot.sh start
```

## Additional Resources

- [API Specification](../specs/API_SPEC.md)
- [Data Models](../specs/DATA_MODELS.md)
- [Business Rules](../specs/BUSINESS_RULES.md)
- [Integration Patterns](../specs/INTEGRATION_SPEC.md)
- [Architecture Documentation](../docs/architecture/)

## Contributing

To add new examples:

1. Create JSON files for request/response pairs
2. Add descriptive filenames (e.g., `trade-execute-limit-buy-request.json`)
3. Include comments in JSON where helpful
4. Update this README with links to new examples
5. Add corresponding test cases if applicable

## Support

For questions or issues with examples:
- Check [TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md)
- Review [API Documentation](../docs/API_DOCUMENTATION.md)
- Open an issue on GitHub
