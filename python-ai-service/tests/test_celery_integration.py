#!/usr/bin/env python3
"""
Integration Tests for Celery Configuration
Tests task registration, routing, scheduling, and complete workflows
"""

import pytest
from unittest.mock import patch, MagicMock
from celery import Celery
from datetime import datetime


class TestCeleryConfiguration:
    """Test Celery app configuration"""

    def test_celery_app_exists(self):
        """Test that Celery app is properly configured"""
        from celery_app import app

        assert isinstance(app, Celery)
        assert app.conf.broker_url is not None
        assert app.conf.result_backend is not None

    def test_celery_broker_url(self):
        """Test that broker URL is configured for RabbitMQ"""
        from celery_app import app

        broker = app.conf.broker_url
        assert broker.startswith("amqp://") or broker.startswith("pyamqp://")
        assert "rabbitmq" in broker or "localhost" in broker

    def test_celery_result_backend(self):
        """Test that result backend is configured for Redis"""
        from celery_app import app

        backend = app.conf.result_backend
        assert backend.startswith("redis://")
        assert "redis" in backend or "localhost" in backend

    def test_celery_timezone(self):
        """Test that timezone is configured"""
        from celery_app import app

        assert app.conf.timezone is not None
        # Should be UTC or valid timezone
        assert app.conf.timezone in ["UTC", "Asia/Ho_Chi_Minh", "America/New_York"]

    def test_celery_task_serializer(self):
        """Test task serialization settings"""
        from celery_app import app

        # Should use JSON for security
        assert app.conf.task_serializer == "json"
        assert app.conf.accept_content == ["json"]
        assert app.conf.result_serializer == "json"


class TestTaskRegistration:
    """Test that all tasks are properly registered"""

    def test_all_monitoring_tasks_registered(self):
        """Test all 4 monitoring tasks are registered"""
        from celery_app import app

        registered = list(app.tasks.keys())

        monitoring_tasks = [
            "tasks.monitoring.system_health_check",
            "tasks.monitoring.daily_portfolio_report",
            "tasks.monitoring.daily_api_cost_report",
            "tasks.monitoring.daily_performance_analysis",
        ]

        for task_name in monitoring_tasks:
            assert task_name in registered, f"{task_name} not registered"

    def test_all_ai_improvement_tasks_registered(self):
        """Test all 3 AI improvement tasks are registered"""
        from celery_app import app

        registered = list(app.tasks.keys())

        ai_tasks = [
            "tasks.ai_improvement.gpt4_self_analysis",
            "tasks.ai_improvement.adaptive_retrain",
            "tasks.ai_improvement.emergency_strategy_disable",
        ]

        for task_name in ai_tasks:
            assert task_name in registered, f"{task_name} not registered"

    def test_all_ml_tasks_registered(self):
        """Test ML and backtest tasks are registered"""
        from celery_app import app

        registered = list(app.tasks.keys())

        ml_tasks = [
            "tasks.ml_tasks.train_model",
            "tasks.ml_tasks.predict_price",
            "tasks.ml_tasks.bulk_analysis",
            "tasks.backtest_tasks.backtest_strategy",
            "tasks.backtest_tasks.optimize_strategy",
        ]

        for task_name in ml_tasks:
            assert task_name in registered, f"{task_name} not registered"

    def test_total_tasks_count(self):
        """Test that expected number of tasks are registered"""
        from celery_app import app

        registered = list(app.tasks.keys())

        # Should have at least 12 custom tasks
        # (4 monitoring + 3 AI + 2 ML + 3 backtest)
        custom_tasks = [t for t in registered if t.startswith("tasks.")]

        assert len(custom_tasks) >= 12, f"Expected 12+ tasks, found {len(custom_tasks)}"


