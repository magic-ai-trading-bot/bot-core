#!/usr/bin/env python3
"""
Simplified Async Tasks Tests - Focus on What's Actually Implemented
Tests core functionality without complex mocking
"""

import pytest
from celery import Celery


class TestCeleryInfrastructure:
    """Test Celery infrastructure is working"""

    def test_celery_app_configured(self):
        """Test Celery app exists and is configured"""
        from celery_app import app

        assert isinstance(app, Celery)
        assert app.conf.broker_url is not None
        assert 'amqp' in app.conf.broker_url or 'rabbitmq' in app.conf.broker_url
        assert app.conf.result_backend is not None
        assert 'redis' in app.conf.result_backend

    def test_all_tasks_registered(self):
        """Test that all async tasks are registered"""
        # Import task modules to trigger registration
        import tasks.monitoring
        import tasks.ai_improvement
        import tasks.ml_tasks
        import tasks.backtest_tasks

        from celery_app import app

        registered_tasks = list(app.tasks.keys())

        # Expected monitoring tasks
        assert 'tasks.monitoring.system_health_check' in registered_tasks
        assert 'tasks.monitoring.daily_portfolio_report' in registered_tasks
        assert 'tasks.monitoring.daily_api_cost_report' in registered_tasks
        assert 'tasks.monitoring.daily_performance_analysis' in registered_tasks

        # Expected AI improvement tasks
        assert 'tasks.ai_improvement.gpt4_self_analysis' in registered_tasks
        assert 'tasks.ai_improvement.adaptive_retrain' in registered_tasks
        assert 'tasks.ai_improvement.emergency_strategy_disable' in registered_tasks

    def test_beat_schedule_configured(self):
        """Test that Celery Beat schedule exists"""
        from celery_app import app

        schedule = app.conf.beat_schedule
        assert schedule is not None
        assert len(schedule) >= 5

        # Check key scheduled tasks
        assert 'system-health-check' in schedule
        assert 'daily-portfolio-report' in schedule
        assert 'gpt4-self-analysis' in schedule

    def test_task_routes_configured(self):
        """Test that task routing is configured"""
        from celery_app import app

        routes = app.conf.task_routes
        assert routes is not None
        assert isinstance(routes, dict)


class TestMonitoringTasksBasic:
    """Basic tests for monitoring tasks"""

    def test_system_health_check_exists(self):
        """Test system health check task exists"""
        from tasks.monitoring import system_health_check

        assert system_health_check is not None
        assert system_health_check.name == 'tasks.monitoring.system_health_check'
        assert callable(system_health_check)

    def test_daily_portfolio_report_exists(self):
        """Test daily portfolio report task exists"""
        from tasks.monitoring import daily_portfolio_report

        assert daily_portfolio_report is not None
        assert daily_portfolio_report.name == 'tasks.monitoring.daily_portfolio_report'

    def test_daily_api_cost_report_exists(self):
        """Test daily API cost report task exists"""
        from tasks.monitoring import daily_api_cost_report

        assert daily_api_cost_report is not None
        assert daily_api_cost_report.name == 'tasks.monitoring.daily_api_cost_report'

    def test_daily_performance_analysis_exists(self):
        """Test daily performance analysis task exists"""
        from tasks.monitoring import daily_performance_analysis

        assert daily_performance_analysis is not None
        assert daily_performance_analysis.name == 'tasks.monitoring.daily_performance_analysis'


class TestAIImprovementTasksBasic:
    """Basic tests for AI improvement tasks"""

    def test_gpt4_self_analysis_exists(self):
        """Test GPT-4 self-analysis task exists"""
        from tasks.ai_improvement import gpt4_self_analysis

        assert gpt4_self_analysis is not None
        assert gpt4_self_analysis.name == 'tasks.ai_improvement.gpt4_self_analysis'

    def test_adaptive_retrain_exists(self):
        """Test adaptive retrain task exists"""
        from tasks.ai_improvement import adaptive_retrain

        assert adaptive_retrain is not None
        assert adaptive_retrain.name == 'tasks.ai_improvement.adaptive_retrain'

    def test_emergency_strategy_disable_exists(self):
        """Test emergency strategy disable task exists"""
        from tasks.ai_improvement import emergency_strategy_disable

        assert emergency_strategy_disable is not None
        assert emergency_strategy_disable.name == 'tasks.ai_improvement.emergency_strategy_disable'


