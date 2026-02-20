# xAI Grok API - Migration Research Report

**Date**: 2026-02-20
**Scope**: Replacing OpenAI GPT-4o in python-ai-service + OpenClaw gateway with xAI Grok

---

## 1. Available Grok Models

### Text Models (relevant to this project)

| Model | Context | Input $/1M | Output $/1M | Notes |
|-------|---------|-----------|------------|-------|
| `grok-4-1-fast-non-reasoning` | **2M** | $0.20 | $0.50 | Best for trading analysis - cheap + big ctx |
| `grok-4-1-fast-reasoning` | **2M** | $0.20 | $0.50 | Same price, adds reasoning tokens |
| `grok-3-mini` | 131K | $0.30 | $0.50 | Budget fallback |
| `grok-3` | 131K | $3.00 | $15.00 | Flagship, GPT-4o tier |
| `grok-4-0709` | 256K | $3.00 | $15.00 | Flagship reasoning model |
| `grok-code-fast-1` | 256K | $0.20 | $1.50 | Code-focused |
| `grok-4-fast-non-reasoning` | **2M** | $0.20 | $0.50 | Vision + functions |
| `grok-4-fast-reasoning` | **2M** | $0.20 | $0.50 | Vision + reasoning |

**Recommended for this project**: `grok-4-1-fast-non-reasoning` — cheapest, largest context, full function calling.

Note: There is no "Grok 4.1" as a distinct release separate from `grok-4-1-fast-*`. The "4.1" series is the fast/cheap tier as of Feb 2026.

---

## 2. Pricing Comparison

| Model | Input $/1M | Output $/1M | Relative to GPT-4o |
|-------|-----------|------------|-------------------|
| **GPT-4o** | $2.50 | $10.00 | baseline |
| **GPT-4o-mini** | $0.15 | $0.60 | 17x cheaper in |
| **Grok 4.1 Fast** | $0.20 | $0.50 | **12.5x cheaper in, 20x cheaper out** |
| **Grok 3-mini** | $0.30 | $0.50 | 8x cheaper in |
| **Grok 3 / Grok 4** | $3.00 | $15.00 | more expensive than GPT-4o |

**Bottom line**: Grok 4.1 Fast is comparable to GPT-4o-mini on input but **5x cheaper on output** ($0.50 vs $2.50... wait, vs GPT-4o: $0.50 vs $10.00 = 20x). vs GPT-4o-mini output: $0.50 vs $0.60 — essentially the same price but with 2M context vs 128K.

For trading signal analysis (output-heavy), Grok 4.1 Fast wins significantly.

Additional perks:
- **Batch API**: 50% off for async processing
- **New users**: $25 free credits + $150/month via data-sharing program

---

## 3. OpenAI SDK Compatibility

**YES — fully compatible.** xAI exposes an OpenAI-compatible REST API.

```python
from openai import OpenAI

client = OpenAI(
    api_key=os.getenv("XAI_API_KEY"),
    base_url="https://api.x.ai/v1",
)
```

**Base URL**: `https://api.x.ai/v1`

The existing `DirectOpenAIClient` in `main.py` (line 1211) only needs `base_url` changed from `https://api.openai.com/v1` to `https://api.x.ai/v1` and env var from `OPENAI_API_KEY` to `XAI_API_KEY`.

Endpoints used in codebase (`/chat/completions`) are identical — no payload changes required.

---

## 4. How to Get API Keys

