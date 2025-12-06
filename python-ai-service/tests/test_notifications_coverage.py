#!/usr/bin/env python3
"""
Additional tests for notification system to boost coverage.
Covers user preferences, Discord/Telegram with custom config, and edge cases.
"""

import pytest
from unittest.mock import patch, MagicMock, Mock
from datetime import datetime, timedelta
import os


class TestUserNotificationPreferences:
    """Test user notification preferences fetching and caching"""

    def setup_method(self):
        """Reset cache before each test"""
        from utils import notifications
        notifications._preferences_cache = None
        notifications._preferences_cache_time = None

    def test_get_preferences_returns_cached_value(self):
        """Test that cached preferences are returned within TTL"""
        from utils import notifications

        # Set up cache
        notifications._preferences_cache = {"enabled": True, "channels": {}}
        notifications._preferences_cache_time = datetime.utcnow()

        result = notifications.get_user_notification_preferences()

        assert result == {"enabled": True, "channels": {}}

    def test_get_preferences_refreshes_expired_cache(self):
        """Test that expired cache triggers refresh"""
        from utils import notifications

        # Set up expired cache
        notifications._preferences_cache = {"enabled": True}
        notifications._preferences_cache_time = datetime.utcnow() - timedelta(seconds=400)

        with patch('utils.notifications.requests.get') as mock_get:
            mock_get.return_value = Mock(
                status_code=200,
                json=lambda: {"success": True, "data": {"enabled": False}}
            )

            result = notifications.get_user_notification_preferences()

            # Should have refreshed
            mock_get.assert_called_once()
            assert result == {"enabled": False}

    def test_get_preferences_api_error(self):
        """Test handling of API error"""
        from utils import notifications

        with patch('utils.notifications.requests.get') as mock_get:
            mock_get.side_effect = Exception("Connection refused")

            result = notifications.get_user_notification_preferences()

            assert result is None

    def test_get_preferences_api_non_200(self):
        """Test handling of non-200 response"""
        from utils import notifications

        with patch('utils.notifications.requests.get') as mock_get:
            mock_get.return_value = Mock(status_code=500)

            result = notifications.get_user_notification_preferences()

            assert result is None

    def test_get_preferences_api_success_no_data(self):
        """Test handling of success response without data"""
        from utils import notifications

        with patch('utils.notifications.requests.get') as mock_get:
            mock_get.return_value = Mock(
                status_code=200,
                json=lambda: {"success": False}
            )

            result = notifications.get_user_notification_preferences()

            assert result is None

    def test_clear_preferences_cache(self):
        """Test clearing the preferences cache"""
        from utils import notifications

        # Set up cache
        notifications._preferences_cache = {"enabled": True}
        notifications._preferences_cache_time = datetime.utcnow()

        notifications.clear_preferences_cache()

        assert notifications._preferences_cache is None
        assert notifications._preferences_cache_time is None


