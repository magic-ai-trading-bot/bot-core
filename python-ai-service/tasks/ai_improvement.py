#!/usr/bin/env python3
"""
AI Self-Improvement & Adaptive Retraining Tasks
GPT-4-powered intelligent decision making for model retraining
"""

from typing import Dict, Any, List, Optional
from celery import Task
from celery_app import app
from utils.logger import get_logger
from utils import notifications
from utils.data_storage import storage
from datetime import datetime, timedelta
import requests
import os
import json
from openai import OpenAI

logger = get_logger("AIImprovementTasks")

# OpenAI configuration - v1.0+ uses client-based API
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")

# Service URLs
RUST_API_URL = os.getenv("RUST_API_URL", "http://localhost:8080")
PYTHON_API_URL = os.getenv("PYTHON_API_URL", "http://localhost:8000")

# Performance thresholds
TARGET_WIN_RATE = 70.0
TARGET_AVG_PROFIT = 2.6
TARGET_SHARPE = 2.1
CRITICAL_WIN_RATE = 55.0
CRITICAL_SHARPE = 1.0
CRITICAL_AVG_PROFIT = 1.5

# Model accuracy thresholds
TARGET_MODEL_ACCURACY = 0.70
CRITICAL_MODEL_ACCURACY = 0.65
ACCURACY_DROP_THRESHOLD = 0.05  # 5% drop triggers analysis


class AIImprovementTask(Task):
    """Base task for AI improvement operations"""

    def on_failure(self, exc, task_id, args, kwargs, einfo):
        logger.error(f"❌ AI improvement task {task_id} failed: {exc}")
        notifications.send_critical(
            title=f"AI Task Failed: {task_id}",
            message=f"Task {self.name} failed with error: {exc}",
            data={"task_id": task_id, "error": str(exc)},
        )

    def on_success(self, retval, task_id, args, kwargs):
        logger.info(f"✅ AI improvement task {task_id} completed successfully")


# =============================================================================
# TASK 1: GPT-4 SELF-ANALYSIS (3 AM daily or on-demand)
# =============================================================================


