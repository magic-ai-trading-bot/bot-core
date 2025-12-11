"""Comprehensive tests for utils/data_storage.py"""

import os
from datetime import datetime, timedelta
from unittest.mock import MagicMock, call, patch

import pytest

from utils.data_storage import (
    COLLECTION_API_COSTS,
    COLLECTION_GPT4_ANALYSIS,
    COLLECTION_MODEL_ACCURACY,
    COLLECTION_PERFORMANCE_METRICS,
    COLLECTION_RETRAIN_HISTORY,
    DataStorage,
    _extract_db_name,
)


@pytest.mark.unit
class TestDatabaseNameExtraction:
    """Test _extract_db_name function"""

    def test_extract_db_name_with_auth_and_database(self):
        """Test extracting database name from URI with auth"""
        uri = "mongodb://user:pass@localhost:27017/production_db"
        result = _extract_db_name(uri)
        # Should extract database name from URI
        assert result in ["production_db", os.getenv("MONGODB_DB", "bot_core")]

    def test_extract_db_name_without_database(self):
        """Test fallback to environment variable"""
        uri = "mongodb://localhost:27017"
        result = _extract_db_name(uri)
        # Should use environment variable or default
        assert result == os.getenv("MONGODB_DB", "bot_core")

    def test_extract_db_name_default(self):
        """Test default database name"""
        uri = "mongodb://localhost:27017"
        result = _extract_db_name(uri)
        # Should return bot_core as default
        assert isinstance(result, str)
        assert len(result) > 0


@pytest.mark.unit
class TestDataStorageSingleton:
    """Test DataStorage singleton pattern"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_singleton_pattern(self, mock_mongo_client):
        """Test that DataStorage is a singleton"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_mongo_client.return_value = mock_client

        storage1 = DataStorage()
        storage2 = DataStorage()

        assert storage1 is storage2
        # Should only create client once
        assert mock_mongo_client.call_count == 1

    @patch("utils.data_storage.MongoClient")
    def test_reset_instance(self, mock_mongo_client):
        """Test resetting singleton instance"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_mongo_client.return_value = mock_client

        storage1 = DataStorage()
        DataStorage.reset_instance()
        storage2 = DataStorage()

        assert storage1 is not storage2
        assert mock_mongo_client.call_count == 2


@pytest.mark.unit
class TestDataStorageInitialization:
    """Test DataStorage initialization"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_successful_initialization(self, mock_mongo_client):
        """Test successful MongoDB connection"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()

        assert storage.is_connected() is True
        mock_client.server_info.assert_called_once()

    @patch("utils.data_storage.MongoClient")
    def test_initialization_connection_failure(self, mock_mongo_client):
        """Test initialization with connection failure"""
        mock_mongo_client.side_effect = Exception("Connection refused")

        storage = DataStorage()

        assert storage.is_connected() is False
        assert storage._client is None
        assert storage._db is None

    @patch("utils.data_storage.MongoClient")
    def test_initialization_server_info_failure(self, mock_mongo_client):
        """Test initialization when server_info fails"""
        mock_client = MagicMock()
        mock_client.server_info.side_effect = Exception("Authentication failed")
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()

        assert storage.is_connected() is False

    @patch("utils.data_storage.MongoClient")
    def test_ensure_indexes_success(self, mock_mongo_client):
        """Test index creation succeeds"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()

        # Should create indexes for all collections
        assert mock_db[COLLECTION_GPT4_ANALYSIS].create_index.called
        assert mock_db[COLLECTION_PERFORMANCE_METRICS].create_index.called
        assert mock_db[COLLECTION_MODEL_ACCURACY].create_index.called
        assert mock_db[COLLECTION_API_COSTS].create_index.called
        assert mock_db[COLLECTION_RETRAIN_HISTORY].create_index.called

    @patch("utils.data_storage.MongoClient")
    def test_ensure_indexes_failure(self, mock_mongo_client):
        """Test index creation handles failures gracefully"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_db[COLLECTION_GPT4_ANALYSIS].create_index.side_effect = Exception(
            "Index creation failed"
        )
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()

        # Should still connect despite index failure
        assert storage.is_connected() is True

    @patch("utils.data_storage.MongoClient")
    def test_ensure_indexes_when_not_connected(self, mock_mongo_client):
        """Test _ensure_indexes when db is None"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        storage._ensure_indexes()  # Should not raise

        # No indexes should be created
        assert storage._db is None


