import { describe, expect, it } from 'vitest'
import {
  DEFAULT_CHANNEL_FORWARDING_CONFIGURATION,
  parseMockHeadersJson,
} from '../app/utils/channelRules'

describe('Phase 3 Ingress Tunnel & Auto-Forwarding Rules', () => {
  it('initializes default channel forwarding rule configuration', () => {
    expect(DEFAULT_CHANNEL_FORWARDING_CONFIGURATION.isEnabled).toBe(false)
    expect(DEFAULT_CHANNEL_FORWARDING_CONFIGURATION.targetUrl).toContain('http://127.0.0.1:8000')
    expect(DEFAULT_CHANNEL_FORWARDING_CONFIGURATION.timeoutSeconds).toBe(5)
    expect(DEFAULT_CHANNEL_FORWARDING_CONFIGURATION.customHeaders).toEqual({})
  })

  it('validates forwarding and mock headers parsing correctly', () => {
    const valid = parseMockHeadersJson('{"X-Forward-Key": "test1234", "X-Service": "billing"}')
    expect(valid.isValid).toBe(true)
    expect(valid.headers).toEqual({
      'X-Forward-Key': 'test1234',
      'X-Service': 'billing',
    })

    const invalid = parseMockHeadersJson('not valid json')
    expect(invalid.isValid).toBe(false)
    expect(invalid.headers).toBeNull()
  })
})
