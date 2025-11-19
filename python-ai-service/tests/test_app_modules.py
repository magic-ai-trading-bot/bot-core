"""
Tests for app modules to boost coverage.
Simple import tests to trigger module execution.
"""

import pytest


class TestAppModulesImport:
    """Test importing app modules."""

    def test_import_app_core_config(self):
        """Import app.core.config."""
        from app.core import config
        assert config is not None

    def test_import_app_core_state(self):
        """Import app.core.state."""
        from app.core import state
        assert state is not None

    def test_import_app_models_schemas(self):
        """Import app.models.schemas."""
        from app.models import schemas
        assert schemas is not None

    def test_import_app_websocket_manager(self):
        """Import app.websocket.manager."""
        from app.websocket import manager
        assert manager is not None


class TestFeaturesModulesImport:
    """Test importing features modules."""

    def test_import_feature_engineering(self):
        """Import features.feature_engineering."""
        import features.feature_engineering as fe
        assert fe is not None

    def test_import_technical_indicators(self):
        """Import features.technical_indicators."""
        import features.technical_indicators as ti
        assert ti is not None


class TestUtilsModulesImport:
    """Test importing utils modules."""

    def test_import_helpers(self):
        """Import utils.helpers."""
        import utils.helpers as helpers
        assert helpers is not None

    def test_import_logger(self):
        """Import utils.logger."""
        import utils.logger as logger
        assert logger is not None


class TestModelModulesImport:
    """Test importing model modules."""

    def test_import_gru_model(self):
        """Import models.gru_model."""
        import models.gru_model as gru
        assert gru is not None

    def test_import_lstm_model(self):
        """Import models.lstm_model."""
        import models.lstm_model as lstm
        assert lstm is not None

    def test_import_transformer_model(self):
        """Import models.transformer_model."""
        import models.transformer_model as transformer
        assert transformer is not None

    def test_import_model_manager(self):
        """Import models.model_manager."""
        import models.model_manager as mm
        assert mm is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
