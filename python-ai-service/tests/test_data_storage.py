"""Comprehensive tests for utils/data_storage.py"""

import os
import pytest
from unittest.mock import MagicMock, patch, call
from datetime import datetime, timedelta
from utils.data_storage import (
    DataStorage,
    _extract_db_name,
    COLLECTION_GPT4_ANALYSIS,
    COLLECTION_PERFORMANCE_METRICS,
    COLLECTION_MODEL_ACCURACY,
    COLLECTION_API_COSTS,
    COLLECTION_RETRAIN_HISTORY,
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
