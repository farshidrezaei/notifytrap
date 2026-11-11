import type { NavigationMenuItem } from '@nuxt/ui'

/**
 * Provides project-scoped primary navigation and global utility links synchronized with i18n.
 */
export function useNavigationItems() {
  const { t } = useI18n()
  const { hasActiveScope } = useWorkspaceContext()
  const { unreadCount } = useNotifications()

  const navigationGroups = computed<NavigationMenuItem[][]>(() => {
    const projectItems: NavigationMenuItem[] = [
      {
        label: t('navigation.overview'),
        icon: 'i-lucide-layout-dashboard',
        to: '/',
      },
      {
        label: t('navigation.inbox'),
        icon: 'i-lucide-inbox',
        to: '/inbox',
        disabled: !hasActiveScope.value,
        badge: unreadCount.value > 0 ? String(unreadCount.value) : undefined,
      },
      {
        label: t('navigation.channels'),
        icon: 'i-lucide-radio',
        to: '/channels',
        disabled: !hasActiveScope.value,
      },
    ]

    const utilityItems: NavigationMenuItem[] = [
      {
        label: t('navigation.workspaces'),
        icon: 'i-lucide-building-2',
        to: '/workspaces',
      },
      {
        label: t('navigation.designSystem'),
        icon: 'i-lucide-swatch-book',
        to: '/design-system',
      },
      {
        label: t('navigation.settings'),
        icon: 'i-lucide-settings',
        to: '/settings',
      },
    ]

    return [projectItems, utilityItems]
  })

  return {
    navigationGroups,
  }
}
