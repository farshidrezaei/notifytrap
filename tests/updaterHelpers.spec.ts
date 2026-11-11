import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  AUTO_UPDATE_SETTING_KEY,
  buildAvailableUpdateSummary,
  calculateDownloadProgressPercentage,
  normalizeUpdateReleaseNotes,
  UPDATE_CHECK_TIMEOUT_MILLISECONDS,
} from '../app/utils/updaterHelpers'

describe('application updater helpers & desktop release configuration', () => {
  it('calculates clamped download progress percentages safely', () => {
    expect(calculateDownloadProgressPercentage(0, 1000)).toBe(0)
    expect(calculateDownloadProgressPercentage(512, 1024)).toBe(50)
    expect(calculateDownloadProgressPercentage(2048, 1024)).toBe(100)
    expect(calculateDownloadProgressPercentage(-50, 1024)).toBe(0)
    expect(calculateDownloadProgressPercentage(100, undefined)).toBeNull()
    expect(calculateDownloadProgressPercentage(100, 0)).toBeNull()
  })

  it('normalizes release notes and builds clean update summaries', () => {
    expect(normalizeUpdateReleaseNotes(undefined)).toBe('')
    expect(normalizeUpdateReleaseNotes('   \n  ')).toBe('')
    expect(normalizeUpdateReleaseNotes('  Added updater toast  ')).toBe('Added updater toast')

    const summary = buildAvailableUpdateSummary({
      currentVersion: '0.1.0',
      version: '0.2.0',
      date: '2026-11-12T00:00:00Z',
      body: '  Bug fixes and Linux AppImage updater  ',
    })

    expect(summary).toEqual({
      currentVersion: '0.1.0',
      version: '0.2.0',
      publishedAt: '2026-11-12T00:00:00Z',
      releaseNotes: 'Bug fixes and Linux AppImage updater',
    })
    expect(AUTO_UPDATE_SETTING_KEY).toBe('app.auto_update_check_enabled')
    expect(UPDATE_CHECK_TIMEOUT_MILLISECONDS).toBeGreaterThanOrEqual(5000)
  })

  it('configures signed updater artifacts, cross-platform bundles, and release CI workflow', () => {
    const rootDirectory = process.cwd()
    const tauriConfig = JSON.parse(
      fs.readFileSync(path.join(rootDirectory, 'src-tauri/tauri.conf.json'), 'utf8'),
    ) as {
      bundle?: { createUpdaterArtifacts?: boolean; targets?: string }
      plugins?: { updater?: { pubkey?: string; endpoints?: string[] } }
    }

    expect(tauriConfig.bundle?.targets).toBe('all')
    expect(tauriConfig.bundle?.createUpdaterArtifacts).toBe(true)
    expect(tauriConfig.plugins?.updater?.pubkey?.length ?? 0).toBeGreaterThan(20)
    expect(tauriConfig.plugins?.updater?.endpoints?.length ?? 0).toBeGreaterThan(0)

    const capabilities = JSON.parse(
      fs.readFileSync(path.join(rootDirectory, 'src-tauri/capabilities/main.json'), 'utf8'),
    ) as { permissions: string[] }
    expect(capabilities.permissions).toContain('updater:default')
    expect(capabilities.permissions).toContain('process:allow-restart')

    const releaseWorkflow = fs.readFileSync(
      path.join(rootDirectory, '.github/workflows/release.yml'),
      'utf8',
    )
    expect(releaseWorkflow).toContain('ubuntu-22.04')
    expect(releaseWorkflow).toContain('windows-latest')
    expect(releaseWorkflow).toContain('macos-latest')
    expect(releaseWorkflow).toContain('--target universal-apple-darwin')
    expect(releaseWorkflow).toContain('tauri-apps/tauri-action@v0')
    expect(releaseWorkflow).toContain('TAURI_SIGNING_PRIVATE_KEY')
  })
})
