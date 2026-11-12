import { describe, expect, it } from 'vitest'
import {
  buildDefaultLoopbackReplayUrl,
  isValidLoopbackReplayUrl,
  parseMockHeadersJson,
} from '../app/utils/channelRules'

describe('Phase 3 webhook mock, loopback replay & chaos helpers', () => {
  it('validates strict loopback replay URLs and rejects external targets', () => {
    expect(isValidLoopbackReplayUrl('http://127.0.0.1:3000/hooks/stripe')).toBe(true)
    expect(isValidLoopbackReplayUrl('http://localhost:8080/webhook?debug=1')).toBe(true)
    expect(isValidLoopbackReplayUrl('http://[::1]:9000/events')).toBe(true)

    expect(isValidLoopbackReplayUrl('')).toBe(false)
    expect(isValidLoopbackReplayUrl('https://127.0.0.1:3000/hooks')).toBe(false)
    expect(isValidLoopbackReplayUrl('http://example.com/hooks')).toBe(false)
    expect(isValidLoopbackReplayUrl('http://192.168.1.5:3000/hooks')).toBe(false)
    expect(isValidLoopbackReplayUrl('http://user@127.0.0.1:3000/hooks')).toBe(false)
    expect(isValidLoopbackReplayUrl('http://127.0.0.1:3000/hooks#frag')).toBe(false)
  })

  it('builds default loopback replay URLs cleanly', () => {
    expect(buildDefaultLoopbackReplayUrl('/stripe/events')).toBe('http://127.0.0.1:3000/stripe/events')
    expect(buildDefaultLoopbackReplayUrl('hooks')).toBe('http://127.0.0.1:3000/hooks')
  })

  it('parses valid mock header JSON maps and rejects invalid header keys or arrays', () => {
    expect(parseMockHeadersJson('')).toEqual({ headers: {}, isValid: true })
    expect(parseMockHeadersJson('{"X-Mock":"true","Retry-After":5}')).toEqual({
      headers: { 'X-Mock': 'true', 'Retry-After': '5' },
      isValid: true,
    })
    expect(parseMockHeadersJson('[]').isValid).toBe(false)
    expect(parseMockHeadersJson('{invalid').isValid).toBe(false)
    expect(parseMockHeadersJson('{"Bad Header":"1"}').isValid).toBe(false)
  })
})