# @spec:FR-ASYNC-008 - GPT-4 Self-Analysis for Retraining Decisions
# @ref:specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md#fr-async-008
# @ref:specs/02-design/2.5-components/COMP-PYTHON-ML.md#gpt4-self-analysis
# @test:TC-ASYNC-046, TC-ASYNC-047, TC-ASYNC-048, TC-ASYNC-049, TC-ASYNC-050
@app.task(
    bind=True,
    base=AIImprovementTask,
    name="tasks.ai_improvement.gpt4_self_analysis",
)
def gpt4_self_analysis(self, force_analysis: bool = False) -> Dict[str, Any]:
    """
    Use GPT-4 to analyze performance trends and decide if retraining is needed
    Schedule: 3:00 AM UTC daily (or triggered by performance alerts)

    Process:
    1. Fetch last 7 days performance metrics
    2. Fetch model accuracy trends
    3. Fetch market conditions
    4. Send to GPT-4 for deep analysis
    5. Get recommendation (retrain/wait/optimize_parameters)
    6. If "retrain" + high confidence → Trigger adaptive_retrain task

    Args:
        force_analysis: If True, run analysis even if no alerts

    Returns:
        GPT-4 analysis result with recommendation
    """
    logger.info("🤖 Starting GPT-4 self-analysis...")

    try:
        # Check if OpenAI API key is configured (check dynamically for test compatibility)
        api_key = os.getenv("OPENAI_API_KEY")
        if not api_key:
            logger.warning("⚠️ OPENAI_API_KEY not configured - skipping GPT-4 analysis")
            # Return a default analysis result for tests instead of raising exception
            return {
                "status": "skipped",
                "reason": "OPENAI_API_KEY not configured",
                "analysis": {
                    "recommendation": "wait",
                    "confidence": 0,
                    "reasoning": "Cannot run analysis without OpenAI API key",
                },
                "trigger_retrain": False,
                "task_id": self.request.id if hasattr(self, 'request') else None,
            }

        # Create OpenAI client (v1.0+ syntax)
        client = OpenAI(api_key=api_key)

        # STEP 1: Fetch last 7 days performance data
        logger.info("📊 Fetching 7-day performance data...")

        response = requests.get(
            f"{RUST_API_URL}/api/paper-trading/trades/closed",
            timeout=10,
        )
        response.raise_for_status()
        trades_response = response.json()
        # Extract trades from response data
        trades = trades_response.get("data", []) if trades_response.get("success") else []

        # Calculate daily metrics
        daily_metrics = calculate_daily_metrics(trades)

        # STEP 2: Fetch model accuracy trends
        logger.info("🧠 Fetching model accuracy trends...")

        try:
            response = requests.get(
                f"{PYTHON_API_URL}/ai/model/accuracy_history",
                params={"days": 7},
                timeout=10,
            )
            model_accuracy = response.json() if response.status_code == 200 else {}
        except Exception as e:
            logger.warning(f"⚠️ Could not fetch model accuracy: {e}")
            model_accuracy = {}

        # STEP 3: Fetch market conditions
        logger.info("📈 Fetching market conditions...")

        try:
            response = requests.get(
                "https://api.binance.com/api/v3/ticker/24hr",
                params={"symbol": "BTCUSDT"},
                timeout=10,
            )
            market_data = response.json() if response.status_code == 200 else {}
        except Exception as e:
            logger.warning(f"⚠️ Could not fetch market data: {e}")
            market_data = {}

        # STEP 4: Check if analysis is needed
        if not force_analysis:
            # Check if there are critical performance issues
            latest_win_rate = daily_metrics[-1]["win_rate"] if daily_metrics else 0
            latest_sharpe = daily_metrics[-1]["sharpe_ratio"] if daily_metrics else 0

            if (
                latest_win_rate >= CRITICAL_WIN_RATE
                and latest_sharpe >= CRITICAL_SHARPE
            ):
                logger.info(
                    f"✅ Performance is acceptable (WR: {latest_win_rate:.1f}%, Sharpe: {latest_sharpe:.2f}) - skipping GPT-4 analysis"
                )
                return {
                    "status": "skipped",
                    "reason": "Performance is acceptable, no analysis needed",
                    "metrics": {
                        "win_rate": latest_win_rate,
                        "sharpe_ratio": latest_sharpe,
                    },
                    "task_id": self.request.id,
                }

        # STEP 5: Prepare GPT-4 prompt
        logger.info("🤖 Preparing GPT-4 analysis prompt...")

        prompt = _build_gpt4_analysis_prompt()

        # STEP 6: Call GPT-4 for analysis
        logger.info("🤖 Calling GPT-4 for deep analysis...")

        try:
            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=[
                    {
                        "role": "system",
                        "content": "You are an AI trading bot self-improvement analyst. Analyze performance data and decide if model retraining is needed. Always respond with valid JSON.",
                    },
                    {
                        "role": "user",
                        "content": prompt,
                    },
                ],
                temperature=0.3,  # Lower temperature for more consistent decisions
                max_completion_tokens=1000,
            )

            gpt4_response = response.choices[0].message.content

            # Strip markdown code blocks if present (GPT-4 often wraps JSON in ```json ... ```)
            if gpt4_response.startswith("```"):
                # Remove opening ```json or ```
                lines = gpt4_response.split("\n")
                # Remove first line (```json) and last line (```)
                lines = [l for l in lines if not l.strip().startswith("```")]
                gpt4_response = "\n".join(lines)

            # Parse JSON response
            analysis_result = json.loads(gpt4_response)

            logger.info(f"🤖 GPT-4 Analysis Complete:")
            logger.info(
                f"  📋 Recommendation: {analysis_result.get('recommendation', 'N/A')}"
            )
            logger.info(f"  🎯 Confidence: {analysis_result.get('confidence', 0):.2%}")
            logger.info(f"  ⚡ Urgency: {analysis_result.get('urgency', 'N/A')}")
            logger.info(
                f"  💡 Reasoning: {analysis_result.get('reasoning', 'N/A')[:100]}..."
            )

            # STEP 7: Decide if adaptive retraining should be triggered
            recommendation = analysis_result.get("recommendation", "wait")
            confidence = analysis_result.get("confidence", 0)
            urgency = analysis_result.get("urgency", "low")

            should_retrain = (
                recommendation == "retrain"
                and confidence >= 0.7
                and urgency in ["medium", "high"]
            )

            # STEP 8: Store GPT-4 analysis FIRST (before Celery queue to prevent data loss)
            storage.store_gpt4_analysis(analysis_result, self.request.id)
            logger.info(f"✅ GPT-4 analysis stored in MongoDB")

            # STEP 9: Send GPT-4 analysis notification
            notifications.send_gpt4_analysis(analysis_result)

            trigger_retrain = False
            if should_retrain:
                logger.warning(
                    f"🚨 GPT-4 recommends RETRAIN with {confidence:.0%} confidence (urgency: {urgency})"
                )
                logger.info(f"🚀 Triggering config improvement analysis...")

                # Try to queue via Celery first, fallback to direct call if Redis fails
                try:
                    from tasks.ai_improvement import adaptive_retrain

                    retrain_task = adaptive_retrain.delay(
                        model_types=["lstm", "gru", "transformer"],
                        analysis_result=analysis_result,
                    )

                    logger.info(f"✅ Config improvement task queued via Celery: {retrain_task.id}")
                    analysis_result["retrain_task_id"] = retrain_task.id
                    analysis_result["retrain_triggered"] = True
                    trigger_retrain = True
                except Exception as celery_error:
                    # If Celery/Redis fails, run directly (synchronously)
                    logger.warning(f"⚠️ Celery queue failed ({celery_error}), running config analysis directly...")
                    try:
                        from tasks.ai_improvement import _run_config_analysis_direct
                        direct_result = _run_config_analysis_direct(analysis_result)
                        if direct_result.get("status") == "success":
                            logger.info(f"✅ Config improvement analysis completed directly (bypassed Celery)")
                            analysis_result["retrain_triggered"] = True
                            analysis_result["retrain_mode"] = "direct"
                            trigger_retrain = True
                        else:
                            logger.warning(f"⚠️ Direct config analysis returned: {direct_result.get('status')}")
                            analysis_result["retrain_triggered"] = False
                    except Exception as direct_error:
                        logger.error(f"❌ Direct config analysis also failed: {direct_error}")
                        analysis_result["retrain_triggered"] = False
            else:
                logger.info(
                    f"✅ No immediate retraining needed (recommendation: {recommendation}, confidence: {confidence:.0%})"
                )
                analysis_result["retrain_triggered"] = False

            return {
                "status": "success",
                "analysis": analysis_result,
                "trigger_retrain": trigger_retrain,
                "task_id": self.request.id,
            }

        except json.JSONDecodeError as e:
            logger.error(f"❌ Failed to parse GPT-4 response as JSON: {e}")
            logger.error(f"Raw response: {gpt4_response}")
            raise
        except Exception as e:
            logger.error(f"❌ GPT-4 API call failed: {e}")
            raise

    except Exception as e:
        logger.error(f"❌ GPT-4 self-analysis failed: {e}")
        raise


# =============================================================================
# TASK 2: GPT-4 CONFIG IMPROVEMENT SUGGESTIONS (Triggered by GPT-4 analysis)
# =============================================================================


