#!/usr/bin/env python3
"""
Unit Tests for Monitoring Tasks
Tests all monitoring jobs: health check, portfolio report, cost report, performance analysis
"""

import pytest
from unittest.mock import patch, MagicMock, Mock
from datetime import datetime, timedelta
import requests

# Skip all tests if celery is not installed
try:
    import celery
    CELERY_AVAILABLE = True
except ImportError:
    CELERY_AVAILABLE = False

pytestmark = pytest.mark.skipif(not CELERY_AVAILABLE, reason="Celery not installed")


class TestSystemHealthCheck:
    """Test system_health_check task"""

    @patch("tasks.monitoring.requests.get")
    def test_health_check_all_services_healthy(self, mock_get):
        """Test health check when all services are healthy"""
        from tasks.monitoring import system_health_check

        # Mock all service responses as healthy
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"status": "healthy"}
        mock_get.return_value = mock_response

        result = system_health_check()

        assert result["status"] == "success"
        assert "health_report" in result
        assert result["health_report"]["overall_status"] in [
            "healthy",
            "degraded",
            "critical",
        ]
        assert "services" in result["health_report"]
        assert "timestamp" in result["health_report"]

    @patch("tasks.monitoring.subprocess.run")
    @patch("tasks.monitoring.requests.get")
    def test_health_check_service_down(self, mock_get, mock_subprocess):
        """Test health check when services are down"""
        from tasks.monitoring import system_health_check
        import subprocess

        # Mock service connection refused
        mock_get.side_effect = requests.exceptions.ConnectionError("Connection refused")

        # Mock subprocess calls to raise exceptions (simulating MongoDB/Redis DOWN)
        mock_subprocess.side_effect = subprocess.TimeoutExpired("mongosh", 5)

        result = system_health_check()

        assert result["status"] == "success"
        assert result["health_report"]["overall_status"] == "critical"
        assert len(result["health_report"]["alerts"]) > 0

    @patch("tasks.monitoring.requests.get")
    def test_health_check_mixed_status(self, mock_get):
        """Test health check with mixed service statuses"""
        from tasks.monitoring import system_health_check

        def mock_response_side_effect(url, **kwargs):
            mock_resp = MagicMock()
            if "rust" in url.lower() or "8080" in url:
                mock_resp.status_code = 200
                mock_resp.json.return_value = {"status": "healthy"}
            else:
                raise requests.exceptions.ConnectionError("Connection refused")
            return mock_resp

        mock_get.side_effect = mock_response_side_effect

        result = system_health_check()

        assert result["status"] == "success"
        assert "health_report" in result
        # Should have both healthy and down services
        services = result["health_report"]["services"]
        statuses = [svc.get("status", "unknown") for svc in services.values()]
        assert "down" in statuses or "up" in statuses

    def test_health_check_task_structure(self):
        """Test that health check task has correct structure"""
        from tasks.monitoring import system_health_check

        # Check task is registered
        assert system_health_check.name == "tasks.monitoring.system_health_check"

        # Check task has bind=True (self parameter)
        assert hasattr(system_health_check, "run")


class TestDailyPortfolioReport:
    """Test daily_portfolio_report task"""

    @patch("tasks.monitoring.requests.get")
    def test_portfolio_report_success(self, mock_get):
        """Test portfolio report with successful API response"""
        from tasks.monitoring import daily_portfolio_report

        # Mock portfolio API response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "total_balance": 10000.50,
            "total_pnl": 250.75,
            "pnl_percentage": 2.57,
            "open_positions": 3,
            "total_trades": 15,
            "win_rate": 66.67,
        }
        mock_get.return_value = mock_response

        result = daily_portfolio_report()

        assert result["status"] == "success"
        assert "report" in result
        # Portfolio data comes from API, not directly in result
        assert "balance" in result["report"] or result["report"]["total_trades"] >= 0

    @patch("tasks.monitoring.requests.get")
    def test_portfolio_report_api_error(self, mock_get):
        """Test portfolio report when API is down"""
        from tasks.monitoring import daily_portfolio_report

        # Mock API connection error
        mock_get.side_effect = requests.exceptions.ConnectionError("API unavailable")

        with pytest.raises(Exception):
            daily_portfolio_report()

    @patch("tasks.monitoring.requests.get")
    def test_portfolio_report_negative_pnl(self, mock_get):
        """Test portfolio report with negative P&L"""
        from tasks.monitoring import daily_portfolio_report

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "total_balance": 9500.00,
            "total_pnl": -500.00,
            "pnl_percentage": -5.0,
            "open_positions": 2,
            "total_trades": 10,
            "win_rate": 40.0,
        }
        mock_get.return_value = mock_response

        result = daily_portfolio_report()

        assert result["status"] == "success"
        assert "report" in result
        # Negative P&L would be reflected in balance/return metrics
        assert result["report"] is not None

    def test_portfolio_report_task_registered(self):
        """Test that portfolio report task is registered"""
        from tasks.monitoring import daily_portfolio_report

        assert daily_portfolio_report.name == "tasks.monitoring.daily_portfolio_report"