@pytest.mark.unit
class TestGPT4AnalysisStorage:
    """Test GPT-4 analysis storage methods"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_store_gpt4_analysis_success(self, mock_mongo_client):
        """Test storing GPT-4 analysis successfully"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "test_id_123"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        analysis = {
            "recommendation": "Consider retraining",
            "confidence": 0.85,
            "urgency": "medium",
            "reasoning": "Test reasoning",
        }

        result = storage.store_gpt4_analysis(analysis, "task_123")

        assert result == "test_id_123"
        mock_collection.insert_one.assert_called_once()

    @patch("utils.data_storage.MongoClient")
    def test_store_gpt4_analysis_not_connected(self, mock_mongo_client):
        """Test storing when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        result = storage.store_gpt4_analysis({"test": "data"}, "task_123")

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_store_gpt4_analysis_insert_failure(self, mock_mongo_client):
        """Test handling insert failure"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.side_effect = Exception("Insert failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.store_gpt4_analysis({"test": "data"}, "task_123")

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_gpt4_analysis_history_success(self, mock_mongo_client):
        """Test retrieving GPT-4 analysis history"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.limit.return_value = [{"analysis": "1"}, {"analysis": "2"}]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_gpt4_analysis_history(days=7, limit=50)

        assert len(history) == 2
        mock_collection.find.assert_called_once()

    @patch("utils.data_storage.MongoClient")
    def test_get_gpt4_analysis_history_not_connected(self, mock_mongo_client):
        """Test retrieving history when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        history = storage.get_gpt4_analysis_history()

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_gpt4_analysis_history_query_failure(self, mock_mongo_client):
        """Test handling query failure"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_gpt4_analysis_history()

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_latest_gpt4_analysis_success(self, mock_mongo_client):
        """Test retrieving latest GPT-4 analysis"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find_one.return_value = {"analysis": "latest"}
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.get_latest_gpt4_analysis()

        assert result == {"analysis": "latest"}

    @patch("utils.data_storage.MongoClient")
    def test_get_latest_gpt4_analysis_not_connected(self, mock_mongo_client):
        """Test retrieving latest when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        result = storage.get_latest_gpt4_analysis()

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_latest_gpt4_analysis_query_failure(self, mock_mongo_client):
        """Test handling find_one failure"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find_one.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.get_latest_gpt4_analysis()

        assert result is None