# @spec:FR-ASYNC-009 - GPT-4 Config Improvement Suggestions
# @ref:specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md#fr-async-009
# @test:TC-ASYNC-061, TC-ASYNC-062, TC-ASYNC-063, TC-ASYNC-064, TC-ASYNC-065
@app.task(
    bind=True,
    base=AIImprovementTask,
    name="tasks.ai_improvement.adaptive_retrain",  # Keep name for backward compatibility
    time_limit=300,  # 5 minutes max
)
def adaptive_retrain(
    self,
    model_types: List[str] = None,  # Deprecated, kept for backward compatibility
    analysis_result: Dict[str, Any] = None,
) -> Dict[str, Any]:
    """
    Use GPT-4 to analyze current config and suggest improvements based on trade performance.
    This task does NOT train ML models - it uses GPT-4 to optimize trading config.

    Process:
    1. Fetch current indicator settings from Rust API
    2. Fetch current signal settings from Rust API
    3. Fetch recent closed trades (last 7 days)
    4. Send all data to GPT-4 for deep analysis
    5. GPT-4 suggests specific config changes
    6. (Optional) Auto-apply changes via PUT to Rust API
    7. Store suggestions in MongoDB for review

    Args:
        model_types: Deprecated, ignored
        analysis_result: GPT-4 analysis that triggered this (contains reasoning)

    Returns:
        Config improvement suggestions from GPT-4
    """
    logger.info("🧠 Starting GPT-4 config improvement analysis...")
    if analysis_result:
        logger.info(
            f"📋 Triggered by: {analysis_result.get('reasoning', 'N/A')[:100]}..."
        )

    try:
        # Check OpenAI API key
        api_key = os.getenv("OPENAI_API_KEY")
        if not api_key:
            logger.warning("⚠️ OPENAI_API_KEY not configured - skipping")
            return {
                "status": "skipped",
                "reason": "OPENAI_API_KEY not configured",
            }

        # Create OpenAI client
        client = OpenAI(api_key=api_key)

        # STEP 1: Fetch current indicator settings
        logger.info("📊 Fetching current indicator settings...")
        try:
            response = requests.get(
                f"{RUST_API_URL}/api/paper-trading/indicator-settings",
                timeout=10,
            )
            response.raise_for_status()
            indicator_settings = response.json().get("data", {})
        except Exception as e:
            logger.warning(f"⚠️ Could not fetch indicator settings: {e}")
            indicator_settings = {}

        # STEP 2: Fetch current signal settings
        logger.info("📊 Fetching current signal settings...")
        try:
            response = requests.get(
                f"{RUST_API_URL}/api/paper-trading/signal-settings",
                timeout=10,
            )
            response.raise_for_status()
            signal_settings = response.json().get("data", {})
        except Exception as e:
            logger.warning(f"⚠️ Could not fetch signal settings: {e}")
            signal_settings = {}

        # STEP 3: Fetch recent closed trades
        logger.info("📈 Fetching recent closed trades...")
        try:
            response = requests.get(
                f"{RUST_API_URL}/api/paper-trading/trades/closed",
                timeout=10,
            )
            response.raise_for_status()
            trades_response = response.json()
            trades = trades_response.get("data", []) if trades_response.get("success") else []
        except Exception as e:
            logger.warning(f"⚠️ Could not fetch trades: {e}")
            trades = []

        # Calculate trade statistics
        total_trades = len(trades)
        winning_trades = sum(1 for t in trades if (t.get("pnl_percentage") or 0) > 0)
        losing_trades = sum(1 for t in trades if (t.get("pnl_percentage") or 0) < 0)
        win_rate = (winning_trades / total_trades * 100) if total_trades > 0 else 0
        total_pnl = sum(t.get("pnl_usdt") or 0 for t in trades)
        avg_pnl_percent = sum(t.get("pnl_percentage") or 0 for t in trades) / total_trades if total_trades > 0 else 0

        # STEP 4: Build GPT-4 prompt
        logger.info("🤖 Building GPT-4 config analysis prompt...")

        # Determine risk level based on performance
        is_losing = total_pnl < 0 or win_rate < 50
        data_insufficient = total_trades < 5

        # Risk guidance for GPT-4
        if data_insufficient:
            risk_guidance = f"""
## ⚠️ CRITICAL: INSUFFICIENT DATA WARNING
You only have {total_trades} trades to analyze. This is NOT enough data to make confident suggestions.
- DO NOT suggest aggressive changes
- Set confidence BELOW 0.3
- Set auto_apply_safe to FALSE
- Recommend waiting for more data (at least 10-20 trades)
"""
        elif is_losing:
            risk_guidance = f"""
## ⚠️ CRITICAL: BOT IS LOSING MONEY
Current PnL: ${total_pnl:.2f} (NEGATIVE) | Win Rate: {win_rate:.1f}% (BELOW 50%)

**RISK MANAGEMENT RULES (MUST FOLLOW):**
1. When losing money, NEVER suggest making the bot MORE aggressive
2. DO NOT lower thresholds (lower threshold = easier to trigger = more trades = more risk)
3. DO NOT decrease indicator periods (faster signals = more noise = more false positives)
4. DO NOT decrease min_required_indicators (fewer confirmations = higher risk)

**INSTEAD, consider:**
1. INCREASE thresholds (be more selective with trades)
2. INCREASE indicator periods (smoother signals, less noise)
3. INCREASE min_required_indicators (more confirmations before trading)
4. Wait for better market conditions

**Your priority is CAPITAL PRESERVATION, not maximizing trades.**
"""
        else:
            risk_guidance = """
## ✅ BOT IS PROFITABLE
You can suggest optimizations to improve performance, but still be cautious.
- Prefer small, incremental changes over drastic ones
- Changes should have clear reasoning backed by the data
"""

        prompt = f"""
You are a CONSERVATIVE trading bot configuration optimizer. Your PRIMARY goal is CAPITAL PRESERVATION.

{risk_guidance}

## CURRENT INDICATOR SETTINGS:
{json.dumps(indicator_settings, indent=2)}

## CURRENT SIGNAL SETTINGS:
{json.dumps(signal_settings, indent=2)}

## TRADE PERFORMANCE (Last 7 days):
- Total Trades: {total_trades}
- Winning Trades: {winning_trades}
- Losing Trades: {losing_trades}
- Win Rate: {win_rate:.1f}%
- Total PnL: ${total_pnl:.2f}
- Avg PnL per Trade: {avg_pnl_percent:.2f}%

## RECENT TRADES DETAILS:
{json.dumps(trades[:10], indent=2, default=str)}

## ANALYSIS CONTEXT:
{analysis_result.get('reasoning', 'Performance needs improvement') if analysis_result else 'Routine optimization check'}

## YOUR TASK:
1. Analyze why trades are losing (if any)
2. Identify which config parameters might be causing issues
3. Suggest SPECIFIC parameter changes with exact values
4. **If losing money: suggest MORE CONSERVATIVE settings (higher thresholds, longer periods)**
5. **If insufficient data (<10 trades): DO NOT suggest changes, recommend waiting**

## IMPORTANT RULES:
- If total_trades < 5: Set confidence < 0.3, auto_apply_safe = false
- If total_trades < 10: Set confidence < 0.5, auto_apply_safe = false
- If losing money (PnL < 0): NEVER lower thresholds, NEVER decrease periods
- If win_rate < 50%: Suggest STRICTER criteria (higher thresholds, more confirmations)

## OUTPUT FORMAT (MUST BE VALID JSON):
{{
  "analysis": {{
    "root_cause": "Brief explanation of why performance is poor",
    "key_issues": ["issue1", "issue2", "issue3"],
    "data_quality": "insufficient (<10 trades)" | "limited (10-50 trades)" | "good (50+ trades)"
  }},
  "indicator_suggestions": {{
    "rsi_period": {{"current": X, "suggested": Y, "reason": "..."}},
    "macd_fast": {{"current": X, "suggested": Y, "reason": "..."}},
    ...only include parameters that need changing
  }},
  "signal_suggestions": {{
    "trend_threshold_percent": {{"current": X, "suggested": Y, "reason": "..."}},
    "min_required_indicators": {{"current": X, "suggested": Y, "reason": "..."}},
    ...only include parameters that need changing
  }},
  "confidence": 0.0-1.0,
  "auto_apply_safe": true/false,
  "risk_assessment": "low" | "medium" | "high",
  "summary": "One paragraph summary of all recommendations"
}}
"""

        # STEP 5: Call GPT-4 for analysis
        logger.info("🤖 Calling GPT-4 for config analysis...")

        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[
                {
                    "role": "system",
                    "content": "You are a trading bot configuration optimizer. Analyze config and performance data, then suggest specific improvements. Always respond with valid JSON.",
                },
                {
                    "role": "user",
                    "content": prompt,
                },
            ],
            temperature=0.3,
            max_completion_tokens=2000,
        )

        gpt4_response = response.choices[0].message.content

        # Strip markdown code blocks if present
        if gpt4_response.startswith("```"):
            lines = gpt4_response.split("\n")
            lines = [l for l in lines if not l.strip().startswith("```")]
            gpt4_response = "\n".join(lines)

        # Parse JSON response
        suggestions = json.loads(gpt4_response)

        logger.info("🤖 GPT-4 Config Analysis Complete:")
        logger.info(f"  📋 Root Cause: {suggestions.get('analysis', {}).get('root_cause', 'N/A')}")
        logger.info(f"  🎯 Confidence: {suggestions.get('confidence', 0):.0%}")
        logger.info(f"  ✅ Auto-apply Safe: {suggestions.get('auto_apply_safe', False)}")
        logger.info(f"  💡 Summary: {suggestions.get('summary', 'N/A')[:100]}...")

        # STEP 6: (Optional) Auto-apply if safe and confidence is high
        applied_changes = []
        if suggestions.get("auto_apply_safe") and suggestions.get("confidence", 0) >= 0.8:
            logger.info("🚀 Auto-applying safe config changes...")

            # Apply indicator settings
            if suggestions.get("indicator_suggestions"):
                new_indicators = {}
                for key, change in suggestions["indicator_suggestions"].items():
                    if isinstance(change, dict) and "suggested" in change:
                        new_indicators[key] = change["suggested"]
                        applied_changes.append(f"indicator.{key}: {change.get('current')} → {change['suggested']}")

                if new_indicators:
                    try:
                        response = requests.put(
                            f"{RUST_API_URL}/api/paper-trading/indicator-settings",
                            json=new_indicators,
                            timeout=10,
                        )
                        response.raise_for_status()
                        logger.info(f"✅ Applied indicator changes: {new_indicators}")
                    except Exception as e:
                        logger.error(f"❌ Failed to apply indicator changes: {e}")

            # Apply signal settings
            if suggestions.get("signal_suggestions"):
                new_signal = {}
                for key, change in suggestions["signal_suggestions"].items():
                    if isinstance(change, dict) and "suggested" in change:
                        new_signal[key] = change["suggested"]
                        applied_changes.append(f"signal.{key}: {change.get('current')} → {change['suggested']}")

                if new_signal:
                    try:
                        response = requests.put(
                            f"{RUST_API_URL}/api/paper-trading/signal-settings",
                            json=new_signal,
                            timeout=10,
                        )
                        response.raise_for_status()
                        logger.info(f"✅ Applied signal changes: {new_signal}")
                    except Exception as e:
                        logger.error(f"❌ Failed to apply signal changes: {e}")
        else:
            logger.info("⏸️ Changes NOT auto-applied (confidence < 80% or not marked safe)")

        # STEP 7: Store suggestions in MongoDB
        result = {
            "status": "success",
            "timestamp": datetime.utcnow().isoformat(),
            "trigger_analysis": analysis_result,
            "current_config": {
                "indicators": indicator_settings,
                "signal": signal_settings,
            },
            "trade_stats": {
                "total_trades": total_trades,
                "win_rate": win_rate,
                "total_pnl": total_pnl,
            },
            "suggestions": suggestions,
            "applied_changes": applied_changes,
            "task_id": self.request.id if hasattr(self, 'request') else None,
        }

        storage.store_config_suggestions(result)

        # Send notification
        notifications.send_config_suggestions(result)

        logger.info(f"✅ Config improvement analysis complete")
        if applied_changes:
            logger.info(f"  📝 Applied {len(applied_changes)} changes automatically")
        else:
            logger.info(f"  📝 {len(suggestions.get('indicator_suggestions', {}))} indicator suggestions")
            logger.info(f"  📝 {len(suggestions.get('signal_suggestions', {}))} signal suggestions")

        return result

    except json.JSONDecodeError as e:
        logger.error(f"❌ Failed to parse GPT-4 response: {e}")
        raise
    except Exception as e:
        logger.error(f"❌ Config improvement analysis failed: {e}")
        raise


