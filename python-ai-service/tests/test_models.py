"""Comprehensive tests for all model files to achieve >90% coverage"""

import io
import os
import sys
import tempfile
from unittest.mock import MagicMock, Mock, call, mock_open, patch

import numpy as np
import pandas as pd
import pytest

# Mock TensorFlow before importing models
tf_mock = MagicMock()
tf_mock.keras = MagicMock()
tf_mock.keras.models = MagicMock()
tf_mock.keras.layers = MagicMock()
tf_mock.keras.optimizers = MagicMock()
tf_mock.keras.callbacks = MagicMock()
sys.modules["tensorflow"] = tf_mock
sys.modules["tensorflow.keras"] = tf_mock.keras
sys.modules["tensorflow.keras.models"] = tf_mock.keras.models
sys.modules["tensorflow.keras.layers"] = tf_mock.keras.layers
sys.modules["tensorflow.keras.optimizers"] = tf_mock.keras.optimizers
sys.modules["tensorflow.keras.callbacks"] = tf_mock.keras.callbacks

from models.gru_model import GRUModel
from models.lstm_model import LSTMModel
from models.model_manager import ModelManager
from models.transformer_model import TransformerModel


class TestLSTMModel:
    """Comprehensive tests for LSTM model - targeting >90% coverage"""

    def test_init(self):
        """Test LSTM model initialization"""
        with patch("models.lstm_model.config") as mock_config:
            mock_config.get_model_config.return_value = {
                "hidden_size": 64,
                "dropout": 0.2,
            }
            model = LSTMModel()
            assert model.config is not None
            assert model.model is None
            assert model.history is None

    @patch("models.lstm_model.Sequential")
    @patch("models.lstm_model.Adam")
    @patch("models.lstm_model.config")
    def test_build_model_success(self, mock_config, mock_adam, mock_sequential):
        """Test successful LSTM model building"""
        mock_config.get_model_config.return_value = {
            "hidden_size": 64,
            "dropout": 0.2,
            "learning_rate": 0.001,
        }

        mock_model = MagicMock()
        mock_model.count_params.return_value = 10000
        mock_sequential.return_value = mock_model

        model = LSTMModel()
        result = model.build_model((60, 15))

        assert result is not None
        assert model.model is not None
        mock_sequential.assert_called_once()
        mock_model.compile.assert_called_once()

    @patch("models.lstm_model.Sequential")
    @patch("models.lstm_model.config")
    def test_build_model_error(self, mock_config, mock_sequential):
        """Test build model with error"""
        mock_config.get_model_config.return_value = {"hidden_size": 64, "dropout": 0.2}
        mock_sequential.side_effect = Exception("Build error")

        model = LSTMModel()
        with pytest.raises(Exception, match="Build error"):
            model.build_model((60, 15))

    @patch("models.lstm_model.EarlyStopping")
    @patch("models.lstm_model.ReduceLROnPlateau")
    @patch("models.lstm_model.ModelCheckpoint")
    @patch("models.lstm_model.config")
    def test_train_with_validation_and_save_path(
        self, mock_config, mock_checkpoint, mock_reduce_lr, mock_early_stopping
    ):
        """Test training with validation data and save path"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_history = MagicMock()
        mock_history.history = {
            "loss": [0.5, 0.4, 0.3],
            "accuracy": [0.8, 0.85, 0.9],
            "val_loss": [0.6, 0.5, 0.45],
            "val_accuracy": [0.75, 0.8, 0.82],
        }
        mock_model.fit.return_value = mock_history
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)
        X_val = np.random.rand(20, 60, 15)
        y_val = np.random.rand(20, 1)

        results = model.train(X_train, y_train, X_val, y_val, save_path="/tmp/model.h5")

        assert results is not None
        assert "final_loss" in results
        assert "final_accuracy" in results
        assert "epochs_trained" in results
        assert "best_val_loss" in results
        assert "best_val_accuracy" in results
        assert results["final_loss"] == 0.3
        assert results["final_accuracy"] == 0.9
        assert results["epochs_trained"] == 3
        mock_model.fit.assert_called_once()

    @patch("models.lstm_model.EarlyStopping")
    @patch("models.lstm_model.ReduceLROnPlateau")
    @patch("models.lstm_model.config")
    def test_train_without_validation(
        self, mock_config, mock_reduce_lr, mock_early_stopping
    ):
        """Test training without validation data"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5, 0.4], "accuracy": [0.8, 0.85]}
        mock_model.fit.return_value = mock_history
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        results = model.train(X_train, y_train)

        assert results is not None
        assert results["best_val_loss"] == float("inf")
        assert results["best_val_accuracy"] == 0
        mock_model.fit.assert_called_once()

    @patch("models.lstm_model.Sequential")
    @patch("models.lstm_model.Adam")
    @patch("models.lstm_model.EarlyStopping")
    @patch("models.lstm_model.ReduceLROnPlateau")
    @patch("models.lstm_model.config")
    def test_train_builds_model_if_none(
        self,
        mock_config,
        mock_reduce_lr,
        mock_early_stopping,
        mock_adam,
        mock_sequential,
    ):
        """Test that training builds model if not exists"""
        mock_config.get_model_config.return_value = {
            "epochs": 10,
            "batch_size": 32,
            "hidden_size": 64,
            "dropout": 0.2,
            "learning_rate": 0.001,
        }

        mock_model = MagicMock()
        mock_model.count_params.return_value = 10000
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5], "accuracy": [0.8]}
        mock_model.fit.return_value = mock_history
        mock_sequential.return_value = mock_model

        model = LSTMModel()
        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        results = model.train(X_train, y_train)

        assert results is not None
        mock_sequential.assert_called_once()

    @patch("models.lstm_model.config")
    def test_train_error(self, mock_config):
        """Test training error handling"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.fit.side_effect = Exception("Training error")
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        with pytest.raises(Exception, match="Training error"):
            model.train(X_train, y_train)

    @patch("models.lstm_model.config")
    def test_predict_success(self, mock_config):
        """Test successful prediction"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.predict.return_value = np.array([[0.7], [0.8], [0.6]])
        model.model = mock_model

        X_test = np.random.rand(3, 60, 15)
        predictions = model.predict(X_test)

        assert predictions is not None
        assert len(predictions) == 3
        mock_model.predict.assert_called_once_with(X_test, verbose=0)

    @patch("models.lstm_model.config")
    def test_predict_without_model(self, mock_config):
        """Test prediction without trained model"""
        mock_config.get_model_config.return_value = {}
        model = LSTMModel()
        X_test = np.random.rand(1, 60, 15)

        with pytest.raises(ValueError, match="Model not trained or loaded"):
            model.predict(X_test)

    @patch("models.lstm_model.config")
    def test_predict_error(self, mock_config):
        """Test prediction error handling"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.predict.side_effect = Exception("Prediction error")
        model.model = mock_model

        X_test = np.random.rand(1, 60, 15)

        with pytest.raises(Exception, match="Prediction error"):
            model.predict(X_test)

    @patch("models.lstm_model.config")
    def test_predict_single_2d_input(self, mock_config):
        """Test single prediction with 2D input (reshape required)"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.predict.return_value = np.array([[0.75]])
        model.model = mock_model

        X_test = np.random.rand(60, 15)  # 2D input
        prediction = model.predict_single(X_test)

        assert isinstance(prediction, float)
        assert prediction == 0.75
        mock_model.predict.assert_called_once()

    @patch("models.lstm_model.config")
    def test_predict_single_3d_input(self, mock_config):
        """Test single prediction with 3D input (no reshape)"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.predict.return_value = np.array([[0.65]])
        model.model = mock_model

        X_test = np.random.rand(1, 60, 15)  # 3D input
        prediction = model.predict_single(X_test)

        assert isinstance(prediction, float)
        assert prediction == 0.65

    @patch("models.lstm_model.config")
    def test_predict_single_error_returns_neutral(self, mock_config):
        """Test that predict_single returns 0.5 on error"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.predict.side_effect = Exception("Prediction error")
        model.model = mock_model

        X_test = np.random.rand(60, 15)
        prediction = model.predict_single(X_test)

        assert prediction == 0.5  # Neutral prediction on error

    @patch("models.lstm_model.config")
    def test_evaluate_success(self, mock_config):
        """Test successful model evaluation"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.evaluate.return_value = [0.3, 0.9, 0.85, 0.88]
        mock_model.metrics_names = ["loss", "accuracy", "precision", "recall"]
        model.model = mock_model

        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        metrics = model.evaluate(X_test, y_test)

        assert metrics is not None
        assert isinstance(metrics, dict)
        assert metrics["loss"] == 0.3
        assert metrics["accuracy"] == 0.9
        mock_model.evaluate.assert_called_once_with(X_test, y_test, verbose=0)

    @patch("models.lstm_model.config")
    def test_evaluate_without_model(self, mock_config):
        """Test evaluation without trained model returns empty dict"""
        mock_config.get_model_config.return_value = {}
        model = LSTMModel()
        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        # The method returns empty dict on error instead of raising
        metrics = model.evaluate(X_test, y_test)
        assert metrics == {}

    @patch("models.lstm_model.config")
    def test_evaluate_error_returns_empty_dict(self, mock_config):
        """Test evaluation error returns empty dict"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.evaluate.side_effect = Exception("Evaluation error")
        model.model = mock_model

        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        metrics = model.evaluate(X_test, y_test)
        assert metrics == {}

    @patch("models.lstm_model.config")
    def test_save_model_success(self, mock_config):
        """Test successful model saving"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        model.model = mock_model

        result = model.save_model("/tmp/test_model.h5")

        assert result is True
        mock_model.save.assert_called_once_with("/tmp/test_model.h5")

    @patch("models.lstm_model.config")
    def test_save_model_no_model(self, mock_config):
        """Test saving when no model exists"""
        mock_config.get_model_config.return_value = {}
        model = LSTMModel()

        result = model.save_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.lstm_model.config")
    def test_save_model_error(self, mock_config):
        """Test save model error handling"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()
        mock_model.save.side_effect = Exception("Save error")
        model.model = mock_model

        result = model.save_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.lstm_model.tf.keras.models.load_model")
    @patch("models.lstm_model.config")
    def test_load_model_success(self, mock_config, mock_load):
        """Test successful model loading"""
        mock_config.get_model_config.return_value = {}

        mock_loaded_model = MagicMock()
        mock_load.return_value = mock_loaded_model

        model = LSTMModel()
        result = model.load_model("/tmp/test_model.h5")

        assert result is True
        assert model.model == mock_loaded_model
        mock_load.assert_called_once_with("/tmp/test_model.h5")

    @patch("models.lstm_model.tf.keras.models.load_model")
    @patch("models.lstm_model.config")
    def test_load_model_error(self, mock_config, mock_load):
        """Test load model error handling"""
        mock_config.get_model_config.return_value = {}
        mock_load.side_effect = Exception("Load error")

        model = LSTMModel()
        result = model.load_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.lstm_model.config")
    def test_get_model_summary_with_model(self, mock_config):
        """Test getting model summary when model exists"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_model = MagicMock()

        # Mock the summary method to write to stdout
        def mock_summary():
            print("Model: sequential")
            print("Total params: 10000")

        mock_model.summary = mock_summary
        model.model = mock_model

        summary = model.get_model_summary()

        assert summary is not None
        assert "Model: sequential" in summary
        assert "Total params: 10000" in summary

    @patch("models.lstm_model.config")
    def test_get_model_summary_without_model(self, mock_config):
        """Test getting model summary when no model exists"""
        mock_config.get_model_config.return_value = {}
        model = LSTMModel()

        summary = model.get_model_summary()

        assert summary is None

    @patch("models.lstm_model.config")
    def test_get_training_history_with_history(self, mock_config):
        """Test getting training history when it exists"""
        mock_config.get_model_config.return_value = {}

        model = LSTMModel()
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5, 0.4], "accuracy": [0.8, 0.85]}
        model.history = mock_history

        history = model.get_training_history()

        assert history is not None
        assert history == {"loss": [0.5, 0.4], "accuracy": [0.8, 0.85]}

    @patch("models.lstm_model.config")
    def test_get_training_history_without_history(self, mock_config):
        """Test getting training history when it doesn't exist"""
        mock_config.get_model_config.return_value = {}
        model = LSTMModel()

        history = model.get_training_history()

        assert history is None