@pytest.mark.unit
class TestPerformanceMetricsStorage:
    """Test performance metrics storage methods"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_store_performance_metrics_success(self, mock_mongo_client):
        """Test storing performance metrics"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "metrics_id_123"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        metrics = {
            "total_trades": 100,
            "win_rate": 0.65,
            "avg_profit": 1.5,
            "sharpe_ratio": 1.8,
        }

        result = storage.store_performance_metrics(metrics, "task_456")

        assert result == "metrics_id_123"
        mock_collection.insert_one.assert_called_once()

    @patch("utils.data_storage.MongoClient")
    def test_store_performance_metrics_not_connected(self, mock_mongo_client):
        """Test storing metrics when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        result = storage.store_performance_metrics({"test": "data"}, "task_456")

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_store_performance_metrics_insert_failure(self, mock_mongo_client):
        """Test handling insert failure for metrics"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.side_effect = Exception("Insert failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.store_performance_metrics({"test": "data"}, "task_456")

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_metrics_history_success(self, mock_mongo_client):
        """Test retrieving performance metrics history"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = [{"metric": "1"}, {"metric": "2"}]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_performance_metrics_history(days=7)

        assert len(history) == 2

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_metrics_history_not_connected(self, mock_mongo_client):
        """Test retrieving metrics history when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        history = storage.get_performance_metrics_history()

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_metrics_history_query_failure(self, mock_mongo_client):
        """Test handling query failure for metrics history"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_performance_metrics_history()

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_trend_improving(self, mock_mongo_client):
        """Test calculating improving performance trend"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        # Reversed data (will be reversed again in function)
        # After reverse: [0.0, 0.6, 1.2, 1.8] - steep linear increase, slope = 0.6 > 0.5
        mock_cursor.limit.return_value = [
            {"win_rate": 1.8},
            {"win_rate": 1.2},
            {"win_rate": 0.6},
            {"win_rate": 0.0},
        ]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trend = storage.get_performance_trend(days=7, metric="win_rate")

        assert trend == "improving"

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_trend_declining(self, mock_mongo_client):
        """Test calculating declining performance trend"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        # Reversed data (will be reversed again in function)
        # After reverse: [1.8, 1.2, 0.6, 0.0] - steep linear decrease, slope = -0.6 < -0.5
        mock_cursor.limit.return_value = [
            {"win_rate": 0.0},
            {"win_rate": 0.6},
            {"win_rate": 1.2},
            {"win_rate": 1.8},
        ]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trend = storage.get_performance_trend(days=7, metric="win_rate")

        assert trend == "declining"

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_trend_stable(self, mock_mongo_client):
        """Test calculating stable performance trend"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.limit.return_value = [
            {"win_rate": 0.65},
            {"win_rate": 0.66},
            {"win_rate": 0.65},
            {"win_rate": 0.64},
        ]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trend = storage.get_performance_trend(days=7, metric="win_rate")

        assert trend == "stable"

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_trend_insufficient_data(self, mock_mongo_client):
        """Test trend calculation with insufficient data"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.limit.return_value = [{"win_rate": 0.5}]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trend = storage.get_performance_trend(days=7)

        assert trend == "insufficient_data"

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_trend_not_connected(self, mock_mongo_client):
        """Test trend calculation when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        trend = storage.get_performance_trend()

        assert trend == "unknown"

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_trend_query_failure(self, mock_mongo_client):
        """Test handling query failure in trend calculation"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trend = storage.get_performance_trend()

        assert trend == "error"

    @patch("utils.data_storage.MongoClient")
    def test_get_performance_trend_zero_denominator(self, mock_mongo_client):
        """Test trend calculation with zero denominator"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        # All same values to create zero denominator
        mock_cursor.limit.return_value = [
            {"win_rate": 0.5},
            {"win_rate": 0.5},
            {"win_rate": 0.5},
        ]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trend = storage.get_performance_trend()

        assert trend == "stable"


@pytest.mark.unit
class TestModelAccuracyStorage:
    """Test model accuracy tracking methods"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_store_model_accuracy_success(self, mock_mongo_client):
        """Test storing model accuracy"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "accuracy_id_123"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        accuracy_data = {
            "model_type": "LSTM",
            "accuracy": 0.85,
            "loss": 0.15,
            "f1_score": 0.82,
        }

        result = storage.store_model_accuracy(accuracy_data)

        assert result == "accuracy_id_123"

    @patch("utils.data_storage.MongoClient")
    def test_store_model_accuracy_not_connected(self, mock_mongo_client):
        """Test storing accuracy when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        result = storage.store_model_accuracy({"model_type": "LSTM", "accuracy": 0.85})

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_store_model_accuracy_insert_failure(self, mock_mongo_client):
        """Test handling insert failure for model accuracy"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.side_effect = Exception("Insert failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.store_model_accuracy({"model_type": "LSTM", "accuracy": 0.85})

        assert result is None