# =============================================================================
# TASK 3: EMERGENCY STRATEGY DISABLE (Alert-triggered)
# =============================================================================


# @spec:FR-ASYNC-010 - Emergency Strategy Disable
# @ref:specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md#fr-async-010
# @test:TC-ASYNC-071, TC-ASYNC-072, TC-ASYNC-073
@app.task(
    bind=True,
    base=AIImprovementTask,
    name="tasks.ai_improvement.emergency_strategy_disable",
)
def emergency_strategy_disable(
    self,
    strategy_name: str,
    reason: str,
) -> Dict[str, Any]:
    """
    Emergency disable a failing strategy
    Triggered by: Severe daily loss (>10%) or consecutive losses (>10)

    Args:
        strategy_name: Strategy to disable
        reason: Reason for emergency disable

    Returns:
        Disable status
    """
    logger.error(f"🚨 EMERGENCY: Disabling {strategy_name} strategy!")
    logger.error(f"📋 Reason: {reason}")

    try:
        # Call Rust API to disable strategy
        response = requests.post(
            f"{RUST_API_URL}/api/strategies/{strategy_name}/disable",
            json={"reason": reason, "emergency": True},
            timeout=10,
        )
        response.raise_for_status()

        logger.error(f"✅ Strategy {strategy_name} has been DISABLED")

        # Send CRITICAL emergency notification
        notifications.send_critical(
            title=f"EMERGENCY: Strategy {strategy_name} Disabled",
            message=f"Strategy automatically disabled due to: {reason}",
            data={
                "strategy": strategy_name,
                "reason": reason,
                "disabled_at": datetime.utcnow().isoformat(),
            },
        )

        return {
            "status": "success",
            "strategy": strategy_name,
            "strategy_disabled": strategy_name,
            "reason": reason,
            "disabled_at": datetime.utcnow().isoformat(),
            "task_id": self.request.id,
        }

    except Exception as e:
        logger.error(f"❌ Failed to disable strategy: {e}")
        raise


