import chardet from 'chardet';

const UTF8_ALIASES = new Set(['UTF-8', 'ASCII']);

/**
 * Read a File, detect its character encoding, and return a new File
 * guaranteed to contain UTF-8 text.  If the file is already UTF-8 (or
 * ASCII) it is returned unchanged.
 */
export async function ensureUtf8File(file: File): Promise<File> {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);

  const encoding = chardet.detect(bytes);

  if (!encoding || UTF8_ALIASES.has(encoding)) {
    return file;
  }

  const label = toTextDecoderLabel(encoding);
  const decoder = new TextDecoder(label);
  const text = decoder.decode(bytes);

  const utf8Blob = new Blob([text], { type: file.type || 'text/plain' });
  return new File([utf8Blob], file.name, {
    type: file.type || 'text/plain',
    lastModified: file.lastModified,
  });
}

/**
 * Map chardet encoding names to labels accepted by the WHATWG TextDecoder.
 * Most names already match; this handles the few mismatches.
 */
function toTextDecoderLabel(enc: string): string {
  const map: Record<string, string> = {
    'ISO-8859-1': 'windows-1252',
    'ISO-8859-9': 'windows-1254',
  };
  return map[enc] ?? enc;
}
