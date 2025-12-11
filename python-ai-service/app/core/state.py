"""
Application state management.
Provides thread-safe state management using FastAPI's app.state.
"""

from datetime import datetime
from typing import Any, Optional, Set

from fastapi import WebSocket
from motor.motor_asyncio import AsyncIOMotorClient


class AppState:
    """Central application state manager."""

    def __init__(self):
        # OpenAI client
        self.openai_client: Optional[Any] = None

        # WebSocket connections
        self.websocket_connections: Set[WebSocket] = set()

        # MongoDB
        self.mongodb_client: Optional[AsyncIOMotorClient] = None
        self.mongodb_db: Optional[Any] = None

        # Rate limiting
        self.last_openai_request_time: Optional[datetime] = None
        self.openai_rate_limit_reset_time: Optional[datetime] = None

        # Cost tracking metrics
        self.metrics = {
            "total_input_tokens": 0,
            "total_output_tokens": 0,
            "total_requests_count": 0,
            "total_cost_usd": 0.0,
        }

        # Service start time
        self.start_time: datetime = datetime.now()

    def update_metrics(
        self, input_tokens: int = 0, output_tokens: int = 0, cost: float = 0.0
    ):
        """Update cost tracking metrics in thread-safe manner."""
        self.metrics["total_input_tokens"] += input_tokens
        self.metrics["total_output_tokens"] += output_tokens
        self.metrics["total_requests_count"] += 1
        self.metrics["total_cost_usd"] += cost

    def get_metrics(self) -> dict:
        """Get current metrics snapshot."""
        uptime = (datetime.now() - self.start_time).total_seconds()
        return {
            **self.metrics,
            "uptime_seconds": uptime,
            "active_websocket_connections": len(self.websocket_connections),
        }

    def reset_metrics(self):
        """Reset cost tracking metrics."""
        self.metrics = {
            "total_input_tokens": 0,
            "total_output_tokens": 0,
            "total_requests_count": 0,
            "total_cost_usd": 0.0,
        }
