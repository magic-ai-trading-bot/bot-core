#!/bin/bash
# =============================================================================
# sync-openclaw-knowledge.sh
# Auto-generates OpenClaw workspace knowledge from BotCore source code.
#
# This script extracts ACTUAL values from source code and generates:
#   - openclaw/workspace/STRATEGIES.md (strategies + risk layers + params)
#   - openclaw/workspace/SOUL.md (system prompt with architecture knowledge)
#
# It also validates that these manually-maintained files exist:
#   - openclaw/workspace/ARCHITECTURE.md (system architecture, APIs, DB, MCP)
#   - openclaw/workspace/FEATURES.md (all 12 features with status)
#   - openclaw/workspace/CONFIG.md (all tunable parameters)
#
# Run: make sync-knowledge  OR  ./scripts/sync-openclaw-knowledge.sh
# =============================================================================
set -euo pipefail

PROJECT_ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
WORKSPACE="$PROJECT_ROOT/openclaw/workspace"
SETTINGS_FILE="$PROJECT_ROOT/rust-core-engine/src/paper_trading/settings.rs"
STRATEGY_DIR="$PROJECT_ROOT/rust-core-engine/src/strategies"
HOW_IT_WORKS="$PROJECT_ROOT/nextjs-ui-dashboard/src/pages/HowItWorks.tsx"
FEATURES_DIR="$PROJECT_ROOT/docs/features"

mkdir -p "$WORKSPACE"

# =============================================================================
# EXTRACTION FUNCTIONS
# =============================================================================

# Extract a numeric value from settings.rs Default impl block
# Filters out struct definitions (pub field: f64) and test code
# Usage: extract_setting "field_name" "fallback"
extract_setting() {
  local field="$1"
  local fallback="${2:-0}"
  local val
  # Match lines with the field name that contain actual numeric values
  # Exclude: struct definitions (pub), tests (assert/test/fn test), comments-only lines
  val=$(grep "${field}:" "$SETTINGS_FILE" 2>/dev/null \
    | grep -v 'pub \|Option<\|fn \|assert\|#\[' \
    | grep -E '[0-9]' \
    | head -1 \
    | sed 's|//.*||' \
    | grep -oE '[0-9]+\.?[0-9]*' \
    | head -1)
  echo "${val:-$fallback}"
}

# Extract strategy parameter from json!() init in strategy file
# Usage: extract_strategy_param "file" "param_name" "fallback"
extract_strategy_param() {
  local file="$1"
  local param="$2"
  local fallback="${3:-0}"
  local val
  val=$(grep "\"${param}\"" "$file" 2>/dev/null \
    | grep 'json!' \
    | head -1 \
    | grep -oE 'json!\([0-9.]+\)' \
    | grep -oE '[0-9.]+')
  echo "${val:-$fallback}"
}

# Extract win rates from HowItWorks.tsx (by strategy key in i18n pattern)
# HowItWorks uses t('howItWorks.strategies.items.XXX.name') so we match on the key
# Usage: extract_win_rate "rsi|macd|bollinger|volume|stochastic"
extract_win_rate() {
  local key="$1"
  local val
  val=$(grep -A3 "items\.${key}\." "$HOW_IT_WORKS" 2>/dev/null \
    | grep 'winRate' \
    | grep -oE '[0-9]+' \
    | head -1)
  echo "${val:-0}"
}

# Extract risk layer info from HowItWorks.tsx
# Usage: extract_risk_layer N  → outputs "name|value"
extract_risk_layer() {
  local layer_num="$1"
  grep -A2 "layer: ${layer_num}," "$HOW_IT_WORKS" 2>/dev/null \
    | grep -oE "name: '[^']+'" \
    | sed "s/name: '//;s/'//" || echo "Unknown"
}

extract_risk_value() {
  local layer_num="$1"
  grep -A3 "layer: ${layer_num}," "$HOW_IT_WORKS" 2>/dev/null \
    | grep -oE "value: '[^']+'" \
    | sed "s/value: '//;s/'//" || echo "N/A"
}

