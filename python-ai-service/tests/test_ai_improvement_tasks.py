#!/usr/bin/env python3
"""
Unit Tests for AI Improvement Tasks
Tests GPT-4 self-analysis, adaptive retraining, and emergency strategy disable
"""

import pytest
from unittest.mock import patch, MagicMock, Mock
from datetime import datetime, timedelta
import json

# Skip all tests if celery is not installed
try:
    import celery
    CELERY_AVAILABLE = True
except ImportError:
    CELERY_AVAILABLE = False

pytestmark = pytest.mark.skipif(not CELERY_AVAILABLE, reason="Celery not installed")


class TestGPT4SelfAnalysis:
    """Test gpt4_self_analysis task"""

    @patch.dict("os.environ", {"OPENAI_API_KEY": "test_key"})
    @patch("tasks.ai_improvement.adaptive_retrain")
    @patch("tasks.ai_improvement.requests.get")
    @patch("tasks.ai_improvement.openai.ChatCompletion.create")
    @patch("tasks.ai_improvement.storage")
    def test_gpt4_analysis_recommends_retrain(self, mock_storage, mock_openai, mock_requests, mock_adaptive_retrain):
        """Test GPT-4 analysis recommending retraining"""
        from tasks.ai_improvement import gpt4_self_analysis

        # Mock adaptive_retrain.delay() to prevent Celery/Redis connection
        mock_task = MagicMock()
        mock_task.id = "test-task-id-123"
        mock_adaptive_retrain.delay.return_value = mock_task

        # Mock HTTP requests
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = []  # Empty trades list
        mock_requests.return_value = mock_response

        # Mock performance data showing decline
        mock_storage.get_performance_metrics_history.return_value = [
            {
                "date": datetime.now().date(),
                "win_rate": 45.0,
                "avg_profit": 0.5,
                "sharpe_ratio": 0.8,
                "total_trades": 20,
            }
        ]

        mock_storage.get_model_accuracy_history.return_value = [
            {
                "timestamp": datetime.now(),
                "model_type": "lstm",
                "accuracy": 62.0,
                "loss": 0.45,
            }
        ]

        # Mock GPT-4 response
        mock_openai.return_value = MagicMock(
            choices=[
                MagicMock(
                    message=MagicMock(
                        content=json.dumps(
                            {
                                "recommendation": "retrain",
                                "confidence": 0.85,
                                "reasoning": "Model accuracy has dropped below 65%, win rate declining",
                                "urgency": "high",
                                "suggested_actions": ["retrain_models"],
                                "models_to_retrain": ["lstm", "gru"],
                                "estimated_improvement": "8-12%",
                            }
                        )
                    )
                )
            ],
            usage=MagicMock(
                prompt_tokens=1200, completion_tokens=450, total_tokens=1650
            ),
        )

        result = gpt4_self_analysis()

        assert result["status"] == "success"
        assert result["analysis"]["recommendation"] == "retrain"
        assert result["analysis"]["confidence"] == 0.85
        assert result["trigger_retrain"] is True
        assert "lstm" in result["analysis"]["models_to_retrain"]

    @patch.dict("os.environ", {"OPENAI_API_KEY": "test_key"})
    @patch("tasks.ai_improvement.requests.get")
    @patch("tasks.ai_improvement.openai.ChatCompletion.create")
    @patch("tasks.ai_improvement.storage")
    def test_gpt4_analysis_recommends_wait(self, mock_storage, mock_openai, mock_requests):
        """Test GPT-4 analysis recommending to wait"""
        from tasks.ai_improvement import gpt4_self_analysis

        # Mock HTTP requests
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = []
        mock_requests.return_value = mock_response

        # Mock good performance data
        mock_storage.get_performance_metrics_history.return_value = [
            {
                "date": datetime.now().date(),
                "win_rate": 72.0,
                "avg_profit": 2.3,
                "sharpe_ratio": 1.8,
                "total_trades": 25,
            }
        ]

        mock_storage.get_model_accuracy_history.return_value = [
            {
                "timestamp": datetime.now(),
                "model_type": "lstm",
                "accuracy": 75.0,
                "loss": 0.25,
            }
        ]

        # Mock GPT-4 response
        mock_openai.return_value = MagicMock(
            choices=[
                MagicMock(
                    message=MagicMock(
                        content=json.dumps(
                            {
                                "recommendation": "wait",
                                "confidence": 0.90,
                                "reasoning": "Performance is strong, no retraining needed",
                                "urgency": "low",
                                "suggested_actions": ["continue_monitoring"],
                                "estimated_improvement": "N/A",
                            }
                        )
                    )
                )
            ],
            usage=MagicMock(
                prompt_tokens=1100, completion_tokens=380, total_tokens=1480
            ),
        )

        result = gpt4_self_analysis()

        assert result["status"] == "success"
        assert result["analysis"]["recommendation"] == "wait"
        assert result["analysis"]["confidence"] == 0.90
        assert result["trigger_retrain"] is False

    @patch.dict("os.environ", {"OPENAI_API_KEY": "test_key"})
    @patch("tasks.ai_improvement.requests.get")
    @patch("tasks.ai_improvement.openai.ChatCompletion.create")
    @patch("tasks.ai_improvement.storage")
    def test_gpt4_analysis_recommends_optimize(self, mock_storage, mock_openai, mock_requests):
        """Test GPT-4 analysis recommending parameter optimization"""
        from tasks.ai_improvement import gpt4_self_analysis

        # Mock HTTP requests
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = []
        mock_requests.return_value = mock_response

        mock_storage.get_performance_metrics_history.return_value = [
            {"win_rate": 68.0, "avg_profit": 1.8, "sharpe_ratio": 1.5}
        ]

        mock_storage.get_model_accuracy_history.return_value = [
            {"model_type": "lstm", "accuracy": 70.0}
        ]

        # Mock GPT-4 recommending optimization
        mock_openai.return_value = MagicMock(
            choices=[
                MagicMock(
                    message=MagicMock(
                        content=json.dumps(
                            {
                                "recommendation": "optimize_parameters",
                                "confidence": 75,
                                "reasoning": "Performance is good but can be improved with parameter tuning",
                                "priority": "medium",
                                "parameters_to_optimize": [
                                    "learning_rate",
                                    "batch_size",
                                ],
                                "models_to_retrain": [],
                            }
                        )
                    )
                )
            ],
            usage=MagicMock(
                prompt_tokens=1150, completion_tokens=400, total_tokens=1550
            ),
        )

        result = gpt4_self_analysis()

        assert result["status"] == "success"
        assert result["analysis"]["recommendation"] == "optimize_parameters"
        assert result["trigger_retrain"] is False

    @patch.dict("os.environ", {"OPENAI_API_KEY": "test_key"})
    @patch("tasks.ai_improvement.requests.get")
    @patch("tasks.ai_improvement.storage")
    @patch("tasks.ai_improvement.openai.ChatCompletion.create")
    def test_gpt4_analysis_openai_error(self, mock_openai, mock_storage, mock_requests):
        """Test GPT-4 analysis handles OpenAI API errors"""
        from tasks.ai_improvement import gpt4_self_analysis

        # Mock HTTP requests
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = []
        mock_requests.return_value = mock_response

        # Mock storage
        mock_storage.get_performance_metrics_history.return_value = []
        mock_storage.get_model_accuracy_history.return_value = []

        # Mock OpenAI API error
        mock_openai.side_effect = Exception("OpenAI API rate limit exceeded")

        with pytest.raises(Exception):
            gpt4_self_analysis()

    @patch("tasks.ai_improvement.storage")
    def test_gpt4_analysis_no_api_key(self, mock_storage):
        """Test GPT-4 analysis skips when API key is missing"""
        from tasks.ai_improvement import gpt4_self_analysis
        import os

        # Temporarily remove API key
        old_key = os.environ.get("OPENAI_API_KEY")
        if "OPENAI_API_KEY" in os.environ:
            del os.environ["OPENAI_API_KEY"]

        try:
            result = gpt4_self_analysis()
            # Should return skipped status instead of raising exception
            assert result["status"] == "skipped"
            assert result["reason"] == "OPENAI_API_KEY not configured"
        finally:
            # Restore API key
            if old_key:
                os.environ["OPENAI_API_KEY"] = old_key

    @patch.dict("os.environ", {"OPENAI_API_KEY": "test_key"})
    @patch("tasks.ai_improvement.adaptive_retrain")
    @patch("tasks.ai_improvement.requests.get")
    @patch("tasks.ai_improvement.openai.ChatCompletion.create")
    @patch("tasks.ai_improvement.storage")
    def test_gpt4_analysis_insufficient_data(self, mock_storage, mock_openai, mock_requests, mock_adaptive_retrain):
        """Test GPT-4 analysis with insufficient historical data"""
        from tasks.ai_improvement import gpt4_self_analysis

        # Mock HTTP requests
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = []
        mock_requests.return_value = mock_response

        # Mock insufficient data (less than 3 days)
        mock_storage.get_performance_metrics_history.return_value = [
            {"date": datetime.now().date(), "win_rate": 70.0}
        ]

        mock_storage.get_model_accuracy_history.return_value = []

        # Mock GPT-4 response (even though we expect insufficient data, provide a valid response)
        mock_openai.return_value = MagicMock(
            choices=[
                MagicMock(
                    message=MagicMock(
                        content=json.dumps({
                            "recommendation": "wait",
                            "confidence": 0.50,
                            "reasoning": "Insufficient data for analysis",
                            "urgency": "low",
                            "suggested_actions": ["gather_more_data"],
                        })
                    )
                )
            ],
            usage=MagicMock(prompt_tokens=500, completion_tokens=100, total_tokens=600),
        )

        result = gpt4_self_analysis(force_analysis=False)

        assert result["status"] in ["success", "skipped"]

    def test_gpt4_analysis_task_registered(self):
        """Test that GPT-4 analysis task is registered"""
        from tasks.ai_improvement import gpt4_self_analysis

        assert gpt4_self_analysis.name == "tasks.ai_improvement.gpt4_self_analysis"