# =============================================================================
# TASK 4: ANALYZE INDIVIDUAL TRADES (GPT-4 per-trade analysis)
# =============================================================================


# @spec:FR-ASYNC-011 - GPT-4 Individual Trade Analysis
# @ref:specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md#fr-async-011
# @test:TC-ASYNC-081, TC-ASYNC-082, TC-ASYNC-083
@app.task(
    bind=True,
    base=AIImprovementTask,
    name="tasks.ai_improvement.analyze_trade",
)
def analyze_trade(
    self,
    trade_data: Dict[str, Any],
) -> Dict[str, Any]:
    """
    Use GPT-4 to analyze a single trade and provide detailed feedback.
    Called automatically when a trade is closed (especially losing trades).

    Args:
        trade_data: Complete trade data including entry/exit details

    Returns:
        GPT-4 analysis of the trade
    """
    trade_id = trade_data.get("id") or trade_data.get("trade_id", "unknown")
    logger.info(f"🔍 Analyzing trade {trade_id}...")

    try:
        # Check if already analyzed
        existing = storage.get_trade_analysis(trade_id)
        if existing:
            logger.info(f"✅ Trade {trade_id} already analyzed, returning cached result")
            return {
                "status": "cached",
                "trade_id": trade_id,
                "analysis": existing.get("analysis"),
            }

        # Check OpenAI API key
        api_key = os.getenv("OPENAI_API_KEY")
        if not api_key:
            logger.warning("⚠️ OPENAI_API_KEY not configured - skipping trade analysis")
            return {
                "status": "skipped",
                "reason": "OPENAI_API_KEY not configured",
                "trade_id": trade_id,
            }

        # Create OpenAI client
        client = OpenAI(api_key=api_key)

        # Determine if winning or losing
        pnl = trade_data.get("pnl_usdt") or trade_data.get("profit_usdt", 0)
        pnl_pct = trade_data.get("pnl_percentage") or trade_data.get("profit_percent", 0)
        is_winning = pnl > 0

        # Fetch current market conditions
        symbol = trade_data.get("symbol", "BTCUSDT")
        try:
            response = requests.get(
                f"https://api.binance.com/api/v3/ticker/24hr",
                params={"symbol": symbol},
                timeout=5,
            )
            market_data = response.json() if response.status_code == 200 else {}
        except Exception:
            market_data = {}

        # Build analysis prompt
        trade_type = "WINNING ✅" if is_winning else "LOSING ❌"

        prompt = f"""
You are a professional trading analyst. Analyze this {trade_type} trade and provide actionable insights.

## TRADE DETAILS:
- Trade ID: {trade_id}
- Symbol: {trade_data.get("symbol")}
- Side: {trade_data.get("side")} (Long/Short)
- Entry Price: ${trade_data.get("entry_price", 0):.4f}
- Exit Price: ${trade_data.get("exit_price", 0):.4f}
- Quantity: {trade_data.get("quantity", 0)}
- PnL: ${pnl:.2f} ({pnl_pct:.2f}%)
- Duration: {trade_data.get("duration_seconds", 0)} seconds
- Close Reason: {trade_data.get("close_reason", "unknown")}
- Open Time: {trade_data.get("open_time", "N/A")}
- Close Time: {trade_data.get("close_time", "N/A")}

## ENTRY SIGNALS (Why bot entered this trade):
{json.dumps(trade_data.get("entry_signals", {}), indent=2, default=str)}

## EXIT SIGNALS (Why bot exited this trade):
{json.dumps(trade_data.get("exit_signals", {}), indent=2, default=str)}

## CURRENT MARKET CONDITIONS:
- 24h Price Change: {market_data.get("priceChangePercent", "N/A")}%
- 24h Volume: {market_data.get("volume", "N/A")}
- Current Price: ${market_data.get("lastPrice", "N/A")}

## YOUR ANALYSIS TASKS:
1. **Entry Analysis**: Was the entry timing good? Were signals valid?
2. **Exit Analysis**: Was the exit optimal? Too early? Too late?
3. **What Went {"Right" if is_winning else "Wrong"}**: Key factors that led to this {"profit" if is_winning else "loss"}
4. **Lessons Learned**: What can be improved for future trades?
5. **Actionable Recommendations**: Specific parameter changes if needed

## OUTPUT FORMAT (MUST BE VALID JSON):
{{
  "trade_verdict": "{trade_type}",
  "entry_analysis": {{
    "quality": "good" | "acceptable" | "poor",
    "reasoning": "Why entry was good/bad",
    "signals_valid": true/false
  }},
  "exit_analysis": {{
    "quality": "optimal" | "acceptable" | "suboptimal",
    "reasoning": "Why exit was good/bad",
    "better_exit_point": "Description of better exit if applicable"
  }},
  "key_factors": ["factor1", "factor2", "factor3"],
  "lessons_learned": ["lesson1", "lesson2"],
  "recommendations": {{
    "config_changes": {{"param": "suggested_value", ...}} or null,
    "strategy_improvements": ["improvement1", "improvement2"],
    "risk_management": "Any risk management advice"
  }},
  "confidence": 0.0-1.0,
  "summary": "One paragraph summary of the analysis"
}}
"""

        # Call GPT-4
        logger.info(f"🤖 Calling GPT-4 to analyze trade {trade_id}...")
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[
                {
                    "role": "system",
                    "content": "You are a professional trading analyst. Provide detailed, actionable analysis of trades. Always respond with valid JSON.",
                },
                {
                    "role": "user",
                    "content": prompt,
                },
            ],
            temperature=0.3,
            max_completion_tokens=1500,
        )

        gpt4_response = response.choices[0].message.content

        # Strip markdown code blocks if present
        if gpt4_response.startswith("```"):
            lines = gpt4_response.split("\n")
            lines = [l for l in lines if not l.strip().startswith("```")]
            gpt4_response = "\n".join(lines)

        # Parse JSON response
        analysis = json.loads(gpt4_response)

        logger.info(f"🤖 GPT-4 Trade Analysis Complete for {trade_id}:")
        logger.info(f"  📋 Verdict: {analysis.get('trade_verdict', 'N/A')}")
        logger.info(f"  🎯 Entry Quality: {analysis.get('entry_analysis', {}).get('quality', 'N/A')}")
        logger.info(f"  🎯 Exit Quality: {analysis.get('exit_analysis', {}).get('quality', 'N/A')}")
        logger.info(f"  💡 Summary: {analysis.get('summary', 'N/A')[:100]}...")

        # Store analysis in MongoDB
        storage.store_trade_analysis(trade_id, trade_data, analysis)

        return {
            "status": "success",
            "trade_id": trade_id,
            "is_winning": is_winning,
            "pnl_usdt": pnl,
            "analysis": analysis,
            "task_id": self.request.id if hasattr(self, 'request') else None,
        }

    except json.JSONDecodeError as e:
        logger.error(f"❌ Failed to parse GPT-4 response for trade {trade_id}: {e}")
        raise
    except Exception as e:
        logger.error(f"❌ Failed to analyze trade {trade_id}: {e}")
        raise


