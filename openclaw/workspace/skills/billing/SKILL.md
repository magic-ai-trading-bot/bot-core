---
name: billing
description: Show AI billing costs, token usage, and spending breakdown. Use when the user asks about costs, spending, billing, usage, tokens, or budget.
metadata: {"openclaw":{"emoji":"💰"}}
---

# Billing

Show AI spending from session logs.

Run:
```bash
python3 {baseDir}/scripts/billing.py [ARGS]
```

Send the **raw stdout** as the reply — do not reformat, summarise, or wrap in code blocks.

Valid ARGS (default = dashboard):
`today` `week` `month` `7d` `30d` `total` `models`

Pass user arguments verbatim, e.g. `/billing models` → `python3 … models`.
