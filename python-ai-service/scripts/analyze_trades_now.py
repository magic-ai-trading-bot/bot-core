#!/usr/bin/env python3
"""
Run GPT-4 analysis on closed trades
"""

import requests
import os
import json
from openai import OpenAI
from utils.data_storage import DataStorage

def main():
    # Initialize
    storage = DataStorage()
    client = OpenAI(api_key=os.getenv('OPENAI_API_KEY'))
    RUST_API_URL = os.getenv('RUST_API_URL', 'http://rust-core-engine:8080')

    # Get closed trades
    response = requests.get(f'{RUST_API_URL}/api/paper-trading/trades/closed', timeout=10)
    trades = response.json().get('data', [])

    print(f'Analyzing {len(trades)} trades with GPT-4...')

    for trade in trades:
        trade_id = trade.get('id', 'unknown')
        symbol = trade.get('symbol', 'UNKNOWN')
        trade_type = trade.get('trade_type', 'Unknown')
        pnl = trade.get('pnl', 0)
        pnl_pct = trade.get('pnl_percentage', 0)
        entry_price = trade.get('entry_price', 0)
        exit_price = trade.get('exit_price', 0)
        close_reason = trade.get('close_reason', 'Unknown')

        # Check if already analyzed
        existing = storage.get_trade_analysis(trade_id)
        if existing:
            print(f'  ⏭️ {symbol} already analyzed, skipping')
            continue

        print(f'  🔍 Analyzing {symbol} {trade_type}...')

        # Build prompt
        prompt = f'''Analyze this cryptocurrency trade and provide detailed feedback:

Trade Details:
- Symbol: {symbol}
- Direction: {trade_type}
- Entry Price: ${entry_price:.2f}
- Exit Price: ${exit_price:.2f}
- P&L: ${pnl:.2f} ({pnl_pct:.2f}%)
- Close Reason: {close_reason}

Please provide analysis in JSON format:
{{
    "overall_assessment": "Brief overall assessment of the trade",
    "what_went_wrong": "What went wrong with this trade",
    "key_mistakes": ["mistake 1", "mistake 2"],
    "lessons_learned": ["lesson 1", "lesson 2"],
    "suggested_improvements": ["improvement 1", "improvement 2"],
    "entry_analysis": "Analysis of entry timing and price",
    "exit_analysis": "Analysis of exit timing and price",
    "risk_management_review": "Review of risk management",
    "market_context": "Brief market context"
}}'''

        try:
            response = client.chat.completions.create(
                model='gpt-4o-mini',
                messages=[
                    {'role': 'system', 'content': 'You are a professional crypto trading analyst. Analyze trades and provide actionable feedback. Always respond in valid JSON format.'},
                    {'role': 'user', 'content': prompt}
                ],
                temperature=0.3,
                max_tokens=1000
            )

            analysis_text = response.choices[0].message.content

            # Clean up JSON if wrapped in markdown
            if '```json' in analysis_text:
                analysis_text = analysis_text.split('```json')[1].split('```')[0]
            elif '```' in analysis_text:
                analysis_text = analysis_text.split('```')[1].split('```')[0]

            analysis = json.loads(analysis_text.strip())

            # Store in MongoDB
            result = storage.store_trade_analysis(
                trade_id=trade_id,
                trade_data=trade,
                analysis=analysis
            )

            print(f'  ✅ {symbol} analyzed and saved! (ID: {result})')
            print(f'     Assessment: {analysis.get("overall_assessment", "N/A")[:100]}...')

        except Exception as e:
            print(f'  ❌ Error analyzing {symbol}: {e}')
            import traceback
            traceback.print_exc()

    print('\n✅ Done! Refresh the frontend to see results.')

if __name__ == '__main__':
    main()
