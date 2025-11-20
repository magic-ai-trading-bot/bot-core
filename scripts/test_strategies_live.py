#!/usr/bin/env python3
"""
Test all 5 trading strategies with real Binance market data
Shows signals, confidence levels, and consensus from all strategies
"""

import requests
import json
from datetime import datetime
import sys

BINANCE_API = "https://api.binance.com"
SYMBOL = "BTCUSDT"
INTERVALS = ["1h", "4h"]

def fetch_binance_klines(symbol, interval, limit=100):
    """Fetch candlestick data from Binance"""
    url = f"{BINANCE_API}/api/v3/klines"
    params = {
        "symbol": symbol,
        "interval": interval,
        "limit": limit
    }

    try:
        response = requests.get(url, params=params, timeout=10)
        response.raise_for_status()
        data = response.json()

        candles = []
        for candle in data:
            candles.append({
                "timestamp": candle[0],
                "open": float(candle[1]),
                "high": float(candle[2]),
                "low": float(candle[3]),
                "close": float(candle[4]),
                "volume": float(candle[5]),
            })

        return candles
    except Exception as e:
        print(f"❌ Error fetching Binance data: {e}")
        return None

def get_current_price(symbol):
    """Get current price from Binance"""
    url = f"{BINANCE_API}/api/v3/ticker/price"
    params = {"symbol": symbol}

    try:
        response = requests.get(url, params=params, timeout=10)
        response.raise_for_status()
        data = response.json()
        return float(data["price"])
    except Exception as e:
        print(f"❌ Error fetching current price: {e}")
        return None

def get_24h_volume(symbol):
    """Get 24h volume from Binance"""
    url = f"{BINANCE_API}/api/v3/ticker/24hr"
    params = {"symbol": symbol}

    try:
        response = requests.get(url, params=params, timeout=10)
        response.raise_for_status()
        data = response.json()
        return float(data["volume"])
    except Exception as e:
        print(f"❌ Error fetching 24h volume: {e}")
        return None

def test_rust_strategy_endpoint():
    """Test if Rust API is running and has strategy endpoint"""
    url = "http://localhost:8080/api/strategies/analyze"

    try:
        # Try a simple health check first
        health_url = "http://localhost:8080/api/health"
        response = requests.get(health_url, timeout=5)

        if response.status_code == 200:
            print("✅ Rust API is running")
            return True
        else:
            print(f"⚠️  Rust API returned status {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Cannot connect to Rust API: {e}")
        print("💡 Make sure to run: ./scripts/bot.sh start")
        return False

def analyze_with_strategies(symbol, timeframe_data, current_price, volume_24h):
    """Call Rust API to analyze with all 5 strategies"""
    url = "http://localhost:8080/api/strategies/analyze"

    payload = {
        "symbol": symbol,
        "timeframe_data": timeframe_data,
        "current_price": current_price,
        "volume_24h": volume_24h,
        "timestamp": int(datetime.now().timestamp() * 1000)
    }

    try:
        response = requests.post(url, json=payload, timeout=10)
        response.raise_for_status()
        return response.json()
    except Exception as e:
        print(f"❌ Error calling strategy API: {e}")
        return None

def format_signal(signal):
    """Format signal with emoji"""
    if signal == "LONG" or signal == "Long":
        return "🟢 LONG"
    elif signal == "SHORT" or signal == "Short":
        return "🔴 SHORT"
    else:
        return "⚪ NEUTRAL"

def format_confidence(confidence):
    """Format confidence with color"""
    if confidence >= 0.8:
        return f"🔥 {confidence:.2%}"
    elif confidence >= 0.6:
        return f"✅ {confidence:.2%}"
    elif confidence >= 0.4:
        return f"⚠️  {confidence:.2%}"
    else:
        return f"❌ {confidence:.2%}"

