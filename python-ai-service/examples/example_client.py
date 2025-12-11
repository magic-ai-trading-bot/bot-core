#!/usr/bin/env python3
"""
Example client for the Cryptocurrency AI Trading Service.

This script demonstrates how to:
1. Generate sample data
2. Train an AI model
3. Make trading signal predictions
4. Monitor model performance

Usage:
    python example_client.py
"""

import random
import time
from datetime import datetime, timedelta
from typing import Any, Dict, List

import requests

# Configuration
API_BASE_URL = "http://localhost:8000"
SYMBOL = "BTCUSDT"
TIMEFRAME = "1h"


class TradingServiceClient:
    """Client for interacting with the AI Trading Service."""

    def __init__(self, base_url: str = API_BASE_URL):
        self.base_url = base_url
        self.session = requests.Session()

    def health_check(self) -> Dict[str, Any]:
        """Check service health."""
        try:
            response = self.session.get(f"{self.base_url}/health")
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            print(f"Health check failed: {e}")
            return {}

    def get_config(self) -> Dict[str, Any]:
        """Get service configuration."""
        try:
            response = self.session.get(f"{self.base_url}/config")
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            print(f"Failed to get config: {e}")
            return {}

    def get_model_info(self) -> Dict[str, Any]:
        """Get current model information."""
        try:
            response = self.session.get(f"{self.base_url}/model/info")
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            print(f"Failed to get model info: {e}")
            return {}

    def train_model(
        self, candles: List[Dict], model_type: str = "lstm", retrain: bool = False
    ) -> Dict[str, Any]:
        """Train the AI model with historical data."""
        payload = {
            "symbol": SYMBOL,
            "model_type": model_type,
            "retrain": retrain,
            "candles": candles,
        }

        try:
            response = self.session.post(f"{self.base_url}/train", json=payload)
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            print(f"Training failed: {e}")
            if hasattr(e, "response") and e.response is not None:
                print(f"Error details: {e.response.text}")
            return {}

    def analyze_market(
        self, candles: List[Dict], symbol: str = SYMBOL, timeframe: str = TIMEFRAME
    ) -> Dict[str, Any]:
        """Analyze market data and get trading signals."""
        payload = {"symbol": symbol, "timeframe": timeframe, "candles": candles}

        try:
            response = self.session.post(f"{self.base_url}/analyze", json=payload)
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            print(f"Analysis failed: {e}")
            if hasattr(e, "response") and e.response is not None:
                print(f"Error details: {e.response.text}")
            return {}


def generate_sample_candles(
    count: int = 500, start_price: float = 47000.0
) -> List[Dict[str, Any]]:
    """Generate realistic sample OHLCV data for testing."""
    candles = []
    current_price = start_price
    base_timestamp = int((datetime.now() - timedelta(hours=count)).timestamp() * 1000)

    for i in range(count):
        # Generate realistic price movement (simplified random walk)
        price_change = random.uniform(-0.03, 0.03)  # ±3% max change
        trend = random.uniform(-0.01, 0.01)  # Small trend component

        # Calculate OHLC
        open_price = current_price
        close_price = open_price * (1 + price_change + trend)

        # Ensure high >= max(open, close) and low <= min(open, close)
        high_price = max(open_price, close_price) * (1 + random.uniform(0, 0.01))
        low_price = min(open_price, close_price) * (1 - random.uniform(0, 0.01))

        # Generate realistic volume
        base_volume = 1000
        volume = base_volume * random.uniform(0.5, 2.0)

        candle = {
            "timestamp": base_timestamp + (i * 3600000),  # 1 hour intervals
            "open": round(open_price, 2),
            "high": round(high_price, 2),
            "low": round(low_price, 2),
            "close": round(close_price, 2),
            "volume": round(volume, 2),
        }

        candles.append(candle)
        current_price = close_price

    return candles


def display_signal_analysis(signal_data: Dict[str, Any]):
    """Display trading signal analysis in a formatted way."""
    if not signal_data:
        print("❌ No signal data received")
        return

    print("\n" + "=" * 60)
    print("🎯 TRADING SIGNAL ANALYSIS")
    print("=" * 60)

    # Signal and confidence
    signal = signal_data.get("signal", "unknown").upper()
    confidence = signal_data.get("confidence", 0)
    probability = signal_data.get("probability", 0.5)

    # Color coding for terminal output
    if signal == "LONG":
        signal_color = "\033[92m"  # Green
        signal_emoji = "📈"
    elif signal == "SHORT":
        signal_color = "\033[91m"  # Red
        signal_emoji = "📉"
    else:
        signal_color = "\033[93m"  # Yellow
        signal_emoji = "➡️"

    reset_color = "\033[0m"

    print(f"Symbol: {signal_data.get('symbol', 'N/A')}")
    print(f"Timeframe: {signal_data.get('timeframe', 'N/A')}")
    print(f"Model: {signal_data.get('model_type', 'N/A').upper()}")
    print(f"Timestamp: {signal_data.get('timestamp', 'N/A')}")
    print()
    print(f"{signal_emoji} Signal: {signal_color}{signal}{reset_color}")
    print(f"🎯 Confidence: {confidence:.1f}%")
    print(f"📊 Probability: {probability:.3f}")

    # Confidence interpretation
    if confidence >= 80:
        print("💪 Very High Confidence")
    elif confidence >= 60:
        print("👍 High Confidence")
    elif confidence >= 40:
        print("🤔 Medium Confidence")
    else:
        print("⚠️  Low Confidence")

    print("=" * 60)


