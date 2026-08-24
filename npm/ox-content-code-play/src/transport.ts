import type { CodePlayTransport, TransportRequest, TransportResponse } from "./types";

export function createFetchTransport(fetchImpl: typeof fetch = fetch): CodePlayTransport {
  return {
    async request(input: TransportRequest): Promise<TransportResponse> {
      const response = await fetchImpl(input.url, {
        method: input.method,
        headers: input.headers,
        body: input.body,
        signal: input.signal,
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
    async request(input) {
      if (input.signal?.aborted) {
        throw abortError();
      }
      return handler(input);
    },
  };
}

export class MissingTransportError extends Error {
  constructor(host: string) {
    super(`No Code Play transport is configured for ${host}.`);
    this.name = "MissingTransportError";
  }
}

export function abortError(): Error {
  const error = new Error("The Code Play run was cancelled.");
  error.name = "AbortError";
  return error;
}

export function isAbortError(error: unknown): boolean {
  return Boolean(
    error && typeof error === "object" && "name" in error && error.name === "AbortError",
  );
}

export function createUnavailableTransport(): CodePlayTransport {
  return {
    request(input) {
      if (input.signal?.aborted) {
        return Promise.reject(abortError());
      }
      return Promise.reject(new MissingTransportError(input.url));
    },
  };
}