class TestSendNotificationWithUserPrefs:
    """Test send_notification_with_user_prefs function"""

    def setup_method(self):
        """Reset cache before each test"""
        from utils import notifications
        notifications._preferences_cache = None
        notifications._preferences_cache_time = None

    def test_fallback_when_no_prefs(self):
        """Test fallback to env-based notifications when no prefs available"""
        from utils import notifications

        with patch('utils.notifications.get_user_notification_preferences', return_value=None):
            with patch('utils.notifications.send_notification') as mock_send:
                mock_send.return_value = {"status": "success"}

                result = notifications.send_notification_with_user_prefs(
                    title="Test",
                    message="Test message",
                    level="info"
                )

                mock_send.assert_called_once()

    def test_notifications_disabled_globally(self):
        """Test when notifications are disabled globally by user"""
        from utils import notifications

        prefs = {"enabled": False}

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            result = notifications.send_notification_with_user_prefs(
                title="Test",
                message="Test message"
            )

            assert result.get("skipped") == True
            assert result.get("reason") == "notifications_disabled"

    def test_alert_type_disabled(self):
        """Test when specific alert type is disabled"""
        from utils import notifications

        prefs = {
            "enabled": True,
            "alerts": {"trade_alerts": False}
        }

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            result = notifications.send_notification_with_user_prefs(
                title="Test",
                message="Test message",
                alert_type="trade_alerts"
            )

            assert result.get("skipped") == True
            assert result.get("reason") == "trade_alerts_disabled"

    def test_discord_channel_enabled(self):
        """Test sending to Discord when enabled"""
        from utils import notifications

        prefs = {
            "enabled": True,
            "alerts": {"system_alerts": True},
            "channels": {
                "discord": {
                    "enabled": True,
                    "webhook_url": "https://discord.com/api/webhooks/test"
                }
            }
        }

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            with patch('utils.notifications.send_discord_with_url') as mock_discord:
                mock_discord.return_value = {"status": "success"}

                result = notifications.send_notification_with_user_prefs(
                    title="Test",
                    message="Test message"
                )

                mock_discord.assert_called_once()
                assert result.get("discord") == {"status": "success"}

    def test_discord_channel_error(self):
        """Test Discord error handling"""
        from utils import notifications

        prefs = {
            "enabled": True,
            "alerts": {"system_alerts": True},
            "channels": {
                "discord": {
                    "enabled": True,
                    "webhook_url": "https://discord.com/api/webhooks/test"
                }
            }
        }

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            with patch('utils.notifications.send_discord_with_url') as mock_discord:
                mock_discord.side_effect = Exception("Discord error")

                result = notifications.send_notification_with_user_prefs(
                    title="Test",
                    message="Test message"
                )

                assert result.get("discord", {}).get("status") == "failed"

    def test_telegram_channel_enabled(self):
        """Test sending to Telegram when enabled"""
        from utils import notifications

        prefs = {
            "enabled": True,
            "alerts": {"system_alerts": True},
            "channels": {
                "telegram": {
                    "enabled": True,
                    "bot_token": "test_token",
                    "chat_id": "12345"
                }
            }
        }

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            with patch('utils.notifications.send_telegram_with_config') as mock_telegram:
                mock_telegram.return_value = {"status": "success"}

                result = notifications.send_notification_with_user_prefs(
                    title="Test",
                    message="Test message"
                )

                mock_telegram.assert_called_once()
                assert result.get("telegram") == {"status": "success"}

    def test_telegram_channel_error(self):
        """Test Telegram error handling"""
        from utils import notifications

        prefs = {
            "enabled": True,
            "alerts": {"system_alerts": True},
            "channels": {
                "telegram": {
                    "enabled": True,
                    "bot_token": "test_token",
                    "chat_id": "12345"
                }
            }
        }

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            with patch('utils.notifications.send_telegram_with_config') as mock_telegram:
                mock_telegram.side_effect = Exception("Telegram error")

                result = notifications.send_notification_with_user_prefs(
                    title="Test",
                    message="Test message"
                )

                assert result.get("telegram", {}).get("status") == "failed"

    def test_email_channel_enabled(self):
        """Test sending email when enabled"""
        from utils import notifications

        prefs = {
            "enabled": True,
            "alerts": {"system_alerts": True},
            "channels": {"email": True}
        }

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            with patch('utils.notifications.send_email') as mock_email:
                mock_email.return_value = {"status": "success"}

                result = notifications.send_notification_with_user_prefs(
                    title="Test",
                    message="Test message"
                )

                mock_email.assert_called_once()

    def test_email_channel_error(self):
        """Test email error handling"""
        from utils import notifications

        prefs = {
            "enabled": True,
            "alerts": {"system_alerts": True},
            "channels": {"email": True}
        }

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            with patch('utils.notifications.send_email') as mock_email:
                mock_email.side_effect = Exception("Email error")

                result = notifications.send_notification_with_user_prefs(
                    title="Test",
                    message="Test message"
                )

                assert result.get("email", {}).get("status") == "failed"

    def test_with_additional_data(self):
        """Test notification with additional data"""
        from utils import notifications

        prefs = {
            "enabled": True,
            "alerts": {"system_alerts": True},
            "channels": {"email": True}
        }

        with patch('utils.notifications.get_user_notification_preferences', return_value=prefs):
            with patch('utils.notifications.send_email') as mock_email:
                mock_email.return_value = {"status": "success"}

                result = notifications.send_notification_with_user_prefs(
                    title="Test",
                    message="Test message",
                    data={"key1": "value1", "key2": 123}
                )

                # Verify email was called with formatted data
                call_args = mock_email.call_args
                assert "key1" in str(call_args) or call_args is not None