class TestGRUModel:
    """Comprehensive tests for GRU model - targeting >90% coverage"""

    def test_init(self):
        """Test GRU model initialization"""
        with patch("models.gru_model.config") as mock_config:
            mock_config.get_model_config.return_value = {
                "hidden_size": 64,
                "dropout": 0.2,
            }
            model = GRUModel()
            assert model.config is not None
            assert model.model is None
            assert model.history is None

    @patch("models.gru_model.Sequential")
    @patch("models.gru_model.Adam")
    @patch("models.gru_model.config")
    def test_build_model_success(self, mock_config, mock_adam, mock_sequential):
        """Test successful GRU model building"""
        mock_config.get_model_config.return_value = {
            "hidden_size": 64,
            "dropout": 0.2,
            "learning_rate": 0.001,
        }

        mock_model = MagicMock()
        mock_model.count_params.return_value = 8000
        mock_sequential.return_value = mock_model

        model = GRUModel()
        result = model.build_model((60, 15))

        assert result is not None
        assert model.model is not None
        mock_sequential.assert_called_once()
        mock_model.compile.assert_called_once()

    @patch("models.gru_model.Sequential")
    @patch("models.gru_model.config")
    def test_build_model_error(self, mock_config, mock_sequential):
        """Test build model with error"""
        mock_config.get_model_config.return_value = {"hidden_size": 64, "dropout": 0.2}
        mock_sequential.side_effect = Exception("Build error")

        model = GRUModel()
        with pytest.raises(Exception, match="Build error"):
            model.build_model((60, 15))

    @patch("models.gru_model.EarlyStopping")
    @patch("models.gru_model.ReduceLROnPlateau")
    @patch("models.gru_model.ModelCheckpoint")
    @patch("models.gru_model.config")
    def test_train_with_validation_and_save_path(
        self, mock_config, mock_checkpoint, mock_reduce_lr, mock_early_stopping
    ):
        """Test training with validation data and save path"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = GRUModel()
        mock_model = MagicMock()
        mock_history = MagicMock()
        mock_history.history = {
            "loss": [0.5, 0.4, 0.3],
            "accuracy": [0.8, 0.85, 0.9],
            "val_loss": [0.6, 0.5, 0.45],
            "val_accuracy": [0.75, 0.8, 0.82],
        }
        mock_model.fit.return_value = mock_history
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)
        X_val = np.random.rand(20, 60, 15)
        y_val = np.random.rand(20, 1)

        results = model.train(X_train, y_train, X_val, y_val, save_path="/tmp/model.h5")

        assert results is not None
        assert "final_loss" in results
        assert "final_accuracy" in results
        assert results["final_loss"] == 0.3
        assert results["final_accuracy"] == 0.9

    @patch("models.gru_model.EarlyStopping")
    @patch("models.gru_model.ReduceLROnPlateau")
    @patch("models.gru_model.config")
    def test_train_without_validation(
        self, mock_config, mock_reduce_lr, mock_early_stopping
    ):
        """Test training without validation data"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = GRUModel()
        mock_model = MagicMock()
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5, 0.4], "accuracy": [0.8, 0.85]}
        mock_model.fit.return_value = mock_history
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        results = model.train(X_train, y_train)

        assert results is not None
        assert results["best_val_loss"] == float("inf")
        assert results["best_val_accuracy"] == 0

    @patch("models.gru_model.Sequential")
    @patch("models.gru_model.Adam")
    @patch("models.gru_model.EarlyStopping")
    @patch("models.gru_model.ReduceLROnPlateau")
    @patch("models.gru_model.config")
    def test_train_builds_model_if_none(
        self,
        mock_config,
        mock_reduce_lr,
        mock_early_stopping,
        mock_adam,
        mock_sequential,
    ):
        """Test that training builds model if not exists"""
        mock_config.get_model_config.return_value = {
            "epochs": 10,
            "batch_size": 32,
            "hidden_size": 64,
            "dropout": 0.2,
            "learning_rate": 0.001,
        }

        mock_model = MagicMock()
        mock_model.count_params.return_value = 8000
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5], "accuracy": [0.8]}
        mock_model.fit.return_value = mock_history
        mock_sequential.return_value = mock_model

        model = GRUModel()
        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        results = model.train(X_train, y_train)

        assert results is not None
        mock_sequential.assert_called_once()

    @patch("models.gru_model.config")
    def test_train_error(self, mock_config):
        """Test training error handling"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = GRUModel()
        mock_model = MagicMock()
        mock_model.fit.side_effect = Exception("Training error")
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        with pytest.raises(Exception, match="Training error"):
            model.train(X_train, y_train)

    @patch("models.gru_model.config")
    def test_predict_success(self, mock_config):
        """Test successful prediction"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()
        mock_model.predict.return_value = np.array([[0.7], [0.8]])
        model.model = mock_model

        X_test = np.random.rand(2, 60, 15)
        predictions = model.predict(X_test)

        assert predictions is not None
        assert len(predictions) == 2

    @patch("models.gru_model.config")
    def test_predict_without_model(self, mock_config):
        """Test prediction without trained model"""
        mock_config.get_model_config.return_value = {}
        model = GRUModel()
        X_test = np.random.rand(1, 60, 15)

        with pytest.raises(ValueError, match="Model not trained or loaded"):
            model.predict(X_test)

    @patch("models.gru_model.config")
    def test_predict_error(self, mock_config):
        """Test prediction error handling"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()
        mock_model.predict.side_effect = Exception("Prediction error")
        model.model = mock_model

        X_test = np.random.rand(1, 60, 15)

        with pytest.raises(Exception, match="Prediction error"):
            model.predict(X_test)

    @patch("models.gru_model.config")
    def test_predict_single_2d_input(self, mock_config):
        """Test single prediction with 2D input"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()
        mock_model.predict.return_value = np.array([[0.75]])
        model.model = mock_model

        X_test = np.random.rand(60, 15)
        prediction = model.predict_single(X_test)

        assert isinstance(prediction, float)
        assert prediction == 0.75

    @patch("models.gru_model.config")
    def test_predict_single_error_returns_neutral(self, mock_config):
        """Test that predict_single returns 0.5 on error"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()
        mock_model.predict.side_effect = Exception("Prediction error")
        model.model = mock_model

        X_test = np.random.rand(60, 15)
        prediction = model.predict_single(X_test)

        assert prediction == 0.5

    @patch("models.gru_model.config")
    def test_evaluate_success(self, mock_config):
        """Test successful model evaluation"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()
        mock_model.evaluate.return_value = [0.3, 0.9, 0.85, 0.88]
        mock_model.metrics_names = ["loss", "accuracy", "precision", "recall"]
        model.model = mock_model

        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        metrics = model.evaluate(X_test, y_test)

        assert metrics is not None
        assert isinstance(metrics, dict)
        assert metrics["loss"] == 0.3
        assert metrics["accuracy"] == 0.9

    @patch("models.gru_model.config")
    def test_evaluate_without_model(self, mock_config):
        """Test evaluation without trained model returns empty dict"""
        mock_config.get_model_config.return_value = {}
        model = GRUModel()
        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        # The method returns empty dict on error instead of raising
        metrics = model.evaluate(X_test, y_test)
        assert metrics == {}

    @patch("models.gru_model.config")
    def test_evaluate_error_returns_empty_dict(self, mock_config):
        """Test evaluation error returns empty dict"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()
        mock_model.evaluate.side_effect = Exception("Evaluation error")
        model.model = mock_model

        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        metrics = model.evaluate(X_test, y_test)
        assert metrics == {}

    @patch("models.gru_model.config")
    def test_save_model_success(self, mock_config):
        """Test successful model saving"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()
        model.model = mock_model

        result = model.save_model("/tmp/test_model.h5")

        assert result is True
        mock_model.save.assert_called_once_with("/tmp/test_model.h5")

    @patch("models.gru_model.config")
    def test_save_model_no_model(self, mock_config):
        """Test saving when no model exists"""
        mock_config.get_model_config.return_value = {}
        model = GRUModel()

        result = model.save_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.gru_model.config")
    def test_save_model_error(self, mock_config):
        """Test save model error handling"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()
        mock_model.save.side_effect = Exception("Save error")
        model.model = mock_model

        result = model.save_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.gru_model.tf.keras.models.load_model")
    @patch("models.gru_model.config")
    def test_load_model_success(self, mock_config, mock_load):
        """Test successful model loading"""
        mock_config.get_model_config.return_value = {}

        mock_loaded_model = MagicMock()
        mock_load.return_value = mock_loaded_model

        model = GRUModel()
        result = model.load_model("/tmp/test_model.h5")

        assert result is True
        assert model.model == mock_loaded_model

    @patch("models.gru_model.tf.keras.models.load_model")
    @patch("models.gru_model.config")
    def test_load_model_error(self, mock_config, mock_load):
        """Test load model error handling"""
        mock_config.get_model_config.return_value = {}
        mock_load.side_effect = Exception("Load error")

        model = GRUModel()
        result = model.load_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.gru_model.config")
    def test_get_model_summary_with_model(self, mock_config):
        """Test getting model summary when model exists"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_model = MagicMock()

        def mock_summary():
            print("Model: sequential")
            print("Total params: 8000")

        mock_model.summary = mock_summary
        model.model = mock_model

        summary = model.get_model_summary()

        assert summary is not None
        assert "Model: sequential" in summary
        assert "Total params: 8000" in summary

    @patch("models.gru_model.config")
    def test_get_model_summary_without_model(self, mock_config):
        """Test getting model summary when no model exists"""
        mock_config.get_model_config.return_value = {}
        model = GRUModel()

        summary = model.get_model_summary()

        assert summary is None

    @patch("models.gru_model.config")
    def test_get_training_history_with_history(self, mock_config):
        """Test getting training history when it exists"""
        mock_config.get_model_config.return_value = {}

        model = GRUModel()
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5, 0.4], "accuracy": [0.8, 0.85]}
        model.history = mock_history

        history = model.get_training_history()

        assert history is not None
        assert history == {"loss": [0.5, 0.4], "accuracy": [0.8, 0.85]}

    @patch("models.gru_model.config")
    def test_get_training_history_without_history(self, mock_config):
        """Test getting training history when it doesn't exist"""
        mock_config.get_model_config.return_value = {}
        model = GRUModel()

        history = model.get_training_history()

        assert history is None


