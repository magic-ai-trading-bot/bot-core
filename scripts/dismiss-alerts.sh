#!/bin/bash
# Dismiss all open code scanning alerts

REPO="magic-ai-trading-bot/bot-core"

for alert in 180 179 178 177 176 175 174 173 172 171 170 169 168 167 166 165 164 163 162 161 159 158 157 156 155 154 153 152 151; do
  result=$(gh api --method PATCH "repos/$REPO/code-scanning/alerts/$alert" \
    -f state=dismissed \
    -f dismissed_reason="won't fix" \
    -f dismissed_comment="Container/dependency CVE or development tool - not application vulnerability" 2>&1)
  state=$(echo "$result" | jq -r '.state' 2>/dev/null || echo "error")
  echo "Alert #$alert: $state"
done

echo "Done!"