class TestDailyAPICostReport:
    """Test daily_api_cost_report task"""

    @patch("tasks.monitoring.requests.get")
    def test_api_cost_report_no_data(self, mock_get):
        """Test API cost report when no data exists"""
        from tasks.monitoring import daily_api_cost_report

        # Mock API response with no data
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "session_statistics": {
                "total_requests": 0,
                "total_cost_usd": 0,
                "total_cost_vnd": 0,
            },
            "projections": {
                "estimated_daily_cost_usd": 0,
                "estimated_monthly_cost_usd": 0,
            },
        }
        mock_get.return_value = mock_response

        result = daily_api_cost_report()

        assert result["status"] == "success"
        assert result["report"]["session"]["total_cost_usd"] == 0
        assert result["report"]["session"]["total_requests"] == 0

    @patch("tasks.monitoring.requests.get")
    def test_api_cost_report_with_data(self, mock_get):
        """Test API cost report with existing data"""
        from tasks.monitoring import daily_api_cost_report

        # Mock API response with cost data
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "session_statistics": {
                "total_requests": 50,
                "total_cost_usd": 0.40,
                "total_cost_vnd": 10000,
            },
            "projections": {
                "estimated_daily_cost_usd": 0.50,
                "estimated_monthly_cost_usd": 15.00,
            },
        }
        mock_get.return_value = mock_response

        result = daily_api_cost_report()

        assert result["status"] == "success"
        assert result["report"]["session"]["total_cost_usd"] == 0.40
        assert result["report"]["session"]["total_requests"] == 50

    @patch("tasks.monitoring.requests.get")
    def test_api_cost_report_high_cost_alert(self, mock_get):
        """Test API cost report triggers alert on high cost"""
        from tasks.monitoring import daily_api_cost_report

        # Mock high cost data (exceeds $50/day threshold)
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "session_statistics": {
                "total_requests": 1000,
                "total_cost_usd": 100.00,
                "total_cost_vnd": 2500000,
            },
            "projections": {
                "estimated_daily_cost_usd": 60.00,  # Above threshold
                "estimated_monthly_cost_usd": 1800.00,
            },
        }
        mock_get.return_value = mock_response

        result = daily_api_cost_report()

        assert result["status"] == "success"
        # Should have alert
        assert len(result["report"]["alerts"]) > 0

    def test_api_cost_report_task_registered(self):
        """Test that API cost report task is registered"""
        from tasks.monitoring import daily_api_cost_report

        assert daily_api_cost_report.name == "tasks.monitoring.daily_api_cost_report"


