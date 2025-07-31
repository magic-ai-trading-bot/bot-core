# Trading Bot Specifications

This directory contains the complete specifications for the cryptocurrency trading bot system. These specifications serve as the single source of truth for all development activities.

## 📚 Specification Documents

### 1. [API_SPEC.md](./API_SPEC.md)
Complete API documentation for all services including:
- REST endpoints for Python AI Service and Rust Core Engine
- Request/response formats
- Error codes and handling
- Authentication and rate limiting
- WebSocket protocols

### 2. [DATA_MODELS.md](./DATA_MODELS.md)
Comprehensive data model definitions:
- Core trading models (Orders, Positions, Trades)
- AI service models (Signals, Indicators, Analysis)
- Database schemas
- Validation rules

### 3. [BUSINESS_RULES.md](./BUSINESS_RULES.md)
Business logic and trading rules:
- Position management rules
- Risk management constraints
- Order execution logic
- Money management rules
- Compliance requirements

### 4. [INTEGRATION_SPEC.md](./INTEGRATION_SPEC.md)
Service integration patterns:
- Inter-service communication
- Data synchronization
- Error handling strategies
- Performance optimization
- Deployment guidelines

## 🔄 Using These Specifications

### For Development

1. **Before implementing any feature**, read the relevant specification
2. **Ensure your code matches** the spec exactly - no deviations
3. **If changes are needed**, update the spec first and get approval
4. **Use the examples** in `/examples` directory for testing

### For Testing

1. **Contract Testing**: Validate all APIs against spec
2. **Integration Testing**: Follow patterns in INTEGRATION_SPEC.md
3. **Business Rule Testing**: Verify all rules from BUSINESS_RULES.md

### For Documentation

These specs ARE the documentation. Keep them updated and accurate.

## 📝 Specification Update Process

1. **Propose Change**: Create a detailed proposal
2. **Impact Analysis**: Identify all affected services
3. **Update Spec**: Modify relevant .md files
4. **Update Examples**: Add/modify examples
5. **Implement**: Only after spec is approved
6. **Validate**: Ensure implementation matches spec

## 🎯 Quick Reference

### Common API Patterns

```bash
# Authentication
POST /api/auth/login
Authorization: Bearer <token>

# AI Analysis
POST /ai/analyze
{
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "candles": [...]
}

# Execute Trade
POST /api/trades/execute
{
  "symbol": "BTCUSDT",
  "side": "BUY",
  "quantity": 0.001
}
```

### Standard Error Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {},
    "timestamp": "ISO 8601"
  }
}
```

### WebSocket Message Format

```json
{
  "type": "message_type",
  "data": {},
  "timestamp": "ISO 8601"
}
```

## 🚀 Service Endpoints

- **Python AI Service**: `http://localhost:8000`
- **Rust Core Engine**: `http://localhost:8080`
- **Frontend Dashboard**: `http://localhost:3000`
- **MongoDB Atlas**: `mongodb+srv://...`

## 📊 Key Business Rules Summary

1. **Max Positions**: 10 concurrent positions
2. **Max Leverage**: 20x (testnet), varies by pair
3. **Stop Loss**: Required, max 10%
4. **Daily Loss Limit**: 5% of account
5. **AI Confidence**: Minimum 0.70 for auto-trade
6. **Position Size**: 0.1% - 10% of account

## 🔐 Security Requirements

1. **JWT Authentication**: 24-hour expiry
2. **Rate Limiting**: Per endpoint limits
3. **Internal Service Token**: For service-to-service
4. **No secrets in code**: Use environment variables

## 📈 Performance Targets

1. **API Response Time**: < 200ms (p95)
2. **Trade Execution**: < 1 second
3. **AI Analysis**: < 5 seconds
4. **WebSocket Latency**: < 100ms
5. **System Uptime**: > 99.9%

---

Remember: **The spec is the source of truth**. When in doubt, follow the spec!