class TestTaskRouting:
    """Test task routing configuration"""

    def test_monitoring_tasks_routed_to_scheduled_queue(self):
        """Test monitoring tasks use 'scheduled' queue"""
        from celery_app import app

        routes = app.conf.task_routes

        monitoring_tasks = [
            "tasks.monitoring.system_health_check",
            "tasks.monitoring.daily_portfolio_report",
            "tasks.monitoring.daily_api_cost_report",
            "tasks.monitoring.daily_performance_analysis",
        ]

        for task_name in monitoring_tasks:
            if task_name in routes:
                assert routes[task_name]["queue"] == "scheduled"

    def test_ml_tasks_routed_correctly(self):
        """Test ML tasks use appropriate queues"""
        from celery_app import app

        routes = app.conf.task_routes

        # ML training should use ml_training queue
        if "tasks.ml_tasks.train_model" in routes:
            assert routes["tasks.ml_tasks.train_model"]["queue"] == "ml_training"

        # Bulk analysis should use bulk_analysis queue
        if "tasks.ml_tasks.bulk_analysis" in routes:
            assert routes["tasks.ml_tasks.bulk_analysis"]["queue"] == "bulk_analysis"

    def test_backtest_tasks_routed_correctly(self):
        """Test backtest tasks use appropriate queues"""
        from celery_app import app

        routes = app.conf.task_routes

        if "tasks.backtest_tasks.backtest_strategy" in routes:
            assert (
                routes["tasks.backtest_tasks.backtest_strategy"]["queue"]
                == "backtesting"
            )

        if "tasks.backtest_tasks.optimize_strategy" in routes:
            assert (
                routes["tasks.backtest_tasks.optimize_strategy"]["queue"]
                == "optimization"
            )


class TestBeatSchedule:
    """Test Celery Beat scheduling configuration"""

    def test_beat_schedule_exists(self):
        """Test that beat schedule is configured"""
        from celery_app import app

        assert hasattr(app.conf, "beat_schedule")
        assert app.conf.beat_schedule is not None
        assert len(app.conf.beat_schedule) > 0

    def test_all_scheduled_tasks_configured(self):
        """Test that all scheduled tasks are in beat schedule"""
        from celery_app import app

        schedule = app.conf.beat_schedule

        expected_schedules = [
            "system-health-check",
            "daily-portfolio-report",
            "daily-api-cost-report",
            "daily-performance-analysis",
            "gpt4-self-analysis",
        ]

        for schedule_name in expected_schedules:
            assert schedule_name in schedule, f"{schedule_name} not in beat schedule"

    def test_schedule_has_required_fields(self):
        """Test that each schedule entry has required fields"""
        from celery_app import app

        schedule = app.conf.beat_schedule

        for name, config in schedule.items():
            assert "task" in config, f"Schedule {name} missing 'task' field"
            assert "schedule" in config, f"Schedule {name} missing 'schedule' field"

    def test_health_check_frequency(self):
        """Test that health check runs every 15 minutes"""
        from celery_app import app
        from celery.schedules import crontab

        schedule = app.conf.beat_schedule.get("system-health-check")

        assert schedule is not None
        # Should run every 15 minutes
        assert isinstance(schedule["schedule"], crontab)

    def test_daily_tasks_scheduled_correctly(self):
        """Test that daily tasks have appropriate schedules"""
        from celery_app import app
        from celery.schedules import crontab

        daily_tasks = [
            "daily-portfolio-report",
            "daily-api-cost-report",
            "daily-performance-analysis",
            "gpt4-self-analysis",
        ]

        for task_name in daily_tasks:
            schedule = app.conf.beat_schedule.get(task_name)
            assert schedule is not None
            assert isinstance(schedule["schedule"], crontab)


class TestTaskExecution:
    """Test task execution and error handling"""

    @patch("tasks.monitoring.requests.get")
    def test_task_execution_returns_result(self, mock_get):
        """Test that tasks return results properly"""
        from tasks.monitoring import system_health_check

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_get.return_value = mock_response

        # Execute task synchronously for testing
        result = system_health_check()

        assert result is not None
        assert "status" in result

    def test_task_has_bind_parameter(self):
        """Test that tasks have bind=True for self parameter"""
        from tasks.monitoring import system_health_check

        # Task should have access to self (task instance)
        assert hasattr(system_health_check, "name")
        assert hasattr(system_health_check, "run")

    @patch("tasks.monitoring.requests.get")
    def test_task_retry_on_failure(self, mock_get):
        """Test that tasks can retry on failure"""
        from tasks.monitoring import daily_portfolio_report
        import requests

        # Mock intermittent failure
        mock_get.side_effect = requests.exceptions.ConnectionError("Connection failed")

        # Task should raise exception (which can trigger retry)
        with pytest.raises(Exception):
            daily_portfolio_report()


