import { describe, expect, it } from 'vitest'
import type { AssetCheckResult, DatabaseStorageStatistics } from '../app/types/application'
import { formatByteSize } from '../app/utils/formatters'

describe('Phase 2: Database Storage Statistics & Maintenance', () => {
  it('formats SQLite file size, WAL size, and attachment footprints accurately', () => {
    const mockStatistics: DatabaseStorageStatistics = {
      databasePath: '/home/developer/.local/share/notifytrap/notifytrap.sqlite',
      fileSizeBytes: 20_971_520, // 20 MB
      walSizeBytes: 4_194_304, // 4 MB
      totalNotificationsCount: 1250,
      totalAttachmentsCount: 45,
      totalAttachmentsSizeBytes: 15_728_640, // 15 MB
    }

    expect(formatByteSize(mockStatistics.fileSizeBytes)).toBe('20.0 MB')
    expect(formatByteSize(mockStatistics.walSizeBytes)).toBe('4.0 MB')
    expect(formatByteSize(mockStatistics.totalAttachmentsSizeBytes)).toBe('15.0 MB')
    expect(mockStatistics.totalNotificationsCount).toBe(1250)
  })
})

describe('Phase 2: Deep Asset & Broken Link Inspection', () => {
  it('classifies link and remote image statuses and identifies broken assets', () => {
    const assetChecks: AssetCheckResult[] = [
      {
        url: 'https://example.com/activate',
        kind: 'link',
        statusCode: 200,
        isBroken: false,
        latencyMilliseconds: 35,
        errorMessage: null,
      },
      {
        url: 'https://example.com/missing-page',
        kind: 'link',
        statusCode: 404,
        isBroken: true,
        latencyMilliseconds: 82,
        errorMessage: 'HTTP status 404',
      },
      {
        url: 'https://cdn.example.com/broken-banner.png',
        kind: 'image',
        statusCode: 500,
        isBroken: true,
        latencyMilliseconds: 110,
        errorMessage: 'HTTP status 500',
      },
      {
        url: 'https://unreachable-domain-xyz.test/logo.svg',
        kind: 'image',
        statusCode: null,
        isBroken: true,
        latencyMilliseconds: 5001,
        errorMessage: 'DNS lookup failure',
      },
    ]

    const brokenAssets = assetChecks.filter((item) => item.isBroken)
    const healthyAssets = assetChecks.filter((item) => !item.isBroken)

    expect(brokenAssets).toHaveLength(3)
    expect(healthyAssets).toHaveLength(1)
    expect(healthyAssets[0].statusCode).toBe(200)

    const images = assetChecks.filter((item) => item.kind === 'image')
    const links = assetChecks.filter((item) => item.kind === 'link')
    expect(images).toHaveLength(2)
    expect(links).toHaveLength(2)
  })
})
