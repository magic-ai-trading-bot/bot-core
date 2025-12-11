"""
Tests for app.websocket.manager module to boost coverage.
"""

from datetime import datetime
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from app.websocket.manager import WebSocketManager


class TestWebSocketManager:
    """Test WebSocketManager class."""

    def test_websocket_manager_initialization(self):
        """Test WebSocketManager initializes with empty connections."""
        manager = WebSocketManager()
        assert len(manager.active_connections) == 0
        assert isinstance(manager.active_connections, set)

    @pytest.mark.asyncio
    async def test_connect_adds_connection(self):
        """Test connect method adds WebSocket to active connections."""
        manager = WebSocketManager()

        # Create mock WebSocket
        mock_websocket = AsyncMock()
        mock_websocket.accept = AsyncMock()
        mock_websocket.send_json = AsyncMock()

        # Connect
        await manager.connect(mock_websocket)

        # Verify connection was added
        assert len(manager.active_connections) == 1
        assert mock_websocket in manager.active_connections

        # Verify accept was called
        mock_websocket.accept.assert_called_once()

        # Verify welcome message was sent
        mock_websocket.send_json.assert_called_once()
        call_args = mock_websocket.send_json.call_args[0][0]
        assert call_args["type"] == "connection"
        assert call_args["message"] == "Connected to AI Trading Service"
        assert "timestamp" in call_args

    @pytest.mark.asyncio
    async def test_connect_multiple_connections(self):
        """Test connect method handles multiple connections."""
        manager = WebSocketManager()

        # Create multiple mock WebSockets
        mock_ws1 = AsyncMock()
        mock_ws1.accept = AsyncMock()
        mock_ws1.send_json = AsyncMock()

        mock_ws2 = AsyncMock()
        mock_ws2.accept = AsyncMock()
        mock_ws2.send_json = AsyncMock()

        # Connect both
        await manager.connect(mock_ws1)
        await manager.connect(mock_ws2)

        # Verify both connections were added
        assert len(manager.active_connections) == 2
        assert mock_ws1 in manager.active_connections
        assert mock_ws2 in manager.active_connections

    def test_disconnect_removes_connection(self):
        """Test disconnect method removes WebSocket from active connections."""
        manager = WebSocketManager()

        # Add mock WebSocket
        mock_websocket = MagicMock()
        manager.active_connections.add(mock_websocket)

        # Verify it was added
        assert len(manager.active_connections) == 1

        # Disconnect
        manager.disconnect(mock_websocket)

        # Verify it was removed
        assert len(manager.active_connections) == 0
        assert mock_websocket not in manager.active_connections

    def test_disconnect_handles_non_existent_connection(self):
        """Test disconnect handles WebSocket not in active connections."""
        manager = WebSocketManager()

        # Try to disconnect non-existent WebSocket (should not raise error)
        mock_websocket = MagicMock()
        manager.disconnect(mock_websocket)

        # Verify no error and connections remain empty
        assert len(manager.active_connections) == 0

    @pytest.mark.asyncio
    async def test_broadcast_signal_sends_to_all_connections(self):
        """Test broadcast_signal sends message to all active connections."""
        manager = WebSocketManager()

        # Create multiple mock WebSockets
        mock_ws1 = AsyncMock()
        mock_ws1.send_json = AsyncMock()

        mock_ws2 = AsyncMock()
        mock_ws2.send_json = AsyncMock()

        # Add connections
        manager.active_connections.add(mock_ws1)
        manager.active_connections.add(mock_ws2)

        # Broadcast signal
        signal_data = {
            "symbol": "BTCUSDT",
            "signal": "BUY",
            "confidence": 0.85,
        }
        await manager.broadcast_signal(signal_data)

        # Verify both received the message
        mock_ws1.send_json.assert_called_once()
        mock_ws2.send_json.assert_called_once()

        # Verify message structure
        call_args = mock_ws1.send_json.call_args[0][0]
        assert call_args["type"] == "AISignalReceived"
        assert call_args["data"] == signal_data
        assert "timestamp" in call_args

    @pytest.mark.asyncio
    async def test_broadcast_signal_handles_no_connections(self):
        """Test broadcast_signal handles empty connections gracefully."""
        manager = WebSocketManager()

        # Broadcast to empty connections (should not raise error)
        signal_data = {"symbol": "BTCUSDT", "signal": "BUY"}
        await manager.broadcast_signal(signal_data)

        # Verify no error occurred
        assert len(manager.active_connections) == 0

    @pytest.mark.asyncio
    async def test_broadcast_signal_handles_send_failure(self):
        """Test broadcast_signal handles WebSocket send failures."""
        manager = WebSocketManager()

        # Create mock WebSocket that raises exception
        mock_ws_fail = AsyncMock()
        mock_ws_fail.send_json = AsyncMock(side_effect=Exception("Connection lost"))

        # Create mock WebSocket that succeeds
        mock_ws_success = AsyncMock()
        mock_ws_success.send_json = AsyncMock()

        # Add both connections
        manager.active_connections.add(mock_ws_fail)
        manager.active_connections.add(mock_ws_success)

        # Broadcast signal
        signal_data = {"symbol": "BTCUSDT", "signal": "BUY"}
        await manager.broadcast_signal(signal_data)

        # Verify failed connection was removed
        assert mock_ws_fail not in manager.active_connections

        # Verify successful connection remains
        assert mock_ws_success in manager.active_connections
        assert len(manager.active_connections) == 1

    def test_get_connection_count_returns_correct_count(self):
        """Test get_connection_count returns correct number."""
        manager = WebSocketManager()

        # Initially 0
        assert manager.get_connection_count() == 0

        # Add connections
        mock_ws1 = MagicMock()
        mock_ws2 = MagicMock()
        mock_ws3 = MagicMock()

        manager.active_connections.add(mock_ws1)
        assert manager.get_connection_count() == 1

        manager.active_connections.add(mock_ws2)
        assert manager.get_connection_count() == 2

        manager.active_connections.add(mock_ws3)
        assert manager.get_connection_count() == 3

        # Remove one
        manager.active_connections.discard(mock_ws2)
        assert manager.get_connection_count() == 2


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
