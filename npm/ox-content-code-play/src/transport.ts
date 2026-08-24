import type { CodePlayTransport, TransportRequest, TransportResponse } from "./types";

export function createFetchTransport(fetchImpl: typeof fetch = fetch): CodePlayTransport {
  return {
    async request(input: TransportRequest): Promise<TransportResponse> {
      const response = await fetchImpl(input.url, {
        method: input.method,
        headers: input.headers,
        body: input.body,
      });
      return {
        ok: response.ok,
        status: response.status,
        text: await response.text(),
      };
    },
  };
}

export function createMemoryTransport(
  handler: (input: TransportRequest) => TransportResponse | Promise<TransportResponse>,
): CodePlayTransport {
  return {
    request: (input) => Promise.resolve(handler(input)),
  };
}

export class MissingTransportError extends Error {
  constructor(host: string) {
    super(`No Code Play transport is configured for ${host}.`);
    this.name = "MissingTransportError";
  }
}

export function createUnavailableTransport(): CodePlayTransport {
  return {
    request(input) {
      return Promise.reject(new MissingTransportError(input.url));
    },
  };
}
