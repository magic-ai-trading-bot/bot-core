#!/bin/bash

# Script to safely setup OpenAI API key
# Usage: ./scripts/setup-openai-key.sh

set -e

BOLD='\033[1m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BOLD}🔐 OpenAI API Key Setup${NC}"
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    echo -e "${RED}❌ Error: .env file not found!${NC}"
    echo "Please copy .env.example to .env first:"
    echo "  cp .env.example .env"
    exit 1
fi

# Check if OPENAI_API_KEY already exists
if grep -q "^OPENAI_API_KEY=" .env; then
    CURRENT_KEY=$(grep "^OPENAI_API_KEY=" .env | cut -d'=' -f2)
    if [[ "$CURRENT_KEY" != "your-openai-api-key" ]] && [[ -n "$CURRENT_KEY" ]]; then
        echo -e "${YELLOW}⚠️  Warning: OPENAI_API_KEY already set in .env${NC}"
        echo "Current key: ${CURRENT_KEY:0:20}..."
        echo ""
        read -p "Do you want to replace it? (yes/no): " REPLACE
        if [[ "$REPLACE" != "yes" ]]; then
            echo "Aborted."
            exit 0
        fi
    fi
fi

echo -e "${YELLOW}📝 Instructions:${NC}"
echo "1. Go to: https://platform.openai.com/api-keys"
echo "2. Create a new API key (or use existing)"
echo "3. Copy the key (starts with sk-proj-...)"
echo ""
echo -e "${RED}⚠️  IMPORTANT: Never share your API key publicly!${NC}"
echo ""

# Prompt for API key (hidden input for security)
read -sp "Paste your OpenAI API key: " API_KEY
echo ""

# Validate key format
if [[ ! "$API_KEY" =~ ^sk-[a-zA-Z0-9_-]+$ ]]; then
    echo -e "${RED}❌ Error: Invalid API key format!${NC}"
    echo "OpenAI keys should start with 'sk-'"
    exit 1
fi

# Check key length
if [ ${#API_KEY} -lt 40 ]; then
    echo -e "${RED}❌ Error: API key seems too short!${NC}"
    exit 1
fi

# Backup .env
cp .env .env.backup
echo -e "${GREEN}✅ Backed up .env to .env.backup${NC}"

# Update or add OPENAI_API_KEY
if grep -q "^OPENAI_API_KEY=" .env; then
    # Replace existing
    sed -i.tmp "s|^OPENAI_API_KEY=.*|OPENAI_API_KEY=${API_KEY}|" .env
    rm .env.tmp 2>/dev/null || true
    echo -e "${GREEN}✅ Updated OPENAI_API_KEY in .env${NC}"
else
    # Add new (after BINANCE_SECRET_KEY line)
    sed -i.tmp "/^BINANCE_SECRET_KEY=/a\\
OPENAI_API_KEY=${API_KEY}
" .env
    rm .env.tmp 2>/dev/null || true
    echo -e "${GREEN}✅ Added OPENAI_API_KEY to .env${NC}"
fi

# Verify
if grep -q "^OPENAI_API_KEY=${API_KEY}" .env; then
    echo ""
    echo -e "${GREEN}✅ Success! OpenAI API key configured${NC}"
    echo ""
    echo -e "${BOLD}Next steps:${NC}"
    echo "1. Start services: ./scripts/bot.sh start --memory-optimized"
    echo "2. Check health: curl http://localhost:8000/health | jq"
    echo "3. Monitor costs: curl http://localhost:8000/ai/cost/statistics | jq"
    echo ""
    echo -e "${YELLOW}📊 Expected costs with optimization:${NC}"
    echo "  • Per request: ~\$0.0005 (12 VNĐ)"
    echo "  • Per day: \$0.62 - \$1.20 (14k - 28k VNĐ)"
    echo "  • Per month: \$18.60 - \$36.00 (428k - 828k VNĐ)"
    echo "  • Savings: 63% (vs \$96.90/month before)"
else
    echo -e "${RED}❌ Error: Failed to update .env${NC}"
    echo "Restoring backup..."
    mv .env.backup .env
    exit 1
fi