class TestTransformerModel:
    """Comprehensive tests for Transformer model - targeting >90% coverage"""

    def test_init(self):
        """Test Transformer model initialization"""
        with patch("models.transformer_model.config") as mock_config:
            mock_config.get_model_config.return_value = {
                "hidden_size": 64,
                "dropout": 0.2,
            }
            model = TransformerModel()
            assert model.config is not None
            assert model.model is None
            assert model.history is None

    @patch("models.transformer_model.config")
    def test_transformer_encoder(self, mock_config):
        """Test transformer encoder block"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()

        # Create mock input with shape attribute
        mock_input = MagicMock()
        mock_input.shape = [None, 60, 64]

        # This will call the transformer_encoder method with mocked layers
        result = model.transformer_encoder(
            mock_input, head_size=64, num_heads=4, ff_dim=128, dropout=0.2
        )

        assert result is not None

    @patch("models.transformer_model.Model")
    @patch("models.transformer_model.Adam")
    @patch("models.transformer_model.config")
    def test_build_model_success(self, mock_config, mock_adam, mock_model_class):
        """Test successful Transformer model building"""
        mock_config.get_model_config.return_value = {
            "hidden_size": 64,
            "dropout": 0.2,
            "learning_rate": 0.001,
            "num_layers": 2,
        }

        mock_model = MagicMock()
        mock_model.count_params.return_value = 15000
        mock_model_class.return_value = mock_model

        model = TransformerModel()
        result = model.build_model((60, 15))

        assert result is not None
        assert model.model is not None
        mock_model.compile.assert_called_once()

    @patch("models.transformer_model.Model")
    @patch("models.transformer_model.config")
    def test_build_model_error(self, mock_config, mock_model_class):
        """Test build model with error"""
        mock_config.get_model_config.return_value = {"hidden_size": 64, "dropout": 0.2}
        mock_model_class.side_effect = Exception("Build error")

        model = TransformerModel()
        with pytest.raises(Exception, match="Build error"):
            model.build_model((60, 15))

    @patch("models.transformer_model.EarlyStopping")
    @patch("models.transformer_model.ReduceLROnPlateau")
    @patch("models.transformer_model.ModelCheckpoint")
    @patch("models.transformer_model.config")
    def test_train_with_validation_and_save_path(
        self, mock_config, mock_checkpoint, mock_reduce_lr, mock_early_stopping
    ):
        """Test training with validation data and save path"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_history = MagicMock()
        mock_history.history = {
            "loss": [0.5, 0.4, 0.3],
            "accuracy": [0.8, 0.85, 0.9],
            "val_loss": [0.6, 0.5, 0.45],
            "val_accuracy": [0.75, 0.8, 0.82],
        }
        mock_model.fit.return_value = mock_history
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)
        X_val = np.random.rand(20, 60, 15)
        y_val = np.random.rand(20, 1)

        results = model.train(X_train, y_train, X_val, y_val, save_path="/tmp/model.h5")

        assert results is not None
        assert results["final_loss"] == 0.3
        assert results["final_accuracy"] == 0.9

    @patch("models.transformer_model.EarlyStopping")
    @patch("models.transformer_model.ReduceLROnPlateau")
    @patch("models.transformer_model.config")
    def test_train_without_validation(
        self, mock_config, mock_reduce_lr, mock_early_stopping
    ):
        """Test training without validation data"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5, 0.4], "accuracy": [0.8, 0.85]}
        mock_model.fit.return_value = mock_history
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        results = model.train(X_train, y_train)

        assert results is not None
        assert results["best_val_loss"] == float("inf")

    @patch("models.transformer_model.Model")
    @patch("models.transformer_model.Adam")
    @patch("models.transformer_model.EarlyStopping")
    @patch("models.transformer_model.ReduceLROnPlateau")
    @patch("models.transformer_model.config")
    def test_train_builds_model_if_none(
        self,
        mock_config,
        mock_reduce_lr,
        mock_early_stopping,
        mock_adam,
        mock_model_class,
    ):
        """Test that training builds model if not exists"""
        mock_config.get_model_config.return_value = {
            "epochs": 10,
            "batch_size": 32,
            "hidden_size": 64,
            "dropout": 0.2,
            "learning_rate": 0.001,
            "num_layers": 2,
        }

        mock_model = MagicMock()
        mock_model.count_params.return_value = 15000
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5], "accuracy": [0.8]}
        mock_model.fit.return_value = mock_history
        mock_model_class.return_value = mock_model

        model = TransformerModel()
        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        results = model.train(X_train, y_train)

        assert results is not None

    @patch("models.transformer_model.config")
    def test_train_error(self, mock_config):
        """Test training error handling"""
        mock_config.get_model_config.return_value = {"epochs": 10, "batch_size": 32}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_model.fit.side_effect = Exception("Training error")
        model.model = mock_model

        X_train = np.random.rand(100, 60, 15)
        y_train = np.random.rand(100, 1)

        with pytest.raises(Exception, match="Training error"):
            model.train(X_train, y_train)

    @patch("models.transformer_model.config")
    def test_predict_success(self, mock_config):
        """Test successful prediction"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_model.predict.return_value = np.array([[0.7], [0.8]])
        model.model = mock_model

        X_test = np.random.rand(2, 60, 15)
        predictions = model.predict(X_test)

        assert predictions is not None
        assert len(predictions) == 2

    @patch("models.transformer_model.config")
    def test_predict_without_model(self, mock_config):
        """Test prediction without trained model"""
        mock_config.get_model_config.return_value = {}
        model = TransformerModel()
        X_test = np.random.rand(1, 60, 15)

        with pytest.raises(ValueError, match="Model not trained or loaded"):
            model.predict(X_test)

    @patch("models.transformer_model.config")
    def test_predict_error(self, mock_config):
        """Test prediction error handling"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_model.predict.side_effect = Exception("Prediction error")
        model.model = mock_model

        X_test = np.random.rand(1, 60, 15)

        with pytest.raises(Exception, match="Prediction error"):
            model.predict(X_test)

    @patch("models.transformer_model.config")
    def test_predict_single_2d_input(self, mock_config):
        """Test single prediction with 2D input"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_model.predict.return_value = np.array([[0.75]])
        model.model = mock_model

        X_test = np.random.rand(60, 15)
        prediction = model.predict_single(X_test)

        assert isinstance(prediction, float)
        assert prediction == 0.75

    @patch("models.transformer_model.config")
    def test_predict_single_error_returns_neutral(self, mock_config):
        """Test that predict_single returns 0.5 on error"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_model.predict.side_effect = Exception("Prediction error")
        model.model = mock_model

        X_test = np.random.rand(60, 15)
        prediction = model.predict_single(X_test)

        assert prediction == 0.5

    @patch("models.transformer_model.config")
    def test_evaluate_success(self, mock_config):
        """Test successful model evaluation"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_model.evaluate.return_value = [0.3, 0.9, 0.85, 0.88]
        mock_model.metrics_names = ["loss", "accuracy", "precision", "recall"]
        model.model = mock_model

        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        metrics = model.evaluate(X_test, y_test)

        assert metrics is not None
        assert isinstance(metrics, dict)

    @patch("models.transformer_model.config")
    def test_evaluate_without_model(self, mock_config):
        """Test evaluation without trained model returns empty dict"""
        mock_config.get_model_config.return_value = {}
        model = TransformerModel()
        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        # The method returns empty dict on error instead of raising
        metrics = model.evaluate(X_test, y_test)
        assert metrics == {}

    @patch("models.transformer_model.config")
    def test_evaluate_error_returns_empty_dict(self, mock_config):
        """Test evaluation error returns empty dict"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_model.evaluate.side_effect = Exception("Evaluation error")
        model.model = mock_model

        X_test = np.random.rand(20, 60, 15)
        y_test = np.random.rand(20, 1)

        metrics = model.evaluate(X_test, y_test)
        assert metrics == {}

    @patch("models.transformer_model.config")
    def test_save_model_success(self, mock_config):
        """Test successful model saving"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()
        model.model = mock_model

        result = model.save_model("/tmp/test_model.h5")

        assert result is True

    @patch("models.transformer_model.config")
    def test_save_model_no_model(self, mock_config):
        """Test saving when no model exists"""
        mock_config.get_model_config.return_value = {}
        model = TransformerModel()

        result = model.save_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.transformer_model.config")
    def test_save_model_error(self, mock_config):
        """Test save model error handling"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()
        mock_model.save.side_effect = Exception("Save error")
        model.model = mock_model

        result = model.save_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.transformer_model.tf.keras.models.load_model")
    @patch("models.transformer_model.config")
    def test_load_model_success(self, mock_config, mock_load):
        """Test successful model loading"""
        mock_config.get_model_config.return_value = {}

        mock_loaded_model = MagicMock()
        mock_load.return_value = mock_loaded_model

        model = TransformerModel()
        result = model.load_model("/tmp/test_model.h5")

        assert result is True
        assert model.model == mock_loaded_model

    @patch("models.transformer_model.tf.keras.models.load_model")
    @patch("models.transformer_model.config")
    def test_load_model_error(self, mock_config, mock_load):
        """Test load model error handling"""
        mock_config.get_model_config.return_value = {}
        mock_load.side_effect = Exception("Load error")

        model = TransformerModel()
        result = model.load_model("/tmp/test_model.h5")

        assert result is False

    @patch("models.transformer_model.config")
    def test_get_model_summary_with_model(self, mock_config):
        """Test getting model summary when model exists"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_model = MagicMock()

        def mock_summary():
            print("Model: functional")
            print("Total params: 15000")

        mock_model.summary = mock_summary
        model.model = mock_model

        summary = model.get_model_summary()

        assert summary is not None
        assert "Model: functional" in summary

    @patch("models.transformer_model.config")
    def test_get_model_summary_without_model(self, mock_config):
        """Test getting model summary when no model exists"""
        mock_config.get_model_config.return_value = {}
        model = TransformerModel()

        summary = model.get_model_summary()

        assert summary is None

    @patch("models.transformer_model.config")
    def test_get_training_history_with_history(self, mock_config):
        """Test getting training history when it exists"""
        mock_config.get_model_config.return_value = {}

        model = TransformerModel()
        mock_history = MagicMock()
        mock_history.history = {"loss": [0.5, 0.4], "accuracy": [0.8, 0.85]}
        model.history = mock_history

        history = model.get_training_history()

        assert history is not None

    @patch("models.transformer_model.config")
    def test_get_training_history_without_history(self, mock_config):
        """Test getting training history when it doesn't exist"""
        mock_config.get_model_config.return_value = {}
        model = TransformerModel()

        history = model.get_training_history()

        assert history is None


