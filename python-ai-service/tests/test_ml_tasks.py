#!/usr/bin/env python3
"""
Unit Tests for ML Tasks
Tests model training, prediction, and bulk analysis functionality
"""

import pytest
from datetime import datetime
import numpy as np


class TestTrainModel:
    """Test train_model task"""

    def test_train_model_lstm_success(self):
        """Test successful LSTM model training"""
        from tasks.ml_tasks import train_model

        result = train_model.run("lstm", "BTCUSDT", 1000)

        assert result["status"] == "success"
        assert result["model_type"] == "lstm"
        assert "accuracy" in result
        assert "loss" in result

    def test_train_model_gru_success(self):
        """Test successful GRU model training"""
        from tasks.ml_tasks import train_model

        result = train_model.run("gru", "ETHUSDT", 500)

        assert result["status"] == "success"
        assert result["model_type"] == "gru"

    def test_train_model_transformer_success(self):
        """Test successful Transformer model training"""
        from tasks.ml_tasks import train_model

        result = train_model.run("transformer", "BNBUSDT", 750)

        assert result["status"] == "success"
        assert result["model_type"] == "transformer"


class TestPredictPrice:
    """Test predict_price task"""

    def test_predict_price_lstm_success(self):
        """Test successful price prediction with LSTM"""
        from tasks.ml_tasks import predict_price

        result = predict_price.run("BTCUSDT", "lstm", 24)

        assert result["status"] == "success"
        assert "predictions" in result or "prediction" in result
        assert "model_type" in result

    def test_predict_price_gru_success(self):
        """Test successful price prediction with GRU"""
        from tasks.ml_tasks import predict_price

        result = predict_price.run("ETHUSDT", "gru", 12)

        assert result["status"] == "success"
        assert result["model_type"] == "gru"

    def test_predict_price_transformer_success(self):
        """Test successful price prediction with Transformer"""
        from tasks.ml_tasks import predict_price

        result = predict_price.run("BNBUSDT", "transformer", 6)

        assert result["status"] == "success"
        assert result["model_type"] == "transformer"


class TestBulkAnalysis:
    """Test bulk_analysis task"""

    def test_bulk_analysis_multiple_symbols_models(self):
        """Test successful bulk analysis with multiple symbols and models"""
        from tasks.ml_tasks import bulk_analysis

        result = bulk_analysis.run(["BTCUSDT", "ETHUSDT", "BNBUSDT"], ["lstm", "gru"])

        assert result["status"] == "success"
        assert "results" in result
        assert len(result["results"]) > 0

    def test_bulk_analysis_single_symbol(self):
        """Test bulk analysis with single symbol"""
        from tasks.ml_tasks import bulk_analysis

        result = bulk_analysis.run(["BTCUSDT"], ["lstm"])

        assert result["status"] == "success"
        assert len(result["results"]) >= 1


class TestMLTaskCallbacks:
    """Test ML task callbacks"""

    def test_ml_task_on_failure(self):
        """Test ML task failure callback"""
        from tasks.ml_tasks import MLTask

        task = MLTask()

        # Should not raise exception
        task.on_failure(
            exc=Exception("Training failed"),
            task_id="test-id",
            args=(),
            kwargs={},
            einfo=None
        )

    def test_ml_task_on_success(self):
        """Test ML task success callback"""
        from tasks.ml_tasks import MLTask

        task = MLTask()

        # Should not raise exception
        task.on_success(
            retval={"status": "success", "accuracy": 75.0},
            task_id="test-id",
            args=(),
            kwargs={}
        )


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
