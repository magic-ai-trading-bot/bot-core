"""
Tests for app.core.state module to boost coverage.
"""

import pytest
from datetime import datetime
from app.core.state import AppState


class TestAppState:
    """Test AppState class functionality."""

    def test_app_state_initialization(self):
        """Test AppState initializes with default values."""
        state = AppState()

        # Check initialization
        assert state.openai_client is None
        assert len(state.websocket_connections) == 0
        assert state.mongodb_client is None
        assert state.mongodb_db is None
        assert state.last_openai_request_time is None
        assert state.openai_rate_limit_reset_time is None

        # Check metrics initialization
        assert state.metrics["total_input_tokens"] == 0
        assert state.metrics["total_output_tokens"] == 0
        assert state.metrics["total_requests_count"] == 0
        assert state.metrics["total_cost_usd"] == 0.0

        # Check start time is set
        assert isinstance(state.start_time, datetime)

    def test_update_metrics(self):
        """Test updating metrics."""
        state = AppState()

        # Update with tokens and cost
        state.update_metrics(input_tokens=100, output_tokens=50, cost=0.05)

        assert state.metrics["total_input_tokens"] == 100
        assert state.metrics["total_output_tokens"] == 50
        assert state.metrics["total_requests_count"] == 1
        assert state.metrics["total_cost_usd"] == 0.05

        # Update again
        state.update_metrics(input_tokens=200, output_tokens=100, cost=0.10)

        assert state.metrics["total_input_tokens"] == 300
        assert state.metrics["total_output_tokens"] == 150
        assert state.metrics["total_requests_count"] == 2
        assert round(state.metrics["total_cost_usd"], 2) == 0.15

    def test_get_metrics(self):
        """Test getting metrics snapshot."""
        state = AppState()

        # Update some metrics
        state.update_metrics(input_tokens=100, output_tokens=50, cost=0.05)

        metrics = state.get_metrics()

        # Check all fields present
        assert "total_input_tokens" in metrics
        assert "total_output_tokens" in metrics
        assert "total_requests_count" in metrics
        assert "total_cost_usd" in metrics
        assert "uptime_seconds" in metrics
        assert "active_websocket_connections" in metrics

        # Check values
        assert metrics["total_input_tokens"] == 100
        assert metrics["total_output_tokens"] == 50
        assert metrics["total_requests_count"] == 1
        assert metrics["total_cost_usd"] == 0.05
        assert metrics["uptime_seconds"] >= 0
        assert metrics["active_websocket_connections"] == 0

    def test_reset_metrics(self):
        """Test resetting metrics."""
        state = AppState()

        # Update metrics
        state.update_metrics(input_tokens=100, output_tokens=50, cost=0.05)

        # Verify metrics are set
        assert state.metrics["total_input_tokens"] == 100
        assert state.metrics["total_cost_usd"] == 0.05

        # Reset
        state.reset_metrics()

        # Verify metrics are reset
        assert state.metrics["total_input_tokens"] == 0
        assert state.metrics["total_output_tokens"] == 0
        assert state.metrics["total_requests_count"] == 0
        assert state.metrics["total_cost_usd"] == 0.0

    def test_websocket_connections_tracking(self):
        """Test WebSocket connections tracking."""
        state = AppState()

        # Initially empty
        assert len(state.websocket_connections) == 0

        # Add mock connections
        mock_ws1 = object()
        mock_ws2 = object()

        state.websocket_connections.add(mock_ws1)
        state.websocket_connections.add(mock_ws2)

        assert len(state.websocket_connections) == 2

        # Check metrics reflect this
        metrics = state.get_metrics()
        assert metrics["active_websocket_connections"] == 2

        # Remove one
        state.websocket_connections.remove(mock_ws1)
        assert len(state.websocket_connections) == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
