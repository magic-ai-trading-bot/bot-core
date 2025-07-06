#!/usr/bin/env python3
"""
COMPREHENSIVE TRADE PIPELINE DEBUG

This script tests the complete pipeline:
1. AI Service health check
2. Engine status verification  
3. Manual analysis trigger
4. Symbol settings verification
5. Trade execution monitoring
6. WebSocket message monitoring
"""

import requests
import time
import json

def test_ai_service():
    """Test AI service directly"""
    print("🤖 Testing AI Service...")
    
    data = {
        "symbol": "BTCUSDT",
        "timeframe_data": {
            "1h": [{
                "open_time": int(time.time() * 1000) - 3600000,
                "close_time": int(time.time() * 1000),
                "open": 97000.0,
                "high": 97900.0,
                "low": 96800.0,
                "close": 97600.0,
                "volume": 350.0,
                "quote_volume": 34160000.0,
                "trades": 1800,
                "is_closed": True,
                "timestamp": int(time.time() * 1000)
            }]
        },
        "current_price": 97600.0,
        "volume_24h": 4800.0,
        "timestamp": int(time.time() * 1000),
        "strategy_context": {
            "selected_strategies": ["RSI Strategy", "MACD Strategy", "Volume Strategy", "Bollinger Bands Strategy"],
            "market_condition": "Trending", 
            "risk_level": "Conservative"
        }
    }
    
    try:
        response = requests.post("http://localhost:8000/ai/analyze", json=data, timeout=15)
        if response.status_code == 200:
            result = response.json()
            confidence = result.get('confidence', 0) * 100
            signal = result.get('signal', 'Unknown')
            reasoning = result.get('reasoning', 'N/A')[:80]
            
            print(f"   ✅ AI Service: {signal} ({confidence:.1f}%)")
            print(f"   💭 Reasoning: {reasoning}...")
            
            if confidence >= 45:
                print(f"   🚨 TRADE CRITERIA MET: {confidence:.1f}% >= 45%")
                return True, confidence
            else:
                print(f"   ⚠️ Below threshold: {confidence:.1f}% < 45%")
                return False, confidence
        else:
            print(f"   ❌ AI Service error: {response.status_code}")
            return False, 0
    except Exception as e:
        print(f"   ❌ AI Service error: {e}")
        return False, 0

def test_engine_status():
    """Test engine status and configuration"""
    print("\n🚀 Testing Engine Status...")
    
    try:
        # Engine status
        response = requests.get("http://localhost:8080/api/paper-trading/status", timeout=10)
        if response.status_code == 200:
            data = response.json()['data']
            print(f"   ✅ Engine Running: {data.get('is_running', False)}")
            print(f"   💰 Portfolio: ${data.get('portfolio', {}).get('equity', 0):.2f}")
            print(f"   📊 Total Trades: {data.get('portfolio', {}).get('total_trades', 0)}")
            
            # Strategy settings
            response = requests.get("http://localhost:8080/api/paper-trading/strategy-settings", timeout=10)
            if response.status_code == 200:
                settings = response.json()['data']
                threshold = settings['engine']['min_confidence_threshold'] * 100
                print(f"   🎯 Confidence Threshold: {threshold}%")
                
                if threshold == 45.0:
                    print("   ✅ Low Volatility settings active")
                    return True
                else:
                    print(f"   ⚠️ Expected 45%, got {threshold}%")
                    return False
            else:
                print(f"   ❌ Settings error: {response.status_code}")
                return False
        else:
            print(f"   ❌ Engine error: {response.status_code}")
            return False
    except Exception as e:
        print(f"   ❌ Engine error: {e}")
        return False

def test_symbol_settings():
    """Test symbol configuration"""
    print("\n⚙️ Testing Symbol Settings...")
    
    try:
        response = requests.get("http://localhost:8080/api/paper-trading/symbols", timeout=10)
        if response.status_code == 200:
            symbols = response.json()['data']
            enabled_symbols = [sym for sym, config in symbols.items() if config.get('enabled', False)]
            
            print(f"   📋 Enabled Symbols: {len(enabled_symbols)}")
            for symbol in enabled_symbols:
                config = symbols[symbol]
                print(f"      {symbol}: leverage={config.get('leverage', 1)}x, size={config.get('position_size_pct', 0)}%")
            
            if len(enabled_symbols) > 0:
                print("   ✅ Symbols configured for trading")
                return True
            else:
                print("   ❌ No symbols enabled")
                return False
        else:
            print(f"   ❌ Symbol settings error: {response.status_code}")
            return False
    except Exception as e:
        print(f"   ❌ Symbol settings error: {e}")
        return False