def main():
    """Main example function demonstrating the trading service."""
    print("🚀 Cryptocurrency AI Trading Service - Example Client")
    print("=" * 60)

    # Initialize client
    client = TradingServiceClient()

    # 1. Health Check
    print("\n1️⃣  Checking service health...")
    health = client.health_check()
    if health:
        print(f"✅ Service is {health.get('status', 'unknown')}")
        print(f"   Model loaded: {health.get('model_loaded', False)}")
        print(f"   Version: {health.get('version', 'unknown')}")
    else:
        print(
            "❌ Service unavailable. Make sure the service is running on localhost:8000"
        )
        return

    # 2. Get Configuration
    print("\n2️⃣  Getting service configuration...")
    config = client.get_config()
    if config:
        supported_timeframes = config.get("data_config", {}).get(
            "supported_timeframes", []
        )
        print(f"✅ Supported timeframes: {', '.join(supported_timeframes)}")

        model_config = config.get("model_config", {})
        print(f"   Default model: {model_config.get('type', 'lstm')}")
        print(f"   Sequence length: {model_config.get('sequence_length', 60)}")

    # 3. Check Model Status
    print("\n3️⃣  Checking current model status...")
    model_info = client.get_model_info()
    model_loaded = model_info.get("model_loaded", False)

    if model_loaded:
        print("✅ Model is already loaded")
        print(f"   Type: {model_info.get('model_type', 'unknown')}")
        print(f"   Training samples: {model_info.get('training_samples', 'N/A')}")
        print(f"   Feature count: {model_info.get('feature_count', 'N/A')}")
    else:
        print("⚠️  No model loaded - training required")

    # 4. Generate Sample Data
    print("\n4️⃣  Generating sample OHLCV data...")
    training_candles = generate_sample_candles(count=800)  # More data for training
    analysis_candles = generate_sample_candles(count=150)  # Minimum for analysis

    print(f"✅ Generated {len(training_candles)} training candles")
    print(f"✅ Generated {len(analysis_candles)} analysis candles")

    # Sample data preview
    latest_candle = training_candles[-1]
    print(
        f"   Latest candle: Open={latest_candle['open']}, Close={latest_candle['close']}, Volume={latest_candle['volume']}"
    )

    # 5. Train Model (if needed)
    if not model_loaded:
        print("\n5️⃣  Training AI model...")
        print("   This may take a few minutes depending on your hardware...")

        training_result = client.train_model(training_candles, model_type="lstm")

        if training_result:
            print("✅ Training started successfully")
            print(f"   Model type: {training_result.get('model_type', 'unknown')}")
            print(f"   Training samples: {training_result.get('training_samples', 0)}")
            print("   Training is running in the background...")

            # Wait a bit for training to progress
            print("   Waiting for training to complete...")
            for i in range(10):
                time.sleep(3)
                model_info = client.get_model_info()
                if model_info.get("model_loaded", False):
                    print("✅ Model training completed!")
                    break
                print(f"   Still training... ({i+1}/10)")
            else:
                print(
                    "⚠️  Training is taking longer than expected. Proceeding with analysis attempt..."
                )
        else:
            print("❌ Training failed. Cannot proceed with analysis.")
            return
    else:
        print("\n5️⃣  Skipping training - model already available")

    # 6. Market Analysis
    print("\n6️⃣  Performing market analysis...")

    # Try multiple analysis requests to show different scenarios
    for i in range(3):
        print(f"\n   Analysis #{i+1}:")

        # Use different portions of data to simulate different market conditions
        start_idx = i * 20
        end_idx = start_idx + 120
        analysis_data = analysis_candles[start_idx:end_idx]

        signal_result = client.analyze_market(analysis_data)

        if signal_result:
            display_signal_analysis(signal_result)
        else:
            print("❌ Analysis failed")

        time.sleep(1)  # Brief pause between requests

    # 7. Model Information (after training/analysis)
    print("\n7️⃣  Final model information...")
    final_model_info = client.get_model_info()

    if final_model_info:
        print("✅ Model Status:")
        print(f"   Type: {final_model_info.get('model_type', 'unknown')}")
        print(f"   Loaded: {final_model_info.get('model_loaded', False)}")
        print(f"   Training samples: {final_model_info.get('training_samples', 'N/A')}")
        print(
            f"   Validation samples: {final_model_info.get('validation_samples', 'N/A')}"
        )
        print(f"   Features: {final_model_info.get('feature_count', 'N/A')}")
        print(
            f"   Training accuracy: {final_model_info.get('training_accuracy', 'N/A')}"
        )

        trained_timestamp = final_model_info.get("trained_timestamp")
        if trained_timestamp:
            print(f"   Last trained: {trained_timestamp}")

    print("\n🎉 Example completed successfully!")
    print("\n💡 Tips for production use:")
    print("   • Use real market data from your Rust trading bot")
    print("   • Implement proper error handling and retries")
    print("   • Monitor model performance and retrain periodically")
    print("   • Consider using multiple models for ensemble predictions")
    print("   • Set up proper logging and monitoring")
    print("\n📚 Documentation: http://localhost:8000/docs")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n⏹️  Example interrupted by user")
    except Exception as e:
        print(f"\n❌ Example failed with error: {e}")
        import traceback

        traceback.print_exc()
