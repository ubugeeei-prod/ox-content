import { executeAdapter, typecheckAdapter } from "./adapters";
import { mergeConfig } from "./config";
import type {
  AdapterRequest,
  CodePlayTransport,
  PlaygroundEndpoints,
  ResolvedLanguageEnable,
  RunResult,
  SessionEventMap,
  SessionEventName,
  SessionInput,
  TypeScriptLike,
} from "./types";
import type { LanguageDefinition } from "./types";

type Listener<T> = (value: T) => void;

export class CodePlaySession {
  readonly language: LanguageDefinition;
  code: string;
  config: Record<string, unknown>;
  lastResult: RunResult | undefined;
  private readonly enabled: ResolvedLanguageEnable;
  private readonly timeoutMs: number;
  private readonly transport: CodePlayTransport;
  private readonly endpoints: PlaygroundEndpoints;
  private readonly loadTypeScript?: () => Promise<TypeScriptLike | undefined>;
  private readonly listeners = new Map<SessionEventName, Set<Listener<unknown>>>();

  constructor(input: SessionConstructorInput) {
    this.language = input.definition;
    this.enabled = input.enabled;
    this.code = input.code;
    this.config = mergeConfig(input.definition.id, input.enabled, input.config);
    this.timeoutMs = input.timeoutMs;
    this.transport = input.transport;
    this.endpoints = input.endpoints;
    this.loadTypeScript = input.loadTypeScript;
  }

  on<K extends SessionEventName>(event: K, listener: Listener<SessionEventMap[K]>): () => void {
    const bucket = this.listeners.get(event) ?? new Set();
    bucket.add(listener as Listener<unknown>);
    this.listeners.set(event, bucket);
    return () => bucket.delete(listener as Listener<unknown>);
  }

  setCode(code: string): void {
    this.code = code;
  }

  setConfig(config: Record<string, unknown>): void {
    this.config = { ...this.config, ...config };
    this.emit("config", this.config);
  }

  async run(): Promise<RunResult> {
    return this.dispatch("execute");
  }

  async typecheck(): Promise<RunResult> {
    return this.dispatch("typecheck");
  }

  private async dispatch(action: "execute" | "typecheck"): Promise<RunResult> {
    const request: AdapterRequest = {
      definition: this.language,
      enabled: this.enabled,
      code: this.code,
      config: this.config,
      timeoutMs: this.timeoutMs,
      transport: this.transport,
      loadTypeScript: this.loadTypeScript,
      endpoints: this.endpoints,
    };
    const result =
      action === "typecheck" ? await typecheckAdapter(request) : await executeAdapter(request);
    this.lastResult = result;
    for (const event of result.stdio) {
      this.emit("stdio", event);
    }
    this.emit("result", result);
    return result;
  }

  private emit<K extends SessionEventName>(event: K, value: SessionEventMap[K]): void {
    for (const listener of this.listeners.get(event) ?? []) {
      listener(value);
    }
  }
}

export interface SessionConstructorInput extends SessionInput {
  definition: LanguageDefinition;
  enabled: ResolvedLanguageEnable;
  timeoutMs: number;
  transport: CodePlayTransport;
  endpoints: PlaygroundEndpoints;
  loadTypeScript?: () => Promise<TypeScriptLike | undefined>;
}