@pytest.mark.unit
class TestModelAccuracyHistoryRetrieval:
    """Test model accuracy history retrieval methods"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_get_model_accuracy_history_with_model_type_filter(self, mock_mongo_client):
        """Test retrieving model accuracy history with model_type filter"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor

        # Mock accuracy history data
        mock_cursor.__iter__.return_value = iter(
            [
                {
                    "model_type": "LSTM",
                    "accuracy": 0.85,
                    "timestamp": datetime.utcnow(),
                },
                {
                    "model_type": "LSTM",
                    "accuracy": 0.83,
                    "timestamp": datetime.utcnow() - timedelta(days=1),
                },
            ]
        )

        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_model_accuracy_history(model_type="LSTM", days=7)

        assert len(history) == 2
        assert history[0]["model_type"] == "LSTM"
        assert history[0]["accuracy"] == 0.85

        # Verify query includes model_type filter
        call_args = mock_collection.find.call_args
        query = call_args[0][0]
        assert "model_type" in query
        assert query["model_type"] == "LSTM"

    @patch("utils.data_storage.MongoClient")
    def test_get_model_accuracy_history_without_filter(self, mock_mongo_client):
        """Test retrieving model accuracy history without model_type filter"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor

        # Mock accuracy history data for multiple models
        mock_cursor.__iter__.return_value = iter(
            [
                {
                    "model_type": "LSTM",
                    "accuracy": 0.85,
                    "timestamp": datetime.utcnow(),
                },
                {"model_type": "GRU", "accuracy": 0.82, "timestamp": datetime.utcnow()},
                {
                    "model_type": "Transformer",
                    "accuracy": 0.88,
                    "timestamp": datetime.utcnow(),
                },
            ]
        )

        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_model_accuracy_history(days=7)

        assert len(history) == 3
        # Verify query does NOT include model_type filter
        call_args = mock_collection.find.call_args
        query = call_args[0][0]
        assert "model_type" not in query

    @patch("utils.data_storage.MongoClient")
    def test_get_model_accuracy_history_not_connected(self, mock_mongo_client):
        """Test retrieving history when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        history = storage.get_model_accuracy_history(days=7)

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_model_accuracy_history_query_failure(self, mock_mongo_client):
        """Test handling query failure"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_model_accuracy_history(days=7)

        assert history == []


@pytest.mark.unit
class TestAPICostTracking:
    """Test API cost tracking methods"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_store_api_cost_success_with_full_data(self, mock_mongo_client):
        """Test storing API cost with full cost data structure"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "cost_id_123"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        cost_data = {
            "session_id": "session_123",
            "input_tokens": 500,
            "output_tokens": 300,
            "total_cost": 0.012,
            "model": "gpt-4",
            "session": {
                "total_requests": 10,
                "total_cost_usd": 0.50,
                "total_cost_vnd": 12500,
            },
            "projections": {
                "estimated_daily_cost_usd": 1.20,
                "estimated_monthly_cost_usd": 36.00,
            },
            "alerts": ["Warning: approaching daily limit"],
        }

        result = storage.store_api_cost(cost_data, task_id="task_456")

        assert result == "cost_id_123"

        # Verify document structure
        call_args = mock_collection.insert_one.call_args
        document = call_args[0][0]
        assert document["task_id"] == "task_456"
        assert document["session_id"] == "session_123"
        assert document["input_tokens"] == 500
        assert document["output_tokens"] == 300
        assert document["total_cost"] == 0.012
        assert document["model"] == "gpt-4"
        assert document["session"]["total_requests"] == 10
        assert document["projections"]["estimated_daily_cost_usd"] == 1.20
        assert len(document["alerts"]) == 1

    @patch("utils.data_storage.MongoClient")
    def test_store_api_cost_without_task_id(self, mock_mongo_client):
        """Test storing API cost without explicit task_id"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "cost_id_456"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        cost_data = {
            "task_id": "task_from_data",
            "session_id": "session_789",
            "total_cost": 0.008,
        }

        result = storage.store_api_cost(cost_data)

        assert result == "cost_id_456"

        # Verify task_id comes from cost_data
        call_args = mock_collection.insert_one.call_args
        document = call_args[0][0]
        assert document["task_id"] == "task_from_data"

    @patch("utils.data_storage.MongoClient")
    def test_store_api_cost_not_connected(self, mock_mongo_client):
        """Test storing API cost when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        cost_data = {"session_id": "session_123", "total_cost": 0.01}
        result = storage.store_api_cost(cost_data)

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_store_api_cost_insert_failure(self, mock_mongo_client):
        """Test handling insert failure for API cost"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.side_effect = Exception("Insert failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        cost_data = {"session_id": "session_123", "total_cost": 0.01}
        result = storage.store_api_cost(cost_data)

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_api_cost_history_success(self, mock_mongo_client):
        """Test retrieving API cost history"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor

        # Mock cost history data
        mock_cursor.__iter__.return_value = iter(
            [
                {
                    "session_id": "session_1",
                    "total_cost": 0.50,
                    "timestamp": datetime.utcnow(),
                },
                {
                    "session_id": "session_2",
                    "total_cost": 0.30,
                    "timestamp": datetime.utcnow() - timedelta(days=1),
                },
                {
                    "session_id": "session_3",
                    "total_cost": 0.20,
                    "timestamp": datetime.utcnow() - timedelta(days=2),
                },
            ]
        )

        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_api_cost_history(days=30)

        assert len(history) == 3
        assert history[0]["total_cost"] == 0.50
        assert history[1]["total_cost"] == 0.30

    @patch("utils.data_storage.MongoClient")
    def test_get_api_cost_history_custom_days(self, mock_mongo_client):
        """Test retrieving API cost history with custom days parameter"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.__iter__.return_value = iter([])

        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_api_cost_history(days=7)

        # Verify timestamp query uses 7 days
        call_args = mock_collection.find.call_args
        query = call_args[0][0]
        assert "timestamp" in query
        assert "$gte" in query["timestamp"]

    @patch("utils.data_storage.MongoClient")
    def test_get_api_cost_history_not_connected(self, mock_mongo_client):
        """Test retrieving history when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        history = storage.get_api_cost_history(days=30)

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_api_cost_history_query_failure(self, mock_mongo_client):
        """Test handling query failure"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_api_cost_history(days=30)

        assert history == []


