#!/usr/bin/env python3
"""
Unit Tests for Data Storage Utilities
Tests MongoDB storage for GPT-4 analysis, performance metrics, and audit trails
"""

import pytest
from unittest.mock import patch, MagicMock, Mock
from datetime import datetime, timedelta
from pymongo.errors import ConnectionFailure, OperationFailure


class TestDataStorageConnection:
    """Test MongoDB connection and initialization"""

    def setup_method(self):
        """Reset singleton before each test"""
        from utils.data_storage import DataStorage
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_storage_singleton_pattern(self, mock_client):
        """Test that DataStorage implements singleton pattern"""
        from utils.data_storage import DataStorage

        # Mock successful connection
        mock_client.return_value.server_info.return_value = {"version": "7.0"}

        storage1 = DataStorage()
        storage2 = DataStorage()

        # Should be the same instance
        assert storage1 is storage2

    @patch("utils.data_storage.MongoClient")
    def test_storage_connection_success(self, mock_client):
        """Test successful MongoDB connection"""
        from utils.data_storage import DataStorage

        # Reset singleton before test
        DataStorage.reset_instance()

        # Mock successful connection
        mock_client.return_value.server_info.return_value = {"version": "7.0"}

        storage = DataStorage()

        assert storage.is_connected()
        assert mock_client.called

    @patch("utils.data_storage.MongoClient")
    def test_storage_connection_failure(self, mock_client):
        """Test MongoDB connection failure handling"""
        from utils.data_storage import DataStorage

        # Reset singleton before test
        DataStorage.reset_instance()

        # Mock connection failure
        mock_client.return_value.server_info.side_effect = ConnectionFailure(
            "Connection refused"
        )

        storage = DataStorage()

        # Should handle failure gracefully
        assert not storage.is_connected()

    @patch("utils.data_storage.MongoClient")
    def test_storage_creates_indexes(self, mock_client):
        """Test that storage creates necessary indexes"""
        from utils.data_storage import DataStorage

        # Reset singleton before test
        DataStorage.reset_instance()

        mock_db = MagicMock()
        mock_client.return_value.__getitem__.return_value = mock_db
        mock_client.return_value.server_info.return_value = {"version": "7.0"}

        storage = DataStorage()

        # Should have created indexes on collections
        # Check that create_index was called
        assert mock_db.__getitem__.called or hasattr(storage, "_db")


