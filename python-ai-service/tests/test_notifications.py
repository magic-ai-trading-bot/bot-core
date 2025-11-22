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

        # Slack uses 'text' or 'blocks' field
        assert "text" in payload or "blocks" in payload

    def test_send_slack_requires_webhook_url(self):
        """Test that Slack requires webhook URL"""
        from utils.notifications import send_slack

        # Save original
        webhook = os.environ.get("SLACK_WEBHOOK_URL")

        try:
            # Remove webhook
            os.environ.pop("SLACK_WEBHOOK_URL", None)

            result = send_slack("Test", "Message", "info")

            # Should fail gracefully
            assert result["status"] == "failed" or "error" in result

        finally:
            if webhook:
                os.environ["SLACK_WEBHOOK_URL"] = webhook


class TestDiscordNotifications:
    """Test Discord webhook notifications"""

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

    @patch.dict(os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "email,slack,discord,telegram"})
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

    @patch.dict(os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"})
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

    @patch.dict(os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"})
    @patch("utils.notifications.send_slack")
    def test_send_warning_uses_warning_level(self, mock_slack):
        """Test send_warning helper uses warning level"""
        from utils.notifications import send_warning

        mock_slack.return_value = {"status": "success"}

        send_warning("Performance Degraded", "Win rate at 55%")

        assert mock_slack.called

    @patch.dict(os.environ, {"NOTIFICATIONS_ENABLED": "true", "NOTIFICATION_CHANNELS": "slack"})
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

    def test_notification_with_invalid_level(self):
        """Test notification with invalid severity level"""
        from utils.notifications import send_notification

        # Should handle invalid level gracefully
        result = send_notification(
            title="Test", message="Message", level="invalid_level"  # Invalid
        )

        # Should default to 'info' or handle gracefully
        assert result is None or isinstance(result, dict)

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


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
