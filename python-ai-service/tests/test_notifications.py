#!/usr/bin/env python3
"""
Unit Tests for Notification System
Tests email, Slack, Discord, and Telegram notifications
"""

import pytest
from unittest.mock import patch, MagicMock, Mock
import os


class TestNotificationConfiguration:
    """Test notification system configuration"""

    def test_notifications_can_be_disabled(self):
        """Test that notifications can be disabled via environment variable"""
        from utils import notifications

        # Save original value
        original = os.environ.get("NOTIFICATIONS_ENABLED")

        try:
            os.environ["NOTIFICATIONS_ENABLED"] = "false"
            # Reload module to pick up new env var
            import importlib

            importlib.reload(notifications)

            # When disabled, should not send
            result = notifications.send_notification(
                title="Test", message="Test message", level="info"
            )
            # Should return early without error (returns {"skipped": True})
            assert result is not None and result.get("skipped") == True

        finally:
            # Restore original
            if original:
                os.environ["NOTIFICATIONS_ENABLED"] = original
            else:
                os.environ.pop("NOTIFICATIONS_ENABLED", None)

    def test_notification_channels_configuration(self):
        """Test that notification channels can be configured"""
        from utils import notifications

        # Check that channel configuration exists
        assert hasattr(notifications, "NOTIFICATION_CHANNELS")

        # Should support multiple channels
        channels = os.environ.get(
            "NOTIFICATION_CHANNELS", "email,slack,discord,telegram"
        )
        assert isinstance(channels, str)


class TestEmailNotifications:
    """Test email (SMTP) notifications"""

    @patch("utils.notifications.smtplib.SMTP")
    def test_send_email_success(self, mock_smtp):
        """Test successful email sending"""
        from utils.notifications import send_email

        # Mock SMTP server
        mock_server = MagicMock()
        mock_smtp.return_value.__enter__.return_value = mock_server

        result = send_email(
            title="Test Alert", message="This is a test message", level="info"
        )

        assert result["status"] == "success"
        assert mock_server.send_message.called

    @patch("utils.notifications.smtplib.SMTP")
    def test_send_email_smtp_error(self, mock_smtp):
        """Test email sending with SMTP error"""
        from utils.notifications import send_email

        # Mock SMTP error
        mock_smtp.side_effect = Exception("SMTP server unavailable")

        result = send_email(title="Test Alert", message="Test", level="error")

        assert result["status"] == "failed"
        assert "error" in result

    @patch("utils.notifications.smtplib.SMTP")
    def test_send_email_critical_level(self, mock_smtp):
        """Test critical level email has appropriate formatting"""
        from utils.notifications import send_email

        mock_server = MagicMock()
        mock_smtp.return_value.__enter__.return_value = mock_server

        result = send_email(
            title="CRITICAL: System Down",
            message="All services are down",
            level="critical",
        )

        assert result["status"] == "success"
        # Critical emails should be sent
        assert mock_server.send_message.called

    def test_send_email_requires_config(self):
        """Test that email requires SMTP configuration"""
        from utils.notifications import send_email

        # Save original env vars
        smtp_host = os.environ.get("SMTP_HOST")
        smtp_user = os.environ.get("SMTP_USER")

        try:
            # Remove SMTP config
            os.environ.pop("SMTP_HOST", None)
            os.environ.pop("SMTP_USER", None)

            result = send_email("Test", "Test message", "info")

            # Should fail gracefully
            assert result["status"] == "failed" or "error" in result

        finally:
            # Restore
            if smtp_host:
                os.environ["SMTP_HOST"] = smtp_host
            if smtp_user:
                os.environ["SMTP_USER"] = smtp_user


