import type { TimingPhase, TimingReport } from "./types";

export class PhaseTracker {
  readonly startedAt: number;
  private readonly phases: TimingPhase[] = [];
  private current: { id: string; label: string; startMs: number } | undefined;

  constructor(now: () => number = nowMs) {
    this.now = now;
    this.startedAt = now();
  }

  private readonly now: () => number;

  start(id: string, label: string): void {
    this.stop();
    this.current = { id, label, startMs: this.now() - this.startedAt };
  }

  stop(): void {
    if (!this.current) {
      return;
    }
    const durationMs = Math.max(0, this.now() - this.startedAt - this.current.startMs);
    this.phases.push({
      id: this.current.id,
      label: this.current.label,
      startMs: this.current.startMs,
      durationMs,
    });
    this.current = undefined;
  }

  report(): TimingReport {
    this.stop();
    return {
      totalMs: Math.max(0, this.now() - this.startedAt),
      phases: this.phases.slice(),
    };
  }
}

export function nowMs(): number {
  return typeof performance !== "undefined" ? performance.now() : Date.now();
}

export function emptyTiming(): TimingReport {
  return { totalMs: 0, phases: [] };
}