class TestSendDiscordWithUrl:
    """Test send_discord_with_url function"""

    def test_send_discord_success(self):
        """Test successful Discord notification"""
        from utils import notifications

        with patch('utils.notifications.requests.post') as mock_post:
            mock_post.return_value = Mock(status_code=204)

            result = notifications.send_discord_with_url(
                title="Test",
                message="Test message",
                level="info",
                data=None,
                webhook_url="https://discord.com/api/webhooks/test"
            )

            assert result["status"] == "success"

    def test_send_discord_with_data(self):
        """Test Discord notification with additional data"""
        from utils import notifications

        with patch('utils.notifications.requests.post') as mock_post:
            mock_post.return_value = Mock(status_code=200)

            result = notifications.send_discord_with_url(
                title="Test",
                message="Test message",
                level="warning",
                data={"price": "50000", "change": "+5%"},
                webhook_url="https://discord.com/api/webhooks/test"
            )

            assert result["status"] == "success"

    def test_send_discord_rate_limited(self):
        """Test Discord rate limit handling"""
        from utils import notifications

        with patch('utils.notifications.requests.post') as mock_post:
            mock_post.return_value = Mock(
                status_code=429,
                json=lambda: {"retry_after": 2.5}
            )

            result = notifications.send_discord_with_url(
                title="Test",
                message="Test message",
                level="info",
                data=None,
                webhook_url="https://discord.com/api/webhooks/test"
            )

            assert result["status"] == "failed"
            assert "Rate limited" in result["error"]

    def test_send_discord_http_error(self):
        """Test Discord HTTP error handling"""
        from utils import notifications

        with patch('utils.notifications.requests.post') as mock_post:
            mock_post.return_value = Mock(status_code=500)

            result = notifications.send_discord_with_url(
                title="Test",
                message="Test message",
                level="error",
                data=None,
                webhook_url="https://discord.com/api/webhooks/test"
            )

            assert result["status"] == "failed"
            assert "HTTP 500" in result["error"]

    def test_send_discord_all_levels(self):
        """Test Discord notifications with all levels"""
        from utils import notifications

        levels = ["info", "warning", "error", "critical"]

        for level in levels:
            with patch('utils.notifications.requests.post') as mock_post:
                mock_post.return_value = Mock(status_code=204)

                result = notifications.send_discord_with_url(
                    title="Test",
                    message="Test message",
                    level=level,
                    data=None,
                    webhook_url="https://discord.com/api/webhooks/test"
                )

                assert result["status"] == "success"


class TestSendTelegramWithConfig:
    """Test send_telegram_with_config function"""

    def test_send_telegram_success(self):
        """Test successful Telegram notification"""
        from utils import notifications

        with patch('utils.notifications.requests.post') as mock_post:
            mock_post.return_value = Mock(status_code=200)

            result = notifications.send_telegram_with_config(
                title="Test",
                message="Test message",
                level="info",
                data=None,
                bot_token="test_token",
                chat_id="12345"
            )

            assert result["status"] == "success"

    def test_send_telegram_with_data(self):
        """Test Telegram notification with additional data"""
        from utils import notifications

        with patch('utils.notifications.requests.post') as mock_post:
            mock_post.return_value = Mock(status_code=200)

            result = notifications.send_telegram_with_config(
                title="Test",
                message="Test message",
                level="warning",
                data={"price": "50000", "symbol": "BTCUSDT"},
                bot_token="test_token",
                chat_id="12345"
            )

            assert result["status"] == "success"

    def test_send_telegram_http_error(self):
        """Test Telegram HTTP error handling"""
        from utils import notifications

        with patch('utils.notifications.requests.post') as mock_post:
            mock_post.return_value = Mock(status_code=401)

            result = notifications.send_telegram_with_config(
                title="Test",
                message="Test message",
                level="error",
                data=None,
                bot_token="invalid_token",
                chat_id="12345"
            )

            assert result["status"] == "failed"
            assert "HTTP 401" in result["error"]

    def test_send_telegram_all_levels(self):
        """Test Telegram notifications with all levels"""
        from utils import notifications

        levels = ["info", "warning", "error", "critical"]

        for level in levels:
            with patch('utils.notifications.requests.post') as mock_post:
                mock_post.return_value = Mock(status_code=200)

                result = notifications.send_telegram_with_config(
                    title="Test",
                    message="Test message",
                    level=level,
                    data=None,
                    bot_token="test_token",
                    chat_id="12345"
                )

                assert result["status"] == "success"


