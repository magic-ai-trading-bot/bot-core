#!/usr/bin/env python3
"""
Performance Monitoring Script - Automated tracking of trading performance

This script monitors:
1. Win rate (target: 70% with optimized parameters)
2. Average profit per trade (target: 2.6%)
3. Sharpe ratio (target: 2.1)
4. Strategy consensus patterns
5. Paper trading results

Usage:
    python3 scripts/monitor_performance.py                  # Run once
    python3 scripts/monitor_performance.py --continuous     # Run continuously
    python3 scripts/monitor_performance.py --alert          # Send alerts if performance drops
"""

import requests
import json
import time
import datetime
from collections import defaultdict
import statistics

# Configuration
RUST_API = "http://localhost:8080"
PYTHON_API = "http://localhost:8000"
CHECK_INTERVAL = 300  # 5 minutes

# Performance thresholds (optimized targets)
TARGET_WIN_RATE = 70.0
TARGET_AVG_PROFIT = 2.6
TARGET_SHARPE = 2.1
MIN_TRADES = 10  # Minimum trades before calculating metrics

class PerformanceMonitor:
    def __init__(self):
        self.trades = []
        self.signals = []
        self.last_check = None

    def fetch_paper_trading_data(self):
        """Fetch paper trading portfolio and trades"""
        try:
            response = requests.get(f"{RUST_API}/api/paper-trading/portfolio", timeout=10)
            if response.status_code == 200:
                return response.json()
            else:
                print(f"⚠️  Failed to fetch portfolio: {response.status_code}")
                return None
        except Exception as e:
            print(f"❌ Error fetching portfolio: {e}")
            return None

    def fetch_trading_signals(self):
        """Fetch recent trading signals from all strategies"""
        try:
            # This endpoint may not exist yet, so handle gracefully
            response = requests.get(f"{RUST_API}/api/strategies/signals/recent", timeout=10)
            if response.status_code == 200:
                return response.json()
            else:
                return {"signals": []}
        except Exception:
            return {"signals": []}

    def calculate_win_rate(self, trades):
        """Calculate win rate from trade history"""
        if not trades or len(trades) < MIN_TRADES:
            return None

        winning_trades = [t for t in trades if t.get('pnl', 0) > 0]
        total_trades = len(trades)
        win_rate = (len(winning_trades) / total_trades) * 100 if total_trades > 0 else 0
        return win_rate

    def calculate_avg_profit(self, trades):
        """Calculate average profit per trade"""
        if not trades or len(trades) < MIN_TRADES:
            return None

        profits = [t.get('pnl_percentage', 0) for t in trades if t.get('pnl', 0) > 0]
        if not profits:
            return 0
        return statistics.mean(profits)

    def calculate_sharpe_ratio(self, trades):
        """Calculate Sharpe ratio (simplified)"""
        if not trades or len(trades) < MIN_TRADES:
            return None

        returns = [t.get('pnl_percentage', 0) for t in trades]
        if not returns or len(returns) < 2:
            return 0

        avg_return = statistics.mean(returns)
        std_dev = statistics.stdev(returns) if len(returns) > 1 else 0.0001

        # Annualized Sharpe (assuming daily returns)
        sharpe = (avg_return / std_dev) * (365 ** 0.5) if std_dev > 0 else 0
        return sharpe

    def analyze_strategy_consensus(self, signals):
        """Analyze how strategies are agreeing"""
        if not signals:
            return None

        consensus_patterns = defaultdict(int)
        for signal_group in signals:
            long_count = sum(1 for s in signal_group if s.get('signal') == 'LONG')
            short_count = sum(1 for s in signal_group if s.get('signal') == 'SHORT')
            neutral_count = sum(1 for s in signal_group if s.get('signal') == 'NEUTRAL')

            total = long_count + short_count + neutral_count
            agreement = max(long_count, short_count, neutral_count)

            consensus_patterns[f"{agreement}/{total}"] += 1

        return dict(consensus_patterns)

    def display_dashboard(self, portfolio, trades, signals):
        """Display performance dashboard"""
        print("\n" + "=" * 80)
        print("📊 PERFORMANCE MONITORING DASHBOARD")
        print("=" * 80)
        print()

        # Timestamp
        now = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        print(f"⏰ Last Updated: {now}")
        print()

        # Portfolio Overview
        if portfolio:
            print("💰 PORTFOLIO STATUS")
            print("-" * 80)
            balance = portfolio.get('balance', 0)
            initial_balance = portfolio.get('initial_balance', 10000)
            total_return = ((balance - initial_balance) / initial_balance) * 100 if initial_balance > 0 else 0

            print(f"   Current Balance:   ${balance:,.2f}")
            print(f"   Initial Balance:   ${initial_balance:,.2f}")
            print(f"   Total Return:      {total_return:+.2f}%")
            print(f"   Unrealized PnL:    ${portfolio.get('unrealized_pnl', 0):+,.2f}")
            print()

        # Trade Performance
        if trades and len(trades) >= MIN_TRADES:
            print("📈 TRADING PERFORMANCE")
            print("-" * 80)

            win_rate = self.calculate_win_rate(trades)
            avg_profit = self.calculate_avg_profit(trades)
            sharpe = self.calculate_sharpe_ratio(trades)

            # Win Rate
            if win_rate is not None:
                win_emoji = "🟢" if win_rate >= TARGET_WIN_RATE else ("🟡" if win_rate >= 65 else "🔴")
                print(f"   Win Rate:          {win_emoji} {win_rate:.1f}% ", end="")
                print(f"(Target: {TARGET_WIN_RATE}%)")

            # Avg Profit
            if avg_profit is not None:
                profit_emoji = "🟢" if avg_profit >= TARGET_AVG_PROFIT else ("🟡" if avg_profit >= 2.0 else "🔴")
                print(f"   Avg Profit:        {profit_emoji} {avg_profit:.2f}% ", end="")
                print(f"(Target: {TARGET_AVG_PROFIT}%)")

            # Sharpe Ratio
            if sharpe is not None:
                sharpe_emoji = "🟢" if sharpe >= TARGET_SHARPE else ("🟡" if sharpe >= 1.5 else "🔴")
                print(f"   Sharpe Ratio:      {sharpe_emoji} {sharpe:.2f} ", end="")
                print(f"(Target: {TARGET_SHARPE})")

            print(f"   Total Trades:      {len(trades)}")
            print()
        else:
            print("⏳ WAITING FOR DATA")
            print("-" * 80)
            trades_needed = MIN_TRADES - (len(trades) if trades else 0)
            print(f"   Need {trades_needed} more trades to calculate metrics")
            print()

        # Strategy Signals
        consensus = self.analyze_strategy_consensus(signals)
        if consensus:
            print("🎯 STRATEGY CONSENSUS PATTERNS")
            print("-" * 80)
            for pattern, count in sorted(consensus.items(), reverse=True):
                print(f"   {pattern} agreement: {count} times")
            print()

        # Performance Summary
        print("=" * 80)
        if trades and len(trades) >= MIN_TRADES:
            win_rate = self.calculate_win_rate(trades)
            if win_rate >= TARGET_WIN_RATE:
                print("✅ Performance: EXCELLENT - Meeting or exceeding optimization targets!")
            elif win_rate >= 65:
                print("⚠️  Performance: GOOD - Close to targets, monitor for improvement")
            else:
                print("❌ Performance: NEEDS ATTENTION - Below baseline, consider adjustments")
        else:
            print("⏳ Performance: COLLECTING DATA - Check back after more trades")
        print("=" * 80)
        print()

    def send_alert(self, message):
        """Send performance alert (can be extended to email/Slack/Discord)"""
        print("\n" + "!" * 80)
        print(f"⚠️  ALERT: {message}")
        print("!" * 80 + "\n")

        # Future: Add email/Slack/Discord integration here
        # For now, just print to console

    def check_performance_alerts(self, portfolio, trades):
        """Check if performance needs attention"""
        if not trades or len(trades) < MIN_TRADES:
            return

        win_rate = self.calculate_win_rate(trades)
        avg_profit = self.calculate_avg_profit(trades)

        alerts = []

        # Win rate dropped significantly
        if win_rate is not None and win_rate < 60:
            alerts.append(f"Win rate dropped to {win_rate:.1f}% (threshold: 60%)")

        # Average profit too low
        if avg_profit is not None and avg_profit < 1.5:
            alerts.append(f"Avg profit dropped to {avg_profit:.2f}% (threshold: 1.5%)")

        # Portfolio drawdown
        if portfolio:
            balance = portfolio.get('balance', 0)
            initial = portfolio.get('initial_balance', 10000)
            drawdown = ((initial - balance) / initial) * 100 if initial > 0 else 0

            if drawdown > 10:
                alerts.append(f"Portfolio drawdown: {drawdown:.1f}% (threshold: 10%)")

        # Send alerts
        for alert in alerts:
            self.send_alert(alert)

    def run_once(self, check_alerts=False):
        """Run monitoring once"""
        print("\n🔍 Fetching performance data...")

        portfolio = self.fetch_paper_trading_data()
        signals = self.fetch_trading_signals()

        # Extract trades from portfolio
        trades = []
        if portfolio and 'closed_trades' in portfolio:
            trades = portfolio['closed_trades']

        self.display_dashboard(portfolio, trades, signals)

        if check_alerts:
            self.check_performance_alerts(portfolio, trades)

        self.last_check = datetime.datetime.now()

    def run_continuous(self, check_alerts=False):
        """Run monitoring continuously"""
        print(f"🔄 Starting continuous monitoring (interval: {CHECK_INTERVAL}s)")
        print("Press Ctrl+C to stop\n")

        try:
            while True:
                self.run_once(check_alerts=check_alerts)
                print(f"\n⏳ Next check in {CHECK_INTERVAL} seconds...")
                time.sleep(CHECK_INTERVAL)
        except KeyboardInterrupt:
            print("\n\n👋 Monitoring stopped by user")

def main():
    import sys

    monitor = PerformanceMonitor()

    # Parse arguments
    continuous = '--continuous' in sys.argv
    check_alerts = '--alert' in sys.argv

    if continuous:
        monitor.run_continuous(check_alerts=check_alerts)
    else:
        monitor.run_once(check_alerts=check_alerts)

if __name__ == "__main__":
    main()
