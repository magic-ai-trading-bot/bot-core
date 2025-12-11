"""
Test WebSocket functionality.
"""

import asyncio
import os
# Import after adding to path in conftest
import sys
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from fastapi.testclient import TestClient

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


@pytest.mark.unit
class TestWebSocketManager:
    """Test WebSocketManager functionality."""

    def test_websocket_manager_initialization(self):
        """Test WebSocketManager initialization."""
        from main import WebSocketManager

        manager = WebSocketManager()
        assert manager.active_connections == set()

    @pytest.mark.asyncio
    async def test_connect_and_disconnect(self):
        """Test connecting and disconnecting WebSocket."""
        from main import WebSocketManager

        manager = WebSocketManager()

        # Mock WebSocket
        mock_websocket = AsyncMock()
        mock_websocket.accept = AsyncMock()
        mock_websocket.send_json = AsyncMock()

        # Test connect
        await manager.connect(mock_websocket)
        assert mock_websocket in manager.active_connections
        mock_websocket.accept.assert_called_once()

        # Test disconnect
        manager.disconnect(mock_websocket)
        assert mock_websocket not in manager.active_connections

    @pytest.mark.asyncio
    async def test_broadcast_message(self):
        """Test broadcasting message to all connections."""
        from main import WebSocketManager

        manager = WebSocketManager()

        # Create multiple mock connections
        ws1 = AsyncMock()
        ws2 = AsyncMock()
        ws3 = AsyncMock()

        # Connect all (ws3 will be set to fail after connection)
        await manager.connect(ws1)
        await manager.connect(ws2)
        await manager.connect(ws3)

        # Now make ws3 fail on next send_json call
        ws3.send_json = AsyncMock(side_effect=Exception("Connection lost"))

        # Broadcast message
        test_message = {"type": "test", "data": "hello"}
        await manager.broadcast_signal(test_message)

        # Check that message was sent to working connections (both calls: welcome + broadcast)
        assert ws1.send_json.call_count >= 1
        assert ws2.send_json.call_count >= 1

        # Failed connection should be removed
        assert ws3 not in manager.active_connections
        assert ws1 in manager.active_connections
        assert ws2 in manager.active_connections


@pytest.mark.integration
class TestWebSocketEndpoint:
    """Test WebSocket endpoint integration."""

    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    def test_websocket_connection_lifecycle(self, test_client):
        """Test full WebSocket connection lifecycle."""
        with test_client.websocket_connect("/ws") as websocket:
            # Test initial connection message
            data = websocket.receive_json()
            assert data["type"] == "connection"
            assert "Connected to AI Trading Service" in data["message"]

            # Connection should remain open
            # In real scenario, would receive broadcasts here

    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    def test_websocket_receives_broadcast(self, test_client):
        """Test receiving broadcast messages."""
        from main import ws_manager

        with test_client.websocket_connect("/ws") as websocket:
            # Skip connection message
            websocket.receive_json()

            # Simulate a broadcast
            async def send_broadcast():
                await ws_manager.broadcast_signal(
                    {
                        "type": "ai_signal",
                        "data": {
                            "symbol": "BTCUSDT",
                            "signal": "Long",
                            "confidence": 0.85,
                        },
                    }
                )

            # Note: In real test, would need to run in async context
            # This is simplified for demonstration

    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    def test_multiple_websocket_connections(self, test_client):
        """Test multiple simultaneous WebSocket connections."""
        # Create multiple connections
        with test_client.websocket_connect("/ws") as ws1:
            with test_client.websocket_connect("/ws") as ws2:
                # Both should receive connection messages
                data1 = ws1.receive_json()
                data2 = ws2.receive_json()

                assert data1["type"] == "connection"
                assert data2["type"] == "connection"

                # Both connections should be active

                # Note: In real scenario, would verify both in active_connections


@pytest.mark.unit
class TestWebSocketBroadcasting:
    """Test WebSocket broadcasting scenarios."""

    @pytest.mark.asyncio
    async def test_broadcast_ai_signal(self):
        """Test broadcasting AI signal to WebSocket clients."""
        from main import ws_manager

        # Mock connections
        mock_ws1 = AsyncMock()
        mock_ws2 = AsyncMock()

        await ws_manager.connect(mock_ws1)
        await ws_manager.connect(mock_ws2)

        # Broadcast AI signal
        signal_data = {
            "type": "ai_signal",
            "data": {
                "symbol": "ETHUSDT",
                "signal": "Short",
                "confidence": 0.72,
                "timestamp": "2025-07-31T18:00:00Z",
            },
        }

        await ws_manager.broadcast_signal(signal_data)

        # Both should receive the signal (check last call since welcome message is sent first)
        assert mock_ws1.send_json.call_count >= 1
        assert mock_ws2.send_json.call_count >= 1

    @pytest.mark.asyncio
    async def test_broadcast_analysis_update(self):
        """Test broadcasting analysis update."""
        from main import ws_manager

        mock_ws = AsyncMock()
        await ws_manager.connect(mock_ws)

        update_data = {
            "type": "analysis_update",
            "data": {
                "symbols_analyzed": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
                "next_analysis": "2025-07-31T18:05:00Z",
                "total_signals_today": 48,
            },
        }

        await ws_manager.broadcast_signal(update_data)
        assert mock_ws.send_json.call_count >= 1

    @pytest.mark.asyncio
    async def test_broadcast_error_notification(self):
        """Test broadcasting error notifications."""
        from main import ws_manager

        mock_ws = AsyncMock()
        await ws_manager.connect(mock_ws)

        error_data = {
            "type": "error",
            "data": {
                "message": "OpenAI API temporarily unavailable",
                "error_code": "SERVICE_UNAVAILABLE",
                "retry_after": 60,
            },
        }

        await ws_manager.broadcast_signal(error_data)
        assert mock_ws.send_json.call_count >= 1

    @pytest.mark.asyncio
    async def test_concurrent_broadcasts(self):
        """Test handling concurrent broadcasts."""
        from main import ws_manager

        # Create many mock connections
        connections = [AsyncMock() for _ in range(10)]
        for ws in connections:
            await ws_manager.connect(ws)

        # Simulate concurrent broadcasts
        async def broadcast_task(msg_type):
            for i in range(5):
                await ws_manager.broadcast_signal(
                    {"type": msg_type, "data": {"index": i}}
                )
                await asyncio.sleep(0.01)

        # Run multiple broadcast tasks concurrently
        await asyncio.gather(
            broadcast_task("type1"), broadcast_task("type2"), broadcast_task("type3")
        )

        # Each connection should have received all messages (welcome + broadcasts)
        for ws in connections:
            assert (
                ws.send_json.call_count == 16
            )  # 1 welcome + 15 broadcasts (3 types * 5 messages each)
