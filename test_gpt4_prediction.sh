#!/bin/bash
# Test GPT-4 Trend Prediction Implementation

echo "🧪 Testing GPT-4 Trend Prediction Endpoint"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Python AI service is running
echo "1️⃣ Checking if Python AI service is running..."
if curl -s http://localhost:8000/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Python AI service is running${NC}"
else
    echo -e "${RED}❌ Python AI service is NOT running${NC}"
    echo "   Start it with: cd python-ai-service && python3 main.py"
    exit 1
fi

echo ""

# Check if OpenAI API key is set
echo "2️⃣ Checking OpenAI API key..."
if [ -z "$OPENAI_API_KEY" ]; then
    echo -e "${YELLOW}⚠️  OPENAI_API_KEY not set${NC}"
    echo "   GPT-4 will not be used (will fallback to technical analysis)"
    echo "   To enable: export OPENAI_API_KEY='sk-...'"
else
    echo -e "${GREEN}✅ OPENAI_API_KEY is set${NC}"
fi

echo ""

# Test 1: Predict trend for BTCUSDT
echo "3️⃣ Testing /predict-trend for BTCUSDT..."
response=$(curl -s -X POST http://localhost:8000/predict-trend \
  -H "Content-Type: application/json" \
  -d '{"symbol": "BTCUSDT", "timeframe": "4h"}')

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ API call successful${NC}"
    echo ""
    echo "Response:"
    echo "$response" | python3 -m json.tool 2>/dev/null || echo "$response"
else
    echo -e "${RED}❌ API call failed${NC}"
    exit 1
fi

echo ""
echo "=========================================="

# Extract model used
model=$(echo "$response" | python3 -c "import sys, json; print(json.load(sys.stdin).get('model', 'unknown'))" 2>/dev/null)

if [ "$model" == "GPT-4o-mini" ]; then
    echo -e "${GREEN}🤖 GPT-4 was used successfully!${NC}"
elif [ "$model" == "EMA200-Technical-Fallback" ]; then
    echo -e "${YELLOW}⚠️  Technical fallback was used (GPT-4 unavailable)${NC}"
else
    echo -e "${YELLOW}⚠️  Unknown model: $model${NC}"
fi

echo ""

# Check cost summary
echo "4️⃣ Checking API cost summary..."
cost_response=$(curl -s http://localhost:8000/ai/cost/summary)

if [ $? -eq 0 ]; then
    echo "$cost_response" | python3 -m json.tool 2>/dev/null || echo "$cost_response"
else
    echo -e "${YELLOW}⚠️  Could not fetch cost summary${NC}"
fi

echo ""
echo "=========================================="
echo -e "${GREEN}✅ Testing complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Start paper trading: curl -X POST http://localhost:8080/api/paper-trading/start"
echo "2. Monitor logs: docker logs -f rust-core-engine | grep 'Hybrid filter'"
echo "3. Check costs: curl http://localhost:8000/ai/cost/summary"
echo "4. Read full docs: cat GPT4_TREND_PREDICTION_IMPLEMENTATION.md"
