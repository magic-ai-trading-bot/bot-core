#!/usr/bin/env python3
"""
Notification System for Async Tasks
Supports: Email, Slack, Discord, Telegram webhooks
"""

import os
import smtplib
import requests
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from typing import Optional, Dict, Any, List
from datetime import datetime
from utils.logger import get_logger

logger = get_logger("Notifications")

# Configuration from environment
SMTP_HOST = os.getenv("SMTP_HOST", "smtp.gmail.com")
SMTP_PORT = int(os.getenv("SMTP_PORT", "587"))
SMTP_USER = os.getenv("SMTP_USER", "")
SMTP_PASSWORD = os.getenv("SMTP_PASSWORD", "")
EMAIL_FROM = os.getenv("EMAIL_FROM", SMTP_USER)
EMAIL_TO = os.getenv("EMAIL_TO", "").split(",")  # Comma-separated list

SLACK_WEBHOOK_URL = os.getenv("SLACK_WEBHOOK_URL", "")
DISCORD_WEBHOOK_URL = os.getenv("DISCORD_WEBHOOK_URL", "")
TELEGRAM_BOT_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN", "")
TELEGRAM_CHAT_ID = os.getenv("TELEGRAM_CHAT_ID", "")

# Notification settings
NOTIFICATIONS_ENABLED = os.getenv("NOTIFICATIONS_ENABLED", "false").lower() == "true"
NOTIFICATION_CHANNELS = os.getenv("NOTIFICATION_CHANNELS", "").split(
    ","
)  # email,slack,discord,telegram


class NotificationLevel:
    """Notification severity levels"""

    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    CRITICAL = "critical"


def is_enabled() -> bool:
    """Check if notifications are enabled (dynamically checks env for test compatibility)"""
    enabled = os.getenv("NOTIFICATIONS_ENABLED", "false").lower() == "true"
    channels = os.getenv("NOTIFICATION_CHANNELS", "").split(",")
    return enabled and len(channels) > 0 and channels[0] != ""


def send_notification(
    title: str,
    message: str,
    level: str = NotificationLevel.INFO,
    data: Optional[Dict[str, Any]] = None,
) -> Dict[str, bool]:
    """
    Send notification to all configured channels

    Args:
        title: Notification title
        message: Notification message
        level: Severity level (info/warning/error/critical)
        data: Additional data to include

    Returns:
        Dict with success status for each channel
    """
    if not is_enabled():
        logger.debug("Notifications disabled, skipping")
        return {"skipped": True}

    results = {}
    timestamp = datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S UTC")

    # Prepare full message with metadata
    full_message = f"""
{message}

Level: {level.upper()}
Time: {timestamp}
"""

    if data:
        full_message += f"\nAdditional Data:\n{format_data(data)}"

    # Check notification channels dynamically (for test compatibility)
    channels = os.getenv("NOTIFICATION_CHANNELS", "").split(",")

    # Send to each enabled channel (with error handling to continue on failure)
    if "email" in channels:
        try:
            results["email"] = send_email(title, full_message, level)
        except Exception as e:
            logger.error(f"Failed to send email notification: {e}")
            results["email"] = {"status": "failed", "error": str(e)}

    if "slack" in channels:
        try:
            results["slack"] = send_slack(title, message, level, data)
        except Exception as e:
            logger.error(f"Failed to send Slack notification: {e}")
            results["slack"] = {"status": "failed", "error": str(e)}

    if "discord" in channels:
        try:
            results["discord"] = send_discord(title, message, level, data)
        except Exception as e:
            logger.error(f"Failed to send Discord notification: {e}")
            results["discord"] = {"status": "failed", "error": str(e)}

    if "telegram" in channels:
        try:
            results["telegram"] = send_telegram(title, message, level, data)
        except Exception as e:
            logger.error(f"Failed to send Telegram notification: {e}")
            results["telegram"] = {"status": "failed", "error": str(e)}

    return results


# =============================================================================
# EMAIL NOTIFICATIONS
# =============================================================================


def send_email(title: str, message: str, level: str) -> Dict[str, Any]:
    """Send email notification via SMTP"""
    try:
        # Get SMTP config (check env vars dynamically for tests)
        smtp_user = os.getenv("SMTP_USER", SMTP_USER)
        smtp_password = os.getenv("SMTP_PASSWORD", SMTP_PASSWORD)
        smtp_host = os.getenv("SMTP_HOST", SMTP_HOST)
        smtp_port = int(os.getenv("SMTP_PORT", SMTP_PORT))

        msg = MIMEMultipart()
        msg["From"] = EMAIL_FROM or smtp_user
        msg["To"] = ", ".join(EMAIL_TO) if EMAIL_TO else ""
        msg["Subject"] = f"[{level.upper()}] {title}"

        # Add message body
        msg.attach(MIMEText(message, "plain"))

        # Connect to SMTP server
        with smtplib.SMTP(smtp_host, smtp_port) as server:
            server.starttls()
            server.login(smtp_user, smtp_password)
            server.send_message(msg)

        logger.info(f"📧 Email sent: {title}")
        return {"status": "success"}

    except Exception as e:
        logger.error(f"❌ Failed to send email: {e}")
        return {"status": "failed", "error": str(e)}


