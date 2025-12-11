#!/usr/bin/env python3
"""
Celery Application Configuration for Bot Core
Handles async tasks: ML training, backtesting, bulk analysis, scheduled jobs
"""

import os

from celery import Celery
from celery.schedules import crontab
from kombu import Exchange, Queue

# RabbitMQ connection
RABBITMQ_USER = os.getenv("RABBITMQ_USER", "admin")
RABBITMQ_PASSWORD = os.getenv("RABBITMQ_PASSWORD", "rabbitmq_default_password")
RABBITMQ_HOST = os.getenv("RABBITMQ_HOST", "rabbitmq")
RABBITMQ_PORT = os.getenv("RABBITMQ_PORT", "5672")
RABBITMQ_VHOST = os.getenv("RABBITMQ_VHOST", "bot-core")

# Redis for results backend (optional, can also use RabbitMQ)
REDIS_PASSWORD = os.getenv("REDIS_PASSWORD", "redis_default_password")
REDIS_HOST = os.getenv("REDIS_HOST", "redis")
REDIS_PORT = os.getenv("REDIS_PORT", "6379")

# Broker URL (RabbitMQ)
BROKER_URL = f"amqp://{RABBITMQ_USER}:{RABBITMQ_PASSWORD}@{RABBITMQ_HOST}:{RABBITMQ_PORT}/{RABBITMQ_VHOST}"

# Results backend (Redis)
RESULT_BACKEND = f"redis://:{REDIS_PASSWORD}@{REDIS_HOST}:{REDIS_PORT}/0"

# Create Celery app
app = Celery(
    "bot_core",
    broker=BROKER_URL,
    backend=RESULT_BACKEND,
    include=[
        "tasks.ml_tasks",
        "tasks.backtest_tasks",
        "tasks.monitoring",
        "tasks.ai_improvement",
    ],
)

# Configure Celery
app.conf.update(
    # Task settings
    task_serializer="json",
    accept_content=["json"],
    result_serializer="json",
    timezone="UTC",
    enable_utc=True,
    # Task execution
    task_track_started=True,
    task_time_limit=3600,  # 1 hour max
    task_soft_time_limit=3300,  # 55 minutes soft limit
    task_acks_late=True,  # Acknowledge after task completes
    worker_prefetch_multiplier=1,  # One task at a time per worker
    # Result backend
    result_expires=86400,  # Results expire after 24 hours
    result_backend_transport_options={
        "master_name": "mymaster",
    },
    # Broker settings
    broker_connection_retry_on_startup=True,
    broker_connection_retry=True,
    broker_connection_max_retries=10,
    # Task routes - route different tasks to different queues
    task_routes={
        "tasks.ml_tasks.train_model": {"queue": "ml_training"},
        "tasks.ml_tasks.bulk_analysis": {"queue": "bulk_analysis"},
        "tasks.backtest_tasks.backtest_strategy": {"queue": "backtesting"},
        "tasks.backtest_tasks.optimize_strategy": {"queue": "optimization"},
        "tasks.monitoring.*": {"queue": "scheduled"},
        "tasks.ai_improvement.*": {"queue": "scheduled"},
    },
    # Task priority
    task_default_priority=5,
    task_inherit_parent_priority=True,
    # Monitoring
    worker_send_task_events=True,
    task_send_sent_event=True,
)

# Define custom queues with specific exchanges
app.conf.task_queues = (
    # ML Training queue - long-running ML tasks
    Queue(
        "ml_training",
        Exchange("ai.predictions", type="topic"),
        routing_key="ml.train.*",
        queue_arguments={
            "x-message-ttl": 3600000,  # 1 hour TTL
            "x-max-length": 100,
        },
    ),
    # Bulk Analysis queue - parallel analysis tasks
    Queue(
        "bulk_analysis",
        Exchange("ai.predictions", type="topic"),
        routing_key="ml.analyze.*",
        queue_arguments={
            "x-message-ttl": 1800000,  # 30 min TTL
            "x-max-length": 500,
        },
    ),
    # Backtesting queue
    Queue(
        "backtesting",
        Exchange("trading.events", type="topic"),
        routing_key="backtest.*",
        queue_arguments={
            "x-message-ttl": 7200000,  # 2 hours TTL
            "x-max-length": 50,
        },
    ),
    # Strategy Optimization queue
    Queue(
        "optimization",
        Exchange("trading.events", type="topic"),
        routing_key="optimize.*",
        queue_arguments={
            "x-message-ttl": 7200000,  # 2 hours TTL
            "x-max-length": 50,
        },
    ),
    # Scheduled tasks queue
    Queue(
        "scheduled",
        Exchange("ai.predictions", type="topic"),
        routing_key="scheduled.*",
        queue_arguments={
            "x-message-ttl": 3600000,
            "x-max-length": 100,
        },
    ),
)

# Celery Beat schedule - NEW INTELLIGENT JOBS (replaced 4 bad time-based jobs)
app.conf.beat_schedule = {
    # System health monitoring (every 15 minutes)
    "system-health-check": {
        "task": "tasks.monitoring.system_health_check",
        "schedule": crontab(minute="*/15"),  # Every 15 minutes
    },
    # Daily portfolio report (8 AM UTC)
    "daily-portfolio-report": {
        "task": "tasks.monitoring.daily_portfolio_report",
        "schedule": crontab(hour=8, minute=0),  # 8:00 AM daily
    },
    # Daily API cost report (9 AM UTC)
    "daily-api-cost-report": {
        "task": "tasks.monitoring.daily_api_cost_report",
        "schedule": crontab(hour=9, minute=0),  # 9:00 AM daily
    },
    # Daily performance analysis (1 AM UTC)
    "daily-performance-analysis": {
        "task": "tasks.monitoring.daily_performance_analysis",
        "schedule": crontab(hour=1, minute=0),  # 1:00 AM daily
    },
    # GPT-4 self-analysis for adaptive retraining (every hour)
    "gpt4-self-analysis": {
        "task": "tasks.ai_improvement.gpt4_self_analysis",
        "schedule": crontab(minute=0),  # Every hour at minute 0
        "kwargs": {"force_analysis": True},  # Always run analysis
    },
    # Analyze recent closed trades with GPT-4 (every 30 minutes)
    "analyze-recent-trades": {
        "task": "tasks.ai_improvement.analyze_recent_trades",
        "schedule": crontab(minute="*/30"),  # Every 30 minutes
        "kwargs": {
            "only_losing": False,
            "limit": 10,
        },  # Analyze all trades, max 10 per run
    },
}

# Task result ignore setting
app.conf.result_ignore_backend = False

if __name__ == "__main__":
    app.start()
