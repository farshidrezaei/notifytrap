const SAFE_IMAGE_MIME_TYPES = new Set<string>([
  'image/png',
  'image/jpeg',
  'image/jpg',
  'image/gif',
  'image/webp',
])

const PREVIEWABLE_TEXT_MIME_TYPES = new Set<string>([
  'text/plain',
  'text/csv',
  'text/html',
  'text/xml',
  'application/json',
  'application/xml',
])

/**
 * Returns whether an attachment MIME type is a safe raster image suitable for inline/modal preview.
 */
export function isSafePreviewableImageMime(contentType: string): boolean {
  const normalized = contentType.split(';')[0]?.trim().toLowerCase() ?? ''
  return SAFE_IMAGE_MIME_TYPES.has(normalized)
}

/**
 * Returns whether an attachment MIME type can be safely previewed as UTF-8 text.
 */
export function isPreviewableTextMime(contentType: string): boolean {
  const normalized = contentType.split(';')[0]?.trim().toLowerCase() ?? ''
  return PREVIEWABLE_TEXT_MIME_TYPES.has(normalized)
}

/**
 * Replaces `cid:` image references inside sanitized email HTML with verified local data URLs.
 */
export function resolveInlineCidImages(
  sanitizedHtml: string,
  cidToDataUrl: ReadonlyMap<string, string>,
): string {
  if (!sanitizedHtml || cidToDataUrl.size === 0) {
    return sanitizedHtml
  }

  return sanitizedHtml.replace(
    /(<img\b[^>]*?\bsrc\s*=\s*['"])cid:([^'"]+)(['"][^>]*>)/gi,
    (fullMatch, prefix: string, rawCid: string, suffix: string) => {
      const normalizedCid = rawCid.replace(/^<|>$/g, '').trim().toLowerCase()
      const dataUrl = cidToDataUrl.get(normalizedCid)
      return dataUrl ? `${prefix}${dataUrl}${suffix}` : fullMatch
    },
  )
}

/**
 * Decodes standard base64 into a Uint8Array in both browser and Node/Vitest environments.
 */
export function decodeBase64ToUint8Array(base64Content: string): Uint8Array {
  const binaryString = atob(base64Content)
  const bytes = new Uint8Array(binaryString.length)
  for (let index = 0; index < binaryString.length; index += 1) {
    bytes[index] = binaryString.charCodeAt(index)
  }
  return bytes
}

/**
 * Decodes base64 attachment bytes to UTF-8 text for safe inspection.
 */
export function decodeBase64ToUtf8Text(base64Content: string): string {
  return new TextDecoder('utf-8', { fatal: false }).decode(decodeBase64ToUint8Array(base64Content))
}

/**
 * Triggers a binary attachment download inside the desktop WebView.
 */
export function triggerBinaryFileDownload(
  filename: string,
  base64Content: string,
  contentType: string,
): void {
  if (typeof document === 'undefined' || typeof URL === 'undefined') {
    return
  }
  const bytes = decodeBase64ToUint8Array(base64Content)
  const buffer = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer
  const blob = new Blob([buffer], { type: contentType || 'application/octet-stream' })
  const objectUrl = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = objectUrl
  anchor.download = filename || 'attachment.bin'
  anchor.style.display = 'none'
  document.body.appendChild(anchor)
  anchor.click()
  document.body.removeChild(anchor)
  URL.revokeObjectURL(objectUrl)
}