# =============================================================================
# SLACK NOTIFICATIONS
# =============================================================================


def send_slack(
    title: str,
    message: str,
    level: str,
    data: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    """Send notification to Slack via webhook"""
    try:
        # Get webhook URL (check env vars dynamically for tests)
        webhook_url = os.getenv("SLACK_WEBHOOK_URL", SLACK_WEBHOOK_URL)

        # Check if webhook URL is configured
        if not webhook_url:
            return {"status": "failed", "error": "SLACK_WEBHOOK_URL not configured"}

        # Color based on level
        color_map = {
            NotificationLevel.INFO: "#36a64f",  # Green
            NotificationLevel.WARNING: "#ff9900",  # Orange
            NotificationLevel.ERROR: "#ff0000",  # Red
            NotificationLevel.CRITICAL: "#990000",  # Dark red
        }
        color = color_map.get(level, "#808080")

        # Emoji based on level
        emoji_map = {
            NotificationLevel.INFO: "ℹ️",
            NotificationLevel.WARNING: "⚠️",
            NotificationLevel.ERROR: "❌",
            NotificationLevel.CRITICAL: "🚨",
        }
        emoji = emoji_map.get(level, "📢")

        # Build Slack message
        payload = {
            "attachments": [
                {
                    "color": color,
                    "title": f"{emoji} {title}",
                    "text": message,
                    "fields": [],
                    "footer": "Bot Core Async Tasks",
                    "ts": int(datetime.utcnow().timestamp()),
                }
            ]
        }

        # Add data fields
        if data:
            for key, value in data.items():
                payload["attachments"][0]["fields"].append(
                    {
                        "title": key.replace("_", " ").title(),
                        "value": str(value),
                        "short": True,
                    }
                )

        # Send to Slack
        response = requests.post(
            webhook_url,
            json=payload,
            timeout=10,
        )

        if response.status_code != 200:
            return {"status": "failed", "error": f"HTTP {response.status_code}"}

        logger.info(f"📢 Slack notification sent: {title}")
        return {"status": "success"}

    except Exception as e:
        logger.error(f"❌ Failed to send Slack notification: {e}")
        return {"status": "failed", "error": str(e)}


# =============================================================================
# DISCORD NOTIFICATIONS
# =============================================================================


def send_discord(
    title: str,
    message: str,
    level: str,
    data: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    """Send notification to Discord via webhook"""
    try:
        # Get webhook URL (check env vars dynamically for tests)
        webhook_url = os.getenv("DISCORD_WEBHOOK_URL", DISCORD_WEBHOOK_URL)

        # Check if webhook URL is configured
        if not webhook_url:
            return {"status": "failed", "error": "DISCORD_WEBHOOK_URL not configured"}

        # Color based on level (Discord uses integer colors)
        color_map = {
            NotificationLevel.INFO: 3447003,  # Blue
            NotificationLevel.WARNING: 16776960,  # Yellow
            NotificationLevel.ERROR: 16711680,  # Red
            NotificationLevel.CRITICAL: 10038562,  # Dark red
        }
        color = color_map.get(level, 8421504)

        # Emoji based on level
        emoji_map = {
            NotificationLevel.INFO: "ℹ️",
            NotificationLevel.WARNING: "⚠️",
            NotificationLevel.ERROR: "❌",
            NotificationLevel.CRITICAL: "🚨",
        }
        emoji = emoji_map.get(level, "📢")

        # Build Discord embed
        embed = {
            "title": f"{emoji} {title}",
            "description": message,
            "color": color,
            "fields": [],
            "footer": {
                "text": "Bot Core Async Tasks",
            },
            "timestamp": datetime.utcnow().isoformat(),
        }

        # Add data fields
        if data:
            for key, value in data.items():
                embed["fields"].append(
                    {
                        "name": key.replace("_", " ").title(),
                        "value": str(value),
                        "inline": True,
                    }
                )

        payload = {"embeds": [embed]}

        # Send to Discord
        response = requests.post(
            webhook_url,
            json=payload,
            timeout=10,
        )

        if response.status_code == 429:
            retry_after = response.json().get("retry_after", 1.5)
            return {"status": "failed", "error": f"Rate limited, retry after {retry_after}s"}

        if response.status_code not in [200, 204]:
            return {"status": "failed", "error": f"HTTP {response.status_code}"}

        logger.info(f"📢 Discord notification sent: {title}")
        return {"status": "success"}

    except Exception as e:
        logger.error(f"❌ Failed to send Discord notification: {e}")
        return {"status": "failed", "error": str(e)}


# =============================================================================
# TELEGRAM NOTIFICATIONS
# =============================================================================


def send_telegram(
    title: str,
    message: str,
    level: str,
    data: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    """Send notification to Telegram via bot API"""
    try:
        # Get Telegram config (check env vars dynamically for tests)
        bot_token = os.getenv("TELEGRAM_BOT_TOKEN", TELEGRAM_BOT_TOKEN)
        chat_id = os.getenv("TELEGRAM_CHAT_ID", TELEGRAM_CHAT_ID)

        # Check if Telegram credentials are configured
        if not bot_token or not chat_id:
            return {"status": "failed", "error": "TELEGRAM_BOT_TOKEN or TELEGRAM_CHAT_ID not configured"}

        # Emoji based on level
        emoji_map = {
            NotificationLevel.INFO: "ℹ️",
            NotificationLevel.WARNING: "⚠️",
            NotificationLevel.ERROR: "❌",
            NotificationLevel.CRITICAL: "🚨",
        }
        emoji = emoji_map.get(level, "📢")

        # Build message
        telegram_message = f"{emoji} <b>{title}</b>\n\n{message}"

        if data:
            telegram_message += "\n\n<b>Additional Data:</b>"
            for key, value in data.items():
                telegram_message += f"\n• {key.replace('_', ' ').title()}: {value}"

        # Send to Telegram
        url = f"https://api.telegram.org/bot{bot_token}/sendMessage"
        payload = {
            "chat_id": chat_id,
            "text": telegram_message,
            "parse_mode": "HTML",
        }

        response = requests.post(url, json=payload, timeout=10)

        if response.status_code != 200:
            return {"status": "failed", "error": f"HTTP {response.status_code}"}

        logger.info(f"📢 Telegram notification sent: {title}")
        return {"status": "success"}

    except Exception as e:
        logger.error(f"❌ Failed to send Telegram notification: {e}")
        return {"status": "failed", "error": str(e)}


# =============================================================================
# HELPER FUNCTIONS
# =============================================================================


def format_data(data: Dict[str, Any], indent: int = 0) -> str:
    """Format data dict for display"""
    lines = []
    prefix = "  " * indent

    for key, value in data.items():
        if isinstance(value, dict):
            lines.append(f"{prefix}{key}:")
            lines.append(format_data(value, indent + 1))
        elif isinstance(value, list):
            lines.append(f"{prefix}{key}: [{len(value)} items]")
        else:
            lines.append(f"{prefix}{key}: {value}")

    return "\n".join(lines)


# =============================================================================
# CONVENIENCE FUNCTIONS
# =============================================================================


def send_info(title: str, message: str, data: Optional[Dict] = None) -> Dict[str, bool]:
    """Send INFO notification"""
    return send_notification(title, message, NotificationLevel.INFO, data)


def send_warning(
    title: str, message: str, data: Optional[Dict] = None
) -> Dict[str, bool]:
    """Send WARNING notification"""
    return send_notification(title, message, NotificationLevel.WARNING, data)


def send_error(
    title: str, message: str, data: Optional[Dict] = None
) -> Dict[str, bool]:
    """Send ERROR notification"""
    return send_notification(title, message, NotificationLevel.ERROR, data)


def send_critical(
    title: str, message: str, data: Optional[Dict] = None
) -> Dict[str, bool]:
    """Send CRITICAL notification"""
    return send_notification(title, message, NotificationLevel.CRITICAL, data)


def send_health_alert(service: str, error: str) -> Dict[str, bool]:
    """Send health check alert"""
    return send_critical(
        title=f"Service Down: {service}",
        message=f"Health check failed for {service}",
        data={"service": service, "error": error},
    )


def send_performance_alert(metrics: Dict[str, Any]) -> Dict[str, bool]:
    """Send performance degradation alert"""
    return send_warning(
        title="Performance Degradation Detected",
        message="Trading performance has dropped below acceptable thresholds",
        data=metrics,
    )


def send_cost_alert(cost_data: Dict[str, Any]) -> Dict[str, bool]:
    """Send API cost alert"""
    return send_warning(
        title="API Cost Alert",
        message="API costs have exceeded warning threshold",
        data=cost_data,
    )


def send_gpt4_analysis(analysis: Dict[str, Any]) -> Dict[str, bool]:
    """Send GPT-4 analysis result"""
    level = (
        NotificationLevel.WARNING
        if analysis.get("recommendation") == "retrain"
        else NotificationLevel.INFO
    )

    return send_notification(
        title="GPT-4 Self-Analysis Complete",
        message=f"Recommendation: {analysis.get('recommendation', 'N/A').upper()}\n"
        f"Confidence: {analysis.get('confidence', 0):.0%}\n"
        f"Reasoning: {analysis.get('reasoning', 'N/A')[:200]}...",
        level=level,
        data={
            "urgency": analysis.get("urgency", "low"),
            "suggested_actions": ", ".join(analysis.get("suggested_actions", [])),
            "estimated_improvement": analysis.get("estimated_improvement", "N/A"),
        },
    )


def send_retrain_complete(results: Dict[str, Any]) -> Dict[str, bool]:
    """Send model retraining completion notification"""
    successful = sum(
        1 for m in results.get("models", {}).values() if m.get("status") == "success"
    )
    total = len(results.get("models", {}))

    return send_info(
        title="Model Retraining Complete",
        message=f"Adaptive retraining completed: {successful}/{total} models successful",
        data=results.get("models", {}),
    )