class TestAdaptiveRetrain:
    """Test adaptive_retrain task"""

    @patch("tasks.ai_improvement.requests.post")
    @patch("tasks.ai_improvement.storage")
    def test_adaptive_retrain_single_model(self, mock_storage, mock_post):
        """Test adaptive retraining of a single model"""
        from tasks.ai_improvement import adaptive_retrain

        # Mock training API response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "status": "success",
            "model": "lstm",
            "accuracy_before": 62.0,
            "accuracy_after": 73.0,
            "improvement": 11.0,
            "training_time": 450,
        }
        mock_post.return_value = mock_response

        analysis_result = {
            "recommendation": "retrain",
            "confidence": 85,
            "models_to_retrain": ["lstm"],
            "reasoning": "Accuracy dropped below threshold",
        }

        result = adaptive_retrain(model_types=["lstm"], analysis_result=analysis_result)

        assert result["status"] == "success"
        assert len(result["retrain_results"]) == 1
        assert result["retrain_results"][0]["model"] == "lstm"
        assert result["retrain_results"][0]["improvement"] == 11.0

    @patch("tasks.ai_improvement.requests.post")
    @patch("tasks.ai_improvement.storage")
    def test_adaptive_retrain_multiple_models(self, mock_storage, mock_post):
        """Test adaptive retraining of multiple models"""
        from tasks.ai_improvement import adaptive_retrain

        # Mock different responses for different models
        def mock_post_side_effect(url, **kwargs):
            mock_resp = MagicMock()
            mock_resp.status_code = 200

            if "lstm" in str(kwargs.get("json", {})):
                mock_resp.json.return_value = {
                    "status": "success",
                    "model": "lstm",
                    "accuracy_after": 73.0,
                    "improvement": 11.0,
                }
            elif "gru" in str(kwargs.get("json", {})):
                mock_resp.json.return_value = {
                    "status": "success",
                    "model": "gru",
                    "accuracy_after": 71.0,
                    "improvement": 9.0,
                }
            return mock_resp

        mock_post.side_effect = mock_post_side_effect

        analysis_result = {
            "recommendation": "retrain",
            "models_to_retrain": ["lstm", "gru"],
        }

        result = adaptive_retrain(
            model_types=["lstm", "gru"], analysis_result=analysis_result
        )

        assert result["status"] == "success"
        assert len(result["retrain_results"]) == 2

    @patch("tasks.ai_improvement.storage")
    @patch("tasks.ai_improvement.requests.post")
    def test_adaptive_retrain_api_failure(self, mock_post, mock_storage):
        """Test adaptive retrain handles training API failures gracefully"""
        from tasks.ai_improvement import adaptive_retrain

        # Mock API failure
        mock_post.side_effect = Exception("Training API unavailable")

        analysis_result = {"recommendation": "retrain", "models_to_retrain": ["lstm"]}

        result = adaptive_retrain(model_types=["lstm"], analysis_result=analysis_result)

        # Should return success status but with failed model
        assert result["status"] == "success"
        assert len(result["retrain_results"]) == 1
        assert result["retrain_results"][0]["status"] == "failed"
        assert "Training API unavailable" in result["retrain_results"][0]["error"]

    @patch("tasks.ai_improvement.requests.post")
    @patch("tasks.ai_improvement.storage")
    def test_adaptive_retrain_stores_history(self, mock_storage, mock_post):
        """Test that retrain results are stored in MongoDB"""
        from tasks.ai_improvement import adaptive_retrain

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "status": "success",
            "model": "lstm",
            "accuracy_after": 75.0,
        }
        mock_post.return_value = mock_response

        analysis_result = {"recommendation": "retrain"}

        result = adaptive_retrain(model_types=["lstm"], analysis_result=analysis_result)

        # Verify storage was called
        assert mock_storage.store_retrain_history.called

    def test_adaptive_retrain_task_registered(self):
        """Test that adaptive retrain task is registered"""
        from tasks.ai_improvement import adaptive_retrain

        assert adaptive_retrain.name == "tasks.ai_improvement.adaptive_retrain"


