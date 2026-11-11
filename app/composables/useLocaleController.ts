const LOCALE_STORAGE_KEY = 'notifytrap-locale'

export type SupportedLocaleCode = 'en' | 'fa'

/**
 * Controls bilingual locale switching and persists the chosen locale locally.
 */
export function useLocaleController() {
  const { locale, setLocale } = useI18n()

  const localeOptions: Array<{ code: SupportedLocaleCode; label: string }> = [
    { code: 'en', label: 'English (LTR)' },
    { code: 'fa', label: 'فارسی (RTL)' },
  ]

  async function changeLocale(nextLocale: SupportedLocaleCode): Promise<void> {
    await setLocale(nextLocale)
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, nextLocale)
    }
  }

  async function restorePersistedLocale(): Promise<void> {
    if (typeof window === 'undefined') {
      return
    }
    const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY)
    if ((stored === 'en' || stored === 'fa') && stored !== locale.value) {
      await setLocale(stored)
    }
  }

  return {
    locale,
    localeOptions,
    changeLocale,
    restorePersistedLocale,
  }
}
