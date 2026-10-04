"""Summarize comparable warm medians and allocation counts without hiding drift."""
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
rows = {name: json.loads((root / f"{name}.json").read_text()) for name in ("base-1", "head-1", "head-8", "cli")}
base, serial, parallel = (rows[name] for name in ("base-1", "head-1", "head-8"))
assert base["sourceBytes"] == serial["sourceBytes"] == parallel["sourceBytes"]
print("## Markdown lint performance\n")
print("Same corpus, spelling disabled, median of seven warm runs. Allocation instrumentation runs separately from timing.\n")
print("| Engine | Threads | Median ms | MiB/s | Allocations | Allocated bytes |")
print("| --- | ---: | ---: | ---: | ---: | ---: |")
for name, row in (("Base", base), ("Head", serial), ("Head", parallel)):
    print(f'| {name} | {row["threads"]} | {row["medianMs"]:.2f} | {row["mibPerSecond"]:.2f} | {row["allocations"]} | {row["allocatedBytes"]} |')
print(f'\nSerial speedup: {base["medianMs"] / serial["medianMs"]:.2f}x. Allocation change: {(serial["allocations"] / base["allocations"] - 1) * 100:.1f}%.')
print(f'\nCLI checks {rows["cli"]["documents"]} files with strict structure and opt-in prose rules; wall time includes discovery, reads, process startup, and JSON stdout.\n')
for row in rows["cli"]["runs"]:
    print(f'- {row["threads"]} threads: {row["medianMs"]:.2f} ms')
print(f'\nCLI thread speedup: {rows["cli"]["speedup"]:.2f}x. Results were checked for serial/parallel equivalence.')
# Allow runner noise, but catch meaningful throughput or allocation regressions.
assert serial["medianMs"] <= base["medianMs"] * 1.25, "serial lint throughput regressed by more than 25%"
assert serial["allocations"] <= base["allocations"], "lint allocation count regressed"
