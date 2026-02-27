"""
Configuration module for AI Trading Service.
Centralizes all configuration constants and environment variables.
"""

import os
from typing import List

# === API Configuration ===
# Read from environment variable with fallback to 2 seconds
# xAI Grok has high rate limits
# Delay between requests in seconds
GROK_REQUEST_DELAY = int(os.getenv("GROK_REQUEST_DELAY", os.getenv("OPENAI_REQUEST_DELAY", "2")))

# === Cost Monitoring (Grok pricing) ===
GROK_INPUT_COST_PER_1M = 0.300  # $0.300 per 1M input tokens
GROK_OUTPUT_COST_PER_1M = 0.500  # $0.500 per 1M output tokens

# === MongoDB Configuration ===
AI_ANALYSIS_COLLECTION = "ai_analysis_results"
ANALYSIS_INTERVAL_MINUTES = 10  # Run analysis every 10 minutes (optimized from 5)

# === Symbols to Analyze ===
# DEPRECATED: Do NOT use this hardcoded list!
# Symbols are fetched dynamically from Rust API via fetch_analysis_symbols() in main.py
# This is kept only for backwards compatibility with tests
# Fallback symbols when Rust API is unavailable
ANALYSIS_SYMBOLS: List[str] = [
    "BTCUSDT",
    "ETHUSDT",
    "BNBUSDT",
    "SOLUSDT",
]


def get_mongodb_url() -> str:
    """Get MongoDB URL from environment variable with validation."""
    mongodb_url = os.getenv("DATABASE_URL")
    if not mongodb_url:
        raise ValueError(
            "DATABASE_URL environment variable is required. "
            "Please set it in your .env file."
        )
    return mongodb_url


def get_ai_api_keys() -> List[str]:
    """Get AI (xAI/Grok) API keys from environment variables."""
    keys = []

    # Try primary key
    primary_key = os.getenv("XAI_API_KEY") or os.getenv("OPENAI_API_KEY")
    if primary_key:
        keys.append(primary_key)

    # Try fallback keys
    for i in range(1, 6):  # Support up to 5 fallback keys
        fallback_key = os.getenv(f"XAI_API_KEY_FALLBACK_{i}") or os.getenv(f"OPENAI_API_KEY_FALLBACK_{i}")
        if fallback_key:
            keys.append(fallback_key)

    if not keys:
        raise ValueError(
            "At least one AI API key is required. "
            "Set XAI_API_KEY in your .env file."
        )

    return keys


# === CORS Configuration ===
ALLOWED_ORIGINS = [
    "http://localhost:3000",
    "http://localhost:5173",
    "http://127.0.0.1:3000",
    "http://127.0.0.1:5173",
]
