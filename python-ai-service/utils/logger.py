import sys
from pathlib import Path
from loguru import logger
from config.config import config


def setup_logger() -> None:
    """Setup loguru logger with configuration from config file."""
    # Remove default handler
    logger.remove()

    # Get logging configuration
    log_config = config.get_logging_config()

    # Ensure log directory exists
    log_file = Path(log_config.get("file", "./logs/trading_ai.log"))
    log_file.parent.mkdir(parents=True, exist_ok=True)

    # Add console handler
    logger.add(
        sys.stdout,
        format=log_config.get(
            "format",
            "{time:YYYY-MM-DD HH:mm:ss} | {level} | {name}:{function}:{line} | {message}",
        ),
        level=log_config.get("level", "INFO"),
        colorize=True,
    )

    # Add file handler
    logger.add(
        log_file,
        format=log_config.get(
            "format",
            "{time:YYYY-MM-DD HH:mm:ss} | {level} | {name}:{function}:{line} | {message}",
        ),
        level=log_config.get("level", "INFO"),
        rotation=log_config.get("rotation", "10 MB"),
        retention=log_config.get("retention", "7 days"),
        compression="zip",
    )

    logger.info("Logger initialized successfully")


def get_logger(name: str = None):
    """Get logger instance with optional name."""
    if name:
        return logger.bind(name=name)
    return logger
