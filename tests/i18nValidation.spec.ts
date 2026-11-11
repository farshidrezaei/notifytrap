import { describe, expect, it } from 'vitest'
import {
  extractPlaceholders,
  flattenLocaleDictionary,
  validateLocaleDirectory,
} from '../scripts/validate-i18n.mjs'

describe('i18n translation validation', () => {
  it('passes validation across all project locale files', () => {
    const errors = validateLocaleDirectory()
    expect(errors).toEqual([])
  })

  it('extracts deterministic sorted placeholders', () => {
    expect(extractPlaceholders('Schema v{version} for {project}')).toEqual([
      '{project}',
      '{version}',
    ])
  })

  it('reports empty translation strings and invalid node shapes', () => {
    const errors: string[] = []
    flattenLocaleDictionary(
      {
        valid: 'Hello',
        empty: '   ',
        badArray: ['a'],
      },
      '',
      errors,
    )
    expect(errors).toHaveLength(2)
  })
})