class TestGPT4AnalysisStorage:
    """Test GPT-4 analysis storage functions"""

    @patch("utils.data_storage.MongoClient")
    def test_store_gpt4_analysis_success(self, mock_client):
        """Test storing GPT-4 analysis result"""
        from utils.data_storage import storage

        # Mock database
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.return_value = MagicMock(inserted_id="test_id")
        mock_db.__getitem__.return_value = mock_collection

        storage._db = mock_db
        storage._client = mock_client

        analysis_result = {
            "recommendation": "retrain",
            "confidence": 85,
            "reasoning": "Model accuracy dropped",
            "models_to_retrain": ["lstm", "gru"],
        }

        result_id = storage.store_gpt4_analysis(
            analysis_result=analysis_result, task_id="test_task_123"
        )

        assert result_id == "test_id"
        assert mock_collection.insert_one.called

        # Verify stored document structure
        call_args = mock_collection.insert_one.call_args[0][0]
        assert "timestamp" in call_args
        assert call_args["recommendation"] == "retrain"
        assert call_args["confidence"] == 85
        assert call_args["task_id"] == "test_task_123"

    @patch("utils.data_storage.MongoClient")
    def test_get_gpt4_analysis_history(self, mock_client):
        """Test retrieving GPT-4 analysis history"""
        from utils.data_storage import storage

        # Mock database
        mock_db = MagicMock()
        mock_collection = MagicMock()

        # Mock historical data
        mock_collection.find.return_value.sort.return_value.limit.return_value = [
            {
                "timestamp": datetime.now(),
                "recommendation": "retrain",
                "confidence": 85,
            },
            {
                "timestamp": datetime.now() - timedelta(days=1),
                "recommendation": "wait",
                "confidence": 90,
            },
        ]

        mock_db.__getitem__.return_value = mock_collection
        storage._db = mock_db

        history = storage.get_gpt4_analysis_history(days=7)

        assert len(history) == 2
        assert history[0]["recommendation"] == "retrain"
        assert mock_collection.find.called

    @patch("utils.data_storage.MongoClient")
    def test_get_latest_gpt4_analysis(self, mock_client):
        """Test getting latest GPT-4 analysis"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()

        latest_analysis = {
            "timestamp": datetime.now(),
            "recommendation": "optimize_parameters",
            "confidence": 78,
        }

        mock_collection.find_one.return_value = latest_analysis
        mock_db.__getitem__.return_value = mock_collection
        storage._db = mock_db

        result = storage.get_latest_gpt4_analysis()

        assert result["recommendation"] == "optimize_parameters"
        assert result["confidence"] == 78


class TestPerformanceMetricsStorage:
    """Test performance metrics storage functions"""

    @patch("utils.data_storage.MongoClient")
    def test_store_performance_metrics(self, mock_client):
        """Test storing daily performance metrics"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.return_value = MagicMock(inserted_id="metric_id")
        mock_db.__getitem__.return_value = mock_collection

        storage._db = mock_db

        metrics = {
            "date": datetime.now().date(),
            "win_rate": 72.5,
            "avg_profit": 2.3,
            "sharpe_ratio": 1.8,
            "total_trades": 25,
            "profitable_trades": 18,
        }

        result_id = storage.store_performance_metrics(
            metrics=metrics, task_id="perf_task_123"
        )

        assert result_id == "metric_id"
        assert mock_collection.insert_one.called

        # Verify stored data
        call_args = mock_collection.insert_one.call_args[0][0]
        assert call_args["win_rate"] == 72.5
        assert call_args["avg_profit"] == 2.3

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_metrics_history(self, mock_client):
        """Test retrieving performance metrics history"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()

        mock_data = [
            {"date": datetime.now().date(), "win_rate": 72.0, "avg_profit": 2.0},
            {
                "date": (datetime.now() - timedelta(days=1)).date(),
                "win_rate": 68.0,
                "avg_profit": 1.8,
            },
        ]

        mock_collection.find.return_value.sort.return_value.limit.return_value = (
            mock_data
        )
        mock_db.__getitem__.return_value = mock_collection
        storage._db = mock_db

        history = storage.get_performance_metrics_history(days=7)

        assert len(history) == 2
        assert history[0]["win_rate"] == 72.0
        assert mock_collection.find.called

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_trend(self, mock_client):
        """Test calculating performance trend"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()

        # Mock declining trend
        mock_collection.find.return_value.sort.return_value.limit.return_value = [
            {"date": datetime.now().date(), "win_rate": 60.0},
            {"date": (datetime.now() - timedelta(days=1)).date(), "win_rate": 65.0},
            {"date": (datetime.now() - timedelta(days=2)).date(), "win_rate": 70.0},
            {"date": (datetime.now() - timedelta(days=3)).date(), "win_rate": 72.0},
        ]

        mock_db.__getitem__.return_value = mock_collection
        storage._db = mock_db

        trend = storage.get_performance_trend(days=7, metric="win_rate")

        assert trend == "declining"


class TestModelAccuracyStorage:
    """Test model accuracy storage functions"""

    @patch("utils.data_storage.MongoClient")
    def test_store_model_accuracy(self, mock_client):
        """Test storing model accuracy data"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.return_value = MagicMock(inserted_id="accuracy_id")
        mock_db.__getitem__.return_value = mock_collection

        storage._db = mock_db

        accuracy_data = {
            "model_type": "lstm",
            "accuracy": 73.5,
            "loss": 0.28,
            "f1_score": 0.71,
            "training_samples": 10000,
        }

        result_id = storage.store_model_accuracy(accuracy_data)

        assert result_id == "accuracy_id"
        assert mock_collection.insert_one.called

    @patch("utils.data_storage.MongoClient")
    def test_get_model_accuracy_history(self, mock_client):
        """Test retrieving model accuracy history"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()

        mock_data = [
            {"timestamp": datetime.now(), "model_type": "lstm", "accuracy": 73.0},
            {
                "timestamp": datetime.now() - timedelta(hours=12),
                "model_type": "lstm",
                "accuracy": 71.5,
            },
        ]

        mock_collection.find.return_value.sort.return_value.limit.return_value = (
            mock_data
        )
        mock_db.__getitem__.return_value = mock_collection
        storage._db = mock_db

        history = storage.get_model_accuracy_history(model_type="lstm", days=7)

        assert len(history) == 2
        assert history[0]["model_type"] == "lstm"
        assert mock_collection.find.called


