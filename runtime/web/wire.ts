// The messages between the page and the host (host/src/serve/mod.rs describes them): four bytes with the
// length of a JSON header, the header, and the bytes of the call or of its answer.

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** A call as the host reads it. `argsJson` is the arguments already as JSON. */
export function encodeCall(op: string, call: number, argsJson: string, bytes: ArrayBuffer | undefined): ArrayBuffer {
  const header = encoder.encode(`{"op":${JSON.stringify(op)},"call":${call},"args":${argsJson}}`);
  const payload = bytes === undefined ? 0 : bytes.byteLength;
  const message = new Uint8Array(4 + header.length + payload);
  new DataView(message.buffer).setUint32(0, header.length);
  message.set(header, 4);
  if (bytes !== undefined) {
    message.set(new Uint8Array(bytes), 4 + header.length);
  }
  return message.buffer;
}

/** What the host answered a call with: how it settled, its JSON, and its bytes when it sent any. */
export interface Reply {
  call: number;
  settle: string;
  json: unknown;
  bytes: ArrayBuffer | undefined;
}

export function decodeReply(data: ArrayBuffer): Reply {
  const length = new DataView(data).getUint32(0);
  const header = JSON.parse(decoder.decode(new Uint8Array(data, 4, length)));
  return {
    call: header.call,
    settle: header.settle,
    json: header.json,
    bytes: data.byteLength > 4 + length ? data.slice(4 + length) : undefined,
  };
}
