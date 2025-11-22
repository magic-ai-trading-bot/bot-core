#!/usr/bin/env python3
"""
Unit Tests for Backtesting Tasks
Tests strategy backtesting and optimization functionality
"""

import pytest
from datetime import datetime, timedelta


class TestBacktestStrategy:
    """Test backtest_strategy task"""

    def test_backtest_strategy_success(self):
        """Test successful strategy backtest"""
        from tasks.backtest_tasks import backtest_strategy

        result = backtest_strategy.run(
                "rsi",
                "BTCUSDT",
                "2024-01-01",
                "2024-03-01",
                {"rsi_period": 14, "oversold": 30, "overbought": 70},
            )

        assert result["status"] == "success"
        assert "total_trades" in result["results"]
        assert "win_rate" in result["results"]
        assert "total_profit" in result["results"]

    def test_backtest_strategy_minimal_period(self):
        """Test backtest with short time period"""
        from tasks.backtest_tasks import backtest_strategy

        result = backtest_strategy.run("rsi", "BTCUSDT", "2024-01-01", "2024-01-02", {})

        assert result["status"] in ["success", "failed"]
        # Should handle short periods gracefully

    def test_backtest_strategy_different_strategies(self):
        """Test backtest with different strategy names"""
        from tasks.backtest_tasks import backtest_strategy

        for strategy in ["rsi", "macd", "bollinger", "volume"]:
            result = backtest_strategy.run(strategy, "BTCUSDT", "2024-01-01", "2024-03-01", {})

            assert result is not None
            assert "status" in result


class TestOptimizeStrategy:
    """Test optimize_strategy task"""

    def test_optimize_strategy_success(self):
        """Test successful strategy optimization"""
        from tasks.backtest_tasks import optimize_strategy

        result = optimize_strategy.run(
                "rsi",
                "BTCUSDT",
                "2024-01-01",
                "2024-03-01",
                {
                    "rsi_period": [10, 14, 20],
                    "oversold": [25, 30, 35],
                    "overbought": [65, 70, 75],
                },
            )

        assert result["status"] == "success"
        assert "best_parameters" in result
        assert "best_sharpe_ratio" in result
        assert "optimization_results" in result

    def test_optimize_strategy_minimal_params(self):
        """Test optimization with minimal parameters"""
        from tasks.backtest_tasks import optimize_strategy

        result = optimize_strategy.run("rsi", "BTCUSDT", "2024-01-01", "2024-03-01", {})

        # Should handle minimal params
        assert result is not None
        assert "status" in result


class TestBacktestTaskCallbacks:
    """Test backtest task callbacks"""

    def test_backtest_task_on_failure(self):
        """Test backtest task failure callback"""
        from tasks.backtest_tasks import BacktestTask

        task = BacktestTask()

        # Should not raise exception
        task.on_failure(
            exc=Exception("Test error"),
            task_id="test-id",
            args=(),
            kwargs={},
            einfo=None
        )

    def test_backtest_task_on_success(self):
        """Test backtest task success callback"""
        from tasks.backtest_tasks import BacktestTask

        task = BacktestTask()

        # Should not raise exception
        task.on_success(
            retval={"status": "success"},
            task_id="test-id",
            args=(),
            kwargs={}
        )


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
