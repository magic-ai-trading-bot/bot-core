"""
Configuration module for AI Trading Service.
Centralizes all configuration constants and environment variables.
"""

import os
from typing import List

# === API Configuration ===
# Read from environment variable with fallback to 2 seconds
# GPT-4o-mini has high rate limits (500 RPM for Tier 1, 5000 RPM for Tier 2+)
# So we can safely reduce delay between requests
OPENAI_REQUEST_DELAY = int(os.getenv("OPENAI_REQUEST_DELAY", "2"))

# === Cost Monitoring (GPT-4o-mini pricing as of Nov 2024) ===
GPT4O_MINI_INPUT_COST_PER_1M = 0.150  # $0.150 per 1M input tokens
GPT4O_MINI_OUTPUT_COST_PER_1M = 0.600  # $0.600 per 1M output tokens

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


def get_openai_api_keys() -> List[str]:
    """Get OpenAI API keys from environment variables."""
    keys = []

    # Try primary key
    primary_key = os.getenv("OPENAI_API_KEY")
    if primary_key:
        keys.append(primary_key)

    # Try fallback keys
    for i in range(1, 6):  # Support up to 5 fallback keys
        fallback_key = os.getenv(f"OPENAI_API_KEY_FALLBACK_{i}")
        if fallback_key:
            keys.append(fallback_key)

    if not keys:
        raise ValueError(
            "At least one OpenAI API key is required. "
            "Set OPENAI_API_KEY in your .env file."
        )

    return keys


# === CORS Configuration ===
ALLOWED_ORIGINS = [
    "http://localhost:3000",
    "http://localhost:5173",
    "http://127.0.0.1:3000",
    "http://127.0.0.1:5173",
]
