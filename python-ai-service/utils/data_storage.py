#!/usr/bin/env python3
"""
Data Storage Utilities for Async Tasks
Handles storing analysis results, metrics, and audit trails in MongoDB
"""

import os
from typing import Dict, Any, List, Optional
from datetime import datetime, timedelta
from motor.motor_asyncio import AsyncIOMotorClient
from pymongo import MongoClient, DESCENDING
from utils.logger import get_logger

logger = get_logger("DataStorage")

# MongoDB connection
# Try DATABASE_URL first (used in docker-compose), fallback to MONGODB_URI
MONGODB_URI = os.getenv("DATABASE_URL") or os.getenv("MONGODB_URI", "mongodb://localhost:27017")

# Extract database name from URI or use environment variable or default
def _extract_db_name(uri: str) -> str:
    """Extract database name from MongoDB URI"""
    # URI format: mongodb://user:pass@host:port/database?options
    if "/" in uri.split("@")[-1]:  # Has database name
        parts = uri.split("@")[-1].split("/")
        if len(parts) > 1:
            db_name = parts[1].split("?")[0]  # Remove query params
            if db_name:
                return db_name
    return os.getenv("MONGODB_DB", "bot_core")

MONGODB_DB = _extract_db_name(MONGODB_URI)

# Collections
COLLECTION_GPT4_ANALYSIS = "gpt4_analysis_history"
COLLECTION_PERFORMANCE_METRICS = "performance_metrics"
COLLECTION_MODEL_ACCURACY = "model_accuracy_history"
COLLECTION_API_COSTS = "api_cost_history"
COLLECTION_RETRAIN_HISTORY = "retrain_history"