class TestEmergencyStrategyDisable:
    """Test emergency_strategy_disable task"""

    @patch("tasks.ai_improvement.requests.post")
    @patch("tasks.ai_improvement.storage")
    def test_emergency_disable_high_loss(self, mock_storage, mock_post):
        """Test emergency disable on high daily loss"""
        from tasks.ai_improvement import emergency_strategy_disable

        # Mock API response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"status": "disabled"}
        mock_post.return_value = mock_response

        result = emergency_strategy_disable(
            strategy_name="rsi_strategy",
            reason="Daily loss exceeded 10% threshold (-12.5%)",
        )

        assert result["status"] == "success"
        assert result["strategy_disabled"] == "rsi_strategy"
        assert "Daily loss" in result["reason"]

    @patch("tasks.ai_improvement.requests.post")
    @patch("tasks.ai_improvement.storage")
    def test_emergency_disable_consecutive_losses(self, mock_storage, mock_post):
        """Test emergency disable on consecutive losses"""
        from tasks.ai_improvement import emergency_strategy_disable

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"status": "disabled"}
        mock_post.return_value = mock_response

        result = emergency_strategy_disable(
            strategy_name="macd_strategy", reason="10 consecutive losses detected"
        )

        assert result["status"] == "success"
        assert result["strategy_disabled"] == "macd_strategy"
        assert "consecutive" in result["reason"].lower()

    @patch("tasks.ai_improvement.requests.post")
    def test_emergency_disable_api_failure(self, mock_post):
        """Test emergency disable handles API failures"""
        from tasks.ai_improvement import emergency_strategy_disable

        # Mock API failure
        mock_post.side_effect = Exception("Rust API unavailable")

        with pytest.raises(Exception):
            emergency_strategy_disable(
                strategy_name="test_strategy", reason="Test failure"
            )

    @patch("tasks.ai_improvement.notifications")
    @patch("tasks.ai_improvement.requests.post")
    def test_emergency_disable_sends_notification(self, mock_post, mock_notifications):
        """Test that emergency disable sends critical notification"""
        from tasks.ai_improvement import emergency_strategy_disable

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"status": "disabled"}
        mock_post.return_value = mock_response

        result = emergency_strategy_disable(
            strategy_name="test_strategy", reason="Test emergency"
        )

        # Verify critical notification was sent
        assert mock_notifications.send_critical.called

    def test_emergency_disable_task_registered(self):
        """Test that emergency disable task is registered"""
        from tasks.ai_improvement import emergency_strategy_disable

        assert (
            emergency_strategy_disable.name
            == "tasks.ai_improvement.emergency_strategy_disable"
        )


