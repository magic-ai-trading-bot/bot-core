#!/usr/bin/env python3
"""
Test Setup Script for Cryptocurrency AI Trading Service

This script performs basic tests to verify the installation and setup.
"""

import sys
import os
import importlib
from pathlib import Path

def test_python_version():
    """Test Python version compatibility."""
    print("🔍 Testing Python version...")
    if sys.version_info >= (3, 8):
        print(f"✅ Python {sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro} is compatible")
        return True
    else:
        print(f"❌ Python {sys.version_info.major}.{sys.version_info.minor} is too old. Requires 3.8+")
        return False

def test_imports():
    """Test if all required packages can be imported."""
    print("\n🔍 Testing package imports...")
    
    required_packages = [
        ('fastapi', 'FastAPI'),
        ('uvicorn', 'Uvicorn'),
        ('pydantic', 'Pydantic'),
        ('tensorflow', 'TensorFlow'),
        ('pandas', 'Pandas'),
        ('numpy', 'NumPy'),
        ('sklearn', 'Scikit-learn'),
        ('loguru', 'Loguru'),
        ('yaml', 'PyYAML'),
        ('ta', 'TA (Technical Analysis)'),
        ('joblib', 'Joblib')
    ]
    
    failed_imports = []
    
    for package, name in required_packages:
        try:
            importlib.import_module(package)
            print(f"✅ {name}")
        except ImportError:
            print(f"❌ {name} - Not installed")
            failed_imports.append(package)
    
    if failed_imports:
        print(f"\n⚠️  Missing packages: {', '.join(failed_imports)}")
        print("Run: pip install -r requirements.txt")
        return False
    
    return True

def test_project_structure():
    """Test if project structure is correct."""
    print("\n🔍 Testing project structure...")
    
    required_files = [
        'main.py',
        'config.yaml',
        'requirements.txt',
        'README.md'
    ]
    
    required_dirs = [
        'config',
        'features', 
        'models',
        'utils'
    ]
    
    missing_files = []
    missing_dirs = []
    
    for file in required_files:
        if Path(file).exists():
            print(f"✅ {file}")
        else:
            print(f"❌ {file} - Missing")
            missing_files.append(file)
    
    for directory in required_dirs:
        if Path(directory).is_dir():
            print(f"✅ {directory}/")
        else:
            print(f"❌ {directory}/ - Missing")
            missing_dirs.append(directory)
    
    if missing_files or missing_dirs:
        print(f"\n⚠️  Missing components:")
        if missing_files:
            print(f"   Files: {', '.join(missing_files)}")
        if missing_dirs:
            print(f"   Directories: {', '.join(missing_dirs)}")
        return False
    
    return True

def test_config_loading():
    """Test if configuration can be loaded."""
    print("\n🔍 Testing configuration loading...")
    
    try:
        # Import and test config
        from config.config import config
        
        # Test basic config access
        server_config = config.get_server_config()
        model_config = config.get_model_config()
        
        print(f"✅ Configuration loaded successfully")
        print(f"   Server port: {server_config.get('port', 'N/A')}")
        print(f"   Model type: {model_config.get('type', 'N/A')}")
        return True
        
    except Exception as e:
        print(f"❌ Configuration loading failed: {e}")
        return False

def test_model_imports():
    """Test if model classes can be imported."""
    print("\n🔍 Testing model imports...")
    
    try:
        from models.lstm_model import LSTMModel
        from models.gru_model import GRUModel
        from models.transformer_model import TransformerModel
        from models.model_manager import ModelManager
        
        print("✅ All model classes imported successfully")
        return True
        
    except Exception as e:
        print(f"❌ Model import failed: {e}")
        return False

def test_feature_engineering():
    """Test if feature engineering can be imported."""
    print("\n🔍 Testing feature engineering...")
    
    try:
        from features.technical_indicators import TechnicalIndicators
        from features.feature_engineering import FeatureEngineer
        
        print("✅ Feature engineering classes imported successfully")
        return True
        
    except Exception as e:
        print(f"❌ Feature engineering import failed: {e}")
        return False

def test_utilities():
    """Test if utilities can be imported."""
    print("\n🔍 Testing utilities...")
    
    try:
        from utils.logger import setup_logger, get_logger
        from utils.helpers import validate_ohlcv_data, create_dataframe_from_ohlcv
        
        print("✅ Utility functions imported successfully")
        return True
        
    except Exception as e:
        print(f"❌ Utilities import failed: {e}")
        return False

def create_test_directories():
    """Create necessary directories for testing."""
    print("\n🔧 Creating test directories...")
    
    directories = [
        'models/saved',
        'logs',
        'data'
    ]
    
    for directory in directories:
        Path(directory).mkdir(parents=True, exist_ok=True)
        print(f"✅ Created: {directory}")

def main():
    """Run all tests."""
    print("🧪 Cryptocurrency AI Trading Service - Setup Test")
    print("=" * 60)
    
    tests = [
        ("Python Version", test_python_version),
        ("Package Imports", test_imports),
        ("Project Structure", test_project_structure),
        ("Configuration", test_config_loading),
        ("Model Classes", test_model_imports),
        ("Feature Engineering", test_feature_engineering),
        ("Utilities", test_utilities)
    ]
    
    results = []
    
    for test_name, test_func in tests:
        try:
            result = test_func()
            results.append((test_name, result))
        except Exception as e:
            print(f"❌ {test_name} test crashed: {e}")
            results.append((test_name, False))
    
    # Create directories
    create_test_directories()
    
    # Summary
    print("\n📊 Test Summary")
    print("=" * 60)
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for test_name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{status} {test_name}")
    
    print(f"\n🎯 Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n🎉 All tests passed! Your setup is ready.")
        print("\n📋 Next steps:")
        print("   1. Start the service: python main.py or ./start.sh")
        print("   2. Check health: curl http://localhost:8000/health")
        print("   3. Run example: python example_client.py")
        print("   4. View docs: http://localhost:8000/docs")
        return True
    else:
        print(f"\n⚠️  {total - passed} test(s) failed. Please fix the issues above.")
        return False

if __name__ == "__main__":
    try:
        success = main()
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\n\n⏹️  Test interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Test suite failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1) 