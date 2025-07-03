#!/bin/bash

# Cryptocurrency AI Trading Service - Start Script
# This script helps you get the AI trading service running quickly

set -e

echo "🤖 Cryptocurrency AI Trading Service - Quick Start"
echo "============================================================"

# Check Python version
echo "🔍 Checking Python version..."
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "✅ Python version: $python_version"

# Create necessary directories
echo "🔧 Setting up directories..."
mkdir -p models/saved
mkdir -p logs
mkdir -p data
echo "✅ Directories created"

# Check if virtual environment should be used
if [ ! -d "venv" ] && [ "$1" != "--no-venv" ]; then
    echo "💡 Consider creating a virtual environment:"
    echo "   python3 -m venv venv"
    echo "   source venv/bin/activate"
    echo "   pip install -r requirements.txt"
    echo ""
fi

# Check if requirements are installed
echo "🔍 Checking dependencies..."
if ! python3 -c "import fastapi, tensorflow, pandas" 2>/dev/null; then
    echo "⚠️  Some dependencies may be missing"
    echo "💡 Install them with: pip install -r requirements.txt"
    echo ""
fi

# Set default environment variables
export SERVER_HOST=${SERVER_HOST:-"0.0.0.0"}
export SERVER_PORT=${SERVER_PORT:-"8000"}
export MODEL_TYPE=${MODEL_TYPE:-"lstm"}
export LOG_LEVEL=${LOG_LEVEL:-"INFO"}

echo "📚 Service Information:"
echo "============================================================"
echo "🌐 API Base URL: http://$SERVER_HOST:$SERVER_PORT"
echo "📖 Documentation: http://$SERVER_HOST:$SERVER_PORT/docs"
echo "🏥 Health Check: http://$SERVER_HOST:$SERVER_PORT/health"
echo "⚙️  Configuration: http://$SERVER_HOST:$SERVER_PORT/config"
echo ""
echo "🔗 Key Endpoints:"
echo "   POST /analyze    - Generate trading signals"
echo "   POST /train      - Train AI model"
echo "   GET  /model/info - Get model information"
echo ""
echo "💡 Quick Commands:"
echo "   # Check service health"
echo "   curl http://$SERVER_HOST:$SERVER_PORT/health"
echo ""
echo "   # Run example client"
echo "   python3 example_client.py"
echo ""
echo "🚀 Starting AI Trading Service..."
echo "============================================================"

# Start the service
if [ "$1" = "--dev" ]; then
    echo "🔧 Development mode enabled"
    export SERVER_RELOAD=true
fi

python3 main.py 