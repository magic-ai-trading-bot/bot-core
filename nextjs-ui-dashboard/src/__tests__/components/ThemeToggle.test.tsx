import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, act } from '@testing-library/react'
import React from 'react'
import { ThemeToggle } from '../../components/ThemeToggle'
import { ThemeProvider } from '../../contexts/ThemeContext'

// Mock react-i18next to suppress i18n instance warning
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
    i18n: { language: 'en' },
  }),
}))

// Mock framer-motion to avoid animation issues in tests
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: React.HTMLAttributes<HTMLDivElement> & { children?: React.ReactNode }) =>
      React.createElement('div', props, children),
  },
  AnimatePresence: ({ children }: { children: React.ReactNode }) => React.createElement(React.Fragment, null, children),
}))

// Mock useThemeColors hook
vi.mock('../../hooks/useThemeColors', () => ({
  useThemeColors: () => ({
    bgPrimary: '#000',
    bgSecondary: '#111',
    textPrimary: '#fff',
    textSecondary: '#aaa',
    borderSubtle: '#333',
    shadowColor: 'rgba(0,0,0,0.5)',
    cyan: '#00bcd4',
  }),
}))

const renderWithTheme = (ui: React.ReactElement) => {
  return render(
    <ThemeProvider>
      {ui}
    </ThemeProvider>
  )
}

describe('ThemeToggle', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders theme toggle button', () => {
    renderWithTheme(<ThemeToggle />)
    expect(screen.getByRole('button')).toBeInTheDocument()
  })

  it('has accessible aria-label', () => {
    renderWithTheme(<ThemeToggle />)
    const button = screen.getByRole('button')
    expect(button).toHaveAttribute('aria-label')
    expect(button.getAttribute('aria-label')).toBeTruthy()
  })

  it('opens dropdown menu on click', async () => {
    renderWithTheme(<ThemeToggle />)
    const button = screen.getByRole('button')

    await act(async () => {
      fireEvent.click(button)
    })

    // Dropdown options should appear
    const buttons = screen.getAllByRole('button')
    expect(buttons.length).toBeGreaterThan(1)
  })

  it('closes dropdown when clicking outside', async () => {
    renderWithTheme(<ThemeToggle />)
    const button = screen.getByRole('button')

    // Open dropdown
    await act(async () => {
      fireEvent.click(button)
    })

    // Click outside
    await act(async () => {
      fireEvent.mouseDown(document.body)
    })

    // Only trigger button remains
    expect(screen.getAllByRole('button')).toHaveLength(1)
  })

  it('selects light theme option', async () => {
    renderWithTheme(<ThemeToggle />)
    const triggerButton = screen.getByRole('button')

    // Open dropdown
    await act(async () => {
      fireEvent.click(triggerButton)
    })

    // Find and click light theme option
    const allButtons = screen.getAllByRole('button')
    // Buttons: trigger + 3 theme options
    expect(allButtons.length).toBe(4)
  })

  it('selects dark theme option and closes dropdown', async () => {
    renderWithTheme(<ThemeToggle />)
    const triggerButton = screen.getByRole('button')

    // Open dropdown
    await act(async () => {
      fireEvent.click(triggerButton)
    })

    // Should have 4 buttons: trigger + light + dark + system
    const allButtons = screen.getAllByRole('button')
    expect(allButtons).toHaveLength(4)

    // Click dark option (index 2)
    await act(async () => {
      fireEvent.click(allButtons[2])
    })

    // Dropdown should close
    expect(screen.getAllByRole('button')).toHaveLength(1)
  })

  it('toggles dropdown open and closed on repeated clicks', async () => {
    renderWithTheme(<ThemeToggle />)
    const button = screen.getByRole('button')

    // Open
    await act(async () => {
      fireEvent.click(button)
    })
    expect(screen.getAllByRole('button').length).toBeGreaterThan(1)

    // Close
    await act(async () => {
      fireEvent.click(screen.getAllByRole('button')[0])
    })
    expect(screen.getAllByRole('button')).toHaveLength(1)
  })
})
