import { describe, expect, it } from 'vitest'
import {
  buildSavedFiltersSettingKey,
  parseSavedInboxFilters,
} from '../app/utils/savedFilters'

describe('Phase 4 saved inbox filters', () => {
  it('builds project-scoped setting keys', () => {
    expect(buildSavedFiltersSettingKey('proj-123')).toBe('saved_filters.proj-123')
  })

  it('parses valid saved filters and discards malformed or incomplete entries', () => {
    const raw = JSON.stringify([
      {
        id: 'filter-1',
        name: 'OTP Emails',
        channelType: 'email',
        status: 'captured',
        testRunIdentifier: 'ci-99',
        searchQuery: 'otp:true',
      },
      {
        id: '',
        name: 'Missing ID',
        channelType: 'sms',
      },
      {
        id: 'filter-2',
        name: 'Invalid Enum Fallback',
        channelType: 'unknown',
        status: 'invalid',
        testRunIdentifier: 42,
        searchQuery: '/code-[0-9]+/',
      },
    ])

    expect(parseSavedInboxFilters(raw)).toEqual([
      {
        id: 'filter-1',
        name: 'OTP Emails',
        channelType: 'email',
        status: 'captured',
        testRunIdentifier: 'ci-99',
        searchQuery: 'otp:true',
      },
      {
        id: 'filter-2',
        name: 'Invalid Enum Fallback',
        channelType: 'all',
        status: 'all',
        testRunIdentifier: '',
        searchQuery: '/code-[0-9]+/',
      },
    ])
  })

  it('returns empty array on null or invalid JSON', () => {
    expect(parseSavedInboxFilters(null)).toEqual([])
    expect(parseSavedInboxFilters('')).toEqual([])
    expect(parseSavedInboxFilters('{not-json')).toEqual([])
  })
})