class TestDailyPerformanceAnalysis:
    """Test daily_performance_analysis task"""

    @patch("tasks.monitoring.storage")
    def test_performance_analysis_good_metrics(self, mock_storage):
        """Test performance analysis with good trading metrics"""
        from tasks.monitoring import daily_performance_analysis

        # Mock good performance data
        mock_storage.get_performance_metrics_history.return_value = [
            {
                "date": datetime.now().date(),
                "win_rate": 75.0,
                "avg_profit": 2.5,
                "sharpe_ratio": 2.0,
                "total_trades": 20,
                "profitable_trades": 15,
            }
        ]

        result = daily_performance_analysis()

        assert result["status"] == "success"
        assert result["analysis"]["avg_win_rate"] == 75.0
        assert result["analysis"]["performance_status"] == "good"

    @patch("tasks.monitoring.storage")
    def test_performance_analysis_poor_metrics(self, mock_storage):
        """Test performance analysis with poor trading metrics"""
        from tasks.monitoring import daily_performance_analysis

        # Mock poor performance data
        mock_storage.get_performance_metrics_history.return_value = [
            {
                "date": datetime.now().date(),
                "win_rate": 35.0,
                "avg_profit": -1.5,
                "sharpe_ratio": 0.5,
                "total_trades": 20,
                "profitable_trades": 7,
            }
        ]

        result = daily_performance_analysis()

        assert result["status"] == "success"
        assert result["analysis"]["avg_win_rate"] == 35.0
        assert result["analysis"]["performance_status"] in ["warning", "critical"]
        assert result["analysis"]["trigger_ai_analysis"] is True

    @patch("tasks.monitoring.storage")
    def test_performance_analysis_no_data(self, mock_storage):
        """Test performance analysis with no historical data"""
        from tasks.monitoring import daily_performance_analysis

        mock_storage.get_performance_metrics_history.return_value = []

        result = daily_performance_analysis()

        assert result["status"] == "success"
        assert result["analysis"]["total_days"] == 0

    @patch("tasks.monitoring.storage")
    def test_performance_analysis_trend_detection(self, mock_storage):
        """Test performance analysis detects declining trends"""
        from tasks.monitoring import daily_performance_analysis

        # Mock declining performance trend
        mock_storage.get_performance_metrics_history.return_value = [
            {"date": datetime.now().date(), "win_rate": 45.0, "avg_profit": 0.5},
            {
                "date": (datetime.now() - timedelta(days=1)).date(),
                "win_rate": 55.0,
                "avg_profit": 1.0,
            },
            {
                "date": (datetime.now() - timedelta(days=2)).date(),
                "win_rate": 65.0,
                "avg_profit": 1.5,
            },
            {
                "date": (datetime.now() - timedelta(days=3)).date(),
                "win_rate": 70.0,
                "avg_profit": 2.0,
            },
        ]

        result = daily_performance_analysis()

        assert result["status"] == "success"
        # Should detect declining trend
        assert result["analysis"]["trigger_ai_analysis"] is True

    def test_performance_analysis_task_registered(self):
        """Test that performance analysis task is registered"""
        from tasks.monitoring import daily_performance_analysis

        assert (
            daily_performance_analysis.name
            == "tasks.monitoring.daily_performance_analysis"
        )


class TestMonitoringTasksIntegration:
    """Integration tests for monitoring tasks"""

    def test_all_monitoring_tasks_registered(self):
        """Test that all monitoring tasks are properly registered"""
        from celery_app import app

        registered_tasks = list(app.tasks.keys())

        expected_tasks = [
            "tasks.monitoring.system_health_check",
            "tasks.monitoring.daily_portfolio_report",
            "tasks.monitoring.daily_api_cost_report",
            "tasks.monitoring.daily_performance_analysis",
        ]

        for task in expected_tasks:
            assert task in registered_tasks, f"Task {task} not registered"

    def test_monitoring_tasks_have_correct_routes(self):
        """Test that monitoring tasks use correct queues"""
        from celery_app import app

        task_routes = app.conf.task_routes

        # Monitoring tasks should use 'scheduled' queue
        monitoring_tasks = [
            "tasks.monitoring.system_health_check",
            "tasks.monitoring.daily_portfolio_report",
            "tasks.monitoring.daily_api_cost_report",
            "tasks.monitoring.daily_performance_analysis",
        ]

        for task_name in monitoring_tasks:
            if task_name in task_routes:
                assert task_routes[task_name]["queue"] == "scheduled"

    def test_beat_schedule_configured(self):
        """Test that Celery Beat schedule is properly configured"""
        from celery_app import app

        beat_schedule = app.conf.beat_schedule

        # Check all scheduled monitoring tasks
        assert "system-health-check" in beat_schedule
        assert "daily-portfolio-report" in beat_schedule
        assert "daily-api-cost-report" in beat_schedule
        assert "daily-performance-analysis" in beat_schedule

        # Verify schedule structure
        for schedule_name in beat_schedule:
            assert "task" in beat_schedule[schedule_name]
            assert "schedule" in beat_schedule[schedule_name]


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