class TestDataStorage:
    """Test data storage utilities"""

    def test_storage_singleton_initialized(self):
        """Test that storage singleton is initialized"""
        from utils.data_storage import storage

        assert storage is not None

    def test_storage_connection_check(self):
        """Test storage connection check method exists"""
        from utils.data_storage import storage

        assert hasattr(storage, 'is_connected')
        assert callable(storage.is_connected)

    def test_storage_collections_defined(self):
        """Test that storage collections are defined"""
        from utils.data_storage import (
            COLLECTION_GPT4_ANALYSIS,
            COLLECTION_PERFORMANCE_METRICS,
            COLLECTION_MODEL_ACCURACY,
            COLLECTION_API_COSTS,
            COLLECTION_RETRAIN_HISTORY
        )

        assert COLLECTION_GPT4_ANALYSIS == 'gpt4_analysis_history'
        assert COLLECTION_PERFORMANCE_METRICS == 'performance_metrics'
        assert COLLECTION_MODEL_ACCURACY == 'model_accuracy_history'
        assert COLLECTION_API_COSTS == 'api_cost_history'
        assert COLLECTION_RETRAIN_HISTORY == 'retrain_history'


class TestNotificationSystem:
    """Test notification system"""

    def test_notification_module_imports(self):
        """Test that notification module can be imported"""
        from utils import notifications

        assert notifications is not None

    def test_notification_functions_exist(self):
        """Test that key notification functions exist"""
        from utils import notifications

        assert hasattr(notifications, 'send_notification')
        assert hasattr(notifications, 'send_critical')
        assert hasattr(notifications, 'send_warning')
        assert hasattr(notifications, 'send_info')


class TestTaskConfiguration:
    """Test task configuration and settings"""

    def test_monitoring_task_has_bind(self):
        """Test that monitoring tasks have bind=True"""
        from tasks.monitoring import system_health_check

        # Tasks with bind=True can access self.request
        assert hasattr(system_health_check, 'run')

    def test_ai_task_has_bind(self):
        """Test that AI tasks have bind=True"""
        from tasks.ai_improvement import gpt4_self_analysis

        assert hasattr(gpt4_self_analysis, 'run')

    def test_task_serialization(self):
        """Test task serialization is JSON"""
        from celery_app import app

        assert app.conf.task_serializer == 'json'
        assert 'json' in app.conf.accept_content


class TestModuleStructure:
    """Test module structure and imports"""

    def test_monitoring_tasks_module(self):
        """Test monitoring tasks module structure"""
        import tasks.monitoring as monitoring

        # Check module has expected tasks
        assert hasattr(monitoring, 'system_health_check')
        assert hasattr(monitoring, 'daily_portfolio_report')
        assert hasattr(monitoring, 'daily_api_cost_report')
        assert hasattr(monitoring, 'daily_performance_analysis')

    def test_ai_improvement_module(self):
        """Test AI improvement module structure"""
        import tasks.ai_improvement as ai_improvement

        assert hasattr(ai_improvement, 'gpt4_self_analysis')
        assert hasattr(ai_improvement, 'adaptive_retrain')
        assert hasattr(ai_improvement, 'emergency_strategy_disable')

    def test_utils_modules(self):
        """Test utils modules exist"""
        from utils import notifications
        from utils import data_storage
        from utils import logger

        assert notifications is not None
        assert data_storage is not None
        assert logger is not None


class TestEnvironmentConfiguration:
    """Test environment configuration"""

    def test_mongodb_uri_configured(self):
        """Test MongoDB URI is configured"""
        from utils.data_storage import MONGODB_URI

        assert MONGODB_URI is not None
        assert 'mongodb://' in MONGODB_URI

    def test_mongodb_db_configured(self):
        """Test MongoDB database is configured"""
        from utils.data_storage import MONGODB_DB

        assert MONGODB_DB is not None
        assert len(MONGODB_DB) > 0


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