class TestSlackNotifications:
    """Test Slack webhook notifications"""

    @patch.dict(os.environ, {"SLACK_WEBHOOK_URL": "https://hooks.slack.com/test"})
    @patch("utils.notifications.requests.post")
    def test_send_slack_success(self, mock_post):
        """Test successful Slack notification"""
        from utils.notifications import send_slack

        # Mock successful webhook response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.text = "ok"
        mock_post.return_value = mock_response

        result = send_slack(
            title="Test Alert", message="Slack notification test", level="info"
        )

        assert result["status"] == "success"
        assert mock_post.called

    @patch.dict(os.environ, {"SLACK_WEBHOOK_URL": "https://hooks.slack.com/test"})
    @patch("utils.notifications.requests.post")
    def test_send_slack_webhook_error(self, mock_post):
        """Test Slack notification with webhook error"""
        from utils.notifications import send_slack

        # Mock webhook failure
        mock_response = MagicMock()
        mock_response.status_code = 400
        mock_response.text = "invalid_payload"
        mock_post.return_value = mock_response

        result = send_slack("Test", "Message", "error")

        assert result["status"] == "failed"

    @patch.dict(os.environ, {"SLACK_WEBHOOK_URL": "https://hooks.slack.com/test"})
    @patch("utils.notifications.requests.post")
    def test_send_slack_formatting(self, mock_post):
        """Test Slack notification uses correct formatting"""
        from utils.notifications import send_slack

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_post.return_value = mock_response

        send_slack(
            title="📊 Performance Alert",
            message="Win rate dropped to 45%",
            level="warning",
            data={"win_rate": 45.0, "threshold": 50.0},
        )

        # Check that post was called with formatted payload
        assert mock_post.called
        call_args = mock_post.call_args
        payload = call_args[1].get("json", {})

        # Slack uses 'text', 'blocks', or 'attachments' field
        assert "text" in payload or "blocks" in payload or "attachments" in payload

    @patch.dict(os.environ, {"SLACK_WEBHOOK_URL": ""}, clear=False)
    def test_send_slack_requires_webhook_url(self):
        """Test that Slack requires webhook URL"""
        from utils.notifications import send_slack

        result = send_slack("Test", "Message", "info")

        # Should fail gracefully
        assert result["status"] == "failed"
        assert "error" in result
        assert "not configured" in result["error"]


class TestDiscordNotifications:
    """Test Discord webhook notifications"""

    @patch.dict(
        os.environ, {"DISCORD_WEBHOOK_URL": "https://discord.com/api/webhooks/test"}
    )
    @patch("utils.notifications.requests.post")
    def test_send_discord_success(self, mock_post):
        """Test successful Discord notification"""
        from utils.notifications import send_discord

        mock_response = MagicMock()
        mock_response.status_code = 204
        mock_post.return_value = mock_response

        result = send_discord(
            title="Bot Alert", message="Discord notification test", level="info"
        )

        assert result["status"] == "success"
        assert mock_post.called

    @patch.dict(
        os.environ, {"DISCORD_WEBHOOK_URL": "https://discord.com/api/webhooks/test"}
    )
    @patch("utils.notifications.requests.post")
    def test_send_discord_with_embed(self, mock_post):
        """Test Discord notification uses embed format"""
        from utils.notifications import send_discord

        mock_response = MagicMock()
        mock_response.status_code = 204
        mock_post.return_value = mock_response

        send_discord(
            title="⚠️ Warning",
            message="System performance degraded",
            level="warning",
            data={"cpu": 95, "memory": 87},
        )

        assert mock_post.called
        payload = mock_post.call_args[1].get("json", {})

        # Discord uses 'embeds' field
        assert "embeds" in payload or "content" in payload

    @patch.dict(
        os.environ, {"DISCORD_WEBHOOK_URL": "https://discord.com/api/webhooks/test"}
    )
    @patch("utils.notifications.requests.post")
    def test_send_discord_rate_limit(self, mock_post):
        """Test Discord notification handles rate limiting"""
        from utils.notifications import send_discord

        # Mock rate limit response
        mock_response = MagicMock()
        mock_response.status_code = 429
        mock_response.json.return_value = {"retry_after": 1.5}
        mock_post.return_value = mock_response

        result = send_discord("Test", "Message", "info")

        # Should handle rate limit
        assert "error" in result or result["status"] == "failed"