class TestAIImprovementTasksIntegration:
    """Integration tests for AI improvement tasks"""

    def test_all_ai_tasks_registered(self):
        """Test that all AI improvement tasks are registered"""
        from celery_app import app

        registered_tasks = list(app.tasks.keys())

        expected_tasks = [
            "tasks.ai_improvement.gpt4_self_analysis",
            "tasks.ai_improvement.adaptive_retrain",
            "tasks.ai_improvement.emergency_strategy_disable",
        ]

        for task in expected_tasks:
            assert task in registered_tasks, f"Task {task} not registered"

    def test_gpt4_triggers_adaptive_retrain(self):
        """Test that GPT-4 analysis can trigger adaptive retraining"""
        from tasks.ai_improvement import gpt4_self_analysis, adaptive_retrain

        # This is more of a workflow test
        # GPT-4 returns recommendation -> adaptive_retrain is called
        # We verify the task structure supports this workflow

        assert hasattr(gpt4_self_analysis, "run")
        assert hasattr(adaptive_retrain, "run")

    @patch.dict("os.environ", {"OPENAI_API_KEY": "test_key"})
    @patch("tasks.ai_improvement.requests.get")
    @patch("tasks.ai_improvement.storage")
    def test_ai_tasks_use_data_storage(self, mock_storage, mock_requests):
        """Test that AI tasks properly use data storage"""
        from tasks.ai_improvement import gpt4_self_analysis

        # Mock HTTP requests
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = []
        mock_requests.return_value = mock_response

        # Mock data
        mock_storage.get_performance_metrics_history.return_value = []
        mock_storage.get_model_accuracy_history.return_value = []

        # Even with empty data, storage should be called
        try:
            gpt4_self_analysis(force_analysis=False)
        except Exception:
            pass  # Expected to fail without proper mocks

        assert mock_storage.get_performance_metrics_history.called
        assert mock_storage.get_model_accuracy_history.called


