import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import React from 'react'
import { LanguageProvider, useLanguage, SUPPORTED_LANGUAGES } from '../../contexts/LanguageContext'

// Mock i18n config
vi.mock('../../i18n/config', () => ({
  SUPPORTED_LANGUAGES: {
    en: { name: 'English', nativeName: 'English', flag: '🇺🇸' },
    vi: { name: 'Vietnamese', nativeName: 'Tiếng Việt', flag: '🇻🇳' },
    fr: { name: 'French', nativeName: 'Français', flag: '🇫🇷' },
    zh: { name: 'Chinese', nativeName: '中文', flag: '🇨🇳' },
    ja: { name: 'Japanese', nativeName: '日本語', flag: '🇯🇵' },
  },
  DEFAULT_LANGUAGE: 'en',
  changeLanguage: vi.fn().mockResolvedValue(undefined),
}))

// Mock react-i18next
const mockI18n = {
  language: 'en',
  on: vi.fn(),
  off: vi.fn(),
  changeLanguage: vi.fn().mockResolvedValue(undefined),
}

vi.mock('react-i18next', () => ({
  useTranslation: vi.fn(() => ({
    i18n: mockI18n,
    t: (key: string) => key,
  })),
}))

describe('LanguageContext', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockI18n.on.mockClear()
    mockI18n.off.mockClear()
  })

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <LanguageProvider>{children}</LanguageProvider>
  )

  it('provides supported languages list with 5 languages', () => {
    const { result } = renderHook(() => useLanguage(), { wrapper })

    expect(Object.keys(result.current.languages)).toHaveLength(5)
    expect(Object.keys(result.current.languages)).toEqual(['en', 'vi', 'fr', 'zh', 'ja'])
  })

  it('initializes with english as default language', () => {
    const { result } = renderHook(() => useLanguage(), { wrapper })

    expect(result.current.language).toBe('en')
  })

  it('provides setLanguage function', () => {
    const { result } = renderHook(() => useLanguage(), { wrapper })

    expect(typeof result.current.setLanguage).toBe('function')
  })

  it('provides isRTL flag defaulting to false for LTR languages', () => {
    const { result } = renderHook(() => useLanguage(), { wrapper })

    expect(result.current.isRTL).toBe(false)
  })

  it('registers and cleans up languageChanged event listener', () => {
    const { unmount } = renderHook(() => useLanguage(), { wrapper })

    expect(mockI18n.on).toHaveBeenCalledWith('languageChanged', expect.any(Function))

    unmount()

    expect(mockI18n.off).toHaveBeenCalledWith('languageChanged', expect.any(Function))
  })

  it('throws error when useLanguage called outside provider', () => {
    expect(() => {
      renderHook(() => useLanguage())
    }).toThrow('useLanguage must be used within a LanguageProvider')
  })

  it('re-exports SUPPORTED_LANGUAGES from config', () => {
    expect(SUPPORTED_LANGUAGES).toBeDefined()
    expect(Object.keys(SUPPORTED_LANGUAGES)).toHaveLength(5)
  })

  it('calls changeLanguage when setLanguage is invoked', async () => {
    const { changeLanguage } = await import('../../i18n/config')
    const { result } = renderHook(() => useLanguage(), { wrapper })

    await act(async () => {
      await result.current.setLanguage('vi')
    })

    expect(changeLanguage).toHaveBeenCalledWith('vi')
  })

  it('provides non-RTL flag for all supported languages', () => {
    const { result } = renderHook(() => useLanguage(), { wrapper })

    // All supported languages (en, vi, fr, zh, ja) are LTR
    expect(result.current.isRTL).toBe(false)
  })
})