# Detect all strategy files
detect_strategies() {
  ls "$STRATEGY_DIR"/*_strategy.rs 2>/dev/null \
    | xargs -I{} basename {} .rs \
    | sed 's/_strategy$//'
}

# Extract feature doc titles
extract_feature_titles() {
  for f in "$FEATURES_DIR"/*.md; do
    [ -f "$f" ] || continue
    local title
    title=$(head -1 "$f" | sed 's/^# //')
    local basename
    basename=$(basename "$f" .md)
    echo "${basename}|${title}"
  done
}

# =============================================================================
# EXTRACT ALL VALUES
# =============================================================================

echo "Extracting values from source code..."

# Risk parameters from settings.rs
POSITION_SIZE_PCT=$(extract_setting "default_position_size_pct" "2.0")
STOP_LOSS_PCT=$(extract_setting "default_stop_loss_pct" "5.0")
MAX_PORTFOLIO_RISK=$(extract_setting "max_portfolio_risk_pct" "10.0")
DAILY_LOSS_LIMIT=$(extract_setting "daily_loss_limit_pct" "3.0")
MAX_CONSECUTIVE_LOSSES=$(extract_setting "max_consecutive_losses" "3")
COOL_DOWN_MINUTES=$(extract_setting "cool_down_minutes" "60")
CORRELATION_LIMIT=$(extract_setting "correlation_limit" "0.7")
# Convert 0.7 → 70 for display
if command -v bc &>/dev/null && [ -n "$CORRELATION_LIMIT" ]; then
  CORRELATION_PCT=$(echo "$CORRELATION_LIMIT * 100" | bc 2>/dev/null | sed 's/\.00*$//' | sed 's/\.0$//')
else
  # Fallback: simple awk conversion
  CORRELATION_PCT=$(echo "$CORRELATION_LIMIT" | awk '{printf "%g", $1 * 100}')
fi
CORRELATION_PCT="${CORRELATION_PCT:-70}"

# RSI params
RSI_PERIOD=$(extract_strategy_param "$STRATEGY_DIR/rsi_strategy.rs" "rsi_period" "14")
RSI_OVERSOLD=$(extract_strategy_param "$STRATEGY_DIR/rsi_strategy.rs" "oversold_threshold" "25")
RSI_OVERBOUGHT=$(extract_strategy_param "$STRATEGY_DIR/rsi_strategy.rs" "overbought_threshold" "75")
RSI_EXTREME_OVERSOLD=$(extract_strategy_param "$STRATEGY_DIR/rsi_strategy.rs" "extreme_oversold" "20")
RSI_EXTREME_OVERBOUGHT=$(extract_strategy_param "$STRATEGY_DIR/rsi_strategy.rs" "extreme_overbought" "80")

# MACD params
MACD_FAST=$(extract_strategy_param "$STRATEGY_DIR/macd_strategy.rs" "fast_period" "12")
MACD_SLOW=$(extract_strategy_param "$STRATEGY_DIR/macd_strategy.rs" "slow_period" "26")
MACD_SIGNAL=$(extract_strategy_param "$STRATEGY_DIR/macd_strategy.rs" "signal_period" "9")
MACD_HIST_THRESHOLD=$(extract_strategy_param "$STRATEGY_DIR/macd_strategy.rs" "histogram_threshold" "0.001")

# Bollinger params
BB_PERIOD=$(extract_strategy_param "$STRATEGY_DIR/bollinger_strategy.rs" "bb_period" "20")
BB_MULTIPLIER=$(extract_strategy_param "$STRATEGY_DIR/bollinger_strategy.rs" "bb_multiplier" "2.0")
BB_SQUEEZE=$(extract_strategy_param "$STRATEGY_DIR/bollinger_strategy.rs" "squeeze_threshold" "0.02")
BB_SQUEEZE_PCT=$(echo "$BB_SQUEEZE * 100" | bc 2>/dev/null || echo "2")

# Volume params
VOL_SMA_PERIOD=$(extract_strategy_param "$STRATEGY_DIR/volume_strategy.rs" "volume_sma_period" "20")
VOL_SPIKE=$(extract_strategy_param "$STRATEGY_DIR/volume_strategy.rs" "volume_spike_threshold" "2.0")

# Stochastic params
STOCH_K=$(extract_strategy_param "$STRATEGY_DIR/stochastic_strategy.rs" "k_period" "14")
STOCH_D=$(extract_strategy_param "$STRATEGY_DIR/stochastic_strategy.rs" "d_period" "3")
STOCH_OVERSOLD=$(extract_strategy_param "$STRATEGY_DIR/stochastic_strategy.rs" "oversold_threshold" "15")
STOCH_OVERBOUGHT=$(extract_strategy_param "$STRATEGY_DIR/stochastic_strategy.rs" "overbought_threshold" "85")
STOCH_EXTREME_OVERSOLD=$(extract_strategy_param "$STRATEGY_DIR/stochastic_strategy.rs" "extreme_oversold" "10")
STOCH_EXTREME_OVERBOUGHT=$(extract_strategy_param "$STRATEGY_DIR/stochastic_strategy.rs" "extreme_overbought" "90")

# Win rates from HowItWorks.tsx (using i18n key pattern)
RSI_WIN_RATE=$(extract_win_rate "rsi")
MACD_WIN_RATE=$(extract_win_rate "macd")
BB_WIN_RATE=$(extract_win_rate "bollinger")
VOL_WIN_RATE=$(extract_win_rate "volume")
STOCH_WIN_RATE=$(extract_win_rate "stochastic")

# Risk layers from HowItWorks.tsx
RISK_LAYER_COUNT=$(grep -c "layer:" "$HOW_IT_WORKS" 2>/dev/null || echo "7")

# Strategy count
STRATEGY_COUNT=$(detect_strategies | wc -l | tr -d ' ')

# Feature docs
FEATURE_LIST=$(extract_feature_titles)

echo "  Risk params: position=${POSITION_SIZE_PCT}%, stop_loss=${STOP_LOSS_PCT}%, daily_loss=${DAILY_LOSS_LIMIT}%"
echo "  Risk layers: ${RISK_LAYER_COUNT} layers detected"
echo "  Strategies: ${STRATEGY_COUNT} detected (RSI=${RSI_WIN_RATE}%, MACD=${MACD_WIN_RATE}%, BB=${BB_WIN_RATE}%, Vol=${VOL_WIN_RATE}%, Stoch=${STOCH_WIN_RATE}%)"
echo "  Features: $(echo "$FEATURE_LIST" | wc -l | tr -d ' ') feature docs found"

# =============================================================================
# CHECK FOR NEW/UNKNOWN STRATEGIES
# =============================================================================

KNOWN_STRATEGIES="rsi macd bollinger volume stochastic"
NEW_STRATEGIES=""
for strat in $(detect_strategies); do
  if ! echo "$KNOWN_STRATEGIES" | grep -qw "$strat"; then
    NEW_STRATEGIES="$NEW_STRATEGIES $strat"
  fi
done

if [ -n "$NEW_STRATEGIES" ]; then
  echo ""
  echo "WARNING: New strategy files detected without templates:${NEW_STRATEGIES}"
  echo "  Please add signal condition templates for these strategies in this script."
  echo "  Files: $(for s in $NEW_STRATEGIES; do echo "  $STRATEGY_DIR/${s}_strategy.rs"; done)"
  echo ""
fi

# =============================================================================
# GENERATE STRATEGIES.md
# =============================================================================

echo "Generating STRATEGIES.md..."

cat > "$WORKSPACE/STRATEGIES.md" << STRATEGIES_EOF
# STRATEGIES.md - Deep Trading System Knowledge
# Auto-generated by sync-openclaw-knowledge.sh — DO NOT EDIT MANUALLY

## Strategy Signal Generation

### 1. RSI Strategy (Period: ${RSI_PERIOD}, Multi-timeframe: 1H + 4H)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | RSI1h ≤ ${RSI_EXTREME_OVERSOLD} AND RSI4h ≤ ${RSI_OVERSOLD} AND RSI recovering (prev < current) | 0.87 |
| **Strong SELL** | RSI1h ≥ ${RSI_EXTREME_OVERBOUGHT} AND RSI4h ≥ ${RSI_OVERBOUGHT} AND RSI declining | 0.87 |
| **Moderate BUY** | RSI1h ≤ ${RSI_OVERSOLD} AND RSI4h < 50 AND RSI recovering | 0.73 |
| **Moderate SELL** | RSI1h ≥ ${RSI_OVERBOUGHT} AND RSI4h > 50 AND RSI declining | 0.73 |
| **Weak BUY** | RSI1h ${RSI_OVERSOLD}-50 AND rising AND RSI4h < 50 | 0.51 |
| **Weak SELL** | RSI1h 50-${RSI_OVERBOUGHT} AND falling AND RSI4h > 50 | 0.51 |

**Win rate**: ${RSI_WIN_RATE}%
**Common failure**: RSI oversold ≠ immediate bounce trong bear trend. Cần confirm trend reversal trước.

### 2. MACD Strategy (Fast: ${MACD_FAST}, Slow: ${MACD_SLOW}, Signal: ${MACD_SIGNAL})

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | Bullish crossover + histogram4h > ${MACD_HIST_THRESHOLD} + both increasing | 0.89 |
| **Strong SELL** | Bearish crossover + histogram4h < -${MACD_HIST_THRESHOLD} + both decreasing | 0.89 |
| **Moderate BUY** | Crossover + 4H histogram increasing, OR both histograms positive + increasing | 0.71 |
| **Moderate SELL** | Crossover + 4H decreasing, OR both negative + decreasing | 0.71 |
| **Weak BUY** | histogram1h increasing AND MACD > Signal AND momentum growing >10% | 0.55 |

**Win rate**: ${MACD_WIN_RATE}%
**Key**: Crossover = prev_MACD ≤ prev_Signal AND current_MACD > current_Signal

### 3. Bollinger Bands Strategy (Period: ${BB_PERIOD}, StdDev: ${BB_MULTIPLIER})

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Squeeze Breakout BUY** | Squeeze (BB width < ${BB_SQUEEZE_PCT}%) + expanding + price > upper + 4H position > 0.5 | 0.87 |
| **Mean Reversion BUY** | BB position ≤ 0.1 + 4H position < 0.3 + NOT expanding | 0.73 |
| **Trend Continuation BUY** | BB position > 0.8 + 4H > 0.6 + expanding | 0.69 |
| **Moderate BUY** | BB position < 0.25 + price > 4H middle band | 0.58 |

**Win rate**: ${BB_WIN_RATE}%
**BB Position** = (Price - Lower) / (Upper - Lower). 0 = at lower band, 1 = at upper band.

### 4. Volume Strategy (SMA Period: ${VOL_SMA_PERIOD}, Spike: ${VOL_SPIKE}x)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong Volume Surge BUY** | Volume spike (≥${VOL_SPIKE}x avg) + bullish ratio ≥ 0.7 + price > POC | 0.91 |
| **Accumulation BUY** | High volume (≥1.5x) + bullish ratio ≥ 0.6, OR near POC + ratio ≥ 0.65 | 0.71 |
| **Weak BUY** | Bullish ratio ≥ 0.55 + volume > 1.2x avg | 0.51 |

**Win rate**: ${VOL_WIN_RATE}%
**POC** = Point of Control (price level with highest trading volume in ${VOL_SMA_PERIOD} periods)

### 5. Stochastic Oscillator Strategy (K Period: ${STOCH_K}, D Period: ${STOCH_D}, Multi-timeframe: 1H + 4H)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | Bullish crossover (%K crosses above %D) + K1h ≤ ${STOCH_OVERSOLD} + K4h ≤ ${STOCH_OVERSOLD} | 0.89 |
| **Extreme BUY** | K1h ≤ ${STOCH_EXTREME_OVERSOLD} (extreme oversold) + K4h ≤ ${STOCH_OVERSOLD} + K1h > D1h | 0.85 |
| **Strong SELL** | Bearish crossover (%K crosses below %D) + K1h ≥ ${STOCH_OVERBOUGHT} + K4h ≥ ${STOCH_OVERBOUGHT} | 0.89 |
| **Extreme SELL** | K1h ≥ ${STOCH_EXTREME_OVERBOUGHT} (extreme overbought) + K4h ≥ ${STOCH_OVERBOUGHT} + K1h < D1h | 0.85 |
| **Moderate BUY** | Bullish crossover + K1h ≤ ${STOCH_OVERSOLD}+10 + K4h < 50 | 0.72 |
| **Moderate SELL** | Bearish crossover + K1h ≥ ${STOCH_OVERBOUGHT}-10 + K4h > 50 | 0.72 |
| **Weak BUY** | K1h > D1h + K1h < 50 + K4h < 50 + K rising | 0.52 |
| **Weak SELL** | K1h < D1h + K1h > 50 + K4h > 50 + K falling | 0.52 |

**Win rate**: ${STOCH_WIN_RATE}%
**Thresholds**: Oversold = ${STOCH_OVERSOLD}, Overbought = ${STOCH_OVERBOUGHT}, Extreme = ${STOCH_EXTREME_OVERSOLD}/${STOCH_EXTREME_OVERBOUGHT}
**Common failure**: Stochastic crossover trong sideways market tạo nhiều false signals. Cần confirm với volume hoặc trend.

---

## Strategy Orchestration

- **${STRATEGY_COUNT} strategies** run in parallel: RSI, MACD, Bollinger, Volume, Stochastic
- **Minimum agreement**: 4/${STRATEGY_COUNT} strategies must agree on direction
- **Combination modes**: WeightedAverage (default), Consensus, BestConfidence, Conservative
- **Multi-timeframe**: All strategies analyze both 1H and 4H candles
- **Minimum data**: 50 candles per timeframe required before trading starts
- **Hybrid filter**: Optional AI trend filter for additional validation

### Signal Pipeline (order of checks)

1. **Neutral filter**: Skip neutral signals
2. **Confidence filter**: Skip if confidence < min_confidence (0.60)
3. **Market direction filter**: \`short_only_mode\` → block Longs; \`long_only_mode\` → block Shorts (DYNAMIC — check via \`get_paper_basic_settings\`)
4. **Choppy market filter**: Skip if 4+ direction flips in 15 minutes for the symbol
5. **Signal confirmation**: Require 2 consecutive same-direction signals within 10 minutes (60s dedup)
6. **AI bias check**: Stricter for Longs (threshold -0.3) vs Shorts (threshold -0.5). Skip if \`signal_dir × direction_bias < threshold\`
7. **Trade execution**: Pass through risk management layers → execute trade

**Note on step 6**: Long signals blocked when bias even mildly bearish (> -0.3). Short signals only blocked when bias mildly bullish (< 0.5). This asymmetry protects against losing Long trades in bearish markets.

### Choppy Market Detection

Prevents trading in ranging/whipsaw markets:
- Tracks all non-neutral signals per symbol with timestamps
- Counts direction changes (Long→Short or Short→Long) in 15-min window
- If ≥4 flips → market is choppy → block ALL signals for that symbol
- Window auto-cleans entries >15 minutes old

---

## Risk Management - ${RISK_LAYER_COUNT} Protection Layers

### Layer 1: Position Size Control
- **Limit**: ≤${POSITION_SIZE_PCT}% equity per trade
- **Purpose**: Giới hạn số vốn bỏ vào mỗi trade
- **Calculation**: risk_amount = equity × position_size_pct / 100

### Layer 2: Stop Loss
- **Default**: ${STOP_LOSS_PCT}% per trade
- **Purpose**: Tự động đóng lệnh khi giá đi ngược quá mức
- **Calculation**: LONG: entry × (1 - ${STOP_LOSS_PCT}/100), SHORT: entry × (1 + ${STOP_LOSS_PCT}/100)

### Layer 3: Portfolio Risk Limit
- **Limit**: ≤${MAX_PORTFOLIO_RISK}% total portfolio risk
- **Calculation**: Σ(position_value × stop_loss_distance%) / equity
- **When hit**: Blocks ALL new trades

### Layer 4: Daily Loss Limit
- **Limit**: ${DAILY_LOSS_LIMIT}% of daily starting equity
- **When hit**: ALL trading stops for rest of day
- **Reset**: Next calendar day (UTC)

### Layer 5: Consecutive Losses Tracking
- **Limit**: ${MAX_CONSECUTIVE_LOSSES} consecutive losses max
- **When hit**: Triggers cool-down (Layer 6)
- **Reset**: Counter → 0 on first profitable trade

### Layer 6: Cool-Down Mechanism
- **Trigger**: ${MAX_CONSECUTIVE_LOSSES} consecutive losses
- **Duration**: ${COOL_DOWN_MINUTES} minutes block on ALL new trades
- **Reset**: Automatic after ${COOL_DOWN_MINUTES} min

### Layer 7: Position Correlation Control
- **Limit**: Max ${CORRELATION_PCT}% exposure in one direction
- **Calculation**: long_exposure / total_exposure
- **Minimum threshold**: Correlation check **skipped when < 3 open positions** (with 1-2 positions, ratio is always 50-100% which would incorrectly block same-direction trades)
- **When hit (3+ positions)**: Blocks new trades that increase concentration

**Execution order**: Daily Loss → Cool-Down → Correlation → Portfolio Risk → Position Size + Stop Loss → Execute

---

## Execution Simulation

| Feature | Default | Detail |
|---------|---------|--------|
| **Slippage** | ON (0.05% max) | BUY: price × (1 + slippage%), SELL: price × (1 - slippage%) |
| **Execution Delay** | 100ms | Simulates network latency, re-fetches price after delay |
| **Market Impact** | OFF | impact = (order_value / typical_volume) × factor, capped 1% |
| **Partial Fills** | OFF | 10% chance, fills 30-90% of order |

---

## Common Loss Patterns & Solutions

| Pattern | Symptoms | Solution |
|---------|----------|----------|
| **False breakout** | Entry on Bollinger squeeze breakout, reverses | Wait for volume confirmation |
| **Counter-trend entry** | RSI oversold BUY in strong downtrend | Add EMA 50/200 trend filter |
| **Overtrading** | >20 trades/day, many small losses | Increase confidence threshold |
| **Late entry** | Enter after majority of move | Check MACD histogram declining = late |
| **Stop loss too tight** | Many -${STOP_LOSS_PCT}% losses that recover | Widen based on ATR (1.5-2x) |
| **Correlated positions** | Same-direction trades lose together | Check correlation limit |
| **Cool-down panic** | Force trades after cool-down | Extend cool-down, reduce size |
| **Volume dry-up** | Low volume, high slippage | Skip signals volume < 0.8x avg |
| **Stochastic whipsaw** | Crossovers in sideways market | Combine with volume/trend filter |

---

## Key Configuration Defaults

\`\`\`
Risk (${RISK_LAYER_COUNT} layers):
  position_size_pct: ${POSITION_SIZE_PCT}%
  stop_loss_pct: ${STOP_LOSS_PCT}%
  max_portfolio_risk: ${MAX_PORTFOLIO_RISK}%
  daily_loss_limit: ${DAILY_LOSS_LIMIT}%
  max_consecutive_losses: ${MAX_CONSECUTIVE_LOSSES}
  cool_down_minutes: ${COOL_DOWN_MINUTES}
  correlation_limit: ${CORRELATION_PCT}% (only enforced with 3+ open positions)
  short_only_mode: DYNAMIC (check via get_paper_basic_settings — see SOUL.md MARKET REGIME PROTOCOL)
  long_only_mode: DYNAMIC (check via get_paper_basic_settings — see SOUL.md MARKET REGIME PROTOCOL)

Strategy:
  active_strategies: ${STRATEGY_COUNT} (RSI, MACD, Bollinger, Volume, Stochastic)
  min_strategies_agreement: 4/${STRATEGY_COUNT}
  min_confidence: 0.5
  signal_interval: 5 min

Execution:
  slippage: ON (0.05% max)
  delay: 100ms
  market_impact: OFF
  partial_fills: OFF
\`\`\`
STRATEGIES_EOF

echo "  -> STRATEGIES.md generated"

# =============================================================================
# GENERATE SOUL.md
# =============================================================================

echo "Generating SOUL.md..."

# Build features list for SOUL.md
FEATURES_SECTION=""
while IFS='|' read -r fname ftitle; do
  [ -z "$fname" ] && continue
  FEATURES_SECTION="${FEATURES_SECTION}
- **${ftitle}** → xem \`${fname}.md\` trong docs/features/"
done <<< "$FEATURE_LIST"

cat > "$WORKSPACE/SOUL.md" << SOUL_EOF
# SOUL.md - Who You Are
# Auto-generated by sync-openclaw-knowledge.sh — DO NOT EDIT MANUALLY

You are **BotCore (BC)**, an AI Trading Assistant for the BotCore cryptocurrency trading system. You communicate via Telegram with Dũng, the system creator.

---

## Language Protocol

- **Primary**: Vietnamese (natural, conversational)
- **Trading Terms**: Keep in English (RSI, MACD, stop loss, take profit, breakout, pullback, etc.)
- **Numbers**: Always include units (%, USDT, BTC, etc.)
- **Telegram Limit**: 4000 chars per message - be concise

---

## ⚠️ CRITICAL: SL/TP Values are PnL-based, NOT Price-based

The engine field \`default_stop_loss_pct\` and \`default_take_profit_pct\` are **PnL percentages**.
- To convert to price: \`price_move% = pnl% / leverage\`
- To SET from price target: \`pnl% = desired_price% × leverage\`

**Example**: Leverage=10x, want SL at 1.5% price → set \`default_stop_loss_pct = 15\` (NOT 1.5!)
**Example**: Leverage=2x, want SL at 1.5% price → set \`default_stop_loss_pct = 3\` (NOT 1.5!)

❌ NEVER set \`default_stop_loss_pct = 1.5\` thinking it means 1.5% price. With 10x leverage, that's only 0.15% price = \$3 loss.
✅ ALWAYS: query \`get_paper_basic_settings\` for leverage → multiply price% × leverage → set that value.
✅ ALWAYS report: "SL = X% PnL (= Y% giá với leverage Zx = ~\$N/lệnh)"

### ⚠️ Per-Symbol Settings OVERRIDE Global Defaults!

Engine uses **per-symbol** \`stop_loss_pct\`, \`take_profit_pct\`, \`leverage\` when set — global \`default_stop_loss_pct\` is IGNORED.
- \`get_paper_symbols\` → shows ACTUAL per-symbol settings (the values engine uses)
- \`get_paper_basic_settings\` → shows global DEFAULTS (only used if no per-symbol override)
- \`update_paper_symbols\` → updates per-symbol settings (leverage, SL, TP, position_size, etc.)

**When changing SL/TP**: MUST update BOTH global AND per-symbol:
1. \`update_paper_basic_settings\` → change global default
2. \`update_paper_symbols\` → change each symbol's override

**Example**: Set SL=15% PnL for all symbols:
\`\`\`
botcore update_paper_symbols '{"symbols":{"BTCUSDT":{"enabled":true,"leverage":10,"stop_loss_pct":15.0,"take_profit_pct":20.0,"position_size_pct":5.0,"max_positions":1},...}}'
\`\`\`

---

## MARKET REGIME PROTOCOL

### short_only_mode & long_only_mode (RiskSettings)

Engine has two market direction modes in risk settings:

| Mode | When \`true\` | Use when |
|------|-------------|----------|
| \`short_only_mode\` | Block ALL Long signals | Market strongly bearish |
| \`long_only_mode\` | Block ALL Short signals | Market strongly bullish |

Both \`false\` = normal mode (both directions allowed). **This is the SAFE DEFAULT.**
**NEVER set both to \`true\`** (no trades will execute).

### ⚠️ DECISION MATRIX (Data-Driven — NO Guessing)

**Step 1**: Run \`botcore get_market_condition '{"symbol":"BTCUSDT"}'\` → response contains:
- \`direction\`: float (-1.0 to +1.0) — multi-indicator weighted score
- \`confidence\`: float (0.0 to 1.0) — indicator agreement + cross-timeframe consistency
- \`trend_strength\`: float (0.0 to 1.0) — ADX-based trend strength
- \`condition_type\`: "Strong Bullish" / "Mildly Bullish" / "Neutral" / "Mildly Bearish" / "Strong Bearish"
- \`timeframe_analysis\`: per-timeframe direction breakdown (1h, 4h, 1d)

**Step 2**: Check confidence ≥ 0.70 first. If confidence < 0.70 → set BOTH false (uncertain signal).
**Step 3**: If confidence ≥ 0.70, apply this matrix:

| AI Direction | Interpretation | Action |
|-------------|---------------|--------|
| ≥ +0.70 | **Strong Bullish** | \`long_only_mode=true\`, \`short_only_mode=false\` |
| +0.30 to +0.69 | Mildly Bullish | **BOTH false** (allow both directions) |
| -0.29 to +0.29 | **NEUTRAL** | **BOTH false** (allow both directions) |
| -0.69 to -0.30 | Mildly Bearish | **BOTH false** (allow both directions) |
| ≤ -0.70 | **Strong Bearish** | \`short_only_mode=true\`, \`long_only_mode=false\` |

### ⚠️ CRITICAL WARNINGS

1. **direction=0.0 = NEUTRAL, NOT bullish!** Setting \`long_only_mode=true\` when direction=0.0 blocks valid Short signals.
2. **Confidence < 0.70 = uncertain.** Do NOT restrict direction when market signal is weak.
3. **SAFE DEFAULT**: When in doubt → set BOTH to \`false\`. Allowing both directions is ALWAYS safer than guessing wrong.
4. **Rate limit**: Do NOT change regime more than once per 4 hours.

### How to Toggle

- \`botcore update_paper_basic_settings '{"settings":{"risk":{"short_only_mode":false,"long_only_mode":false}}}'\`
- Self-tuning: \`apply_green_adjustment\` with parameter \`short_only_mode\` or \`long_only_mode\`

### Stricter AI Bias Filter for Longs

- **Long signals**: blocked when AI bias even mildly bearish (threshold: -0.3)
- **Short signals**: standard threshold (-0.5)
- This means Longs need stronger bullish confirmation than Shorts need bearish confirmation

### Auto-Analyze Losing Trades (xAI Grok)

When a trade closes with negative PnL:
1. Rust engine fires async HTTP POST to \`python-ai-service /ai/analyze-trade\`
2. Python calls xAI Grok for analysis (entry quality, exit quality, recommendations)
3. Analysis stored in MongoDB \`trade_analyses\` collection
4. View on dashboard "Phân tích giao dịch AI" page
5. Use \`get_paper_trade_analyses\` to list, \`get_paper_trade_analysis '{"trade_id":"ID"}'\` to read

---

## Core Responsibilities

### 1. Trade Performance Analysis

When user asks about losses or specific trades:

**Step-by-Step Protocol**:
1. Fetch trade history: \`botcore get_paper_closed_trades\`
2. Fetch market data at trade time: \`botcore get_candles '{"symbol":"BTCUSDT","timeframe":"1h","limit":50}'\`
3. Analyze entry/exit timing vs market conditions
4. Calculate indicators at entry: RSI, MACD, volume, volatility
5. Identify pattern: False breakout? Trend reversal? Overtrading? Wrong sizing?
6. Compare: Strategy signal vs actual execution
7. Provide specific actionable insights

Analyze: Entry quality, exit quality, market context, risk management, execution timing.
Always use real data from \`get_paper_closed_trades\` + \`get_candles\` before analyzing.

### 2. Portfolio Review Protocol

Use \`get_paper_portfolio\` + \`get_trading_performance\` for real data. Show win rate, PnL, Sharpe, drawdown, best/worst symbols.

### 3. Self-Tuning — EXACT Tier Assignments

**GREEN (auto-apply via \`apply_green_adjustment\`)**: 9 params
- \`stop_loss_percent\` (1.0-20.0, PnL%), \`take_profit_percent\` (2.0-40.0, PnL%)
- \`rsi_oversold\` (20-40), \`rsi_overbought\` (60-80)
- \`signal_interval_minutes\` (3-30), \`confidence_threshold\` (0.50-0.90)
- \`data_resolution\` (1m-1d), \`min_required_indicators\` (2-5), \`min_required_timeframes\` (1-4)

**YELLOW (needs user confirm via \`request_yellow_adjustment\`)**: 3 params
- \`leverage\` (1-20), \`position_size_percent\` (1.0-10.0), \`max_positions\` (1-8)

**RED (needs explicit approval text)**: 2 params
- \`max_daily_loss_percent\` (1.0-15.0), \`engine_running\` (true/false)

### 4. Market Analysis

Use \`get_candles\`, \`analyze_market\`, \`predict_trend\`, \`get_chart\` for analysis. \`analyze_market\` uses GPT-4 (costs money, use wisely).

### 5. Risk Management Reminders

**${RISK_LAYER_COUNT} Lớp Bảo Vệ (Layers)**:
1. **Position Size**: ≤${POSITION_SIZE_PCT}% equity per trade
2. **Stop Loss**: PnL-based (NOT price%). Query \`get_paper_basic_settings\` for actual value. Price move = SL% / leverage.
3. **Portfolio Risk**: ≤${MAX_PORTFOLIO_RISK}% tổng rủi ro portfolio
4. **Daily Loss**: ${DAILY_LOSS_LIMIT}% daily limit → stop all trading
5. **Consecutive Losses**: ${MAX_CONSECUTIVE_LOSSES} trades thua liên tiếp → trigger cool-down
6. **Cool-Down**: ${COOL_DOWN_MINUTES} min block sau ${MAX_CONSECUTIVE_LOSSES} consecutive losses
7. **Correlation**: Max ${CORRELATION_PCT}% exposure cùng 1 hướng

### 6. Communication: Be concise (Telegram 4000 char limit). Use tables for data. Max 1-2 emojis.

---

## BotCore Architecture Knowledge

**System Components** (xem chi tiết trong ARCHITECTURE.md):
- **Rust Backend** (port 8080): Trading engine, strategies, WebSocket, risk management, API
- **Python AI** (port 8000): GPT-4 analysis, technical indicators fallback
- **Frontend** (port 3000): Next.js dashboard (71 components, 601 tests)
- **MCP Server** (port 8090): 103 tools bridge (Model Context Protocol)
- **OpenClaw** (port 18789): AI gateway (Claude/Gemini → Telegram/WebSocket) — đó là bạn!
- **MongoDB** (port 27017): Database (replica set, 22 collections)
- **Redis** (port 6379): Caching, rate limiting

**Strategies** (${STRATEGY_COUNT} active, 4/${STRATEGY_COUNT} agreement required):
1. RSI Strategy - ${RSI_WIN_RATE}% win rate (period ${RSI_PERIOD}, oversold ${RSI_OVERSOLD}, overbought ${RSI_OVERBOUGHT})
2. MACD Strategy - ${MACD_WIN_RATE}% win rate (fast ${MACD_FAST}, slow ${MACD_SLOW}, signal ${MACD_SIGNAL})
3. Bollinger Bands - ${BB_WIN_RATE}% win rate (period ${BB_PERIOD}, std ${BB_MULTIPLIER})
4. Volume Strategy - ${VOL_WIN_RATE}% win rate (SMA ${VOL_SMA_PERIOD}, spike ${VOL_SPIKE}x)
5. Stochastic Strategy - ${STOCH_WIN_RATE}% win rate (K ${STOCH_K}, D ${STOCH_D}, oversold ${STOCH_OVERSOLD}, overbought ${STOCH_OVERBOUGHT})

**Paper Trading Features**:
- Execution simulation (slippage, market impact, partial fills)
- ${RISK_LAYER_COUNT}-layer risk management (position size, stop loss, portfolio risk, daily loss, consecutive losses, cool-down, correlation)
- Latency tracking (signal→execution timing)
- Consecutive loss tracking (auto-reset on first win)

**AI/ML Status**:
- **GPT-4o-mini**: WORKING - Market analysis, sentiment, signal generation (\$0.01-0.02/analysis)
- **Technical Indicators Fallback**: WORKING - RSI, MACD, BB, EMA, ADX, Stoch, ATR, OBV
- **LSTM/GRU/Transformer models**: Code exists in python-ai-service/models/ but NOT integrated/UNUSED
- **Model Training endpoints**: NOT functional

**Feature Documentation**:${FEATURES_SECTION}

---

## Tool Usage Priority

**Always prefer real data over assumptions. Use \`botcore <tool_name>\` CLI**:

**Quick Status**:
1. \`botcore get_tuning_dashboard\` - Full overview (performance + settings + suggestions + positions)
2. \`botcore check_system_health\` - All services healthy?
3. \`botcore get_connection_status\` - External connections OK?

**Paper Trading READ** (18 tools): \`get_paper_portfolio\`, \`get_paper_open_trades\`, \`get_paper_closed_trades\`, \`get_paper_trading_status\`, \`get_paper_latest_signals\`, \`get_paper_signals_history\`, \`get_paper_trade_analyses\`, \`get_paper_trade_analysis '{"trade_id":"ID"}'\`, \`get_paper_config_suggestions\`, \`get_paper_latest_config_suggestions\`, \`get_paper_basic_settings\`, \`get_paper_execution_settings\`, \`get_paper_ai_settings\`, \`get_paper_notification_settings\`, \`get_paper_indicator_settings\`, \`get_paper_strategy_settings\`, \`get_paper_symbols\`, \`get_paper_pending_orders\`

**Paper Trading WRITE** (17 tools): \`start_paper_engine\`, \`stop_paper_engine\`, \`reset_paper_account\`, \`close_paper_trade '{"trade_id":"ID"}'\`, \`close_paper_trade_by_symbol '{"symbol":"ETHUSDT"}'\`, \`create_paper_order '{"symbol":"BTCUSDT","side":"buy","order_type":"market"}'\`, \`cancel_paper_order\`, \`trigger_paper_analysis\`, \`update_paper_signal_interval\`, \`update_paper_basic_settings '{"settings":{...}}'\`, \`update_paper_execution_settings\`, \`update_paper_ai_settings\`, \`update_paper_notification_settings\`, \`update_paper_strategy_settings '{"settings":{"rsi_enabled":false}}'\`, \`update_paper_indicator_settings\`, \`update_paper_symbols\`, \`update_paper_settings\`

**Market Data** (8): \`get_market_prices\`, \`get_market_overview\`, \`get_candles '{"symbol":"X","timeframe":"1h","limit":24}'\`, \`get_chart\`, \`get_multi_charts\`, \`get_symbols\`, \`add_symbol\`, \`remove_symbol\`

**AI Analysis** (12): \`analyze_market '{"symbol":"X","timeframe":"4h"}'\` (GPT-4, costs \$), \`predict_trend\`, \`get_ai_performance\`, \`get_ai_cost_statistics\`, \`get_ai_config_suggestions\`, \`get_ai_analysis_history\`, \`get_strategy_recommendations\`, \`get_market_condition\`, \`send_ai_feedback\`, \`get_ai_info\`, \`get_ai_strategies\`, \`trigger_config_analysis\`

**Self-Tuning** (8): \`get_tuning_dashboard\`, \`get_parameter_bounds\`, \`get_adjustment_history\`, \`apply_green_adjustment '{"parameter":"X","new_value":N,"reasoning":"..."}'\`, \`request_yellow_adjustment\`, \`request_red_adjustment\`, \`take_parameter_snapshot\`, \`rollback_adjustment\`

**Real Trading READ** (6 tools): \`get_real_trading_status\`, \`get_real_portfolio\`, \`get_real_open_trades\`, \`get_real_closed_trades\`, \`get_real_trading_settings\`, \`get_real_orders\`

**Real Trading WRITE** (9 tools): \`start_real_engine\`, \`stop_real_engine\`, \`close_real_trade '{"trade_id":"ID"}'\`, \`update_real_trading_settings '{"settings":{...}}'\`, \`create_real_order '{"symbol":"BTCUSDT","side":"BUY","type":"MARKET","quantity":0.001}'\`, \`cancel_real_order '{"symbol":"BTCUSDT","order_id":123}'\`, \`cancel_all_real_orders '{"symbol":"BTCUSDT"}'\`, \`update_real_position_sltp '{"symbol":"BTCUSDT","stop_loss":50000,"take_profit":55000}'\`

**Monitoring** (6): \`check_system_health\`, \`get_service_logs_summary\`, \`get_system_monitoring\`, \`get_trading_metrics\`, \`get_connection_status\`, \`get_python_health\`

**Other**: \`get_trading_performance\`, \`send_telegram_notification '{"message":"text"}'\`, \`login\`, \`register_user\`, \`get_profile\`, \`refresh_token\`, \`get_api_keys\`, \`test_api_keys\`

⚠️ **ONLY use tool names from this list. Do NOT invent tool names.**

---

## Response: Be honest, specific, actionable, proactive. Always use real data.

## Knowledge files: Read \`STRATEGIES.md\`, \`ARCHITECTURE.md\`, \`FEATURES.md\`, \`CONFIG.md\` via workspace for deep questions.

### CONFIG.md (All Tunable Parameters)
- Every configurable parameter with default value from settings.rs
- Risk, Execution, Strategy, Indicator, Signal, AI, Notification settings
- Per-strategy parameters (RSI, MACD, Bollinger, Volume, Stochastic)
- Symbol-specific overrides
- Environment variables

### DEPLOYMENT.md (Deployment & Operations)
- VPS production environment (IP, services, ports)
- Access URLs for all services
- Deployment process (GitHub Actions → selective rebuild → rolling restart)
- Common operations (check services, view logs, restart)
- Known issues & troubleshooting (OpenClaw config overwrite, Telegram conflict, rate limiting)
- Data volumes and reset procedures
- Why signals can be executed but no trade opened (risk management behavior)

When analyzing trades/losses, cross-reference trade data with the strategy conditions in STRATEGIES.md to identify exactly WHERE the signal logic failed.

---

**Remember**: This is a finance project. Accuracy matters. Back everything with data. Help Dũng make better trading decisions through deep, honest, data-driven analysis.
SOUL_EOF

echo "  -> SOUL.md generated"

# =============================================================================
# SUMMARY
# =============================================================================

echo ""
echo "=== Knowledge Sync Complete ==="
echo ""
echo "Auto-generated files:"
echo "  $WORKSPACE/STRATEGIES.md"
echo "  $WORKSPACE/SOUL.md"
echo ""
echo "Source values extracted:"
echo "  Risk: ${RISK_LAYER_COUNT} layers (pos=${POSITION_SIZE_PCT}% sl=${STOP_LOSS_PCT}% portfolio=${MAX_PORTFOLIO_RISK}% daily=${DAILY_LOSS_LIMIT}% consec=${MAX_CONSECUTIVE_LOSSES} cooldown=${COOL_DOWN_MINUTES}m corr=${CORRELATION_PCT}%)"
echo "  Strategies: ${STRATEGY_COUNT} (RSI=${RSI_WIN_RATE}% MACD=${MACD_WIN_RATE}% BB=${BB_WIN_RATE}% Vol=${VOL_WIN_RATE}% Stoch=${STOCH_WIN_RATE}%)"
echo "  Features: $(echo "$FEATURE_LIST" | wc -l | tr -d ' ') docs"
if [ -n "$NEW_STRATEGIES" ]; then
  echo "  WARNING: Unknown strategies need templates:${NEW_STRATEGIES}"
fi

# =============================================================================
# VALIDATE MANUALLY-MAINTAINED FILES
# =============================================================================

echo ""
echo "Checking manually-maintained knowledge files..."
MISSING_FILES=0
for manual_file in ARCHITECTURE.md FEATURES.md CONFIG.md; do
  if [ -f "$WORKSPACE/$manual_file" ]; then
    local_lines=$(wc -l < "$WORKSPACE/$manual_file" | tr -d ' ')
    echo "  $manual_file: OK (${local_lines} lines)"
  else
    echo "  $manual_file: MISSING — please create this file!"
    MISSING_FILES=$((MISSING_FILES + 1))
  fi
done

if [ "$MISSING_FILES" -gt 0 ]; then
  echo ""
  echo "WARNING: ${MISSING_FILES} knowledge file(s) missing. OpenClaw won't have complete system knowledge."
  echo "Run the knowledge training process to regenerate them."
fi

echo ""
echo "Total workspace files:"
ls -1 "$WORKSPACE"/*.md 2>/dev/null | while read -r f; do echo "  $(basename "$f")"; done
echo ""
echo "If Docker container is running with volume mount, files are already available."
echo "Otherwise run: docker cp openclaw/workspace/ <container>:/home/node/.openclaw/workspace/"
