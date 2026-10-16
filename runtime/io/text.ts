// `TextEncoder` and `TextDecoder`, for UTF-8 only: the host moves bytes, and the app
// turns them into text.

/** How many UTF-16 units go to one `String.fromCharCode` call. */
const CHUNK = 8192;

const UTF8_LABELS = new Set(["utf-8", "utf8", "unicode-1-1-utf-8"]);

/** Turns text into UTF-8 bytes. A lone surrogate becomes U+FFFD, as in browsers. */
export class TextEncoder {
  readonly encoding = "utf-8";

  encode(input = ""): Uint8Array {
    const text = String(input);
    const bytes = new Uint8Array(text.length * 3);
    return bytes.slice(0, this.encodeInto(text, bytes).written);
  }

  encodeInto(source: string, destination: Uint8Array): { read: number; written: number } {
    let read = 0;
    let written = 0;
    while (read < source.length) {
      let point = source.charCodeAt(read);
      let units = 1;
      if (point >= 0xd800 && point <= 0xdbff && read + 1 < source.length) {
        const next = source.charCodeAt(read + 1);
        if (next >= 0xdc00 && next <= 0xdfff) {
          point = 0x10000 + ((point - 0xd800) << 10) + (next - 0xdc00);
          units = 2;
        }
      }
      if (point >= 0xd800 && point <= 0xdfff) {
        point = 0xfffd;
      }
      const size = point < 0x80 ? 1 : point < 0x800 ? 2 : point < 0x10000 ? 3 : 4;
      if (written + size > destination.length) {
        break;
      }
      written = writePoint(destination, written, point, size);
      read += units;
    }
    return { read, written };
  }
}

function writePoint(bytes: Uint8Array, at: number, point: number, size: number): number {
  if (size === 1) {
    bytes[at++] = point;
  } else if (size === 2) {
    bytes[at++] = 0xc0 | (point >> 6);
    bytes[at++] = 0x80 | (point & 0x3f);
  } else if (size === 3) {
    bytes[at++] = 0xe0 | (point >> 12);
    bytes[at++] = 0x80 | ((point >> 6) & 0x3f);
    bytes[at++] = 0x80 | (point & 0x3f);
  } else {
    bytes[at++] = 0xf0 | (point >> 18);
    bytes[at++] = 0x80 | ((point >> 12) & 0x3f);
    bytes[at++] = 0x80 | ((point >> 6) & 0x3f);
    bytes[at++] = 0x80 | (point & 0x3f);
  }
  return at;
}

/** Turns UTF-8 bytes into text. Invalid bytes become U+FFFD, or throw with `fatal`. */
export class TextDecoder {
  readonly encoding = "utf-8";
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;
  /** The start of a character that a streamed chunk ended in the middle of. */
  private pending: number[] = [];
  private seenStart = false;

  constructor(label = "utf-8", options: { fatal?: boolean; ignoreBOM?: boolean } = {}) {
    if (!UTF8_LABELS.has(String(label).trim().toLowerCase())) {
      throw new RangeError(`The encoding label provided ('${label}') is invalid; only utf-8 is supported.`);
    }
    this.fatal = options.fatal === true;
    this.ignoreBOM = options.ignoreBOM === true;
  }

  decode(input?: ArrayBuffer | ArrayBufferView, options: { stream?: boolean } = {}): string {
    const chunk = toBytes(input);
    let bytes = chunk;
    if (this.pending.length > 0) {
      bytes = new Uint8Array(this.pending.length + chunk.length);
      bytes.set(this.pending);
      bytes.set(chunk, this.pending.length);
      this.pending = [];
    }
    const streaming = options.stream === true;
    const end = streaming ? completeLength(bytes) : bytes.length;
    if (end < bytes.length) {
      this.pending = Array.from(bytes.subarray(end));
    }
    let text = this.decodeComplete(bytes.subarray(0, end));
    if (!this.seenStart && text.length > 0) {
      this.seenStart = true;
      if (!this.ignoreBOM && text.charCodeAt(0) === 0xfeff) {
        text = text.slice(1);
      }
    }
    if (!streaming) {
      this.seenStart = false;
    }
    return text;
  }

  private decodeComplete(bytes: Uint8Array): string {
    // Every character takes at most as many UTF-16 units as it has bytes.
    const units = new Uint16Array(bytes.length);
    let length = 0;
    let at = 0;
    while (at < bytes.length) {
      if (bytes[at] < 0x80) {
        units[length++] = bytes[at++];
        continue;
      }
      const [point, size] = readPoint(bytes, at);
      at += size;
      if (point === -1) {
        if (this.fatal) {
          throw new TypeError("The encoded data was not valid for encoding utf-8");
        }
        units[length++] = 0xfffd;
      } else if (point >= 0x10000) {
        units[length++] = 0xd800 + ((point - 0x10000) >> 10);
        units[length++] = 0xdc00 + ((point - 0x10000) & 0x3ff);
      } else {
        units[length++] = point;
      }
    }
    const parts: string[] = [];
    for (let start = 0; start < length; start += CHUNK) {
      const chunk = units.subarray(start, Math.min(start + CHUNK, length));
      parts.push(String.fromCharCode.apply(null, chunk as unknown as number[]));
    }
    return parts.join("");
  }
}

function toBytes(input: ArrayBuffer | ArrayBufferView | undefined): Uint8Array {
  if (input === undefined) {
    return new Uint8Array(0);
  }
  if (ArrayBuffer.isView(input)) {
    return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  }
  return new Uint8Array(input);
}

/** How many leading bytes hold only whole characters; a character cut off at the end is held back. */
function completeLength(bytes: Uint8Array): number {
  for (let back = 1; back <= Math.min(3, bytes.length); back++) {
    const byte = bytes[bytes.length - back];
    if ((byte & 0xc0) === 0x80) {
      continue;
    }
    const need = byte >= 0xf0 ? 4 : byte >= 0xe0 ? 3 : byte >= 0xc0 ? 2 : 1;
    return need > back ? bytes.length - back : bytes.length;
  }
  return bytes.length;
}

/** Reads one character at `at`: its code point (or -1 if invalid) and how many bytes it used. */
function readPoint(bytes: Uint8Array, at: number): [number, number] {
  const first = bytes[at];
  if (first < 0x80) {
    return [first, 1];
  }
  const size = first >= 0xf0 && first <= 0xf4 ? 4 : first >= 0xe0 && first < 0xf0 ? 3 : first >= 0xc2 && first < 0xe0 ? 2 : 0;
  if (size === 0) {
    return [-1, 1];
  }
  let point = first & (0xff >> (size + 1));
  for (let index = 1; index < size; index++) {
    const byte = bytes[at + index];
    if (byte === undefined || (byte & 0xc0) !== 0x80) {
      return [-1, index];
    }
    point = (point << 6) | (byte & 0x3f);
  }
  const overlong = (size === 3 && point < 0x800) || (size === 4 && point < 0x10000);
  const surrogate = point >= 0xd800 && point <= 0xdfff;
  if (overlong || surrogate || point > 0x10ffff) {
    return [-1, size];
  }
  return [point, size];
}