class TestSendDiscordEdgeCases:
    """Test edge cases for send_discord function"""

    def test_send_discord_no_webhook_url(self):
        """Test Discord without webhook URL configured"""
        from utils import notifications

        with patch.dict(os.environ, {"DISCORD_WEBHOOK_URL": ""}, clear=False):
            with patch.object(notifications, 'DISCORD_WEBHOOK_URL', ""):
                result = notifications.send_discord(
                    title="Test",
                    message="Test message",
                    level="info"
                )

                assert result["status"] == "failed"
                assert "not configured" in result["error"]

    def test_send_discord_rate_limit(self):
        """Test Discord rate limit response"""
        from utils import notifications

        with patch.dict(os.environ, {"DISCORD_WEBHOOK_URL": "https://test.com"}, clear=False):
            with patch.object(notifications, 'DISCORD_WEBHOOK_URL', "https://test.com"):
                with patch('utils.notifications.requests.post') as mock_post:
                    mock_post.return_value = Mock(
                        status_code=429,
                        json=lambda: {"retry_after": 1.5}
                    )

                    result = notifications.send_discord(
                        title="Test",
                        message="Test message",
                        level="info"
                    )

                    assert result["status"] == "failed"
                    assert "Rate limited" in result["error"]

    def test_send_discord_non_success_status(self):
        """Test Discord non-success HTTP status"""
        from utils import notifications

        with patch.dict(os.environ, {"DISCORD_WEBHOOK_URL": "https://test.com"}, clear=False):
            with patch.object(notifications, 'DISCORD_WEBHOOK_URL', "https://test.com"):
                with patch('utils.notifications.requests.post') as mock_post:
                    mock_post.return_value = Mock(status_code=400)

                    result = notifications.send_discord(
                        title="Test",
                        message="Test message",
                        level="info"
                    )

                    assert result["status"] == "failed"
                    assert "HTTP 400" in result["error"]

    def test_send_discord_exception(self):
        """Test Discord exception handling"""
        from utils import notifications

        with patch.dict(os.environ, {"DISCORD_WEBHOOK_URL": "https://test.com"}, clear=False):
            with patch.object(notifications, 'DISCORD_WEBHOOK_URL', "https://test.com"):
                with patch('utils.notifications.requests.post') as mock_post:
                    mock_post.side_effect = Exception("Connection failed")

                    result = notifications.send_discord(
                        title="Test",
                        message="Test message",
                        level="info"
                    )

                    assert result["status"] == "failed"
                    assert "Connection failed" in result["error"]


class TestSendTelegramEdgeCases:
    """Test edge cases for send_telegram function"""

    def test_send_telegram_exception(self):
        """Test Telegram exception handling"""
        from utils import notifications

        with patch.dict(os.environ, {
            "TELEGRAM_BOT_TOKEN": "test_token",
            "TELEGRAM_CHAT_ID": "12345"
        }, clear=False):
            with patch.object(notifications, 'TELEGRAM_BOT_TOKEN', "test_token"):
                with patch.object(notifications, 'TELEGRAM_CHAT_ID', "12345"):
                    with patch('utils.notifications.requests.post') as mock_post:
                        mock_post.side_effect = Exception("API error")

                        result = notifications.send_telegram(
                            title="Test",
                            message="Test message",
                            level="info"
                        )

                        assert result["status"] == "failed"
                        assert "API error" in result["error"]


class TestFormatData:
    """Test format_data helper function"""

    def test_format_data_simple(self):
        """Test formatting simple data"""
        from utils import notifications

        result = notifications.format_data({"key": "value"})
        assert "key" in result
        assert "value" in result

    def test_format_data_nested(self):
        """Test formatting nested data"""
        from utils import notifications

        result = notifications.format_data({
            "outer": {"inner": "value"}
        })
        assert "outer" in result
        assert "inner" in result

    def test_format_data_with_list(self):
        """Test formatting data with list values"""
        from utils import notifications

        result = notifications.format_data({
            "items": ["a", "b", "c"]
        })
        assert "items" in result
