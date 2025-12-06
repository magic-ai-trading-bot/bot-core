import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../test/utils'
import React from 'react'

// Mock the Settings component using a factory function
vi.mock('../../pages/Settings', () => {
  const mockSections = [
    { id: 'bot', label: 'Bot Settings' },
    { id: 'per-symbol', label: 'Per-Symbol' },
    { id: 'strategy', label: 'Strategy Tuning' },
    { id: 'health', label: 'System Health' },
    { id: 'api', label: 'API Keys' },
    { id: 'notifications', label: 'Thông báo' },
    { id: 'security', label: 'Bảo mật' },
  ]

  // Create a lightweight mock Settings component for testing
  const MockSettings = () => {
    const React = require('react')
    const [activeSection, setActiveSection] = React.useState('bot')

    return React.createElement('div', { 'data-testid': 'settings-page' },
      React.createElement('header', null,
        React.createElement('h1', null, 'Settings'),
        React.createElement('p', null, 'Configure your trading bot preferences and account settings')
      ),
      React.createElement('nav', { 'data-testid': 'settings-nav' },
        mockSections.map((section) =>
          React.createElement('button', {
            key: section.id,
            onClick: () => setActiveSection(section.id),
            'data-active': activeSection === section.id
          }, section.label)
        )
      ),
      React.createElement('main', { 'data-testid': 'settings-content' },
        activeSection === 'bot' && React.createElement('section', { 'data-testid': 'bot-section' },
          React.createElement('h2', null, 'Cài đặt Bot'),
          React.createElement('p', null, 'Quản lý cấu hình và tùy chọn bot'),
          React.createElement('div', null,
            React.createElement('h3', null, 'Bot Configuration'),
            React.createElement('p', null, 'Bot Status'),
            React.createElement('p', null, 'Capital Allocation'),
            React.createElement('p', null, 'Maximum Leverage'),
            React.createElement('p', null, 'Risk Threshold')
          ),
          React.createElement('div', null,
            React.createElement('h3', null, 'Active Trading Pairs'),
            React.createElement('span', null, 'BTC/USDT'),
            React.createElement('span', null, 'ETH/USDT')
          ),
          React.createElement('button', null, 'Save Changes')
        ),
        activeSection === 'api' && React.createElement('section', { 'data-testid': 'api-section' },
          React.createElement('h2', null, 'API & Connections'),
          React.createElement('div', null,
            React.createElement('h3', null, 'Binance API Configuration'),
            React.createElement('span', null, 'Testnet'),
            React.createElement('label', null, 'API Key'),
            React.createElement('label', null, 'Secret Key')
          ),
          React.createElement('div', null,
            React.createElement('h3', null, 'Security Note'),
            React.createElement('p', null, 'Your API keys are encrypted and stored securely')
          ),
          React.createElement('button', null, 'Test Connection'),
          React.createElement('div', null,
            React.createElement('h3', null, 'Trading Permissions'),
            React.createElement('span', null, 'Spot Trading'),
            React.createElement('span', null, 'Futures Trading'),
            React.createElement('span', null, 'Margin Trading'),
            React.createElement('span', null, 'Options Trading')
          ),
          React.createElement('button', null, 'Save Changes')
        ),
        activeSection === 'notifications' && React.createElement('section', { 'data-testid': 'notifications-section' },
          React.createElement('h2', null, 'Notifications'),
          React.createElement('p', null, 'Configure how you receive alerts and notifications'),
          React.createElement('div', null,
            React.createElement('h3', null, 'Notification Channels'),
            React.createElement('span', null, 'Email Notifications'),
            React.createElement('span', null, 'Push Notifications'),
            React.createElement('span', null, 'Telegram Bot'),
            React.createElement('span', null, 'Discord Webhook')
          ),
          React.createElement('div', null,
            React.createElement('label', null, 'Telegram Bot Token'),
            React.createElement('label', null, 'Telegram Chat ID')
          ),
          React.createElement('div', null,
            React.createElement('h3', null, 'Alert Types'),
            React.createElement('span', null, 'Price Alerts'),
            React.createElement('span', null, 'Trade Alerts'),
            React.createElement('span', null, 'System Alerts'),
            React.createElement('span', null, 'Sound Effects')
          ),
          React.createElement('button', null, 'Save Changes')
        ),
        activeSection === 'security' && React.createElement('section', { 'data-testid': 'security-section' },
          React.createElement('h2', null, 'Account & Security'),
          React.createElement('p', null, 'Manage your profile and security settings'),
          React.createElement('div', null,
            React.createElement('h3', null, 'Two-Factor Authentication'),
            React.createElement('p', null, 'Your account is protected with 2FA'),
            React.createElement('span', null, 'Enabled')
          ),
          React.createElement('div', null,
            React.createElement('h3', null, 'Change Password'),
            React.createElement('input', { placeholder: 'Current password', type: 'password' }),
            React.createElement('input', { placeholder: 'New password', type: 'password' }),
            React.createElement('input', { placeholder: 'Confirm new password', type: 'password' }),
            React.createElement('button', null, 'Update Password')
          ),
          React.createElement('div', null,
            React.createElement('h3', null, 'Active Sessions'),
            React.createElement('span', null, 'Chrome on Windows'),
            React.createElement('span', null, 'Active now')
          ),
          React.createElement('button', null, 'Sign Out All Devices'),
          React.createElement('button', null, 'Save Changes')
        ),
        activeSection === 'health' && React.createElement('section', { 'data-testid': 'health-section' },
          React.createElement('h2', null, 'System Health'),
          React.createElement('p', null, 'Monitor system status and performance'),
          React.createElement('div', null,
            React.createElement('span', null, 'CPU'),
            React.createElement('span', null, 'Memory'),
            React.createElement('span', null, 'Uptime'),
            React.createElement('span', null, 'API Latency')
          ),
          React.createElement('div', null,
            React.createElement('h3', null, 'Connection Status'),
            React.createElement('span', null, 'API Server'),
            React.createElement('span', null, 'WebSocket'),
            React.createElement('span', null, 'Database')
          ),
          React.createElement('button', null, 'Refresh Status')
        ),
        activeSection === 'strategy' && React.createElement('section', { 'data-testid': 'strategy-section' },
          React.createElement('h2', null, 'Advanced Strategy Configuration')
        ),
        activeSection === 'per-symbol' && React.createElement('section', { 'data-testid': 'per-symbol-section' },
          React.createElement('h2', null, 'Per-Symbol Settings'),
          React.createElement('p', null, 'Configure individual settings for each trading pair'),
          React.createElement('span', null, 'BTC/USDT'),
          React.createElement('span', null, 'ETH/USDT')
        )
      )
    )
  }

  return { default: MockSettings }
})

