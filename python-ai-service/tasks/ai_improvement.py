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
import openai

logger = get_logger("AIImprovementTasks")

# OpenAI configuration
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
if OPENAI_API_KEY:
    openai.api_key = OPENAI_API_KEY

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

        # Set OpenAI API key for this request
        openai.api_key = api_key

        # STEP 1: Fetch last 7 days performance data
        logger.info("📊 Fetching 7-day performance data...")

        response = requests.get(
            f"{RUST_API_URL}/api/paper-trading/trades",
            params={"days": 7, "limit": 1000},
            timeout=10,
        )
        response.raise_for_status()
        trades = response.json()

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
            response = openai.ChatCompletion.create(
                model="gpt-4-turbo-preview",
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
                max_tokens=1000,
            )

            gpt4_response = response.choices[0].message.content

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

            if should_retrain:
                logger.warning(
                    f"🚨 GPT-4 recommends RETRAIN with {confidence:.0%} confidence (urgency: {urgency})"
                )
                logger.info(f"🚀 Triggering adaptive retraining task...")

                # Queue adaptive retraining task
                from tasks.ai_improvement import adaptive_retrain

                retrain_task = adaptive_retrain.delay(
                    model_types=["lstm", "gru", "transformer"],
                    analysis_result=analysis_result,
                )

                logger.info(f"✅ Adaptive retraining task queued: {retrain_task.id}")

                analysis_result["retrain_task_id"] = retrain_task.id
                analysis_result["retrain_triggered"] = True
                trigger_retrain = True
            else:
                logger.info(
                    f"✅ No immediate retraining needed (recommendation: {recommendation}, confidence: {confidence:.0%})"
                )
                analysis_result["retrain_triggered"] = False
                trigger_retrain = False

            # STEP 8: Send GPT-4 analysis notification
            notifications.send_gpt4_analysis(analysis_result)

            # STEP 9: Store GPT-4 analysis in MongoDB for audit trail
            storage.store_gpt4_analysis(analysis_result, self.request.id)

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
# TASK 2: ADAPTIVE MODEL RETRAINING (Triggered by GPT-4 recommendation)
# =============================================================================


# @spec:FR-ASYNC-009 - Adaptive Model Retraining
# @ref:specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md#fr-async-009
# @test:TC-ASYNC-061, TC-ASYNC-062, TC-ASYNC-063, TC-ASYNC-064, TC-ASYNC-065
@app.task(
    bind=True,
    base=AIImprovementTask,
    name="tasks.ai_improvement.adaptive_retrain",
    time_limit=7200,  # 2 hours max
)
def adaptive_retrain(
    self,
    model_types: List[str],
    analysis_result: Dict[str, Any],
) -> Dict[str, Any]:
    """
    Adaptively retrain ML models based on GPT-4 recommendation
    Triggered by: gpt4_self_analysis task

    Process:
    1. Fetch last 60 days of market data
    2. Train models (LSTM, GRU, Transformer)
    3. Validate on holdout set
    4. Compare with current model accuracy
    5. Deploy if accuracy improves
    6. Send notification with before/after metrics

    Args:
        model_types: List of models to retrain
        analysis_result: GPT-4 analysis that triggered this

    Returns:
        Retraining results with before/after comparison
    """
    logger.info(f"🚀 Starting adaptive retraining for {len(model_types)} models...")
    logger.info(
        f"📋 Triggered by GPT-4 recommendation: {analysis_result.get('reasoning', 'N/A')[:100]}..."
    )

    try:
        results = {
            "timestamp": datetime.utcnow().isoformat(),
            "trigger": "gpt4_recommendation",
            "analysis_reasoning": analysis_result.get("reasoning"),
            "models": {},
        }

        for model_type in model_types:
            logger.info(f"🧠 Retraining {model_type.upper()} model...")

            # Update Celery task state only if we have a valid task context
            if hasattr(self, 'request') and self.request.id:
                self.update_state(
                    state="PROGRESS",
                    meta={
                        "current_model": model_type,
                        "status": f"Retraining {model_type} model...",
                    },
                )

            try:
                # Call Python AI service to retrain model
                response = requests.post(
                    f"{PYTHON_API_URL}/ai/train",
                    json={
                        "model_type": model_type,
                        "days_of_data": 60,  # Last 60 days
                        "retrain": True,
                    },
                    timeout=3600,  # 1 hour timeout per model
                )
                response.raise_for_status()
                retrain_result = response.json()

                results["models"][model_type] = {
                    "status": "success",
                    "old_accuracy": retrain_result.get("old_accuracy", 0),
                    "new_accuracy": retrain_result.get("new_accuracy", 0),
                    "improvement": retrain_result.get("improvement", 0),
                    "deployed": retrain_result.get("deployed", False),
                }

                logger.info(
                    f"✅ {model_type.upper()}: {retrain_result.get('new_accuracy', 0):.2%} accuracy"
                )

            except Exception as e:
                logger.error(f"❌ Failed to retrain {model_type}: {e}")
                results["models"][model_type] = {
                    "status": "failed",
                    "error": str(e),
                }

        # Summary
        successful_retrains = sum(
            1 for m in results["models"].values() if m.get("status") == "success"
        )
        deployed_models = sum(
            1 for m in results["models"].values() if m.get("deployed", False)
        )

        logger.info(f"✅ Adaptive retraining complete:")
        logger.info(f"  📊 Successful: {successful_retrains}/{len(model_types)} models")
        logger.info(f"  🚀 Deployed: {deployed_models} improved models")

        # Send completion notification
        notifications.send_retrain_complete(results)

        # Store retrain results in MongoDB
        storage.store_retrain_history(
            retrain_data=results,
            task_id=self.request.id if hasattr(self, 'request') else None,
        )

        return {
            "status": "success",
            "retrain_results": [{"model": m, **results["models"][m]} for m in results["models"]],
            "results": results,
            "task_id": self.request.id if hasattr(self, 'request') else None,
        }

    except Exception as e:
        logger.error(f"❌ Adaptive retraining failed: {e}")
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
        date = trade.get("executed_at", "")[:10]  # Extract YYYY-MM-DD
        daily_trades[date].append(trade)

    # Calculate metrics for each day
    daily_metrics = []
    for date in sorted(daily_trades.keys()):
        day_trades = daily_trades[date]

        winning = sum(1 for t in day_trades if t.get("profit_percent", 0) > 0)
        total = len(day_trades)
        win_rate = (winning / total * 100) if total > 0 else 0

        profits = [t.get("profit_percent", 0) for t in day_trades]
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
