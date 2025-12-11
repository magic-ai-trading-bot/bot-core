"""
WebSocket connection manager for real-time AI signal broadcasting.
"""

import logging
from datetime import datetime, timezone
from typing import Any, Dict, Set

from fastapi import WebSocket

logger = logging.getLogger(__name__)


class WebSocketManager:
    """Manages WebSocket connections for real-time AI signal broadcasting."""

    def __init__(self):
        self.active_connections: Set[WebSocket] = set()

    async def connect(self, websocket: WebSocket):
        """Accept new WebSocket connection."""
        await websocket.accept()
        self.active_connections.add(websocket)
        logger.info(
            f"🔗 New WebSocket connection. " f"Total: {len(self.active_connections)}"
        )

        # Send welcome message
        await websocket.send_json(
            {
                "type": "connection",
                "message": "Connected to AI Trading Service",
                "timestamp": datetime.now(timezone.utc).isoformat(),
            }
        )

    def disconnect(self, websocket: WebSocket):
        """Remove WebSocket connection."""
        self.active_connections.discard(websocket)
        logger.info(
            f"🔌 WebSocket disconnected. " f"Remaining: {len(self.active_connections)}"
        )

    async def broadcast_signal(self, signal_data: Dict[str, Any]):
        """Broadcast AI signal to all connected clients."""
        if not self.active_connections:
            return

        message = {
            "type": "AISignalReceived",
            "data": signal_data,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }

        # Send to all connections
        disconnected = []
        for connection in self.active_connections.copy():
            try:
                await connection.send_json(message)
            except Exception as e:
                logger.warning(f"Failed to send to WebSocket: {e}")
                disconnected.append(connection)

        # Clean up disconnected clients
        for conn in disconnected:
            self.active_connections.discard(conn)

        logger.info(
            f"📡 Broadcasted AI signal to " f"{len(self.active_connections)} clients"
        )

    def get_connection_count(self) -> int:
        """Get number of active connections."""
        return len(self.active_connections)