def trigger_manual_analysis():
    """Trigger manual analysis and monitor results"""
    print("\n🔧 Triggering Manual Analysis...")
    
    try:
        # Trigger analysis
        response = requests.post("http://localhost:8080/api/paper-trading/trigger-analysis", timeout=15)
        if response.status_code == 200:
            print("   ✅ Manual analysis triggered")
            
            # Wait and check for trades
            print("   ⏳ Waiting 10 seconds for trade creation...")
            time.sleep(10)
            
            # Check trades
            response = requests.get("http://localhost:8080/api/paper-trading/status", timeout=10)
            if response.status_code == 200:
                data = response.json()['data']
                total_trades = data.get('portfolio', {}).get('total_trades', 0)
                open_trades = len(data.get('open_trades', []))
                
                print(f"   📊 After Analysis - Total: {total_trades}, Open: {open_trades}")
                
                if total_trades > 0:
                    print("   🎉 TRADES CREATED! Pipeline working!")
                    return True
                else:
                    print("   ❌ No trades created - pipeline issue")
                    return False
            else:
                print(f"   ❌ Status check error: {response.status_code}")
                return False
        else:
            print(f"   ❌ Trigger error: {response.status_code}")
            return False
    except Exception as e:
        print(f"   ❌ Manual analysis error: {e}")
        return False

def check_ai_service_logs():
    """Check if AI service is being called by engine"""
    print("\n📋 Checking AI Service Logs...")
    
    import subprocess
    try:
        # Check AI service logs for recent requests
        result = subprocess.run(['docker-compose', 'logs', '--tail=20', 'python-ai-service'], 
                              capture_output=True, text=True, timeout=10)
        
        logs = result.stdout
        if '/ai/analyze' in logs:
            print("   ✅ AI service receiving requests from engine")
            # Count recent requests
            recent_requests = logs.count('/ai/analyze')
            print(f"   📊 Recent AI requests: {recent_requests}")
            return True
        else:
            print("   ❌ No AI analysis requests found in logs")
            print("   💡 Engine may not be calling AI service automatically")
            return False
    except Exception as e:
        print(f"   ❌ Log check error: {e}")
        return False

def diagnose_pipeline():
    """Run complete pipeline diagnosis"""
    print("🔍 COMPREHENSIVE TRADE PIPELINE DIAGNOSIS")
    print("=" * 60)
    
    # Test each component
    ai_working, ai_confidence = test_ai_service()
    engine_working = test_engine_status()
    symbols_working = test_symbol_settings()
    logs_working = check_ai_service_logs()
    
    print("\n" + "=" * 60)
    print("📊 DIAGNOSIS SUMMARY:")
    print("=" * 60)
    print(f"✅ AI Service:           {'✅ WORKING' if ai_working else '❌ ISSUES'}")
    print(f"✅ Engine Status:        {'✅ WORKING' if engine_working else '❌ ISSUES'}")
    print(f"✅ Symbol Configuration: {'✅ WORKING' if symbols_working else '❌ ISSUES'}")
    print(f"✅ AI Service Usage:     {'✅ WORKING' if logs_working else '❌ ISSUES'}")
    
    if ai_working and engine_working and symbols_working:
        print(f"\n🚨 SHOULD BE CREATING TRADES: AI {ai_confidence:.1f}% >= 45%")
        
        if not logs_working:
            print("\n💡 LIKELY ISSUE: Engine not calling AI service automatically")
            print("   - Analysis loop may not be running")
            print("   - AI service URL may be misconfigured")
            print("   - Signal processing may have errors")
        
        # Try manual trigger
        print("\n🔧 TESTING MANUAL TRIGGER:")
        manual_working = trigger_manual_analysis()
        
        if not manual_working:
            print("\n🚨 CRITICAL: Manual trigger also fails!")
            print("   - Check engine analysis implementation")
            print("   - Verify AI service connection in engine")
            print("   - Debug signal processing logic")
        else:
            print("\n✅ Manual trigger works - automatic loop issue")
    else:
        print("\n❌ PIPELINE HAS ISSUES - Fix prerequisites first")
    
    return {
        'ai_service': ai_working,
        'engine': engine_working, 
        'symbols': symbols_working,
        'ai_usage': logs_working,
        'ai_confidence': ai_confidence
    }

if __name__ == "__main__":
    results = diagnose_pipeline()
    
    if all([results['ai_service'], results['engine'], results['symbols']]):
        print(f"\n🎯 NEXT STEPS:")
        if not results['ai_usage']:
            print("1. Debug engine AI service connection")
            print("2. Check analysis loop implementation") 
            print("3. Verify automatic signal processing")
        else:
            print("1. All components working!")
            print("2. Monitor for automatic trades")
    else:
        print(f"\n🔧 FIX REQUIRED:")
        if not results['ai_service']:
            print("- Fix AI service connection")
        if not results['engine']:
            print("- Fix engine configuration")
        if not results['symbols']:
            print("- Configure trading symbols") 