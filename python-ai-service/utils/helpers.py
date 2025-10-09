import os
import pandas as pd
from pathlib import Path
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone


def ensure_directory_exists(directory_path: str) -> None:
    """Ensure that a directory exists, create if it doesn't."""
    Path(directory_path).mkdir(parents=True, exist_ok=True)


def validate_ohlcv_data(data: Dict[str, Any]) -> bool:
    """Validate OHLCV data structure and content."""
    required_fields = ["open", "high", "low", "close", "volume", "timestamp"]

    # Check if all required fields are present
    if not all(field in data for field in required_fields):
        return False

    # Check if data has the expected structure
    if not isinstance(data.get("candles"), list):
        return False

    # Check if each candle has the required fields
    for candle in data.get("candles", []):
        if not all(field in candle for field in required_fields):
            return False

        # Validate data types and ranges
        try:
            open_price = float(candle["open"])
            high_price = float(candle["high"])
            low_price = float(candle["low"])
            close_price = float(candle["close"])
            volume = float(candle["volume"])

            # Basic price validation
            if not (
                low_price <= open_price <= high_price
                and low_price <= close_price <= high_price
            ):
                return False

            # Volume should be non-negative
            if volume < 0:
                return False

        except (ValueError, TypeError):
            return False

    return True


def convert_timeframe_to_minutes(timeframe: str) -> int:
    """Convert timeframe string to minutes."""
    timeframe_map = {
        "1m": 1,
        "5m": 5,
        "15m": 15,
        "30m": 30,
        "1h": 60,
        "4h": 240,
        "1d": 1440,
    }
    return timeframe_map.get(timeframe, 1)


def create_dataframe_from_ohlcv(data: Dict[str, Any]) -> Optional[pd.DataFrame]:
    """Convert OHLCV data to pandas DataFrame."""
    try:
        candles = data.get("candles", [])
        if not candles:
            return None

        df_data = []
        for candle in candles:
            df_data.append(
                {
                    "timestamp": pd.to_datetime(candle["timestamp"], unit="ms"),
                    "open": float(candle["open"]),
                    "high": float(candle["high"]),
                    "low": float(candle["low"]),
                    "close": float(candle["close"]),
                    "volume": float(candle["volume"]),
                }
            )

        df = pd.DataFrame(df_data)
        df.set_index("timestamp", inplace=True)
        df.sort_index(inplace=True)

        return df
    except Exception:
        return None


def get_current_timestamp() -> str:
    """Get current timestamp in ISO format."""
    return datetime.now(timezone.utc).isoformat()


def calculate_percentage_change(old_value: float, new_value: float) -> float:
    """Calculate percentage change between two values."""
    if old_value == 0:
        return 0.0
    return ((new_value - old_value) / old_value) * 100


def format_confidence_score(confidence: float) -> float:
    """Format confidence score to 2 decimal places."""
    return round(confidence * 100, 2)
