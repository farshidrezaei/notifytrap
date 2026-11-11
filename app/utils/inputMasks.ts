import { Mask, type MaskInputOptions } from 'maska'
import { UNICODE_DIGIT_CHARACTER_PATTERN } from './numberNormalization'

export type InputMaskKey =
  | 'displayName'
  | 'singleLineAsciiSafe'
  | 'descriptionText'
  | 'searchKeyword'
  | 'slugIdentifier'
  | 'tcpPort'
  | 'ipv4Address'
  | 'phoneE164'
  | 'verificationCode'
  | 'integerCount'
  | 'payloadMegabytes'
  | 'retentionDays'

const SINGLE_LINE_CONTROL_SAFE_TOKENS: MaskInputOptions = {
  mask: 'c',
  tokens: {
    c: {
      // eslint-disable-next-line no-control-regex
      pattern: /[^\r\n\t\x00-\x1f\x7f]/,
      repeated: true,
    },
  },
}

const DIGIT_HASH_TOKEN: MaskInputOptions['tokens'] = {
  '#': {
    pattern: UNICODE_DIGIT_CHARACTER_PATTERN,
  },
}

/**
 * Centralized Maska preset definitions shared by all masked form inputs.
 */
export const INPUT_MASKS: Record<InputMaskKey, MaskInputOptions> = {
  displayName: SINGLE_LINE_CONTROL_SAFE_TOKENS,
  singleLineAsciiSafe: SINGLE_LINE_CONTROL_SAFE_TOKENS,
  descriptionText: {
    mask: 'd',
    tokens: {
      d: {
        // eslint-disable-next-line no-control-regex
        pattern: /[^\x00-\x08\x0b\x0c\x0e-\x1f\x7f]/,
        repeated: true,
      },
    },
  },
  searchKeyword: SINGLE_LINE_CONTROL_SAFE_TOKENS,
  slugIdentifier: {
    mask: 's',
    tokens: {
      s: {
        pattern: /[a-z0-9-]/i,
        repeated: true,
        transform: (character: string) => character.toLowerCase(),
      },
    },
  },
  tcpPort: {
    mask: '#####',
    tokens: DIGIT_HASH_TOKEN,
  },
  ipv4Address: {
    mask: 'd.d.d.d',
    tokens: {
      d: {
        pattern: UNICODE_DIGIT_CHARACTER_PATTERN,
        multiple: true,
      },
    },
  },
  phoneE164: {
    mask: '+## ### ### ####',
    tokens: DIGIT_HASH_TOKEN,
  },
  verificationCode: {
    mask: '###-###',
    tokens: DIGIT_HASH_TOKEN,
  },
  integerCount: {
    mask: 'd',
    tokens: {
      d: {
        pattern: UNICODE_DIGIT_CHARACTER_PATTERN,
        repeated: true,
      },
    },
  },
  payloadMegabytes: {
    mask: '###',
    tokens: DIGIT_HASH_TOKEN,
  },
  retentionDays: {
    mask: '####',
    tokens: DIGIT_HASH_TOKEN,
  },
}

/**
 * Applies a named NotifyTrap mask deterministically to a raw input string.
 */
export function applyInputMask(maskKey: InputMaskKey, rawValue: string): string {
  const mask = new Mask(INPUT_MASKS[maskKey])
  return mask.masked(rawValue)
}

/**
 * Returns the unmasked token value extracted by a named NotifyTrap mask.
 */
export function unmaskInputValue(maskKey: InputMaskKey, maskedValue: string): string {
  const mask = new Mask(INPUT_MASKS[maskKey])
  return mask.unmasked(maskedValue)
}