class TestAITasksHelperFunctions:
    """Test helper functions used by AI tasks"""

    @patch("tasks.ai_improvement.storage")
    def test_build_gpt4_prompt(self, mock_storage):
        """Test GPT-4 prompt building function"""
        from tasks.ai_improvement import _build_gpt4_analysis_prompt

        mock_storage.get_performance_metrics_history.return_value = [
            {"win_rate": 70.0, "avg_profit": 2.0}
        ]

        mock_storage.get_model_accuracy_history.return_value = [
            {"model_type": "lstm", "accuracy": 72.0}
        ]

        prompt = _build_gpt4_analysis_prompt()

        assert isinstance(prompt, str)
        assert len(prompt) > 100
        assert "win_rate" in prompt.lower() or "performance" in prompt.lower()

    def test_calculate_model_metrics(self):
        """Test model metrics calculation"""
        from tasks.ai_improvement import _calculate_model_metrics

        accuracy_data = [
            {"model_type": "lstm", "accuracy": 72.0, "timestamp": datetime.now()},
            {
                "model_type": "lstm",
                "accuracy": 68.0,
                "timestamp": datetime.now() - timedelta(days=1),
            },
            {
                "model_type": "lstm",
                "accuracy": 75.0,
                "timestamp": datetime.now() - timedelta(days=2),
            },
        ]

        metrics = _calculate_model_metrics(accuracy_data, "lstm")

        assert "avg_accuracy" in metrics
        assert "trend" in metrics
        assert metrics["avg_accuracy"] > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