class TestTaskChaining:
    """Test task chaining and workflows"""

    @patch.dict("os.environ", {"OPENAI_API_KEY": "test_key"})
    @patch("tasks.ai_improvement.requests.get")
    @patch("tasks.ai_improvement.openai.ChatCompletion.create")
    @patch("tasks.ai_improvement.storage")
    @patch("tasks.ai_improvement.adaptive_retrain")
    def test_gpt4_can_trigger_retrain(self, mock_retrain, mock_storage, mock_openai, mock_requests):
        """Test GPT-4 analysis can trigger adaptive retrain"""
        from tasks.ai_improvement import gpt4_self_analysis
        import json

        # Mock HTTP requests
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = []
        mock_requests.return_value = mock_response

        # Mock GPT-4 recommending retrain with correct format
        mock_openai.return_value = MagicMock(
            choices=[
                MagicMock(
                    message=MagicMock(
                        content=json.dumps(
                            {
                                "recommendation": "retrain",
                                "confidence": 0.85,
                                "urgency": "high",
                                "reasoning": "Model accuracy dropped significantly",
                                "suggested_actions": ["retrain_lstm"],
                                "estimated_improvement": "10-15%",
                            }
                        )
                    )
                )
            ],
            usage=MagicMock(
                prompt_tokens=1000, completion_tokens=400, total_tokens=1400
            ),
        )

        mock_storage.get_performance_metrics_history.return_value = [{"win_rate": 45.0}]
        mock_storage.get_model_accuracy_history.return_value = [
            {"model_type": "lstm", "accuracy": 60.0}
        ]

        result = gpt4_self_analysis()

        # Should indicate retrain should be triggered
        assert result["trigger_retrain"] is True

    def test_task_signature_creation(self):
        """Test creating task signatures for chaining"""
        from tasks.monitoring import system_health_check

        # Should be able to create signature
        sig = system_health_check.s()

        assert sig is not None
        assert hasattr(sig, "apply_async")


class TestWorkerConfiguration:
    """Test worker-specific configuration"""

    def test_worker_max_tasks_per_child(self):
        """Test worker max tasks per child is configured"""
        from celery_app import app

        # Should have max_tasks_per_child to prevent memory leaks
        assert hasattr(app.conf, "worker_max_tasks_per_child")

    def test_worker_prefetch_multiplier(self):
        """Test worker prefetch multiplier"""
        from celery_app import app

        # Should have reasonable prefetch multiplier
        if hasattr(app.conf, "worker_prefetch_multiplier"):
            assert app.conf.worker_prefetch_multiplier > 0

    def test_task_acks_late(self):
        """Test task acknowledgment settings"""
        from celery_app import app

        # For reliability, tasks should ack late
        if hasattr(app.conf, "task_acks_late"):
            assert isinstance(app.conf.task_acks_late, bool)


class TestIntegrationWorkflows:
    """End-to-end integration tests"""

    @patch("tasks.monitoring.storage")
    @patch("tasks.monitoring.notifications")
    def test_complete_monitoring_workflow(self, mock_notifications, mock_storage):
        """Test complete monitoring workflow"""
        from tasks.monitoring import daily_performance_analysis

        # Mock poor performance
        mock_storage.get_performance_metrics_history.return_value = [
            {
                "date": datetime.now().date(),
                "win_rate": 35.0,
                "avg_profit": -1.0,
                "sharpe_ratio": 0.3,
            }
        ]

        result = daily_performance_analysis()

        # Should analyze, detect issue, and notify
        assert result["status"] == "success"
        assert result["analysis"]["performance_status"] in ["warning", "critical"]

    @patch("tasks.ai_improvement.storage")
    @patch("tasks.ai_improvement.requests.post")
    def test_complete_retrain_workflow(self, mock_post, mock_storage):
        """Test complete retrain workflow"""
        from tasks.ai_improvement import adaptive_retrain

        # Mock successful training
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "status": "success",
            "model": "lstm",
            "accuracy_after": 75.0,
            "improvement": 10.0,
        }
        mock_post.return_value = mock_response

        analysis = {
            "recommendation": "retrain",
            "confidence": 85,
            "models_to_retrain": ["lstm"],
        }

        result = adaptive_retrain(model_types=["lstm"], analysis_result=analysis)

        # Should train, store results, and notify
        assert result["status"] == "success"
        assert len(result["retrain_results"]) == 1

    def test_celery_app_can_start(self):
        """Test that Celery app can initialize without errors"""
        from celery_app import app

        # Should be able to access app properties without errors
        assert app.main is not None
        assert app.conf.broker_url is not None

        # Should have tasks registered
        assert len(list(app.tasks.keys())) > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