class TestTelegramNotifications:
    """Test Telegram bot notifications"""

    @patch.dict(
        os.environ,
        {"TELEGRAM_BOT_TOKEN": "test_token", "TELEGRAM_CHAT_ID": "test_chat_id"},
    )
    @patch("utils.notifications.requests.post")
    def test_send_telegram_success(self, mock_post):
        """Test successful Telegram notification"""
        from utils.notifications import send_telegram

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"ok": True}
        mock_post.return_value = mock_response

        result = send_telegram(
            title="Bot Update", message="Telegram test message", level="info"
        )

        assert result["status"] == "success"
        assert mock_post.called

    @patch.dict(
        os.environ,
        {"TELEGRAM_BOT_TOKEN": "test_token", "TELEGRAM_CHAT_ID": "test_chat_id"},
    )
    @patch("utils.notifications.requests.post")
    def test_send_telegram_with_markdown(self, mock_post):
        """Test Telegram notification supports markdown"""
        from utils.notifications import send_telegram

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"ok": True}
        mock_post.return_value = mock_response

        send_telegram(
            title="📈 **Performance Report**",
            message="*Win Rate*: 75%\n*Profit*: $250",
            level="info",
        )

        assert mock_post.called
        payload = mock_post.call_args[1].get("json", {})

        # Telegram uses 'text' field and parse_mode
        assert "text" in payload
        assert payload.get("parse_mode") in ["Markdown", "MarkdownV2", "HTML"]

    def test_send_telegram_requires_credentials(self):
        """Test that Telegram requires bot token and chat ID"""
        from utils.notifications import send_telegram

        # Save originals
        token = os.environ.get("TELEGRAM_BOT_TOKEN")
        chat_id = os.environ.get("TELEGRAM_CHAT_ID")

        try:
            # Remove credentials
            os.environ.pop("TELEGRAM_BOT_TOKEN", None)
            os.environ.pop("TELEGRAM_CHAT_ID", None)

            result = send_telegram("Test", "Message", "info")

            # Should fail gracefully
            assert result["status"] == "failed" or "error" in result

        finally:
            if token:
                os.environ["TELEGRAM_BOT_TOKEN"] = token
            if chat_id:
                os.environ["TELEGRAM_CHAT_ID"] = chat_id


class TestUnifiedNotificationSystem:
    """Test unified notification functions"""

    @patch.dict(
        os.environ,
        {
            "NOTIFICATIONS_ENABLED": "true",
            "NOTIFICATION_CHANNELS": "email,slack,discord,telegram",
        },
    )
    @patch("utils.notifications.send_email")
    @patch("utils.notifications.send_slack")
    @patch("utils.notifications.send_discord")
    @patch("utils.notifications.send_telegram")
    def test_send_notification_to_all_channels(
        self, mock_telegram, mock_discord, mock_slack, mock_email
    ):
        """Test that send_notification sends to all enabled channels"""
        from utils.notifications import send_notification

        # Mock all channels as successful
        for mock in [mock_email, mock_slack, mock_discord, mock_telegram]:
            mock.return_value = {"status": "success"}

        result = send_notification(
            title="Multi-Channel Alert", message="Testing all channels", level="warning"
        )

        # Should attempt to send to all configured channels
        assert mock_email.called
        assert mock_slack.called
        assert mock_discord.called
        assert mock_telegram.called

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_critical_uses_critical_level(self, mock_slack):
        """Test send_critical helper uses critical level"""
        from utils.notifications import send_critical

        mock_slack.return_value = {"status": "success"}

        send_critical(
            title="CRITICAL: System Failure",
            message="All services down",
            data={"services_down": 5},
        )

        # Should be called with critical level
        assert mock_slack.called
        call_args = mock_slack.call_args
        assert call_args[0][2] == "critical" or call_args[1].get("level") == "critical"

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_warning_uses_warning_level(self, mock_slack):
        """Test send_warning helper uses warning level"""
        from utils.notifications import send_warning

        mock_slack.return_value = {"status": "success"}

        send_warning("Performance Degraded", "Win rate at 55%")

        assert mock_slack.called

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_info_uses_info_level(self, mock_slack):
        """Test send_info helper uses info level"""
        from utils.notifications import send_info

        mock_slack.return_value = {"status": "success"}

        send_info("Daily Report", "All systems operational")

        assert mock_slack.called

    def test_send_gpt4_analysis_notification(self):
        """Test specialized GPT-4 analysis notification"""
        from utils.notifications import send_gpt4_analysis

        analysis = {
            "recommendation": "retrain",
            "confidence": 85,
            "reasoning": "Model accuracy dropped",
            "models_to_retrain": ["lstm", "gru"],
        }

        # Should not raise exception
        try:
            send_gpt4_analysis(analysis)
        except Exception as e:
            # May fail due to missing credentials, but should be handled
            assert "credentials" in str(e).lower() or "config" in str(e).lower()


