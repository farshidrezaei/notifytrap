import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

describe('Bulk Operations & Tagging i18n and contracts', () => {
  it('enforces complete symmetric translations for all bulk operations', () => {
    const en = JSON.parse(fs.readFileSync(path.resolve(process.cwd(), 'i18n/locales/en.json'), 'utf8'))
    const fa = JSON.parse(fs.readFileSync(path.resolve(process.cwd(), 'i18n/locales/fa.json'), 'utf8'))

    expect(en.inbox.bulk).toBeDefined()
    expect(fa.inbox.bulk).toBeDefined()

    const requiredKeys = [
      'selectedCount',
      'selectAll',
      'deselectAll',
      'markRead',
      'markUnread',
      'archive',
      'delete',
      'tag',
      'tagTitle',
      'tagPlaceholder',
      'tagSubmit',
      'deleteConfirmTitle',
      'deleteConfirmDescription',
    ]

    for (const key of requiredKeys) {
      expect(en.inbox.bulk[key]).toBeDefined()
      expect(fa.inbox.bulk[key]).toBeDefined()
      expect(typeof en.inbox.bulk[key]).toBe('string')
      expect(typeof fa.inbox.bulk[key]).toBe('string')
      expect(en.inbox.bulk[key].length).toBeGreaterThan(0)
      expect(fa.inbox.bulk[key].length).toBeGreaterThan(0)
    }

    const toastKeys = ['deleted', 'statusUpdated', 'tagged']
    for (const key of toastKeys) {
      expect(en.inbox.bulk.toast[key]).toBeDefined()
      expect(fa.inbox.bulk.toast[key]).toBeDefined()
    }
  })
})