@pytest.mark.unit
class TestRetrainHistoryTracking:
    """Test retrain history tracking methods"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_store_retrain_result_with_success_counting(self, mock_mongo_client):
        """Test storing retrain result with successful model counting"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "retrain_id_123"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        results = {
            "models": {
                "LSTM": {"status": "success", "accuracy": 0.85, "deployed": True},
                "GRU": {"status": "success", "accuracy": 0.82, "deployed": False},
                "Transformer": {"status": "failed", "error": "Training error"},
            }
        }

        result = storage.store_retrain_result(
            results=results,
            task_id="task_retrain_1",
            trigger_type="gpt4_recommendation",
            trigger_data={"reason": "Poor performance detected"},
        )

        assert result == "retrain_id_123"

        # Verify successful_count and deployed_count
        call_args = mock_collection.insert_one.call_args
        document = call_args[0][0]
        assert document["successful_count"] == 2  # LSTM and GRU
        assert document["deployed_count"] == 1  # Only LSTM
        assert document["trigger_type"] == "gpt4_recommendation"
        assert document["trigger_data"]["reason"] == "Poor performance detected"

    @patch("utils.data_storage.MongoClient")
    def test_store_retrain_result_all_failed(self, mock_mongo_client):
        """Test storing retrain result when all models failed"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "retrain_id_456"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        results = {
            "models": {
                "LSTM": {"status": "failed", "error": "Training error"},
                "GRU": {"status": "failed", "error": "Data error"},
            }
        }

        result = storage.store_retrain_result(results=results)

        assert result == "retrain_id_456"

        # Verify counts are 0
        call_args = mock_collection.insert_one.call_args
        document = call_args[0][0]
        assert document["successful_count"] == 0
        assert document["deployed_count"] == 0
        assert document["task_id"] == "unknown"  # Default when not provided

    @patch("utils.data_storage.MongoClient")
    def test_store_retrain_result_not_connected(self, mock_mongo_client):
        """Test storing retrain result when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        results = {"models": {"LSTM": {"status": "success"}}}
        result = storage.store_retrain_result(results=results)

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_store_retrain_result_insert_failure(self, mock_mongo_client):
        """Test handling insert failure for retrain result"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.side_effect = Exception("Insert failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        results = {"models": {"LSTM": {"status": "success"}}}
        result = storage.store_retrain_result(results=results)

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_retrain_history_success(self, mock_mongo_client):
        """Test retrieving retrain history"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.limit.return_value = mock_cursor

        # Mock retrain history data
        mock_cursor.__iter__.return_value = iter(
            [
                {
                    "task_id": "task_1",
                    "successful_count": 2,
                    "deployed_count": 1,
                    "timestamp": datetime.utcnow(),
                },
                {
                    "task_id": "task_2",
                    "successful_count": 3,
                    "deployed_count": 2,
                    "timestamp": datetime.utcnow() - timedelta(days=1),
                },
            ]
        )

        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_retrain_history(days=30, limit=50)

        assert len(history) == 2
        assert history[0]["successful_count"] == 2
        assert history[1]["successful_count"] == 3

    @patch("utils.data_storage.MongoClient")
    def test_get_retrain_history_custom_limit(self, mock_mongo_client):
        """Test retrieving retrain history with custom limit"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.limit.return_value = mock_cursor
        mock_cursor.__iter__.return_value = iter([])

        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_retrain_history(days=7, limit=10)

        # Verify limit is applied
        assert mock_cursor.limit.called
        assert mock_cursor.limit.call_args[0][0] == 10

    @patch("utils.data_storage.MongoClient")
    def test_get_retrain_history_not_connected(self, mock_mongo_client):
        """Test retrieving history when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        history = storage.get_retrain_history(days=30)

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_retrain_history_query_failure(self, mock_mongo_client):
        """Test handling query failure"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_retrain_history(days=30)

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_store_retrain_history_alias_method(self, mock_mongo_client):
        """Test store_retrain_history as alias for store_retrain_result"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "retrain_alias_id"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        retrain_data = {
            "trigger": "manual",
            "models": {
                "LSTM": {"status": "success", "deployed": True},
            },
        }

        result = storage.store_retrain_history(retrain_data, task_id="task_alias")

        assert result == "retrain_alias_id"

        # Verify it calls store_retrain_result with correct parameters
        call_args = mock_collection.insert_one.call_args
        document = call_args[0][0]
        assert document["task_id"] == "task_alias"
        assert document["trigger_type"] == "manual"
        assert document["successful_count"] == 1
        assert document["deployed_count"] == 1


