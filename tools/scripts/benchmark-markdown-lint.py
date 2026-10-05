"""Measure native CLI wall time, including reads and clean/diagnostic JSON stdout."""
import hashlib
import json
import pathlib
import statistics
import subprocess
import sys
import tempfile
import time

binary = str(pathlib.Path(sys.argv[1]).resolve())
profile = sys.argv[2] if len(sys.argv) > 2 else "legacy"
expected_status = lambda diagnostics: 1 if profile == "markdownlint" and diagnostics else 0


def measure(root, diagnostics):
    rows = []
    expected = None
    for threads in (1, 8):
        command = [binary, "lint", "--threads", str(threads), "--strict", "--max-warnings", "100000", "--format", "json"]
        if profile == "markdownlint":
            command.remove("--strict")
        sample = subprocess.run(command, cwd=root, capture_output=True)
        assert sample.returncode == expected_status(diagnostics), sample.stderr
        report = json.loads(sample.stdout)
        assert report["checkedFileCount"] == 1024
        assert len(report["diagnostics"]) == diagnostics
        stable = {key: value for key, value in report.items() if key != "durationMs"}
        if expected is None:
            expected = stable
        assert stable == expected, "parallel output must match serial output"
        elapsed = []
        for _ in range(7):
            start = time.perf_counter()
            measured = subprocess.run(command, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            assert measured.returncode == expected_status(diagnostics), measured.stderr
            elapsed.append(time.perf_counter() - start)
        rows.append({"threads": threads, "medianMs": statistics.median(elapsed) * 1000})
    return {"profile": profile, "documents": 1024, "sourceBytes": sum(path.stat().st_size for path in root.glob("*.md")),
            "diagnosticCount": diagnostics, "outputBytes": len(sample.stdout),
            "resultHash": hashlib.sha256(json.dumps(expected, sort_keys=True).encode()).hexdigest(),
            "runs": rows, "speedup": rows[0]["medianMs"] / rows[1]["medianMs"]}


with tempfile.TemporaryDirectory(prefix="ox-lint-perf-") as directory:
    root = pathlib.Path(directory)
    paragraph = "This is clear prose with a [visible link](https://example.com) and `code`.\n\n"
    for index in range(1024):
        (root / f"page-{index:04d}.md").write_text("# Guide\n\n" + paragraph * 100)
    (root / ".oxlint.json").write_text(json.dumps({"textRules": {"sentenceLength": 100, "maxTen": 3, "noTodo": True}}))
    if profile == "markdownlint":
        (root / ".oxlint.json").write_text(json.dumps({"markdownlint": {}}))
    clean = measure(root, 0)
    for path in root.glob("*.md"):
        path.write_text("#  Guide!\n\nText.\n" if profile == "markdownlint" else "# Guide\n\n" + "TODO: finish this paragraph.\n\n" * 8)
    clean["diagnosticOutput"] = measure(root, 2048 if profile == "markdownlint" else 8192)
    print(json.dumps(clean))