class DataStorage:
    """Singleton class for MongoDB data storage"""

    _instance = None
    _client = None
    _db = None

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(DataStorage, cls).__new__(cls)
            cls._instance._initialize()
        return cls._instance

    def _initialize(self):
        """Initialize MongoDB connection"""
        try:
            self._client = MongoClient(MONGODB_URI, serverSelectionTimeoutMS=5000)
            # Test connection
            self._client.server_info()
            self._db = self._client[MONGODB_DB]
            logger.info(f"✅ Connected to MongoDB: {MONGODB_DB}")
            self._ensure_indexes()
        except Exception as e:
            logger.error(f"❌ Failed to connect to MongoDB: {e}")
            self._client = None
            self._db = None

    def _ensure_indexes(self):
        """Create indexes for efficient queries"""
        if self._db is None:
            return

        try:
            # GPT-4 analysis indexes
            self._db[COLLECTION_GPT4_ANALYSIS].create_index([("timestamp", DESCENDING)])
            self._db[COLLECTION_GPT4_ANALYSIS].create_index("recommendation")

            # Performance metrics indexes
            self._db[COLLECTION_PERFORMANCE_METRICS].create_index([("timestamp", DESCENDING)])
            self._db[COLLECTION_PERFORMANCE_METRICS].create_index([("date", DESCENDING)])

            # Model accuracy indexes
            self._db[COLLECTION_MODEL_ACCURACY].create_index([("timestamp", DESCENDING)])
            self._db[COLLECTION_MODEL_ACCURACY].create_index("model_type")

            # API cost indexes
            self._db[COLLECTION_API_COSTS].create_index([("timestamp", DESCENDING)])

            # Retrain history indexes
            self._db[COLLECTION_RETRAIN_HISTORY].create_index([("timestamp", DESCENDING)])
            self._db[COLLECTION_RETRAIN_HISTORY].create_index("trigger_type")

            logger.info("✅ MongoDB indexes created")
        except Exception as e:
            logger.warning(f"⚠️ Failed to create indexes: {e}")

    def is_connected(self) -> bool:
        """Check if MongoDB is connected"""
        return self._client is not None and self._db is not None

    # =========================================================================
    # GPT-4 ANALYSIS STORAGE
    # =========================================================================

    def store_gpt4_analysis(
        self,
        analysis_result: Dict[str, Any],
        task_id: str,
    ) -> Optional[str]:
        """
        Store GPT-4 self-analysis result for audit trail

        Args:
            analysis_result: GPT-4 analysis output
            task_id: Celery task ID

        Returns:
            Document ID if successful, None otherwise
        """
        if not self.is_connected():
            logger.warning("⚠️ MongoDB not connected, skipping GPT-4 analysis storage")
            return None

        try:
            document = {
                "timestamp": datetime.utcnow(),
                "task_id": task_id,
                "recommendation": analysis_result.get("recommendation"),
                "confidence": analysis_result.get("confidence"),
                "urgency": analysis_result.get("urgency"),
                "reasoning": analysis_result.get("reasoning"),
                "suggested_actions": analysis_result.get("suggested_actions", []),
                "estimated_improvement": analysis_result.get("estimated_improvement"),
                "retrain_triggered": analysis_result.get("retrain_triggered", False),
                "retrain_task_id": analysis_result.get("retrain_task_id"),
                "full_analysis": analysis_result,
            }

            result = self._db[COLLECTION_GPT4_ANALYSIS].insert_one(document)
            logger.info(f"✅ Stored GPT-4 analysis: {result.inserted_id}")
            return str(result.inserted_id)

        except Exception as e:
            logger.error(f"❌ Failed to store GPT-4 analysis: {e}")
            return None

    def get_gpt4_analysis_history(
        self,
        days: int = 30,
        limit: int = 100,
    ) -> List[Dict[str, Any]]:
        """Get GPT-4 analysis history"""
        if not self.is_connected():
            return []

        try:
            since = datetime.utcnow() - timedelta(days=days)
            cursor = self._db[COLLECTION_GPT4_ANALYSIS].find(
                {"timestamp": {"$gte": since}}
            ).sort("timestamp", DESCENDING).limit(limit)

            return list(cursor)

        except Exception as e:
            logger.error(f"❌ Failed to retrieve GPT-4 history: {e}")
            return []

    # =========================================================================
    # PERFORMANCE METRICS STORAGE
    # =========================================================================

    def store_performance_metrics(
        self,
        metrics: Dict[str, Any],
        task_id: str,
    ) -> Optional[str]:
        """
        Store daily performance metrics

        Args:
            metrics: Performance metrics (win_rate, avg_profit, sharpe, etc.)
            task_id: Celery task ID

        Returns:
            Document ID if successful, None otherwise
        """
        if not self.is_connected():
            logger.warning("⚠️ MongoDB not connected, skipping performance metrics storage")
            return None

        try:
            document = {
                "timestamp": datetime.utcnow(),
                "date": metrics.get("date", datetime.utcnow().strftime("%Y-%m-%d")),
                "task_id": task_id,
                "total_trades": metrics.get("total_trades"),
                "win_rate": metrics.get("win_rate"),
                "avg_profit_per_trade": metrics.get("avg_profit_per_trade"),
                "sharpe_ratio": metrics.get("sharpe_ratio"),
                "performance_status": {
                    "win_rate_status": metrics.get("performance", {}).get("win_rate_status"),
                    "avg_profit_status": metrics.get("performance", {}).get("avg_profit_status"),
                    "sharpe_status": metrics.get("performance", {}).get("sharpe_status"),
                },
                "alerts": metrics.get("alerts", []),
                "trigger_ai_analysis": metrics.get("trigger_ai_analysis", False),
                "full_metrics": metrics,
            }

            result = self._db[COLLECTION_PERFORMANCE_METRICS].insert_one(document)
            logger.info(f"✅ Stored performance metrics: {result.inserted_id}")
            return str(result.inserted_id)

        except Exception as e:
            logger.error(f"❌ Failed to store performance metrics: {e}")
            return None

    def get_performance_metrics_history(
        self,
        days: int = 7,
    ) -> List[Dict[str, Any]]:
        """Get performance metrics history for trend analysis"""
        if not self.is_connected():
            return []

        try:
            since = datetime.utcnow() - timedelta(days=days)
            cursor = self._db[COLLECTION_PERFORMANCE_METRICS].find(
                {"timestamp": {"$gte": since}}
            ).sort("timestamp", DESCENDING)

            return list(cursor)

        except Exception as e:
            logger.error(f"❌ Failed to retrieve performance history: {e}")
            return []

    # =========================================================================
    # MODEL ACCURACY TRACKING
    # =========================================================================

    def store_model_accuracy(
        self,
        model_type: str,
        accuracy: float,
        additional_metrics: Optional[Dict[str, Any]] = None,
    ) -> Optional[str]:
        """
        Store model accuracy for tracking model performance over time

        Args:
            model_type: Model type (lstm, gru, transformer)
            accuracy: Model accuracy (0.0-1.0)
            additional_metrics: Additional metrics (loss, f1_score, etc.)

        Returns:
            Document ID if successful, None otherwise
        """
        if not self.is_connected():
            logger.warning("⚠️ MongoDB not connected, skipping model accuracy storage")
            return None

        try:
            document = {
                "timestamp": datetime.utcnow(),
                "model_type": model_type,
                "accuracy": accuracy,
                "additional_metrics": additional_metrics or {},
            }

            result = self._db[COLLECTION_MODEL_ACCURACY].insert_one(document)
            logger.info(f"✅ Stored model accuracy for {model_type}: {accuracy:.2%}")
            return str(result.inserted_id)

        except Exception as e:
            logger.error(f"❌ Failed to store model accuracy: {e}")
            return None

    def get_model_accuracy_history(
        self,
        model_type: Optional[str] = None,
        days: int = 7,
    ) -> List[Dict[str, Any]]:
        """Get model accuracy history"""
        if not self.is_connected():
            return []

        try:
            since = datetime.utcnow() - timedelta(days=days)
            query = {"timestamp": {"$gte": since}}

            if model_type:
                query["model_type"] = model_type

            cursor = self._db[COLLECTION_MODEL_ACCURACY].find(query).sort("timestamp", DESCENDING)

            return list(cursor)

        except Exception as e:
            logger.error(f"❌ Failed to retrieve model accuracy history: {e}")
            return []

    # =========================================================================
    # API COST TRACKING
    # =========================================================================

    def store_api_cost(
        self,
        cost_data: Dict[str, Any],
        task_id: str,
    ) -> Optional[str]:
        """
        Store API cost data for tracking GPT-4 usage

        Args:
            cost_data: Cost data from daily_api_cost_report
            task_id: Celery task ID

        Returns:
            Document ID if successful, None otherwise
        """
        if not self.is_connected():
            logger.warning("⚠️ MongoDB not connected, skipping API cost storage")
            return None

        try:
            document = {
                "timestamp": datetime.utcnow(),
                "date": cost_data.get("date", datetime.utcnow().strftime("%Y-%m-%d")),
                "task_id": task_id,
                "session": cost_data.get("session", {}),
                "projections": cost_data.get("projections", {}),
                "alerts": cost_data.get("alerts", []),
                "full_data": cost_data,
            }

            result = self._db[COLLECTION_API_COSTS].insert_one(document)
            logger.info(f"✅ Stored API cost data: {result.inserted_id}")
            return str(result.inserted_id)

        except Exception as e:
            logger.error(f"❌ Failed to store API cost: {e}")
            return None

    def get_api_cost_history(
        self,
        days: int = 30,
    ) -> List[Dict[str, Any]]:
        """Get API cost history"""
        if not self.is_connected():
            return []

        try:
            since = datetime.utcnow() - timedelta(days=days)
            cursor = self._db[COLLECTION_API_COSTS].find(
                {"timestamp": {"$gte": since}}
            ).sort("timestamp", DESCENDING)

            return list(cursor)

        except Exception as e:
            logger.error(f"❌ Failed to retrieve API cost history: {e}")
            return []

    # =========================================================================
    # RETRAIN HISTORY TRACKING
    # =========================================================================

    def store_retrain_result(
        self,
        results: Dict[str, Any],
        task_id: str,
        trigger_type: str = "gpt4_recommendation",
        trigger_data: Optional[Dict[str, Any]] = None,
    ) -> Optional[str]:
        """
        Store model retraining results

        Args:
            results: Retrain results from adaptive_retrain task
            task_id: Celery task ID
            trigger_type: What triggered the retrain
            trigger_data: Additional trigger information

        Returns:
            Document ID if successful, None otherwise
        """
        if not self.is_connected():
            logger.warning("⚠️ MongoDB not connected, skipping retrain result storage")
            return None

        try:
            document = {
                "timestamp": datetime.utcnow(),
                "task_id": task_id,
                "trigger_type": trigger_type,
                "trigger_data": trigger_data or {},
                "models": results.get("models", {}),
                "successful_count": sum(
                    1 for m in results.get("models", {}).values()
                    if m.get("status") == "success"
                ),
                "deployed_count": sum(
                    1 for m in results.get("models", {}).values()
                    if m.get("deployed", False)
                ),
                "full_results": results,
            }

            result = self._db[COLLECTION_RETRAIN_HISTORY].insert_one(document)
            logger.info(f"✅ Stored retrain result: {result.inserted_id}")
            return str(result.inserted_id)

        except Exception as e:
            logger.error(f"❌ Failed to store retrain result: {e}")
            return None

    def get_retrain_history(
        self,
        days: int = 30,
        limit: int = 50,
    ) -> List[Dict[str, Any]]:
        """Get retrain history"""
        if not self.is_connected():
            return []

        try:
            since = datetime.utcnow() - timedelta(days=days)
            cursor = self._db[COLLECTION_RETRAIN_HISTORY].find(
                {"timestamp": {"$gte": since}}
            ).sort("timestamp", DESCENDING).limit(limit)

            return list(cursor)

        except Exception as e:
            logger.error(f"❌ Failed to retrieve retrain history: {e}")
            return []


# Singleton instance
storage = DataStorage()