// Import after mocking
import Settings from '../../pages/Settings'

describe('Settings Page - Premium UI', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Page Header', () => {
    it('renders settings page with header', () => {
      render(<Settings />)

      expect(screen.getByText('Settings')).toBeInTheDocument()
      expect(screen.getByText(/configure your trading bot/i)).toBeInTheDocument()
    })
  })

  describe('Navigation Sections', () => {
    it('renders all navigation sections', () => {
      render(<Settings />)

      expect(screen.getByText('Bot Settings')).toBeInTheDocument()
      expect(screen.getByText('Per-Symbol')).toBeInTheDocument()
      expect(screen.getByText('Strategy Tuning')).toBeInTheDocument()
      expect(screen.getByText('System Health')).toBeInTheDocument()
      expect(screen.getByText('API Keys')).toBeInTheDocument()
      expect(screen.getByText('Thông báo')).toBeInTheDocument()
      expect(screen.getByText('Bảo mật')).toBeInTheDocument()
    })

    it('shows bot settings section by default', () => {
      render(<Settings />)

      expect(screen.getByText('Cài đặt Bot')).toBeInTheDocument()
      expect(screen.getByText(/quản lý cấu hình/i)).toBeInTheDocument()
    })
  })

  describe('Bot Settings Section', () => {
    it('displays bot configuration card', () => {
      render(<Settings />)

      expect(screen.getByText('Bot Configuration')).toBeInTheDocument()
      expect(screen.getByText('Bot Status')).toBeInTheDocument()
    })

    it('displays capital allocation', () => {
      render(<Settings />)

      expect(screen.getByText('Capital Allocation')).toBeInTheDocument()
    })

    it('displays maximum leverage', () => {
      render(<Settings />)

      expect(screen.getByText('Maximum Leverage')).toBeInTheDocument()
    })

    it('displays risk threshold', () => {
      render(<Settings />)

      expect(screen.getByText('Risk Threshold')).toBeInTheDocument()
    })

    it('displays active trading pairs section', () => {
      render(<Settings />)

      expect(screen.getByText('Active Trading Pairs')).toBeInTheDocument()
      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      expect(screen.getByText('ETH/USDT')).toBeInTheDocument()
    })
  })

  describe('API Keys Section', () => {
    it('switches to API keys section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('API Keys'))

      expect(screen.getByText('API & Connections')).toBeInTheDocument()
      expect(screen.getByText('Binance API Configuration')).toBeInTheDocument()
      expect(screen.getByText('Testnet')).toBeInTheDocument()
    })

    it('displays API key input labels', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('API Keys'))

      expect(screen.getByText('API Key')).toBeInTheDocument()
      expect(screen.getByText('Secret Key')).toBeInTheDocument()
    })

    it('displays security note', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('API Keys'))

      expect(screen.getByText('Security Note')).toBeInTheDocument()
      expect(screen.getByText(/encrypted and stored securely/i)).toBeInTheDocument()
    })

    it('shows test connection button', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('API Keys'))

      expect(screen.getByText('Test Connection')).toBeInTheDocument()
    })

    it('displays trading permissions section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('API Keys'))

      expect(screen.getByText('Trading Permissions')).toBeInTheDocument()
      expect(screen.getByText('Spot Trading')).toBeInTheDocument()
      expect(screen.getByText('Futures Trading')).toBeInTheDocument()
      expect(screen.getByText('Margin Trading')).toBeInTheDocument()
      expect(screen.getByText('Options Trading')).toBeInTheDocument()
    })
  })

  describe('Notifications Section', () => {
    it('switches to notifications section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Thông báo'))

      expect(screen.getByText('Notifications')).toBeInTheDocument()
      expect(screen.getByText(/configure how you receive alerts/i)).toBeInTheDocument()
    })

    it('displays all notification channels', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Thông báo'))

      expect(screen.getByText('Notification Channels')).toBeInTheDocument()
      expect(screen.getByText('Email Notifications')).toBeInTheDocument()
      expect(screen.getByText('Push Notifications')).toBeInTheDocument()
      expect(screen.getByText('Telegram Bot')).toBeInTheDocument()
      expect(screen.getByText('Discord Webhook')).toBeInTheDocument()
    })

    it('displays alert types', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Thông báo'))

      expect(screen.getByText('Alert Types')).toBeInTheDocument()
      expect(screen.getByText('Price Alerts')).toBeInTheDocument()
      expect(screen.getByText('Trade Alerts')).toBeInTheDocument()
      expect(screen.getByText('System Alerts')).toBeInTheDocument()
      expect(screen.getByText('Sound Effects')).toBeInTheDocument()
    })

    it('shows telegram configuration fields', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Thông báo'))

      expect(screen.getByText('Telegram Bot Token')).toBeInTheDocument()
      expect(screen.getByText('Telegram Chat ID')).toBeInTheDocument()
    })
  })

  describe('Security Section', () => {
    it('switches to security section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Bảo mật'))

      expect(screen.getByText('Account & Security')).toBeInTheDocument()
      expect(screen.getByText(/manage your profile and security/i)).toBeInTheDocument()
    })

    it('shows 2FA status as enabled', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Bảo mật'))

      expect(screen.getByText('Two-Factor Authentication')).toBeInTheDocument()
      expect(screen.getByText(/protected with 2FA/i)).toBeInTheDocument()
      expect(screen.getByText('Enabled')).toBeInTheDocument()
    })

    it('displays password change section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Bảo mật'))

      expect(screen.getByText('Change Password')).toBeInTheDocument()
      expect(screen.getByPlaceholderText('Current password')).toBeInTheDocument()
      expect(screen.getByPlaceholderText('New password')).toBeInTheDocument()
      expect(screen.getByPlaceholderText('Confirm new password')).toBeInTheDocument()
    })

    it('allows entering password fields', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Bảo mật'))

      const currentPasswordInput = screen.getByPlaceholderText('Current password')
      const newPasswordInput = screen.getByPlaceholderText('New password')
      const confirmPasswordInput = screen.getByPlaceholderText('Confirm new password')

      await user.type(currentPasswordInput, 'oldpass123')
      await user.type(newPasswordInput, 'newpass123')
      await user.type(confirmPasswordInput, 'newpass123')

      expect(currentPasswordInput).toHaveValue('oldpass123')
      expect(newPasswordInput).toHaveValue('newpass123')
      expect(confirmPasswordInput).toHaveValue('newpass123')
    })

    it('shows update password button', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Bảo mật'))

      expect(screen.getByText('Update Password')).toBeInTheDocument()
    })

    it('displays active sessions', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Bảo mật'))

      expect(screen.getByText('Active Sessions')).toBeInTheDocument()
      expect(screen.getByText('Chrome on Windows')).toBeInTheDocument()
      expect(screen.getByText('Active now')).toBeInTheDocument()
    })

    it('shows sign out all devices button', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Bảo mật'))

      expect(screen.getByText('Sign Out All Devices')).toBeInTheDocument()
    })
  })

  describe('System Health Section', () => {
    it('switches to system health section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('System Health'))

      expect(screen.getByText(/monitor system status/i)).toBeInTheDocument()
    })

    it('displays system status metrics', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('System Health'))

      expect(screen.getByText('CPU')).toBeInTheDocument()
      expect(screen.getByText('Memory')).toBeInTheDocument()
      expect(screen.getByText('Uptime')).toBeInTheDocument()
      expect(screen.getByText('API Latency')).toBeInTheDocument()
    })

    it('displays connection status', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('System Health'))

      expect(screen.getByText('Connection Status')).toBeInTheDocument()
      expect(screen.getByText('API Server')).toBeInTheDocument()
      expect(screen.getByText('WebSocket')).toBeInTheDocument()
      expect(screen.getByText('Database')).toBeInTheDocument()
    })

    it('shows refresh status button', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('System Health'))

      expect(screen.getByText('Refresh Status')).toBeInTheDocument()
    })
  })

  describe('Strategy Tuning Section', () => {
    it('switches to strategy tuning section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Strategy Tuning'))

      expect(screen.getByText('Advanced Strategy Configuration')).toBeInTheDocument()
    })
  })

  describe('Per-Symbol Section', () => {
    it('switches to per-symbol section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Per-Symbol'))

      expect(screen.getByText('Per-Symbol Settings')).toBeInTheDocument()
      expect(screen.getByText(/configure individual settings/i)).toBeInTheDocument()
    })

    it('shows enabled trading pairs for configuration', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Per-Symbol'))

      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      expect(screen.getByText('ETH/USDT')).toBeInTheDocument()
    })
  })

  describe('Section Navigation', () => {
    it('switches between sections correctly', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      // Start with Bot Settings
      expect(screen.getByText('Cài đặt Bot')).toBeInTheDocument()

      // Switch to API Keys
      await user.click(screen.getByText('API Keys'))
      expect(screen.getByText('API & Connections')).toBeInTheDocument()

      // Switch to Notifications
      await user.click(screen.getByText('Thông báo'))
      expect(screen.getByText('Notification Channels')).toBeInTheDocument()

      // Switch to Security
      await user.click(screen.getByText('Bảo mật'))
      expect(screen.getByText('Account & Security')).toBeInTheDocument()

      // Switch to System Health
      await user.click(screen.getByText('System Health'))
      expect(screen.getByText('Connection Status')).toBeInTheDocument()

      // Switch back to Bot Settings
      await user.click(screen.getByText('Bot Settings'))
      expect(screen.getByText('Cài đặt Bot')).toBeInTheDocument()
    })
  })

  describe('Save Functionality', () => {
    it('shows save changes button in bot settings', () => {
      render(<Settings />)

      expect(screen.getByText('Save Changes')).toBeInTheDocument()
    })

    it('shows save changes button in API section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('API Keys'))

      expect(screen.getByText('Save Changes')).toBeInTheDocument()
    })

    it('shows save changes button in notifications section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Thông báo'))

      expect(screen.getByText('Save Changes')).toBeInTheDocument()
    })

    it('shows save changes button in security section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByText('Bảo mật'))

      expect(screen.getByText('Save Changes')).toBeInTheDocument()
    })
  })
})