class TestModelManager:
    """Comprehensive tests for ModelManager - targeting >90% coverage"""

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_init(self, mock_config, mock_fe, mock_ensure_dir):
        """Test ModelManager initialization"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        assert manager.current_model is None
        assert manager.model_type == "lstm"
        mock_ensure_dir.assert_called_once()

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.LSTMModel")
    @patch("models.model_manager.config")
    def test_create_model_lstm(self, mock_config, mock_lstm, mock_fe, mock_ensure_dir):
        """Test creating LSTM model"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()
        result = manager.create_model("lstm")

        assert result is not None
        assert manager.model_type == "lstm"
        mock_lstm.assert_called_once()

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.GRUModel")
    @patch("models.model_manager.config")
    def test_create_model_gru(self, mock_config, mock_gru, mock_fe, mock_ensure_dir):
        """Test creating GRU model"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()
        result = manager.create_model("gru")

        assert result is not None
        assert manager.model_type == "gru"
        mock_gru.assert_called_once()

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.TransformerModel")
    @patch("models.model_manager.config")
    def test_create_model_transformer(
        self, mock_config, mock_transformer, mock_fe, mock_ensure_dir
    ):
        """Test creating Transformer model"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()
        result = manager.create_model("transformer")

        assert result is not None
        assert manager.model_type == "transformer"
        mock_transformer.assert_called_once()

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_create_model_invalid_type(self, mock_config, mock_fe, mock_ensure_dir):
        """Test creating model with invalid type"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        with pytest.raises(ValueError, match="Unsupported model type"):
            manager.create_model("invalid")

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_create_model_error(self, mock_config, mock_fe, mock_ensure_dir):
        """Test create model error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        with patch(
            "models.model_manager.LSTMModel",
            side_effect=Exception("Model creation error"),
        ):
            manager = ModelManager()

            with pytest.raises(Exception, match="Model creation error"):
                manager.create_model("lstm")

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_train_model_success(
        self, mock_config, mock_fe_class, mock_ensure_dir, mock_timestamp, mock_datetime
    ):
        """Test successful model training"""
        mock_config.get_model_config.return_value = {
            "type": "lstm",
            "validation_split": 0.2,
            "sequence_length": 60,
        }
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"
        mock_datetime.now.return_value.strftime.return_value = "20240101_000000"

        # Mock feature engineer
        mock_fe = MagicMock()
        mock_fe.prepare_features.return_value = pd.DataFrame()
        mock_fe.create_sequences.return_value = (
            np.random.rand(100, 60, 15),
            np.random.rand(100, 1),
        )
        mock_fe.scale_features.return_value = np.random.rand(100, 60, 15)
        mock_fe.get_features_count.return_value = 15
        mock_fe_class.return_value = mock_fe

        manager = ModelManager()

        # Mock current model
        mock_model = MagicMock()
        mock_model.train.return_value = {
            "final_loss": 0.3,
            "final_accuracy": 0.9,
            "epochs_trained": 10,
        }
        manager.current_model = mock_model

        df = pd.DataFrame({"close": np.random.rand(200)})
        results = manager.train_model(df)

        assert results is not None
        assert "final_loss" in results
        mock_model.train.assert_called_once()

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_train_model_no_sequences(
        self, mock_config, mock_fe_class, mock_ensure_dir
    ):
        """Test training with no sequences created"""
        mock_config.get_model_config.return_value = {
            "type": "lstm",
            "validation_split": 0.2,
        }
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        # Mock feature engineer to return empty sequences
        mock_fe = MagicMock()
        mock_fe.prepare_features.return_value = pd.DataFrame()
        mock_fe.create_sequences.return_value = (np.array([]), np.array([]))
        mock_fe_class.return_value = mock_fe

        manager = ModelManager()
        df = pd.DataFrame({"close": np.random.rand(10)})

        with pytest.raises(ValueError, match="No sequences created from data"):
            manager.train_model(df)

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.LSTMModel")
    @patch("models.model_manager.config")
    def test_train_model_creates_model_if_none(
        self,
        mock_config,
        mock_lstm,
        mock_fe_class,
        mock_ensure_dir,
        mock_timestamp,
        mock_datetime,
    ):
        """Test that training creates model if none exists"""
        mock_config.get_model_config.return_value = {
            "type": "lstm",
            "validation_split": 0.2,
            "sequence_length": 60,
        }
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"
        mock_datetime.now.return_value.strftime.return_value = "20240101_000000"

        # Mock feature engineer
        mock_fe = MagicMock()
        mock_fe.prepare_features.return_value = pd.DataFrame()
        mock_fe.create_sequences.return_value = (
            np.random.rand(100, 60, 15),
            np.random.rand(100, 1),
        )
        mock_fe.scale_features.return_value = np.random.rand(100, 60, 15)
        mock_fe.get_features_count.return_value = 15
        mock_fe_class.return_value = mock_fe

        # Mock model
        mock_model = MagicMock()
        mock_model.train.return_value = {"final_loss": 0.3}
        mock_lstm.return_value = mock_model

        manager = ModelManager()
        df = pd.DataFrame({"close": np.random.rand(200)})

        results = manager.train_model(df)

        assert results is not None
        mock_lstm.assert_called()

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_train_model_error(self, mock_config, mock_fe_class, mock_ensure_dir):
        """Test training error handling"""
        mock_config.get_model_config.return_value = {
            "type": "lstm",
            "validation_split": 0.2,
        }
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        # Mock feature engineer to raise error
        mock_fe = MagicMock()
        mock_fe.prepare_features.side_effect = Exception("Feature preparation error")
        mock_fe_class.return_value = mock_fe

        manager = ModelManager()
        df = pd.DataFrame({"close": np.random.rand(100)})

        with pytest.raises(Exception, match="Feature preparation error"):
            manager.train_model(df)

    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_predict_success(
        self, mock_config, mock_fe_class, mock_ensure_dir, mock_timestamp
    ):
        """Test successful prediction"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {
            "long_threshold": 0.6,
            "short_threshold": 0.4,
        }
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"

        # Mock feature engineer
        mock_fe = MagicMock()
        mock_fe.prepare_for_inference.return_value = np.random.rand(1, 60, 15)
        mock_fe_class.return_value = mock_fe

        manager = ModelManager()

        # Mock model
        mock_model = MagicMock()
        mock_model.predict_single.return_value = 0.7  # Long signal
        manager.current_model = mock_model
        manager.model_type = "lstm"

        df = pd.DataFrame({"close": np.random.rand(100)})
        result = manager.predict(df)

        assert result is not None
        assert result["signal"] == "long"
        assert result["probability"] == 0.7
        assert result["model_type"] == "lstm"

    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_predict_short_signal(
        self, mock_config, mock_fe_class, mock_ensure_dir, mock_timestamp
    ):
        """Test prediction with short signal"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {
            "long_threshold": 0.6,
            "short_threshold": 0.4,
        }
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"

        mock_fe = MagicMock()
        mock_fe.prepare_for_inference.return_value = np.random.rand(1, 60, 15)
        mock_fe_class.return_value = mock_fe

        manager = ModelManager()
        mock_model = MagicMock()
        mock_model.predict_single.return_value = 0.3  # Short signal
        manager.current_model = mock_model

        df = pd.DataFrame({"close": np.random.rand(100)})
        result = manager.predict(df)

        assert result["signal"] == "short"

    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_predict_neutral_signal(
        self, mock_config, mock_fe_class, mock_ensure_dir, mock_timestamp
    ):
        """Test prediction with neutral signal"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {
            "long_threshold": 0.6,
            "short_threshold": 0.4,
        }
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"

        mock_fe = MagicMock()
        mock_fe.prepare_for_inference.return_value = np.random.rand(1, 60, 15)
        mock_fe_class.return_value = mock_fe

        manager = ModelManager()
        mock_model = MagicMock()
        mock_model.predict_single.return_value = 0.5  # Neutral signal
        manager.current_model = mock_model

        df = pd.DataFrame({"close": np.random.rand(100)})
        result = manager.predict(df)

        assert result["signal"] == "neutral"

    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_predict_no_model(
        self, mock_config, mock_fe_class, mock_ensure_dir, mock_timestamp
    ):
        """Test prediction without model"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"

        manager = ModelManager()
        df = pd.DataFrame({"close": np.random.rand(100)})

        result = manager.predict(df)

        assert result["signal"] == "neutral"
        assert "error" in result

    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_predict_prepare_inference_fails(
        self, mock_config, mock_fe_class, mock_ensure_dir, mock_timestamp
    ):
        """Test prediction when prepare_for_inference returns None"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"

        mock_fe = MagicMock()
        mock_fe.prepare_for_inference.return_value = None
        mock_fe_class.return_value = mock_fe

        manager = ModelManager()
        manager.current_model = MagicMock()

        df = pd.DataFrame({"close": np.random.rand(100)})
        result = manager.predict(df)

        assert result["signal"] == "neutral"
        assert "error" in result

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_determine_signal_long(self, mock_config, mock_fe, mock_ensure_dir):
        """Test determine signal for long"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {
            "long_threshold": 0.6,
            "short_threshold": 0.4,
        }

        manager = ModelManager()
        signal = manager._determine_signal(0.7)

        assert signal == "long"

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_determine_signal_short(self, mock_config, mock_fe, mock_ensure_dir):
        """Test determine signal for short"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {
            "long_threshold": 0.6,
            "short_threshold": 0.4,
        }

        manager = ModelManager()
        signal = manager._determine_signal(0.3)

        assert signal == "short"

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_determine_signal_neutral(self, mock_config, mock_fe, mock_ensure_dir):
        """Test determine signal for neutral"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {
            "long_threshold": 0.6,
            "short_threshold": 0.4,
        }

        manager = ModelManager()
        signal = manager._determine_signal(0.5)

        assert signal == "neutral"

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_calculate_confidence(self, mock_config, mock_fe, mock_ensure_dir):
        """Test confidence calculation"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        # Test extreme confidence
        conf1 = manager._calculate_confidence(0.0)
        assert conf1 == 100.0

        conf2 = manager._calculate_confidence(1.0)
        assert conf2 == 100.0

        # Test neutral confidence
        conf3 = manager._calculate_confidence(0.5)
        assert conf3 == 0.0

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_save_model_success(
        self, mock_config, mock_fe, mock_ensure_dir, mock_timestamp, mock_datetime
    ):
        """Test successful model saving"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"
        mock_datetime.now.return_value.strftime.return_value = "20240101_000000"

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.save_model.return_value = True
        manager.current_model = mock_model

        with patch("models.model_manager.joblib.dump"):
            result = manager.save_model()

        assert result is True

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_save_model_no_model(self, mock_config, mock_fe, mock_ensure_dir):
        """Test saving when no model exists"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        result = manager.save_model()

        assert result is False

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_save_model_error(
        self, mock_config, mock_fe, mock_ensure_dir, mock_timestamp, mock_datetime
    ):
        """Test save model error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"
        mock_datetime.now.return_value.strftime.return_value = "20240101_000000"

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.save_model.side_effect = Exception("Save error")
        manager.current_model = mock_model

        result = manager.save_model()

        assert result is False

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.get_current_timestamp")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_save_model_returns_false(
        self, mock_config, mock_fe, mock_ensure_dir, mock_timestamp, mock_datetime
    ):
        """Test save model when save_model returns False"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_timestamp.return_value = "2024-01-01T00:00:00Z"
        mock_datetime.now.return_value.strftime.return_value = "20240101_000000"

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.save_model.return_value = False  # Returns False instead of True
        manager.current_model = mock_model

        with patch("models.model_manager.joblib.dump"):
            result = manager.save_model()

        assert result is False

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.LSTMModel")
    @patch("models.model_manager.config")
    def test_load_model_success(
        self, mock_config, mock_lstm, mock_fe, mock_ensure_dir, mock_exists
    ):
        """Test successful model loading"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = True

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.load_model.return_value = True
        mock_lstm.return_value = mock_model

        with patch(
            "models.model_manager.joblib.load", return_value={"model_type": "lstm"}
        ):
            result = manager.load_model("/tmp/lstm_model_20240101_000000.h5")

        assert result is True

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.LSTMModel")
    @patch("models.model_manager.config")
    def test_load_model_without_metadata(
        self, mock_config, mock_lstm, mock_fe, mock_ensure_dir, mock_exists
    ):
        """Test successful model loading when metadata loading fails"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        # First call for model file, second call for metadata file (not found)
        mock_exists.side_effect = [True, False]

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.load_model.return_value = True
        mock_lstm.return_value = mock_model

        result = manager.load_model("/tmp/lstm_model_20240101_000000.h5")

        assert result is True

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_load_model_file_not_found(
        self, mock_config, mock_fe, mock_ensure_dir, mock_exists
    ):
        """Test loading model when file doesn't exist"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = False

        manager = ModelManager()

        result = manager.load_model("/tmp/nonexistent.h5")

        assert result is False

    @patch("models.model_manager.os.listdir")
    @patch("models.model_manager.os.path.getmtime")
    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.LSTMModel")
    @patch("models.model_manager.config")
    def test_load_model_finds_latest(
        self,
        mock_config,
        mock_lstm,
        mock_fe,
        mock_ensure_dir,
        mock_exists,
        mock_getmtime,
        mock_listdir,
    ):
        """Test loading latest model when no path specified"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = True
        mock_listdir.return_value = ["model1.h5", "model2.h5"]
        mock_getmtime.side_effect = [100, 200]

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.load_model.return_value = True
        mock_lstm.return_value = mock_model

        with patch(
            "models.model_manager.joblib.load", return_value={"model_type": "lstm"}
        ):
            result = manager.load_model()

        assert result is True

    @patch("models.model_manager.os.listdir")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_load_model_no_models_found(
        self, mock_config, mock_fe, mock_ensure_dir, mock_listdir
    ):
        """Test loading when no models exist"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_listdir.return_value = []

        manager = ModelManager()

        result = manager.load_model()

        assert result is False

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.LSTMModel")
    @patch("models.model_manager.config")
    def test_load_model_error(
        self, mock_config, mock_lstm, mock_fe, mock_ensure_dir, mock_exists
    ):
        """Test load model error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = True

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.load_model.side_effect = Exception("Load error")
        mock_lstm.return_value = mock_model

        with patch(
            "models.model_manager.joblib.load", return_value={"model_type": "lstm"}
        ):
            result = manager.load_model("/tmp/model.h5")

        assert result is False

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.LSTMModel")
    @patch("models.model_manager.config")
    def test_load_model_returns_false(
        self, mock_config, mock_lstm, mock_fe, mock_ensure_dir, mock_exists
    ):
        """Test load model when load_model returns False"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = True

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.load_model.return_value = False  # Returns False instead of True
        mock_lstm.return_value = mock_model

        with patch(
            "models.model_manager.joblib.load", return_value={"model_type": "lstm"}
        ):
            result = manager.load_model("/tmp/model.h5")

        assert result is False

    @patch("models.model_manager.os.listdir")
    @patch("models.model_manager.os.path.getmtime")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_find_latest_model(
        self, mock_config, mock_fe, mock_ensure_dir, mock_getmtime, mock_listdir
    ):
        """Test finding latest model"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_listdir.return_value = ["model1.h5", "model2.h5", "other.txt"]
        mock_getmtime.side_effect = [100, 200]

        manager = ModelManager()

        result = manager._find_latest_model()

        assert result is not None
        assert "model2.h5" in result

    @patch("models.model_manager.os.listdir")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_find_latest_model_no_models(
        self, mock_config, mock_fe, mock_ensure_dir, mock_listdir
    ):
        """Test finding latest model when none exist"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_listdir.return_value = []

        manager = ModelManager()

        result = manager._find_latest_model()

        assert result is None

    @patch("models.model_manager.os.listdir")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_find_latest_model_error(
        self, mock_config, mock_fe, mock_ensure_dir, mock_listdir
    ):
        """Test find latest model error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_listdir.side_effect = Exception("Directory error")

        manager = ModelManager()

        result = manager._find_latest_model()

        assert result is None

    @patch("models.model_manager.joblib.dump")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_save_feature_engineer_success(
        self, mock_config, mock_fe, mock_ensure_dir, mock_dump
    ):
        """Test saving feature engineer"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        result = manager._save_feature_engineer("20240101_000000")

        assert result is True
        mock_dump.assert_called_once()

    @patch("models.model_manager.joblib.dump")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_save_feature_engineer_error(
        self, mock_config, mock_fe, mock_ensure_dir, mock_dump
    ):
        """Test save feature engineer error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_dump.side_effect = Exception("Save error")

        manager = ModelManager()

        result = manager._save_feature_engineer("20240101_000000")

        assert result is False

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.joblib.load")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_load_feature_engineer_success(
        self, mock_config, mock_fe_class, mock_ensure_dir, mock_load, mock_exists
    ):
        """Test loading feature engineer"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = True
        mock_load.return_value = MagicMock()

        manager = ModelManager()

        result = manager._load_feature_engineer("20240101_000000")

        assert result is True

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_load_feature_engineer_not_found(
        self, mock_config, mock_fe, mock_ensure_dir, mock_exists
    ):
        """Test loading feature engineer when file not found"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = False

        manager = ModelManager()

        result = manager._load_feature_engineer("20240101_000000")

        assert result is False

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.joblib.load")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_load_feature_engineer_error(
        self, mock_config, mock_fe, mock_ensure_dir, mock_load, mock_exists
    ):
        """Test load feature engineer error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = True
        mock_load.side_effect = Exception("Load error")

        manager = ModelManager()

        result = manager._load_feature_engineer("20240101_000000")

        assert result is False

    @patch("models.model_manager.joblib.dump")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_save_metadata_success(
        self, mock_config, mock_fe, mock_ensure_dir, mock_dump
    ):
        """Test saving metadata"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        result = manager._save_metadata("20240101_000000")

        assert result is True

    @patch("models.model_manager.joblib.dump")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_save_metadata_error(
        self, mock_config, mock_fe, mock_ensure_dir, mock_dump
    ):
        """Test save metadata error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_dump.side_effect = Exception("Save error")

        manager = ModelManager()

        result = manager._save_metadata("20240101_000000")

        assert result is False

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.joblib.load")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_load_metadata_success(
        self, mock_config, mock_fe, mock_ensure_dir, mock_load, mock_exists
    ):
        """Test loading metadata"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = True
        mock_load.return_value = {"model_type": "lstm"}

        manager = ModelManager()

        result = manager._load_metadata("20240101_000000")

        assert result is True

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_load_metadata_not_found(
        self, mock_config, mock_fe, mock_ensure_dir, mock_exists
    ):
        """Test loading metadata when file not found"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = False

        manager = ModelManager()

        result = manager._load_metadata("20240101_000000")

        assert result is False

    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.joblib.load")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_load_metadata_error(
        self, mock_config, mock_fe, mock_ensure_dir, mock_load, mock_exists
    ):
        """Test load metadata error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_exists.return_value = True
        mock_load.side_effect = Exception("Load error")

        manager = ModelManager()

        result = manager._load_metadata("20240101_000000")

        assert result is False

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_should_retrain_no_metadata(
        self, mock_config, mock_fe, mock_ensure_dir, mock_datetime
    ):
        """Test should_retrain with no metadata"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        result = manager.should_retrain()

        assert result is True

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_should_retrain_no_timestamp(
        self, mock_config, mock_fe, mock_ensure_dir, mock_datetime
    ):
        """Test should_retrain with no timestamp in metadata"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()
        manager.model_metadata = {"model_type": "lstm"}

        result = manager.should_retrain()

        assert result is True

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_should_retrain_interval_passed(
        self, mock_config, mock_fe, mock_ensure_dir, mock_datetime_module
    ):
        """Test should_retrain when interval has passed"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/",
            "retrain_interval_hours": 24,
        }
        mock_config.get_trading_config.return_value = {}

        from datetime import datetime, timedelta

        old_time = datetime.now() - timedelta(hours=25)
        mock_datetime_module.now.return_value = datetime.now()
        mock_datetime_module.fromisoformat.return_value = old_time

        manager = ModelManager()
        manager.model_metadata = {"trained_timestamp": old_time.isoformat() + "Z"}

        result = manager.should_retrain()

        assert result is True

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_should_retrain_interval_not_passed(
        self, mock_config, mock_fe, mock_ensure_dir, mock_datetime_module
    ):
        """Test should_retrain when interval has not passed"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/",
            "retrain_interval_hours": 24,
        }
        mock_config.get_trading_config.return_value = {}

        from datetime import datetime, timedelta

        recent_time = datetime.now() - timedelta(hours=1)
        future_time = datetime.now() + timedelta(hours=1)
        mock_datetime_module.now.return_value = datetime.now()
        mock_datetime_module.fromisoformat.return_value = recent_time

        manager = ModelManager()
        manager.model_metadata = {"trained_timestamp": recent_time.isoformat() + "Z"}

        result = manager.should_retrain()

        assert result is False

    @patch("models.model_manager.datetime")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_should_retrain_error(
        self, mock_config, mock_fe, mock_ensure_dir, mock_datetime
    ):
        """Test should_retrain error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_datetime.fromisoformat.side_effect = Exception("Parse error")

        manager = ModelManager()
        manager.model_metadata = {"trained_timestamp": "invalid"}

        result = manager.should_retrain()

        assert result is False

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_get_model_info_no_model(self, mock_config, mock_fe, mock_ensure_dir):
        """Test get_model_info without model"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        info = manager.get_model_info()

        assert info["model_type"] == "lstm"
        assert info["model_loaded"] is False

    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_get_model_info_with_model(self, mock_config, mock_fe, mock_ensure_dir):
        """Test get_model_info with model"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}

        manager = ModelManager()

        mock_model = MagicMock()
        mock_model.get_model_summary.return_value = "Model summary"
        mock_model.get_training_history.return_value = {"loss": [0.5]}
        manager.current_model = mock_model

        info = manager.get_model_info()

        assert info["model_loaded"] is True
        assert "summary" in info
        assert "training_history" in info

    @patch("models.model_manager.os.listdir")
    @patch("models.model_manager.os.path.getmtime")
    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.os.remove")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_cleanup_old_models(
        self,
        mock_config,
        mock_fe,
        mock_ensure_dir,
        mock_remove,
        mock_exists,
        mock_getmtime,
        mock_listdir,
    ):
        """Test cleanup of old models"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/",
            "backup_count": 2,
        }
        mock_config.get_trading_config.return_value = {}
        mock_listdir.return_value = ["model1.h5", "model2.h5", "model3.h5"]
        mock_getmtime.side_effect = [100, 200, 300]
        mock_exists.side_effect = [
            True,
            True,
        ]  # For feature engineer and metadata files

        manager = ModelManager()

        deleted_count = manager.cleanup_old_models()

        assert deleted_count == 1
        assert mock_remove.call_count >= 1

    @patch("models.model_manager.os.listdir")
    @patch("models.model_manager.os.path.getmtime")
    @patch("models.model_manager.os.path.exists")
    @patch("models.model_manager.os.remove")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_cleanup_old_models_delete_error(
        self,
        mock_config,
        mock_fe,
        mock_ensure_dir,
        mock_remove,
        mock_exists,
        mock_getmtime,
        mock_listdir,
    ):
        """Test cleanup with deletion error"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/",
            "backup_count": 1,
        }
        mock_config.get_trading_config.return_value = {}
        mock_listdir.return_value = ["model1.h5", "model2.h5"]
        mock_getmtime.side_effect = [100, 200]
        mock_remove.side_effect = [Exception("Delete error"), None, None]
        mock_exists.return_value = False

        manager = ModelManager()

        deleted_count = manager.cleanup_old_models()

        # Should still be 0 because the main file deletion failed
        assert deleted_count == 0

    @patch("models.model_manager.os.listdir")
    @patch("models.model_manager.ensure_directory_exists")
    @patch("models.model_manager.FeatureEngineer")
    @patch("models.model_manager.config")
    def test_cleanup_old_models_error(
        self, mock_config, mock_fe, mock_ensure_dir, mock_listdir
    ):
        """Test cleanup error handling"""
        mock_config.get_model_config.return_value = {"type": "lstm"}
        mock_config.get_model_management_config.return_value = {
            "model_save_path": "./models/saved/"
        }
        mock_config.get_trading_config.return_value = {}
        mock_listdir.side_effect = Exception("Directory error")

        manager = ModelManager()

        deleted_count = manager.cleanup_old_models()

        assert deleted_count == 0
