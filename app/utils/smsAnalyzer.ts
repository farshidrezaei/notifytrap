export type SmsEncoding = 'GSM-7' | 'UCS-2'

export interface SmsAnalysis {
  encoding: SmsEncoding
  characterCount: number
  unitCount: number
  segmentCount: number
  maxSingleSegmentUnits: number
  maxMultiSegmentUnits: number
  charactersRemainingInSegment: number
}

const GSM7_BASIC_CHARS = new Set<string>([
  '@', '£', '$', '¥', 'è', 'é', 'ù', 'ì', 'ò', 'Ç', '\n', 'Ø', 'ø', '\r', 'Å', 'å',
  'Δ', '_', 'Φ', 'Γ', 'Λ', 'Ω', 'Π', 'Ψ', 'Σ', 'Θ', 'Ξ', 'Æ', 'æ', 'ß', 'É',
  ' ', '!', '"', '#', '¤', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
  '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
  '¡', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
  'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'Ä', 'Ö', 'Ñ', 'Ü', '§',
  '¿', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
  'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'ä', 'ö', 'ñ', 'ü', 'à',
])

const GSM7_EXTENSION_CHARS = new Set<string>([
  '\f', '^', '{', '}', '\\', '[', '~', ']', '|', '€',
])

const GSM7_SINGLE_LIMIT = 160
const GSM7_MULTI_LIMIT = 153
const UCS2_SINGLE_LIMIT = 70
const UCS2_MULTI_LIMIT = 67

/**
 * Analyzes an SMS body for GSM-7 vs UCS-2 encoding, septet/code-unit usage, and carrier segments.
 */
export function analyzeSmsMessage(messageText: string): SmsAnalysis {
  const codePoints = Array.from(messageText)
  let isGsm7 = true
  let gsmSeptets = 0

  for (const character of codePoints) {
    if (GSM7_BASIC_CHARS.has(character)) {
      gsmSeptets += 1
    } else if (GSM7_EXTENSION_CHARS.has(character)) {
      gsmSeptets += 2
    } else {
      isGsm7 = false
      break
    }
  }

  const encoding: SmsEncoding = isGsm7 ? 'GSM-7' : 'UCS-2'
  const unitCount = isGsm7 ? gsmSeptets : messageText.length
  const maxSingleSegmentUnits = isGsm7 ? GSM7_SINGLE_LIMIT : UCS2_SINGLE_LIMIT
  const maxMultiSegmentUnits = isGsm7 ? GSM7_MULTI_LIMIT : UCS2_MULTI_LIMIT

  if (unitCount === 0) {
    return {
      encoding,
      characterCount: 0,
      unitCount: 0,
      segmentCount: 0,
      maxSingleSegmentUnits,
      maxMultiSegmentUnits,
      charactersRemainingInSegment: maxSingleSegmentUnits,
    }
  }

  if (unitCount <= maxSingleSegmentUnits) {
    return {
      encoding,
      characterCount: codePoints.length,
      unitCount,
      segmentCount: 1,
      maxSingleSegmentUnits,
      maxMultiSegmentUnits,
      charactersRemainingInSegment: maxSingleSegmentUnits - unitCount,
    }
  }

  const segmentCount = Math.ceil(unitCount / maxMultiSegmentUnits)
  const capacity = segmentCount * maxMultiSegmentUnits
  return {
    encoding,
    characterCount: codePoints.length,
    unitCount,
    segmentCount,
    maxSingleSegmentUnits,
    maxMultiSegmentUnits,
    charactersRemainingInSegment: capacity - unitCount,
  }
}