1. Go to [console.x.ai](https://console.x.ai)
2. Sign in with X account (Twitter) or email
3. Create a Team (required)
4. Navigate to **API Keys** → **Create API Key**
5. Set permission: `chat:write` for standard completions
6. Copy key → save as `XAI_API_KEY` in `.env`

**Requirement**: Active X Premium subscription (for API access tier beyond free credits).

---

## 5. Context Window & Capabilities vs GPT-4o

| Capability | GPT-4o | Grok 4.1 Fast |
|-----------|--------|---------------|
| Context window | 128K | **2,000,000** (15x larger) |
| Function calling | Yes | Yes |
| Structured output | Yes | Yes |
| Vision (image input) | Yes | Yes (`grok-4-fast-*` variants) |
| Reasoning tokens | No | Yes (optional) |
| Web search (native) | No | Yes ($5/1K calls) |
| Code execution (native) | No | Yes |
| JSON mode | Yes | Yes |
| Streaming | Yes | Yes |

**No regressions** for current usage patterns (chat completions, JSON output, no vision needed in python-ai-service).

---

## 6. Migration Plan for This Project

### 6a. python-ai-service/main.py

Current code uses `DirectOpenAIClient` with hardcoded `base_url = "https://api.openai.com/v1"`.

**Changes needed** (minimal):

```python
# main.py - DirectOpenAIClient.__init__
self.base_url = os.getenv("AI_BASE_URL", "https://api.x.ai/v1")  # was api.openai.com/v1

# main.py - lifespan()
api_key_string = os.getenv("XAI_API_KEY") or os.getenv("OPENAI_API_KEY", "")
```

```python
# Wherever GPT model is specified (search for "gpt-4" in main.py)
model = os.getenv("AI_MODEL", "grok-4-1-fast-non-reasoning")  # was gpt-4o-mini
```

**.env additions**:
```bash
XAI_API_KEY=xai-your-key-here
AI_MODEL=grok-4-1-fast-non-reasoning
AI_BASE_URL=https://api.x.ai/v1
```

**Cost tracking** (update constants in main.py ~line 74):
```python
GPT4O_MINI_INPUT_COST_PER_1M = 0.20   # Grok 4.1 Fast
GPT4O_MINI_OUTPUT_COST_PER_1M = 0.50  # Grok 4.1 Fast
```

### 6b. OpenClaw Gateway

Current config (`openclaw.json` / `openclaw.production.json`):
```json
"model": { "primary": "openai/gpt-4.1" }
```

**Option A — Direct xAI** (recommended for cost):
```json
"model": { "primary": "xai/grok-4-1-fast-non-reasoning" }
```
Requires env var `XAI_API_KEY` in OpenClaw container.

**Option B — Via OpenRouter** (if you want unified billing):
```json
"model": { "primary": "openrouter/x-ai/grok-4.1-fast" }
```
Requires `OPENROUTER_API_KEY`.

OpenClaw provider name for direct xAI is `xai` (not `x-ai`). Model format: `xai/<model-id>`.

---

## 7. Limitations / Risks

| Area | Status | Notes |
|------|--------|-------|
| API stability | Stable | xAI in production, $25M+ VC backed |
| Rate limits | Unknown exact | Monitor via `x-ratelimit-*` headers (same as OpenAI) |
| Model name changes | Low risk | `grok-4-1-fast-*` just released Nov 2025 |
| Tool/function format | Compatible | Same JSON schema format as OpenAI |
| Retry/backoff logic | No change | `DirectOpenAIClient` handles this already |
| Error codes | Similar | Same HTTP codes (429 = rate limit, etc.) |
| Multi-key rotation | Works | `XAI_BACKUP_API_KEYS` env pattern can be added |
| xAI native features (web search, code exec) | Optional | Not needed for current use case; costs extra |

**One known gap**: xAI does not support the OpenAI `Responses API` (newer endpoint). But this codebase uses `/chat/completions` — no issue.

---

## 8. Estimated Cost Savings

Assuming current usage pattern from main.py (1,200 output tokens avg, many analysis requests):

| Scenario | GPT-4o | Grok 4.1 Fast | Savings |
|---------|--------|---------------|---------|
| 1K requests/day, 500 in + 1200 out tokens | $12.70/day | $0.70/day | **~94% cheaper** |
| Monthly (30 days) | ~$381 | ~$21 | **$360/month saved** |

Note: actual savings depend on real token counts in production.

---

## Unresolved Questions

1. Does xAI API support `OPENAI_BACKUP_API_KEYS` pattern (multiple key rotation) at the same reliability? Need to test 429 response format matches OpenAI exactly for `DirectOpenAIClient` retry logic.
2. Does OpenClaw's `xai` provider support streaming (`streamMode: "partial"` in Telegram config)?
3. xAI requires X Premium subscription — confirm this is available for the account being used.
4. Is `grok-4-1-fast-reasoning` worth it over `grok-4-1-fast-non-reasoning` for trading analysis quality? Same price, just adds reasoning overhead/tokens.

---

## Sources

- [xAI Models and Pricing](https://docs.x.ai/developers/models)
- [xAI API Quickstart](https://docs.x.ai/developers/quickstart)
- [xAI Migration Guide](https://docs.x.ai/docs/guides/migration)
- [Grok 4.1 Fast on OpenRouter](https://openrouter.ai/x-ai/grok-4.1-fast)
- [OpenClaw Model Providers](https://docs.openclaw.ai/concepts/model-providers)
- [AI API Pricing Comparison 2026](https://intuitionlabs.ai/articles/ai-api-pricing-comparison-grok-gemini-openai-claude)
- [OpenAI Pricing](https://platform.openai.com/docs/pricing)
- [Grok 4.1 Fast Agent Tools Announcement](https://x.ai/news/grok-4-1-fast)