# @spec:FR-ASYNC-012 - Batch Trade Analysis
# @ref:specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md#fr-async-012
# @test:TC-ASYNC-091, TC-ASYNC-092
@app.task(
    bind=True,
    base=AIImprovementTask,
    name="tasks.ai_improvement.analyze_recent_trades",
)
def analyze_recent_trades(
    self,
    only_losing: bool = True,
    limit: int = 10,
) -> Dict[str, Any]:
    """
    Analyze multiple recent trades that haven't been analyzed yet.
    Called periodically (e.g., every hour) to catch up on unanalyzed trades.

    Args:
        only_losing: If True, only analyze losing trades
        limit: Maximum number of trades to analyze

    Returns:
        Summary of analyzed trades
    """
    logger.info(f"🔍 Analyzing recent {'losing ' if only_losing else ''}trades (limit: {limit})...")

    try:
        # Fetch recent closed trades
        response = requests.get(
            f"{RUST_API_URL}/api/paper-trading/trades/closed",
            timeout=10,
        )
        response.raise_for_status()
        trades_response = response.json()
        trades = trades_response.get("data", []) if trades_response.get("success") else []

        if not trades:
            logger.info("✅ No trades to analyze")
            return {"status": "success", "analyzed_count": 0, "message": "No trades found"}

        # Filter losing trades if requested
        if only_losing:
            trades = [t for t in trades if (t.get("pnl_usdt") or 0) < 0]

        if not trades:
            logger.info("✅ No losing trades to analyze")
            return {"status": "success", "analyzed_count": 0, "message": "No losing trades found"}

        # Get trade IDs
        trade_ids = [t.get("id") or t.get("trade_id") for t in trades if t.get("id") or t.get("trade_id")]

        # Filter out already analyzed trades
        unanalyzed_ids = storage.get_unanalyzed_trade_ids(trade_ids)

        if not unanalyzed_ids:
            logger.info("✅ All trades already analyzed")
            return {"status": "success", "analyzed_count": 0, "message": "All trades already analyzed"}

        # Limit to requested number
        unanalyzed_ids = unanalyzed_ids[:limit]

        # Get full trade data for unanalyzed trades
        trades_to_analyze = [t for t in trades if (t.get("id") or t.get("trade_id")) in unanalyzed_ids]

        logger.info(f"📊 Found {len(trades_to_analyze)} trades to analyze")

        # Queue individual analysis tasks
        results = []
        for trade in trades_to_analyze:
            try:
                result = analyze_trade(trade)
                results.append({
                    "trade_id": trade.get("id") or trade.get("trade_id"),
                    "status": result.get("status"),
                    "is_winning": result.get("is_winning"),
                })
            except Exception as e:
                logger.error(f"❌ Failed to analyze trade: {e}")
                results.append({
                    "trade_id": trade.get("id") or trade.get("trade_id"),
                    "status": "error",
                    "error": str(e),
                })

        successful = sum(1 for r in results if r.get("status") == "success")
        logger.info(f"✅ Analyzed {successful}/{len(results)} trades successfully")

        return {
            "status": "success",
            "analyzed_count": successful,
            "total_attempted": len(results),
            "results": results,
            "task_id": self.request.id if hasattr(self, 'request') else None,
        }

    except Exception as e:
        logger.error(f"❌ Batch trade analysis failed: {e}")
        raise


# =============================================================================
# HELPER FUNCTIONS
# =============================================================================


def calculate_daily_metrics(trades: List[Dict]) -> List[Dict]:
    """Calculate daily performance metrics from trades"""
    if not trades:
        return []

    # Group trades by date
    from collections import defaultdict

    daily_trades = defaultdict(list)

    for trade in trades:
        # Support both old format (executed_at) and new format (close_time)
        timestamp = trade.get("close_time") or trade.get("executed_at", "")
        date = timestamp[:10] if timestamp else ""  # Extract YYYY-MM-DD
        if date:
            daily_trades[date].append(trade)

    # Calculate metrics for each day
    daily_metrics = []
    for date in sorted(daily_trades.keys()):
        day_trades = daily_trades[date]

        # Support both old format (profit_percent) and new format (pnl_percentage)
        winning = sum(1 for t in day_trades if (t.get("pnl_percentage") or t.get("profit_percent", 0)) > 0)
        total = len(day_trades)
        win_rate = (winning / total * 100) if total > 0 else 0

        profits = [t.get("pnl_percentage") or t.get("profit_percent", 0) for t in day_trades]
        avg_profit = sum(profits) / len(profits) if profits else 0

        # Simple Sharpe approximation
        returns = [p / 100 for p in profits]
        avg_return = sum(returns) / len(returns) if returns else 0
        std_dev = (
            (sum((r - avg_return) ** 2 for r in returns) / len(returns)) ** 0.5
            if returns
            else 1
        )
        sharpe = (avg_return / std_dev) * (252**0.5) if std_dev > 0 else 0

        daily_metrics.append(
            {
                "date": date,
                "total_trades": total,
                "win_rate": round(win_rate, 2),
                "avg_profit": round(avg_profit, 2),
                "sharpe_ratio": round(sharpe, 2),
            }
        )

    return daily_metrics


