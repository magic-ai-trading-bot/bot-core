import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../test/utils'
import Settings from '../../pages/Settings'

// Mock DashboardHeader
vi.mock('../../components/dashboard/DashboardHeader', () => ({
  DashboardHeader: () => <div data-testid="dashboard-header">Dashboard Header</div>,
}))

// Mock BotSettings
vi.mock('../../components/dashboard/BotSettings', () => ({
  BotSettings: () => <div data-testid="bot-settings">Bot Settings Component</div>,
}))

// Mock ChatBot
vi.mock('../../components/ChatBot', () => ({
  default: () => null,
}))

// Mock PaperTradingContext (Settings uses usePaperTradingContext)
vi.mock('../../contexts/PaperTradingContext', () => ({
  usePaperTradingContext: vi.fn(() => ({
    portfolio: {
      current_balance: 10000,
      equity: 10000,
      total_pnl: 0,
      total_pnl_percentage: 0,
    },
  })),
  PaperTradingProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

describe('Settings', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders settings page with header', () => {
    render(<Settings />)

    expect(screen.getByText('Cài đặt Bot')).toBeInTheDocument()
    expect(screen.getByText(/quản lý cấu hình và tùy chọn/i)).toBeInTheDocument()
    expect(screen.getByTestId('dashboard-header')).toBeInTheDocument()
  })

  it('renders all tabs', () => {
    render(<Settings />)

    expect(screen.getByRole('tab', { name: /bot settings/i })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /api keys/i })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /thông báo/i })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /bảo mật/i })).toBeInTheDocument()
  })

  it('shows bot settings tab by default', () => {
    render(<Settings />)

    expect(screen.getByTestId('bot-settings')).toBeInTheDocument()
  })

  describe('API Keys Tab', () => {
    it('switches to API keys tab', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      expect(screen.getByText('Binance API Configuration')).toBeInTheDocument()
      expect(screen.getByText('Testnet')).toBeInTheDocument()
    })

    it('displays API key input fields', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      expect(screen.getByLabelText('API Key')).toBeInTheDocument()
      expect(screen.getByLabelText('Secret Key')).toBeInTheDocument()
    })

    it('displays masked API keys', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      const apiKeyInput = screen.getByLabelText('API Key') as HTMLInputElement
      const secretKeyInput = screen.getByLabelText('Secret Key') as HTMLInputElement

      expect(apiKeyInput.value).toContain('****')
      expect(secretKeyInput.value).toContain('****')
    })

    it('allows updating API keys', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      const apiKeyInput = screen.getByLabelText('API Key')
      await user.clear(apiKeyInput)
      await user.type(apiKeyInput, 'new-api-key-123')

      expect(apiKeyInput).toHaveValue('new-api-key-123')
    })

    it('displays security warning for API keys', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      expect(screen.getByText(/lưu ý bảo mật/i)).toBeInTheDocument()
      expect(screen.getByText(/api keys được mã hóa/i)).toBeInTheDocument()
    })

    it('shows test connection button', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      expect(screen.getByRole('button', { name: /test connection/i })).toBeInTheDocument()
    })

    it('shows save API keys button', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      expect(screen.getByRole('button', { name: /lưu api keys/i })).toBeInTheDocument()
    })

    it('displays trading permissions section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      expect(screen.getByText(/quyền hạn trading/i)).toBeInTheDocument()
      expect(screen.getByText('Spot Trading')).toBeInTheDocument()
      expect(screen.getByText('Futures Trading')).toBeInTheDocument()
      expect(screen.getByText('Margin Trading')).toBeInTheDocument()
      expect(screen.getByText('Options Trading')).toBeInTheDocument()
    })

    it('shows futures trading as enabled and disabled', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      // Futures trading should be enabled and disabled (can't toggle)
      const switches = screen.getAllByRole('switch')
      const futuresSwitch = switches.find(s =>
        s.closest('.p-3')?.querySelector('.font-semibold')?.textContent === 'Futures Trading'
      )

      expect(futuresSwitch).toBeDisabled()
    })
  })

  describe('Notifications Tab', () => {
    it('switches to notifications tab', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /thông báo/i }))

      expect(screen.getByText(/tùy chọn thông báo/i)).toBeInTheDocument()
    })

    it('displays all notification options', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /thông báo/i }))

      expect(screen.getByText('Email Notifications')).toBeInTheDocument()
      expect(screen.getByText('Push Notifications')).toBeInTheDocument()
      expect(screen.getByText('Telegram Bot')).toBeInTheDocument()
      expect(screen.getByText('Discord Webhook')).toBeInTheDocument()
    })

    it('shows email notifications enabled by default', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /thông báo/i }))

      const switches = screen.getAllByRole('switch')
      const emailSwitch = switches.find(s =>
        s.closest('.p-3')?.querySelector('.font-semibold')?.textContent === 'Email Notifications'
      )

      expect(emailSwitch).toBeChecked()
    })

    it('toggles push notifications', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /thông báo/i }))

      const switches = screen.getAllByRole('switch')
      const pushSwitch = switches.find(s =>
        s.closest('.p-3')?.querySelector('.font-semibold')?.textContent === 'Push Notifications'
      )

      expect(pushSwitch).not.toBeChecked()

      if (pushSwitch) {
        await user.click(pushSwitch)
        expect(pushSwitch).toBeChecked()
      }
    })

    it('shows telegram token input when telegram is enabled', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /thông báo/i }))

      // Telegram should be enabled by default
      expect(screen.getByLabelText(/telegram bot token/i)).toBeInTheDocument()
    })

    it('hides telegram token input when telegram is disabled', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /thông báo/i }))

      const switches = screen.getAllByRole('switch')
      const telegramSwitch = switches.find(s =>
        s.closest('.p-3')?.querySelector('.font-semibold')?.textContent === 'Telegram Bot'
      )

      if (telegramSwitch) {
        await user.click(telegramSwitch)
        expect(screen.queryByLabelText(/telegram bot token/i)).not.toBeInTheDocument()
      }
    })

    it('shows save notification settings button', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /thông báo/i }))

      expect(screen.getByRole('button', { name: /lưu cài đặt thông báo/i })).toBeInTheDocument()
    })
  })

  describe('Security Tab', () => {
    it('switches to security tab', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /bảo mật/i }))

      expect(screen.getByText(/bảo mật tài khoản/i)).toBeInTheDocument()
    })

    it('shows 2FA status as enabled', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /bảo mật/i }))

      expect(screen.getByText('Two-Factor Authentication')).toBeInTheDocument()
      expect(screen.getByText(/xác thực 2 yếu tố đã được bật/i)).toBeInTheDocument()
      expect(screen.getByText('Đã kích hoạt')).toBeInTheDocument()
    })

    it('displays password change section', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /bảo mật/i }))

      expect(screen.getByText(/đổi mật khẩu/i)).toBeInTheDocument()
      expect(screen.getByPlaceholderText('Mật khẩu hiện tại')).toBeInTheDocument()
      expect(screen.getByPlaceholderText('Mật khẩu mới')).toBeInTheDocument()
      expect(screen.getByPlaceholderText('Xác nhận mật khẩu mới')).toBeInTheDocument()
    })

    it('allows updating password', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /bảo mật/i }))

      const currentPasswordInput = screen.getByPlaceholderText('Mật khẩu hiện tại')
      const newPasswordInput = screen.getByPlaceholderText('Mật khẩu mới')
      const confirmPasswordInput = screen.getByPlaceholderText('Xác nhận mật khẩu mới')

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

      await user.click(screen.getByRole('tab', { name: /bảo mật/i }))

      expect(screen.getByRole('button', { name: /cập nhật mật khẩu/i })).toBeInTheDocument()
    })

    it('displays active sessions', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /bảo mật/i }))

      expect(screen.getByText(/phiên đăng nhập/i)).toBeInTheDocument()
      expect(screen.getByText('Chrome on Windows')).toBeInTheDocument()
      expect(screen.getByText('Active now')).toBeInTheDocument()
      expect(screen.getByText('Mobile App')).toBeInTheDocument()
      expect(screen.getByText('2 hours ago')).toBeInTheDocument()
    })

    it('shows logout all devices button', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /bảo mật/i }))

      expect(screen.getByRole('button', { name: /đăng xuất tất cả thiết bị/i })).toBeInTheDocument()
    })
  })

  describe('Tab Navigation', () => {
    it('switches between tabs correctly', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      // Start with Bot Settings
      expect(screen.getByTestId('bot-settings')).toBeInTheDocument()

      // Switch to API Keys
      await user.click(screen.getByRole('tab', { name: /api keys/i }))
      expect(screen.getByText('Binance API Configuration')).toBeInTheDocument()

      // Switch to Notifications
      await user.click(screen.getByRole('tab', { name: /thông báo/i }))
      expect(screen.getByText(/tùy chọn thông báo/i)).toBeInTheDocument()

      // Switch to Security
      await user.click(screen.getByRole('tab', { name: /bảo mật/i }))
      expect(screen.getByText(/bảo mật tài khoản/i)).toBeInTheDocument()

      // Switch back to Bot Settings
      await user.click(screen.getByRole('tab', { name: /bot settings/i }))
      expect(screen.getByTestId('bot-settings')).toBeInTheDocument()
    })
  })

  describe('Form Interactions', () => {
    it('updates API key input values', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /api keys/i }))

      const apiKeyInput = screen.getByLabelText('API Key')
      const secretKeyInput = screen.getByLabelText('Secret Key')

      await user.clear(apiKeyInput)
      await user.type(apiKeyInput, 'new-api-key')
      await user.clear(secretKeyInput)
      await user.type(secretKeyInput, 'new-secret-key')

      expect(apiKeyInput).toHaveValue('new-api-key')
      expect(secretKeyInput).toHaveValue('new-secret-key')
    })

    it('updates telegram token input', async () => {
      const user = userEvent.setup()
      render(<Settings />)

      await user.click(screen.getByRole('tab', { name: /thông báo/i }))

      const telegramTokenInput = screen.getByLabelText(/telegram bot token/i)
      await user.type(telegramTokenInput, '123456:ABC-DEF')

      expect(telegramTokenInput).toHaveValue('123456:ABC-DEF')
    })
  })
})
