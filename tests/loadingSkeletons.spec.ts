import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const ROOT_DIRECTORY = process.cwd()

function readProjectFile(relativePath: string): string {
  return fs.readFileSync(path.resolve(ROOT_DIRECTORY, relativePath), 'utf8')
}

interface SkeletonStructuralContract {
  skeletonFile: string
  targetFile: string
  expectedClasses: string[]
  requiresCardWrapper?: boolean
}

const SKELETON_CONTRACTS: SkeletonStructuralContract[] = [
  {
    skeletonFile: 'app/components/skeleton/DashboardView.vue',
    targetFile: 'app/pages/index.vue',
    expectedClasses: ['grid gap-4 md:grid-cols-3', 'grid gap-6 lg:grid-cols-2'],
    requiresCardWrapper: true,
  },
  {
    skeletonFile: 'app/components/skeleton/DashboardChannelCards.vue',
    targetFile: 'app/pages/index.vue',
    expectedClasses: ['grid gap-3 sm:grid-cols-2 lg:grid-cols-4', 'size-9'],
    requiresCardWrapper: true,
  },
  {
    skeletonFile: 'app/components/skeleton/ChannelCardList.vue',
    targetFile: 'app/components/channels/ChannelCredentialCard.vue',
    expectedClasses: ['grid gap-3 md:grid-cols-2', 'border-t border-default/70 pt-3'],
    requiresCardWrapper: true,
  },
  {
    skeletonFile: 'app/components/skeleton/WorkspaceCardList.vue',
    targetFile: 'app/pages/workspaces.vue',
    expectedClasses: ['space-y-4', 'grid gap-2 sm:grid-cols-2 lg:grid-cols-3'],
    requiresCardWrapper: true,
  },
  {
    skeletonFile: 'app/components/skeleton/ProjectGrid.vue',
    targetFile: 'app/pages/projects.vue',
    expectedClasses: ['grid gap-4 sm:grid-cols-2', 'border-t border-default pt-3'],
    requiresCardWrapper: true,
  },
  {
    skeletonFile: 'app/components/skeleton/InboxView.vue',
    targetFile: 'app/pages/inbox.vue',
    expectedClasses: ['grid min-h-0 flex-1 gap-4 lg:grid-cols-12', 'min-h-[320px] lg:col-span-5', 'min-h-[380px] lg:col-span-7'],
  },
  {
    skeletonFile: 'app/components/skeleton/InboxMessageList.vue',
    targetFile: 'app/components/inbox/InboxMessageList.vue',
    expectedClasses: ['divide-y divide-default', 'p-3'],
  },
  {
    skeletonFile: 'app/components/skeleton/InboxInspector.vue',
    targetFile: 'app/components/inbox/NotificationInspector.vue',
    expectedClasses: ['h-[420px]', 'border-b border-default pb-3'],
  },
  {
    skeletonFile: 'app/components/skeleton/SettingsFormGrid.vue',
    targetFile: 'app/pages/settings.vue',
    expectedClasses: ['grid grid-cols-1 gap-4 sm:grid-cols-3', 'border-t border-default pt-5'],
  },
]

describe('isomorphic loading skeleton contracts', () => {
  it.each(SKELETON_CONTRACTS)(
    '$skeletonFile mirrors layout geometry of $targetFile without custom animate-pulse replacements',
    ({ skeletonFile, targetFile, expectedClasses, requiresCardWrapper }) => {
      const skeletonSource = readProjectFile(skeletonFile)
      const targetSource = readProjectFile(targetFile)

      expect(skeletonSource).not.toContain('animate-pulse')
      expect(skeletonSource).toContain('USkeleton')

      if (requiresCardWrapper) {
        expect(skeletonSource).toContain('<UCard')
      }

      for (const expectedClass of expectedClasses) {
        expect(skeletonSource).toContain(expectedClass)
        if (targetFile === 'app/components/inbox/NotificationInspector.vue' && expectedClass === 'h-[420px]') {
          const emailInspectorSource = readProjectFile('app/components/inbox/EmailInspectorView.vue')
          expect(emailInspectorSource).toContain('h-[420px]')
        } else {
          expect(targetSource).toContain(expectedClass)
        }
      }
    },
  )

  it('guards all workspace-scoped pages against empty-state flash during initial hydration', () => {
    const pagesRequiringInitializationGuard = [
      'app/pages/index.vue',
      'app/pages/channels.vue',
      'app/pages/inbox.vue',
      'app/pages/workspaces.vue',
      'app/pages/projects.vue',
    ]

    for (const pagePath of pagesRequiringInitializationGuard) {
      const source = readProjectFile(pagePath)
      expect(source).toContain('isWorkspaceInitializing')
      expect(source).toMatch(/<Skeleton[A-Z]/)
    }
  })
})