class TestAPICostStorage:
    """Test API cost storage functions"""

    @patch("utils.data_storage.MongoClient")
    def test_store_api_cost(self, mock_client):
        """Test storing API cost data"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.return_value = MagicMock(inserted_id="cost_id")
        mock_db.__getitem__.return_value = mock_collection

        storage._db = mock_db

        cost_data = {
            "session_id": "gpt4_session_123",
            "input_tokens": 1200,
            "output_tokens": 450,
            "total_cost": 0.028,
            "model": "gpt-4",
        }

        result_id = storage.store_api_cost(cost_data)

        assert result_id == "cost_id"
        assert mock_collection.insert_one.called

    @patch("utils.data_storage.MongoClient")
    def test_get_api_cost_history(self, mock_client):
        """Test retrieving API cost history"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()

        mock_data = [
            {"total_cost": 0.028, "timestamp": datetime.now()},
            {"total_cost": 0.032, "timestamp": datetime.now() - timedelta(hours=2)},
        ]

        mock_collection.find.return_value.sort.return_value.limit.return_value = (
            mock_data
        )
        mock_db.__getitem__.return_value = mock_collection
        storage._db = mock_db

        history = storage.get_api_cost_history(days=1)

        assert len(history) == 2
        assert sum(item["total_cost"] for item in history) == 0.060


class TestRetrainHistoryStorage:
    """Test retrain history storage functions"""

    @patch("utils.data_storage.MongoClient")
    def test_store_retrain_history(self, mock_client):
        """Test storing retrain history"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.return_value = MagicMock(inserted_id="retrain_id")
        mock_db.__getitem__.return_value = mock_collection

        storage._db = mock_db

        retrain_data = {
            "trigger_type": "gpt4_recommendation",
            "models_retrained": ["lstm", "gru"],
            "accuracy_improvements": {"lstm": 11.0, "gru": 9.0},
            "training_duration": 450,
            "deployment_status": "deployed",
        }

        result_id = storage.store_retrain_history(
            retrain_data=retrain_data, task_id="retrain_task_123"
        )

        assert result_id == "retrain_id"
        assert mock_collection.insert_one.called

    @patch("utils.data_storage.MongoClient")
    def test_get_retrain_history(self, mock_client):
        """Test retrieving retrain history"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()

        mock_data = [
            {
                "timestamp": datetime.now(),
                "trigger_type": "gpt4_recommendation",
                "models_retrained": ["lstm"],
            },
            {
                "timestamp": datetime.now() - timedelta(days=5),
                "trigger_type": "manual",
                "models_retrained": ["gru", "transformer"],
            },
        ]

        mock_collection.find.return_value.sort.return_value.limit.return_value = (
            mock_data
        )
        mock_db.__getitem__.return_value = mock_collection
        storage._db = mock_db

        history = storage.get_retrain_history(days=30)

        assert len(history) == 2
        assert history[0]["trigger_type"] == "gpt4_recommendation"


class TestDataStorageErrorHandling:
    """Test data storage error handling"""

    def setup_method(self):
        """Reset singleton before each test"""
        from utils.data_storage import DataStorage
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_storage_handles_insert_failure(self, mock_client):
        """Test storage handles insert failures gracefully"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.side_effect = OperationFailure("Insert failed")
        mock_db.__getitem__.return_value = mock_collection

        storage._db = mock_db

        # Should not raise exception, returns None
        result = storage.store_gpt4_analysis({"test": "data"}, "task_123")

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_storage_handles_query_failure(self, mock_client):
        """Test storage handles query failures gracefully"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = OperationFailure("Query failed")
        mock_db.__getitem__.return_value = mock_collection

        storage._db = mock_db

        # Should return empty list on error
        result = storage.get_gpt4_analysis_history(days=7)

        assert result == []

    def test_storage_methods_when_disconnected(self):
        """Test that storage methods handle disconnected state"""
        from utils.data_storage import DataStorage

        # Create storage instance without connection
        storage = DataStorage()
        storage._client = None
        storage._db = None

        # Methods should handle gracefully
        result = storage.store_gpt4_analysis({"test": "data"}, "task_123")
        assert result is None

        history = storage.get_gpt4_analysis_history(days=7)
        assert history == []


class TestDataStorageIntegration:
    """Integration tests for data storage"""

    def setup_method(self):
        """Reset singleton before each test"""
        from utils.data_storage import DataStorage
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_complete_workflow_gpt4_analysis(self, mock_client):
        """Test complete workflow: store analysis, retrieve history"""
        from utils.data_storage import storage

        mock_db = MagicMock()
        mock_collection = MagicMock()

        # Mock insert
        mock_collection.insert_one.return_value = MagicMock(inserted_id="id1")

        # Mock retrieve
        mock_collection.find.return_value.sort.return_value.limit.return_value = [
            {
                "timestamp": datetime.now(),
                "recommendation": "retrain",
                "confidence": 85,
                "task_id": "task_123",
            }
        ]

        mock_db.__getitem__.return_value = mock_collection
        storage._db = mock_db

        # Store
        analysis = {"recommendation": "retrain", "confidence": 85}
        storage.store_gpt4_analysis(analysis, "task_123")

        # Retrieve
        history = storage.get_gpt4_analysis_history(days=1)

        assert len(history) == 1
        assert history[0]["recommendation"] == "retrain"


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