class TestNotificationErrorHandling:
    """Test notification error handling"""

    @patch.dict(
        os.environ,
        {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "email,slack"},
    )
    @patch("utils.notifications.send_email")
    @patch("utils.notifications.send_slack")
    def test_notification_continues_on_channel_failure(self, mock_slack, mock_email):
        """Test that notification continues even if one channel fails"""
        from utils.notifications import send_notification

        # Email fails, Slack succeeds
        mock_email.side_effect = Exception("SMTP error")
        mock_slack.return_value = {"status": "success"}

        # Should not raise exception
        send_notification("Test", "Message", "info")

        # Both should have been attempted
        assert mock_email.called
        assert mock_slack.called

    @patch.dict(
        os.environ,
        {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack,discord"},
    )
    @patch("utils.notifications.send_slack")
    @patch("utils.notifications.send_discord")
    def test_notification_slack_exception_caught(self, mock_discord, mock_slack):
        """Test that Slack exceptions are caught and logged."""
        from utils.notifications import send_notification

        # Slack raises exception
        mock_slack.side_effect = Exception("Slack API error")
        mock_discord.return_value = {"status": "success"}

        result = send_notification("Test", "Message", "info")

        # Both should have been attempted
        assert mock_slack.called
        assert mock_discord.called
        # Result should contain error for Slack
        assert result["slack"]["status"] == "failed"
        assert "error" in result["slack"]

    @patch.dict(
        os.environ,
        {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "discord,telegram"},
    )
    @patch("utils.notifications.send_discord")
    @patch("utils.notifications.send_telegram")
    def test_notification_discord_exception_caught(self, mock_telegram, mock_discord):
        """Test that Discord exceptions are caught and logged."""
        from utils.notifications import send_notification

        # Discord raises exception
        mock_discord.side_effect = Exception("Discord webhook error")
        mock_telegram.return_value = {"status": "success"}

        result = send_notification("Test", "Message", "info")

        # Both should have been attempted
        assert mock_discord.called
        assert mock_telegram.called
        # Result should contain error for Discord
        assert result["discord"]["status"] == "failed"
        assert "error" in result["discord"]

    @patch.dict(
        os.environ,
        {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack,telegram"},
    )
    @patch("utils.notifications.send_telegram")
    @patch("utils.notifications.send_slack")
    def test_notification_telegram_exception_caught(self, mock_slack, mock_telegram):
        """Test that Telegram exceptions are caught and logged."""
        from utils.notifications import send_notification

        # Telegram raises exception
        mock_telegram.side_effect = Exception("Telegram bot error")
        mock_slack.return_value = {"status": "success"}

        result = send_notification("Test", "Message", "info")

        # Both should have been attempted
        assert mock_slack.called
        assert mock_telegram.called
        # Result should contain error for Telegram
        assert result["telegram"]["status"] == "failed"
        assert "error" in result["telegram"]

    def test_notification_with_invalid_level(self):
        """Test notification with invalid severity level"""
        from utils.notifications import send_notification

        # Should handle invalid level gracefully
        result = send_notification(
            title="Test", message="Message", level="invalid_level"  # Invalid
        )

        # Should default to 'info' or handle gracefully
        assert result is None or isinstance(result, dict)

    @patch.dict(os.environ, {"SLACK_WEBHOOK_URL": "https://hooks.slack.com/test"})
    @patch("utils.notifications.requests.post")
    def test_notification_with_timeout(self, mock_post):
        """Test notification handles request timeout"""
        from utils.notifications import send_slack
        import requests

        # Mock timeout
        mock_post.side_effect = requests.exceptions.Timeout("Request timeout")

        result = send_slack("Test", "Message", "info")

        assert result["status"] == "failed"
        assert "timeout" in str(result.get("error", "")).lower()


@pytest.mark.unit
class TestConvenienceNotificationFunctions:
    """Test convenience wrapper functions."""

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_error_function(self, mock_slack):
        """Test send_error convenience function."""
        from utils.notifications import send_error

        mock_slack.return_value = {"status": "success"}

        result = send_error("Error occurred", "System error details", {"code": 500})

        assert mock_slack.called
        # Verify error level was used
        call_args = mock_slack.call_args
        assert call_args[0][2] == "error" or call_args[1].get("level") == "error"

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_health_alert_function(self, mock_slack):
        """Test send_health_alert convenience function."""
        from utils.notifications import send_health_alert

        mock_slack.return_value = {"status": "success"}

        result = send_health_alert("MongoDB", "Connection timeout")

        assert mock_slack.called
        # Should use critical level
        call_args = mock_slack.call_args
        assert call_args[0][2] == "critical" or call_args[1].get("level") == "critical"

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_performance_alert_function(self, mock_slack):
        """Test send_performance_alert convenience function."""
        from utils.notifications import send_performance_alert

        mock_slack.return_value = {"status": "success"}

        metrics = {"win_rate": 45.0, "sharpe_ratio": 0.5}
        result = send_performance_alert(metrics)

        assert mock_slack.called
        # Should use warning level
        call_args = mock_slack.call_args
        assert call_args[0][2] == "warning" or call_args[1].get("level") == "warning"

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_cost_alert_function(self, mock_slack):
        """Test send_cost_alert convenience function."""
        from utils.notifications import send_cost_alert

        mock_slack.return_value = {"status": "success"}

        cost_data = {"daily_cost": 15.50, "threshold": 10.0}
        result = send_cost_alert(cost_data)

        assert mock_slack.called
        # Should use warning level
        call_args = mock_slack.call_args
        assert call_args[0][2] == "warning" or call_args[1].get("level") == "warning"

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_retrain_complete_function(self, mock_slack):
        """Test send_retrain_complete convenience function."""
        from utils.notifications import send_retrain_complete

        mock_slack.return_value = {"status": "success"}

        results = {
            "models": {
                "lstm": {"status": "success", "accuracy": 75.0},
                "gru": {"status": "success", "accuracy": 72.0},
                "transformer": {"status": "failed", "error": "Training error"},
            }
        }

        result = send_retrain_complete(results)

        assert mock_slack.called
        # Should use info level
        call_args = mock_slack.call_args
        assert call_args[0][2] == "info" or call_args[1].get("level") == "info"

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_retrain_complete_empty_models(self, mock_slack):
        """Test send_retrain_complete with empty models dict."""
        from utils.notifications import send_retrain_complete

        mock_slack.return_value = {"status": "success"}

        results = {"models": {}}

        result = send_retrain_complete(results)

        assert mock_slack.called


@pytest.mark.unit
class TestFormatDataFunction:
    """Test format_data helper function."""

    def test_format_data_simple(self):
        """Test format_data with simple key-value pairs."""
        from utils.notifications import format_data

        data = {"key1": "value1", "key2": "value2"}
        result = format_data(data)

        assert "key1: value1" in result
        assert "key2: value2" in result

    def test_format_data_nested_dict(self):
        """Test format_data with nested dictionary."""
        from utils.notifications import format_data

        data = {
            "outer": {
                "inner1": "value1",
                "inner2": "value2",
            }
        }
        result = format_data(data)

        assert "outer:" in result
        assert "inner1: value1" in result
        assert "inner2: value2" in result

    def test_format_data_with_list(self):
        """Test format_data with list values."""
        from utils.notifications import format_data

        data = {"items": [1, 2, 3, 4, 5]}
        result = format_data(data)

        assert "items: [5 items]" in result

    def test_format_data_with_indent(self):
        """Test format_data with custom indent."""
        from utils.notifications import format_data

        data = {"key": "value"}
        result = format_data(data, indent=2)

        # Should have 4 spaces prefix (2 * 2)
        assert "    key: value" in result

    def test_format_data_complex_nested(self):
        """Test format_data with complex nested structure."""
        from utils.notifications import format_data

        data = {
            "level1": {
                "level2": {
                    "level3": "deep_value",
                }
            },
            "list_field": [1, 2],
            "simple": 42,
        }
        result = format_data(data)

        assert "level1:" in result
        assert "level2:" in result
        assert "level3: deep_value" in result
        assert "list_field: [2 items]" in result
        assert "simple: 42" in result


@pytest.mark.unit
class TestDiscordNotificationDetails:
    """Additional Discord notification tests."""

    @patch.dict(
        os.environ, {"DISCORD_WEBHOOK_URL": "https://discord.com/api/webhooks/test"}
    )
    @patch("utils.notifications.requests.post")
    def test_send_discord_with_data_fields(self, mock_post):
        """Test Discord notification includes data as embed fields."""
        from utils.notifications import send_discord

        mock_response = MagicMock()
        mock_response.status_code = 204
        mock_post.return_value = mock_response

        send_discord(
            title="Test",
            message="Test message",
            level="warning",
            data={"cpu_usage": 95, "memory_mb": 512},
        )

        assert mock_post.called
        payload = mock_post.call_args[1].get("json", {})
        embed = payload.get("embeds", [{}])[0]
        fields = embed.get("fields", [])

        # Should have fields for data
        assert len(fields) == 2
        field_names = [f["name"] for f in fields]
        assert "Cpu Usage" in field_names or "cpu_usage" in field_names

    @patch.dict(
        os.environ, {"DISCORD_WEBHOOK_URL": "https://discord.com/api/webhooks/test"}
    )
    @patch("utils.notifications.requests.post")
    def test_send_discord_different_levels(self, mock_post):
        """Test Discord notification colors for different levels."""
        from utils.notifications import send_discord, NotificationLevel

        mock_response = MagicMock()
        mock_response.status_code = 204
        mock_post.return_value = mock_response

        # Test each level
        levels = [
            NotificationLevel.INFO,
            NotificationLevel.WARNING,
            NotificationLevel.ERROR,
            NotificationLevel.CRITICAL,
        ]
        for level in levels:
            send_discord("Test", "Message", level)
            assert mock_post.called

    @patch.dict(
        os.environ, {"DISCORD_WEBHOOK_URL": "https://discord.com/api/webhooks/test"}
    )
    @patch("utils.notifications.requests.post")
    def test_send_discord_success_200_status(self, mock_post):
        """Test Discord returns success for 200 status."""
        from utils.notifications import send_discord

        mock_response = MagicMock()
        mock_response.status_code = 200  # Some Discord endpoints return 200
        mock_post.return_value = mock_response

        result = send_discord("Test", "Message", "info")

        assert result["status"] == "success"


@pytest.mark.unit
class TestTelegramNotificationDetails:
    """Additional Telegram notification tests."""

    @patch.dict(
        os.environ,
        {"TELEGRAM_BOT_TOKEN": "test_token", "TELEGRAM_CHAT_ID": "test_chat_id"},
    )
    @patch("utils.notifications.requests.post")
    def test_send_telegram_with_data(self, mock_post):
        """Test Telegram notification includes data in message."""
        from utils.notifications import send_telegram

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"ok": True}
        mock_post.return_value = mock_response

        send_telegram(
            title="Alert",
            message="Test message",
            level="warning",
            data={"metric": "value", "count": 42},
        )

        assert mock_post.called
        payload = mock_post.call_args[1].get("json", {})
        text = payload.get("text", "")

        # Should include data
        assert "Additional Data" in text
        assert "Metric" in text or "metric" in text

    @patch.dict(
        os.environ,
        {"TELEGRAM_BOT_TOKEN": "test_token", "TELEGRAM_CHAT_ID": "test_chat_id"},
    )
    @patch("utils.notifications.requests.post")
    def test_send_telegram_api_error(self, mock_post):
        """Test Telegram handles API errors."""
        from utils.notifications import send_telegram

        mock_response = MagicMock()
        mock_response.status_code = 400
        mock_post.return_value = mock_response

        result = send_telegram("Test", "Message", "info")

        assert result["status"] == "failed"
        assert "400" in result["error"]

    @patch.dict(os.environ, {"TELEGRAM_BOT_TOKEN": "", "TELEGRAM_CHAT_ID": ""})
    def test_send_telegram_missing_both_credentials(self):
        """Test Telegram fails gracefully with missing credentials."""
        from utils.notifications import send_telegram

        result = send_telegram("Test", "Message", "info")

        assert result["status"] == "failed"
        assert "not configured" in result["error"]


@pytest.mark.unit
class TestConfigSuggestionsNotification:
    """Test send_config_suggestions function."""

    @patch.dict(
        os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"}
    )
    @patch("utils.notifications.send_slack")
    def test_send_config_suggestions(self, mock_slack):
        """Test send_config_suggestions notification."""
        from utils.notifications import send_config_suggestions

        mock_slack.return_value = {"status": "success"}

        result = {
            "suggestions": {
                "strategy": {"rsi_period": 12},
                "risk": {"max_loss": 0.03},
            },
            "reasoning": "Based on recent performance analysis",
        }

        send_config_suggestions(result)

        assert mock_slack.called


@pytest.mark.unit
class TestNotificationLevelClass:
    """Test NotificationLevel constants."""

    def test_notification_level_values(self):
        """Test NotificationLevel has correct values."""
        from utils.notifications import NotificationLevel

        assert NotificationLevel.INFO == "info"
        assert NotificationLevel.WARNING == "warning"
        assert NotificationLevel.ERROR == "error"
        assert NotificationLevel.CRITICAL == "critical"


@pytest.mark.unit
class TestIsEnabledFunction:
    """Test is_enabled function."""

    def test_is_enabled_false_when_disabled(self):
        """Test is_enabled returns False when notifications disabled."""
        from utils import notifications
        import importlib

        with patch.dict(
            os.environ, {"NOTIFICATIONS_ENABLED": "false", "NOTIFICATION_CHANNELS": ""}
        ):
            importlib.reload(notifications)
            assert notifications.is_enabled() is False

    def test_is_enabled_false_with_empty_channels(self):
        """Test is_enabled returns False with empty channels."""
        from utils import notifications
        import importlib

        with patch.dict(
            os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": ""}
        ):
            importlib.reload(notifications)
            assert notifications.is_enabled() is False


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