def _build_gpt4_analysis_prompt() -> str:
    """
    Build GPT-4 analysis prompt with performance data (private helper)
    Gets data from storage and API automatically
    """
    # Fetch data from storage
    daily_metrics = storage.get_performance_metrics_history(days=7) or []
    model_accuracy_history = storage.get_model_accuracy_history(days=7) or []

    # Try to get market data from Binance
    try:
        response = requests.get(
            "https://api.binance.com/api/v3/ticker/24hr",
            params={"symbol": "BTCUSDT"},
            timeout=10,
        )
        market_data = response.json() if response.status_code == 200 else {}
    except Exception:
        market_data = {}

    # Convert model accuracy history to dict format (serialize datetime objects)
    model_accuracy_serializable = []
    for item in model_accuracy_history:
        serialized = {}
        for key, value in item.items():
            if isinstance(value, datetime):
                serialized[key] = value.isoformat()
            else:
                serialized[key] = value
        model_accuracy_serializable.append(serialized)

    model_accuracy = {
        "models": model_accuracy_serializable
    }

    # Extract trends (serialize date objects in daily_metrics)
    daily_metrics_serializable = []
    for metric in (daily_metrics or []):
        serialized = {}
        for key, value in metric.items():
            if hasattr(value, 'isoformat'):  # date or datetime
                serialized[key] = value.isoformat()
            else:
                serialized[key] = value
        daily_metrics_serializable.append(serialized)

    win_rates = [m.get("win_rate", 0) for m in daily_metrics_serializable] if daily_metrics_serializable else []
    profits = [m.get("avg_profit", 0) for m in daily_metrics_serializable] if daily_metrics_serializable else []
    sharpes = [m.get("sharpe_ratio", 0) for m in daily_metrics_serializable] if daily_metrics_serializable else []

    current_win_rate = win_rates[-1] if win_rates else 0
    current_profit = profits[-1] if profits else 0
    current_sharpe = sharpes[-1] if sharpes else 0

    # Market conditions
    volatility = float(market_data.get("priceChangePercent", 0)) if market_data else 0
    volume_change = float(market_data.get("volume", 0)) if market_data else 0

    prompt = f"""
You are a trading bot self-improvement AI. Analyze the following performance data and decide if model retraining is needed.

PERFORMANCE TRENDS (Last 7 days):
- Win Rate: {win_rates} (Target: {TARGET_WIN_RATE}%, Current: {current_win_rate}%)
- Avg Profit: {profits} (Target: {TARGET_AVG_PROFIT}%, Current: {current_profit}%)
- Sharpe Ratio: {sharpes} (Target: {TARGET_SHARPE}, Current: {current_sharpe})

MODEL ACCURACY:
{json.dumps(model_accuracy, indent=2, default=str)}

MARKET CONDITIONS:
- 24h Price Change: {volatility}%
- 24h Volume: {volume_change}

THRESHOLDS:
- Retrain if win rate < {CRITICAL_WIN_RATE}% for 3+ days
- Retrain if model accuracy drops > {ACCURACY_DROP_THRESHOLD * 100}% in 7 days
- Retrain if Sharpe ratio < {CRITICAL_SHARPE}
- Retrain if market regime changed significantly

QUESTION: Should we retrain the ML models? Consider:
1. Is performance degradation temporary or structural?
2. Is it due to model decay or market regime change?
3. Will retraining with recent data help?
4. What's the cost-benefit of retraining now vs waiting?

OUTPUT FORMAT (MUST BE VALID JSON):
{{
  "recommendation": "retrain" | "wait" | "optimize_parameters",
  "confidence": 0.0-1.0,
  "reasoning": "detailed explanation",
  "urgency": "low" | "medium" | "high",
  "suggested_actions": ["action1", "action2"],
  "estimated_improvement": "X% expected win rate improvement"
}}
"""

    return prompt


def _calculate_model_metrics(
    accuracy_data: List[Dict[str, Any]], model_type: str
) -> Dict[str, Any]:
    """
    Calculate model metrics from accuracy history (private helper)

    Args:
        accuracy_data: List of dicts with accuracy history
        model_type: Model type to filter for (e.g., 'lstm', 'gru')

    Returns:
        Dict with avg_accuracy and trend
    """
    # Filter for the specific model type
    model_data = [
        d for d in accuracy_data if d.get("model_type") == model_type
    ]

    if not model_data:
        return {
            "avg_accuracy": 0.0,
            "trend": "unknown",
            "count": 0,
        }

    # Calculate average accuracy
    accuracies = [d.get("accuracy", 0) for d in model_data]
    avg_accuracy = sum(accuracies) / len(accuracies) if accuracies else 0.0

    # Determine trend (simple: compare first half vs second half)
    if len(accuracies) >= 2:
        mid = len(accuracies) // 2
        first_half_avg = sum(accuracies[:mid]) / mid
        second_half_avg = sum(accuracies[mid:]) / (len(accuracies) - mid)

        if second_half_avg > first_half_avg + 2:
            trend = "improving"
        elif second_half_avg < first_half_avg - 2:
            trend = "declining"
        else:
            trend = "stable"
    else:
        trend = "unknown"

    return {
        "avg_accuracy": avg_accuracy,
        "trend": trend,
        "count": len(model_data),
        "latest_accuracy": accuracies[-1] if accuracies else 0.0,
    }