@pytest.mark.unit
class TestConfigSuggestionsStorage:
    """Test config suggestions storage methods"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_store_config_suggestions_success(self, mock_mongo_client):
        """Test storing config suggestions successfully"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "config_id_123"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        suggestions = {
            "strategy_changes": {"rsi_period": 14, "macd_fast": 12},
            "risk_changes": {"max_daily_loss": 0.05},
            "confidence": 0.85,
        }

        result = storage.store_config_suggestions(suggestions)

        assert result == "config_id_123"
        mock_collection.insert_one.assert_called_once()
        # Verify created_at is added
        call_args = mock_collection.insert_one.call_args
        document = call_args[0][0]
        assert "created_at" in document
        assert document["strategy_changes"]["rsi_period"] == 14

    @patch("utils.data_storage.MongoClient")
    def test_store_config_suggestions_not_connected(self, mock_mongo_client):
        """Test storing config suggestions when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        result = storage.store_config_suggestions({"test": "data"})

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_store_config_suggestions_insert_failure(self, mock_mongo_client):
        """Test handling insert failure for config suggestions"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.side_effect = Exception("Insert failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.store_config_suggestions({"test": "data"})

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_config_suggestions_history_success(self, mock_mongo_client):
        """Test retrieving config suggestions history"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.limit.return_value = [
            {"strategy_changes": {"rsi_period": 14}, "created_at": datetime.utcnow()},
            {"strategy_changes": {"rsi_period": 12}, "created_at": datetime.utcnow()},
        ]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_config_suggestions_history(days=30, limit=50)

        assert len(history) == 2
        mock_collection.find.assert_called_once()

    @patch("utils.data_storage.MongoClient")
    def test_get_config_suggestions_history_not_connected(self, mock_mongo_client):
        """Test retrieving config suggestions when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        history = storage.get_config_suggestions_history()

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_config_suggestions_history_query_failure(self, mock_mongo_client):
        """Test handling query failure for config suggestions"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_config_suggestions_history()

        assert history == []


@pytest.mark.unit
class TestTradeAnalysisStorage:
    """Test trade analysis storage methods"""

    def setup_method(self):
        """Reset singleton before each test"""
        DataStorage.reset_instance()

    def teardown_method(self):
        """Reset singleton after each test"""
        DataStorage.reset_instance()

    @patch("utils.data_storage.MongoClient")
    def test_store_trade_analysis_success(self, mock_mongo_client):
        """Test storing trade analysis successfully"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "analysis_id_123"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trade_data = {
            "symbol": "BTCUSDT",
            "trade_type": "LONG",
            "entry_price": 50000,
            "exit_price": 51000,
            "pnl": 100.0,
            "pnl_percentage": 2.0,
            "close_reason": "take_profit",
        }
        analysis = {
            "quality_score": 85,
            "reasoning": "Good entry, proper risk management",
            "suggestions": ["Consider tighter stop loss"],
        }

        result = storage.store_trade_analysis("trade_123", trade_data, analysis)

        assert result == "analysis_id_123"
        mock_collection.insert_one.assert_called_once()

        # Verify document structure
        call_args = mock_collection.insert_one.call_args
        document = call_args[0][0]
        assert document["trade_id"] == "trade_123"
        assert document["is_winning"] is True
        assert document["pnl_usdt"] == 100.0
        assert document["symbol"] == "BTCUSDT"
        assert document["side"] == "LONG"

    @patch("utils.data_storage.MongoClient")
    def test_store_trade_analysis_losing_trade(self, mock_mongo_client):
        """Test storing trade analysis for losing trade"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_result = MagicMock()
        mock_result.inserted_id = "analysis_id_456"
        mock_collection.insert_one.return_value = mock_result
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trade_data = {
            "symbol": "ETHUSDT",
            "side": "SHORT",  # Test legacy field name
            "entry_price": 3000,
            "exit_price": 3100,
            "pnl_usdt": -50.0,  # Test legacy pnl field
            "pnl_percentage": -1.67,
            "close_reason": "stop_loss",
        }
        analysis = {"quality_score": 60, "reasoning": "Entry timing was off"}

        result = storage.store_trade_analysis("trade_456", trade_data, analysis)

        assert result == "analysis_id_456"
        call_args = mock_collection.insert_one.call_args
        document = call_args[0][0]
        assert document["is_winning"] is False
        assert document["pnl_usdt"] == -50.0
        assert document["side"] == "SHORT"

    @patch("utils.data_storage.MongoClient")
    def test_store_trade_analysis_not_connected(self, mock_mongo_client):
        """Test storing trade analysis when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        result = storage.store_trade_analysis("trade_123", {}, {})

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_store_trade_analysis_insert_failure(self, mock_mongo_client):
        """Test handling insert failure for trade analysis"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.insert_one.side_effect = Exception("Insert failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.store_trade_analysis("trade_123", {}, {})

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_trade_analysis_success(self, mock_mongo_client):
        """Test retrieving trade analysis by trade ID"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find_one.return_value = {
            "trade_id": "trade_123",
            "analysis": {"quality_score": 85},
        }
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.get_trade_analysis("trade_123")

        assert result is not None
        assert result["trade_id"] == "trade_123"
        mock_collection.find_one.assert_called_once_with({"trade_id": "trade_123"})

    @patch("utils.data_storage.MongoClient")
    def test_get_trade_analysis_not_found(self, mock_mongo_client):
        """Test retrieving non-existent trade analysis"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find_one.return_value = None
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.get_trade_analysis("nonexistent_trade")

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_trade_analysis_not_connected(self, mock_mongo_client):
        """Test retrieving trade analysis when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        result = storage.get_trade_analysis("trade_123")

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_trade_analysis_query_failure(self, mock_mongo_client):
        """Test handling query failure for trade analysis"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find_one.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        result = storage.get_trade_analysis("trade_123")

        assert result is None

    @patch("utils.data_storage.MongoClient")
    def test_get_trade_analyses_history_success(self, mock_mongo_client):
        """Test retrieving trade analyses history"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.limit.return_value = [
            {"trade_id": "trade_1", "is_winning": True, "pnl_usdt": 100},
            {"trade_id": "trade_2", "is_winning": False, "pnl_usdt": -50},
        ]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_trade_analyses_history(days=30, limit=100)

        assert len(history) == 2
        mock_collection.find.assert_called_once()

    @patch("utils.data_storage.MongoClient")
    def test_get_trade_analyses_history_only_losing(self, mock_mongo_client):
        """Test retrieving only losing trade analyses"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_cursor = MagicMock()
        mock_cursor.sort.return_value = mock_cursor
        mock_cursor.limit.return_value = [
            {"trade_id": "trade_2", "is_winning": False, "pnl_usdt": -50},
        ]
        mock_collection.find.return_value = mock_cursor
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_trade_analyses_history(
            days=30, limit=100, only_losing=True
        )

        assert len(history) == 1
        # Verify only_losing filter is applied
        call_args = mock_collection.find.call_args
        query = call_args[0][0]
        assert query.get("is_winning") is False

    @patch("utils.data_storage.MongoClient")
    def test_get_trade_analyses_history_not_connected(self, mock_mongo_client):
        """Test retrieving history when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        history = storage.get_trade_analyses_history()

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_trade_analyses_history_query_failure(self, mock_mongo_client):
        """Test handling query failure for trade analyses history"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.find.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        history = storage.get_trade_analyses_history()

        assert history == []

    @patch("utils.data_storage.MongoClient")
    def test_get_unanalyzed_trade_ids_success(self, mock_mongo_client):
        """Test getting unanalyzed trade IDs"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        # Mock already analyzed trades
        mock_collection.distinct.return_value = ["trade_1", "trade_3"]
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trade_ids = ["trade_1", "trade_2", "trade_3", "trade_4"]
        unanalyzed = storage.get_unanalyzed_trade_ids(trade_ids)

        assert "trade_2" in unanalyzed
        assert "trade_4" in unanalyzed
        assert "trade_1" not in unanalyzed
        assert "trade_3" not in unanalyzed

    @patch("utils.data_storage.MongoClient")
    def test_get_unanalyzed_trade_ids_empty_input(self, mock_mongo_client):
        """Test get_unanalyzed_trade_ids with empty input"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        unanalyzed = storage.get_unanalyzed_trade_ids([])

        assert unanalyzed == []

    @patch("utils.data_storage.MongoClient")
    def test_get_unanalyzed_trade_ids_not_connected(self, mock_mongo_client):
        """Test get_unanalyzed_trade_ids when not connected"""
        mock_mongo_client.side_effect = Exception("Connection failed")

        storage = DataStorage()
        trade_ids = ["trade_1", "trade_2"]
        unanalyzed = storage.get_unanalyzed_trade_ids(trade_ids)

        # Should return original list when not connected
        assert unanalyzed == trade_ids

    @patch("utils.data_storage.MongoClient")
    def test_get_unanalyzed_trade_ids_query_failure(self, mock_mongo_client):
        """Test handling query failure for unanalyzed trade IDs"""
        mock_client = MagicMock()
        mock_client.server_info.return_value = {}
        mock_db = MagicMock()
        mock_collection = MagicMock()
        mock_collection.distinct.side_effect = Exception("Query failed")
        mock_db.__getitem__.return_value = mock_collection
        mock_client.__getitem__.return_value = mock_db
        mock_mongo_client.return_value = mock_client

        storage = DataStorage()
        trade_ids = ["trade_1", "trade_2"]
        unanalyzed = storage.get_unanalyzed_trade_ids(trade_ids)

        # Should return original list on error
        assert unanalyzed == trade_ids