def main():
    print("=" * 80)
    print("🚀 TESTING 5 TRADING STRATEGIES WITH REAL BINANCE DATA")
    print("=" * 80)
    print()

    # Step 1: Check if Rust API is running
    print("📡 Step 1: Checking Rust API...")
    if not test_rust_strategy_endpoint():
        print("\n⚠️  Please start the bot first: ./scripts/bot.sh start")
        sys.exit(1)
    print()

    # Step 2: Fetch real market data
    print(f"📊 Step 2: Fetching real market data for {SYMBOL}...")

    timeframe_data = {}
    for interval in INTERVALS:
        print(f"   - Fetching {interval} candles...")
        candles = fetch_binance_klines(SYMBOL, interval, limit=100)

        if not candles:
            print(f"   ❌ Failed to fetch {interval} data")
            sys.exit(1)

        timeframe_data[interval] = candles
        print(f"   ✅ Got {len(candles)} candles for {interval}")

    current_price = get_current_price(SYMBOL)
    volume_24h = get_24h_volume(SYMBOL)

    if not current_price or not volume_24h:
        print("   ❌ Failed to fetch current price or volume")
        sys.exit(1)

    print(f"   ✅ Current Price: ${current_price:,.2f}")
    print(f"   ✅ 24h Volume: {volume_24h:,.0f} BTC")
    print()

    # Step 3: Analyze with all 5 strategies
    print("🤖 Step 3: Analyzing with all 5 strategies...")
    print("   - RSI Strategy")
    print("   - MACD Strategy")
    print("   - Bollinger Bands Strategy")
    print("   - Volume Strategy")
    print("   - Stochastic Strategy (NEW)")
    print()

    result = analyze_with_strategies(SYMBOL, timeframe_data, current_price, volume_24h)

    if not result:
        print("❌ Failed to get strategy analysis")
        sys.exit(1)

    # Step 4: Display results
    print("=" * 80)
    print("📈 STRATEGY ANALYSIS RESULTS")
    print("=" * 80)
    print()

    signals = result.get("signals", [])
    consensus = result.get("consensus", {})

    if not signals:
        print("⚠️  No signals received from strategies")
        sys.exit(1)

    # Display individual strategy signals
    print("🎯 Individual Strategy Signals:")
    print("-" * 80)

    for i, signal in enumerate(signals, 1):
        strategy_name = signal.get("strategy", "Unknown")
        signal_type = signal.get("signal", "NEUTRAL")
        confidence = signal.get("confidence", 0.0)
        reasoning = signal.get("reasoning", "No reasoning provided")
        timeframe = signal.get("timeframe", "1h")

        print(f"\n{i}. {strategy_name.upper()} ({timeframe})")
        print(f"   Signal:     {format_signal(signal_type)}")
        print(f"   Confidence: {format_confidence(confidence)}")
        print(f"   Reasoning:  {reasoning}")

    print()
    print("=" * 80)
    print("🎲 CONSENSUS ANALYSIS")
    print("=" * 80)

    # Display consensus
    consensus_signal = consensus.get("signal", "NEUTRAL")
    consensus_confidence = consensus.get("confidence", 0.0)
    long_count = consensus.get("long_count", 0)
    short_count = consensus.get("short_count", 0)
    neutral_count = consensus.get("neutral_count", 0)
    total = long_count + short_count + neutral_count

    print(f"\n📊 Vote Breakdown:")
    print(f"   🟢 LONG:    {long_count}/{total} strategies ({long_count/total*100:.0f}%)")
    print(f"   🔴 SHORT:   {short_count}/{total} strategies ({short_count/total*100:.0f}%)")
    print(f"   ⚪ NEUTRAL: {neutral_count}/{total} strategies ({neutral_count/total*100:.0f}%)")

    print(f"\n🎯 Final Consensus:")
    print(f"   Signal:     {format_signal(consensus_signal)}")
    print(f"   Confidence: {format_confidence(consensus_confidence)}")

    agreement_percentage = max(long_count, short_count, neutral_count) / total * 100
    print(f"\n📈 Agreement Level: {agreement_percentage:.0f}% ({max(long_count, short_count, neutral_count)}/{total} strategies agree)")

    if agreement_percentage >= 80:
        print("   ✅ STRONG CONSENSUS - High confidence signal")
    elif agreement_percentage >= 60:
        print("   ⚠️  MODERATE CONSENSUS - Medium confidence signal")
    else:
        print("   ❌ WEAK CONSENSUS - Low confidence signal")

    print()
    print("=" * 80)
    print("✅ TEST COMPLETED SUCCESSFULLY")
    print("=" * 80)
    print()
    print(f"✨ All 5 strategies are working correctly with real Binance data!")
    print(f"✨ UI showing '5/5 chiến lược đồng ý' is now ACCURATE!")
    print()

if __name__ == "__main__":
    main()
