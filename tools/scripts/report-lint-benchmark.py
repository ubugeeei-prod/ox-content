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
if (root / "japanese-base-1.json").exists():
    japanese = [json.loads((root / f"japanese-{name}.json").read_text()) for name in ("base-1", "head-1", "head-8")]
    before, after, threaded = japanese
    assert before["sourceBytes"] == after["sourceBytes"] == threaded["sourceBytes"]
    print("\nJapanese prose, same rules and API:\n")
    print("| Engine | Threads | Median ms | Allocations | Allocated bytes |")
    print("| --- | ---: | ---: | ---: | ---: |")
    for engine, row in (("Base", before), ("Head", after), ("Head", threaded)):
        print(f'| {engine} | {row["threads"]} | {row["medianMs"]:.2f} | {row["allocations"]} | {row["allocatedBytes"]} |')
    print(f'\nJapanese serial speedup: {before["medianMs"] / after["medianMs"]:.2f}x. Allocated bytes change: {(after["allocatedBytes"] / before["allocatedBytes"] - 1) * 100:.1f}%.')
    assert after["medianMs"] <= before["medianMs"] * 1.25, "Japanese lint throughput regressed"
    assert after["allocations"] <= before["allocations"], "Japanese allocation count regressed"
print(f'\nCLI checks {rows["cli"]["documents"]} files with strict structure and opt-in prose rules; wall time includes discovery, reads, process startup, and JSON stdout.\n')
for row in rows["cli"]["runs"]:
    print(f'- {row["threads"]} threads: {row["medianMs"]:.2f} ms')
print(f'\nCLI thread speedup: {rows["cli"]["speedup"]:.2f}x. Results were checked for serial/parallel equivalence.')
baseline = root / "cli-base.json"
if baseline.exists():
    cli_base = json.loads(baseline.read_text())
    print("\n| CLI corpus | Engine | Threads | Median ms | JSON output bytes |")
    print("| --- | --- | ---: | ---: | ---: |")
    for name, key in (("Clean", None), ("8192 diagnostics", "diagnosticOutput")):
        before = cli_base if key is None else cli_base[key]
        after = rows["cli"] if key is None else rows["cli"][key]
        assert before["sourceBytes"] == after["sourceBytes"]
        assert before["resultHash"] == after["resultHash"], "base/head CLI diagnostics differ"
        for engine, case in (("Base", before), ("Head", after)):
            for row in case["runs"]:
                print(f'| {name} | {engine} | {row["threads"]} | {row["medianMs"]:.2f} | {case["outputBytes"]} |')
        print(f'\n{name} CLI serial speedup: {before["runs"][0]["medianMs"] / after["runs"][0]["medianMs"]:.2f}x.\n')
# Allow runner noise, but catch meaningful throughput or allocation regressions.
assert serial["medianMs"] <= base["medianMs"] * 1.25, "serial lint throughput regressed by more than 25%"
assert serial["allocations"] <= base["allocations"], "lint allocation count regressed"
