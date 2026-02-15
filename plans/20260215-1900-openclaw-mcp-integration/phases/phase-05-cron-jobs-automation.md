# Phase 05: Cron Jobs & Automation

**Status**: Pending | **Est.**: 1 day | **Priority**: P1 (value multiplier)

## Context Links

- [Research: OpenClaw Cron Jobs](../research/researcher-01-openclaw-mcp.md#5-cron-jobs--automation)
- [BotCore AI Analysis API](../../../../specs/02-design/2.3-api/API-PYTHON-AI.md#post-aianalyze)
- [BotCore Performance API](../../../../specs/02-design/2.3-api/API-RUST-CORE.md#get-apitradingperformance)
- Phase 03 (self-tuning tools available)
- Phase 04 (OpenClaw + channels running)

## Overview

Configure scheduled automation via OpenClaw cron jobs. These trigger Claude to perform periodic analysis, generate reports, review performance, and optionally auto-tune parameters -- all sent to the user via Telegram/WhatsApp. Each cron job runs in independent mode (separate context) to avoid interference.

## Key Insights

1. OpenClaw cron jobs are prompts executed on a schedule -- Claude runs the prompt using MCP tools, then sends the result via active channels.
2. **Independent mode** is critical: each cron run gets its own context, preventing state leak between scheduled tasks.
3. Cron jobs should be conservative initially -- start with reports/monitoring, add auto-tuning after user gains confidence.
4. Cost per cron run: ~0 (Claude Max subscription includes API usage). But be mindful of MCP tool calls generating BotCore API load.
5. Error backoff is built into OpenClaw (30s -> 1m -> 5m -> 15m -> 60m).

## Requirements

- Morning market briefing (weekdays 9:00 AM)
- Portfolio report (daily 8:00 PM)
- Performance review (weekly Monday 10:00 AM)
- System health check (every 2 hours)
- Self-tuning review (daily 6:00 AM, before market opens)
- Risk alert monitor (every 30 minutes during active trading)
- All outputs delivered via Telegram/WhatsApp
- Each job runs in independent context
- Jobs configurable (enable/disable, schedule change) via user message

## Architecture

```
OpenClaw Cron Scheduler
  |
  +-- 09:00 Mon-Fri: morning_market_briefing
  |     -> Claude uses: get_market_prices, get_market_overview,
  |        analyze_market, get_market_condition
  |     -> Output: Market summary + AI signals to Telegram
  |
  +-- 20:00 Daily: daily_portfolio_report
  |     -> Claude uses: get_portfolio, get_open_trades,
  |        get_closed_trades, get_trading_performance
  |     -> Output: PnL summary, open positions, daily stats
  |
  +-- 10:00 Monday: weekly_performance_review
  |     -> Claude uses: get_tuning_dashboard, get_adjustment_history,
  |        get_trading_performance, get_ai_performance
  |     -> Output: Week-over-week comparison, recommendations
  |
  +-- */120 * * * *: system_health_check
  |     -> Claude uses: check_system_health, get_system_metrics,
  |        get_connection_status
  |     -> Output: Alert only if unhealthy (no spam for healthy)
  |
  +-- 06:00 Daily: self_tuning_review
  |     -> Claude uses: get_tuning_dashboard, get_parameter_bounds,
  |        get_config_suggestions, suggest_adjustments
  |     -> Output: Proposed adjustments (GREEN auto-apply, YELLOW ask)
  |
  +-- */30 * * * *: risk_monitor
        -> Claude uses: get_portfolio, get_correlation_analysis,
           get_open_trades
        -> Output: Alert only on risk events (drawdown, correlation)
```

## Related Code Files

| File | Purpose |
|------|---------|
| `openclaw/config/cron/` (new) | Cron job definitions directory |
| `openclaw/config/cron/morning-briefing.json` | Morning market briefing job |
| `openclaw/config/cron/daily-portfolio.json` | Daily portfolio report job |
| `openclaw/config/cron/weekly-review.json` | Weekly performance review job |
| `openclaw/config/cron/health-check.json` | System health check job |
| `openclaw/config/cron/self-tuning.json` | Self-tuning review job |
| `openclaw/config/cron/risk-monitor.json` | Risk alert monitor job |

## Implementation Steps

### 1. Morning Market Briefing (~1.5h)

**`morning-briefing.json`**:
```json
{
  "id": "morning-market-briefing",
  "name": "Morning Market Briefing",
  "schedule": "0 9 * * 1-5",
  "mode": "independent",
  "prompt": "Run a morning market briefing. Use MCP tools to: 1) Get current prices for all symbols. 2) Get market overview with 24h stats. 3) Get AI market condition analysis for BTC and ETH. 4) Check if there are any open positions and their PnL. Format as a concise Telegram-friendly report with: current prices, 24h changes (with arrows), AI signals if any, open position status, and one-sentence market outlook. Keep it under 500 characters for readability."
}
```

**Expected Output Format**:
```
MORNING BRIEFING - Feb 15, 2026

BTC: $97,500 (+2.3%)
ETH: $3,850 (-0.5%)
BNB: $625 (+1.1%)
SOL: $185 (+3.2%)

AI Signal: BTC LONG (75% conf)
Open: 2 positions, +$125 unrealized

Outlook: Bullish momentum continues with RSI at 62.
```

### 2. Daily Portfolio Report (~1.5h)

**`daily-portfolio.json`**:
```json
{
  "id": "daily-portfolio-report",
  "name": "Daily Portfolio Report",
  "schedule": "0 20 * * *",
  "mode": "independent",
  "prompt": "Generate a daily portfolio report. Use MCP tools to: 1) Get portfolio summary (balance, PnL). 2) Get today's closed trades with results. 3) Get open positions with current PnL. 4) Get trading performance stats. Format as: portfolio balance, daily PnL, win rate today, list of trades executed today, open positions table. Highlight any notable events (big wins, losses, risk events). End with brief assessment."
}
```

### 3. Weekly Performance Review (~2h)

**`weekly-review.json`**:
```json
{
  "id": "weekly-performance-review",
  "name": "Weekly Performance Review",
  "schedule": "0 10 * * 1",
  "mode": "independent",
  "prompt": "Perform a comprehensive weekly performance review. Use MCP tools to: 1) Get trading performance stats. 2) Get tuning dashboard for parameter analysis. 3) Get adjustment history from the past week. 4) Get AI config suggestions. 5) Get strategy performance breakdown. Create a detailed report with: weekly PnL summary, win rate trend, strategy performance comparison, parameter adjustments made this week and their impact, AI suggestions for improvement, and concrete action items for the coming week. For YELLOW-tier suggestions, present them as confirmable actions."
}
```

### 4. System Health Check (~1h)

**`health-check.json`**:
```json
{
  "id": "system-health-check",
  "name": "System Health Check",
  "schedule": "0 */2 * * *",
  "mode": "independent",
  "prompt": "Check system health. Use MCP tools to check system health, connection status, and trading metrics. ONLY send a message if something is wrong (unhealthy service, high error rate, disconnected WebSocket, etc.). If everything is healthy, respond with just 'HEALTHY' and do NOT send to the user. If there are issues, alert with: which service is affected, what the issue is, and suggested action."
}
```

**Note**: The "HEALTHY = no message" behavior depends on OpenClaw's cron output handling. If OpenClaw always sends output, add a flag in the prompt to suppress healthy-state messages.

### 5. Self-Tuning Review (~2h)

**`self-tuning.json`**:
```json
{
  "id": "self-tuning-review",
  "name": "Daily Self-Tuning Review",
  "schedule": "0 6 * * *",
  "mode": "independent",
  "prompt": "Perform a self-tuning analysis. Use MCP tools to: 1) Get the tuning dashboard with full performance data. 2) Get current parameter bounds and values. 3) Get AI config suggestions from the Python service. 4) Review last 3 days of trading performance trends. Based on the data: A) For GREEN parameters where data strongly supports a change, apply the adjustment automatically and report what you changed and why. B) For YELLOW parameters where you see opportunity, present the suggestion with data and ask for confirmation. C) For RED parameters, only mention if there's a critical need. Always explain your reasoning with numbers. Maximum 2 GREEN auto-adjustments per review."
}
```

### 6. Risk Monitor (~1h)

**`risk-monitor.json`**:
```json
{
  "id": "risk-monitor",
  "name": "Risk Alert Monitor",
  "schedule": "*/30 * * * *",
  "mode": "independent",
  "prompt": "Quick risk check. Use MCP tools to: 1) Get portfolio (check daily PnL). 2) Get correlation analysis. 3) Check if paper trading is running. ONLY alert if: daily loss exceeds 3%, correlation exceeds 0.75, or paper trading has stopped unexpectedly. If no issues, respond with just 'OK' and do NOT send to user. If alerting, be urgent and specific."
}
```

### 7. Cron Registration (~1h)

Register all cron jobs via OpenClaw CLI during container startup:

```bash
# In Dockerfile or entrypoint script
for f in /root/.openclaw/cron/*.json; do
  openclaw cron add --file "$f" || true
done
```

Or via API after container starts:
```bash
docker exec openclaw openclaw cron list  # verify
docker exec openclaw openclaw cron add --file /root/.openclaw/cron/morning-briefing.json
```

### 8. User-Configurable Schedules (~1h)

Enable users to modify cron schedules via chat:
- "Pause morning briefing" -> disable cron job
- "Change portfolio report to 9 PM" -> update schedule
- "Run health check now" -> manual trigger

This works through Claude interpreting the message and using OpenClaw cron API (or MCP tools that wrap cron management).

## Todo List

- [ ] Create `openclaw/config/cron/` directory
- [ ] Write morning market briefing cron job definition
- [ ] Write daily portfolio report cron job definition
- [ ] Write weekly performance review cron job definition
- [ ] Write system health check cron job definition (alert-only)
- [ ] Write self-tuning review cron job definition
- [ ] Write risk monitor cron job definition (alert-only)
- [ ] Implement cron registration in container entrypoint
- [ ] Test: morning briefing executes and sends to Telegram
- [ ] Test: portfolio report shows correct PnL data
- [ ] Test: health check is silent when healthy
- [ ] Test: risk monitor alerts on simulated threshold breach
- [ ] Test: self-tuning applies GREEN adjustment and notifies
- [ ] Test: user can pause/resume cron jobs via chat
- [ ] Tune prompt lengths to avoid excessive Claude API usage
- [ ] Verify cron jobs survive container restart

## Success Criteria

1. All 6 cron jobs registered and running on schedule
2. Morning briefing arrives at 9:00 AM weekdays via Telegram
3. Portfolio report arrives at 8:00 PM daily
4. Health checks only alert when issues detected (not spam)
5. Self-tuning review proposes actionable adjustments with data
6. Risk monitor alerts within 30 minutes of threshold breach
7. Cron jobs persist across container restarts
8. User can modify schedules via natural language in chat

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Cron spam (too many messages) | Medium | Medium | Alert-only for health/risk, concise formats |
| Claude API rate limits during cron burst | Low | Medium | Stagger cron schedules, 5-min gaps between jobs |
| Stale data in independent context | Low | Low | Each run fetches fresh data via MCP |
| Self-tuning cron applies bad adjustments | Low | High | Max 2 GREEN auto-adjustments per run, bounds enforced |

## Security Considerations

- Cron prompts should not contain secrets (they are stored in JSON files)
- Self-tuning cron can only apply GREEN adjustments automatically
- YELLOW/RED changes from cron require user interaction (confirmation message sent)
- Health check and risk monitor prompts designed to minimize data exposure in alerts
- All cron job outputs are delivered only to allowlisted users

## Next Steps

After this phase: proceed to Phase 06 for comprehensive integration testing and security audit of the entire system.