def _run_config_analysis_direct(analysis_result: Dict[str, Any] = None) -> Dict[str, Any]:
    """
    Run config improvement analysis DIRECTLY (bypass Celery/Redis).
    This is a fallback when Redis is unavailable.

    Args:
        analysis_result: GPT-4 analysis that triggered this (optional)

    Returns:
        Config improvement suggestions from GPT-4
    """
    logger.info("🧠 Running config improvement analysis DIRECTLY (bypassing Celery)...")

    try:
        # Check OpenAI API key
        api_key = os.getenv("OPENAI_API_KEY")
        if not api_key:
            logger.warning("⚠️ OPENAI_API_KEY not configured - skipping")
            return {"status": "skipped", "reason": "OPENAI_API_KEY not configured"}

        # Create OpenAI client
        client = OpenAI(api_key=api_key)

        # STEP 1: Fetch current indicator settings
        logger.info("📊 Fetching current indicator settings...")
        try:
            response = requests.get(
                f"{RUST_API_URL}/api/paper-trading/indicator-settings",
                timeout=10,
            )
            response.raise_for_status()
            indicator_settings = response.json().get("data", {})
        except Exception as e:
            logger.warning(f"⚠️ Could not fetch indicator settings: {e}")
            indicator_settings = {}

        # STEP 2: Fetch current signal settings
        logger.info("📊 Fetching current signal settings...")
        try:
            response = requests.get(
                f"{RUST_API_URL}/api/paper-trading/signal-settings",
                timeout=10,
            )
            response.raise_for_status()
            signal_settings = response.json().get("data", {})
        except Exception as e:
            logger.warning(f"⚠️ Could not fetch signal settings: {e}")
            signal_settings = {}

        # STEP 3: Fetch recent closed trades
        logger.info("📈 Fetching recent closed trades...")
        try:
            response = requests.get(
                f"{RUST_API_URL}/api/paper-trading/trades/closed",
                timeout=10,
            )
            response.raise_for_status()
            trades_response = response.json()
            trades = trades_response.get("data", []) if trades_response.get("success") else []
        except Exception as e:
            logger.warning(f"⚠️ Could not fetch trades: {e}")
            trades = []

        # Calculate trade statistics
        total_trades = len(trades)
        winning_trades = sum(1 for t in trades if (t.get("pnl_percentage") or 0) > 0)
        losing_trades = sum(1 for t in trades if (t.get("pnl_percentage") or 0) < 0)
        win_rate = (winning_trades / total_trades * 100) if total_trades > 0 else 0
        total_pnl = sum(t.get("pnl_usdt") or 0 for t in trades)
        avg_pnl_percent = sum(t.get("pnl_percentage") or 0 for t in trades) / total_trades if total_trades > 0 else 0

        # Determine risk level
        is_losing = total_pnl < 0 or win_rate < 50
        data_insufficient = total_trades < 5

        # Risk guidance
        if data_insufficient:
            risk_guidance = f"""
## ⚠️ CRITICAL: INSUFFICIENT DATA WARNING
You only have {total_trades} trades to analyze. This is NOT enough data.
- DO NOT suggest aggressive changes
- Set confidence BELOW 0.3
- Set auto_apply_safe to FALSE
"""
        elif is_losing:
            risk_guidance = f"""
## ⚠️ CRITICAL: BOT IS LOSING MONEY
Current PnL: ${total_pnl:.2f} | Win Rate: {win_rate:.1f}%
- NEVER lower thresholds (more risk)
- Consider INCREASING thresholds (be more selective)
- Priority is CAPITAL PRESERVATION
"""
        else:
            risk_guidance = "## ✅ BOT IS PROFITABLE - Cautious optimization allowed"

        prompt = f"""
You are a CONSERVATIVE trading bot configuration optimizer.

{risk_guidance}

## CURRENT INDICATOR SETTINGS:
{json.dumps(indicator_settings, indent=2)}

## CURRENT SIGNAL SETTINGS:
{json.dumps(signal_settings, indent=2)}

## TRADE PERFORMANCE:
- Total Trades: {total_trades}
- Win Rate: {win_rate:.1f}%
- Total PnL: ${total_pnl:.2f}
- Avg PnL per Trade: {avg_pnl_percent:.2f}%

## RECENT TRADES (last 5):
{json.dumps(trades[:5], indent=2, default=str)}

## OUTPUT FORMAT (VALID JSON):
{{
  "analysis": {{
    "root_cause": "Brief explanation",
    "key_issues": ["issue1", "issue2"],
    "data_quality": "insufficient/limited/good"
  }},
  "indicator_suggestions": {{}},
  "signal_suggestions": {{}},
  "confidence": 0.0-1.0,
  "auto_apply_safe": false,
  "risk_assessment": "low/medium/high",
  "summary": "One paragraph summary"
}}
"""

        # Call GPT-4
        logger.info("🤖 Calling GPT-4 for config analysis...")
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[
                {
                    "role": "system",
                    "content": "You are a trading bot config optimizer. Always respond with valid JSON.",
                },
                {"role": "user", "content": prompt},
            ],
            temperature=0.3,
            max_completion_tokens=2000,
        )

        gpt4_response = response.choices[0].message.content

        # Strip markdown code blocks
        if gpt4_response.startswith("```"):
            lines = gpt4_response.split("\n")
            lines = [l for l in lines if not l.strip().startswith("```")]
            gpt4_response = "\n".join(lines)

        suggestions = json.loads(gpt4_response)

        logger.info(f"🤖 GPT-4 Config Analysis Complete:")
        logger.info(f"  📋 Root Cause: {suggestions.get('analysis', {}).get('root_cause', 'N/A')}")
        logger.info(f"  🎯 Confidence: {suggestions.get('confidence', 0):.0%}")

        # Store in MongoDB
        result = {
            "status": "success",
            "timestamp": datetime.utcnow().isoformat(),
            "mode": "direct_bypass_celery",
            "trigger_analysis": analysis_result,
            "current_config": {
                "indicators": indicator_settings,
                "signal": signal_settings,
            },
            "trade_stats": {
                "total_trades": total_trades,
                "win_rate": win_rate,
                "total_pnl": total_pnl,
            },
            "suggestions": suggestions,
            "applied_changes": [],  # Never auto-apply in direct mode
            "task_id": None,
        }

        storage.store_config_suggestions(result)

        # Send notification
        notifications.send_config_suggestions(result)

        logger.info("✅ Config improvement analysis complete (direct mode)")
        return result

    except json.JSONDecodeError as e:
        logger.error(f"❌ Failed to parse GPT-4 response: {e}")
        return {"status": "error", "error": str(e)}
    except Exception as e:
        logger.error(f"❌ Direct config analysis failed: {e}")
        return {"status": "error", "error": str(e)}
