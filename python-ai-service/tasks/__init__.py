"""Celery tasks for async operations"""

# Lazy loading using __getattr__ to avoid importing celery at module load time
# Allows @patch("tasks.ai_improvement.storage") to work in tests
__all__ = ["ai_improvement", "monitoring", "ml_tasks", "backtest_tasks"]


def __getattr__(name):
    """Lazy import task modules to avoid importing celery unnecessarily"""
    if name in __all__:
        import importlib

        module = importlib.import_module(f".{name}", __package__)
        globals()[name] = module
        return module
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")
