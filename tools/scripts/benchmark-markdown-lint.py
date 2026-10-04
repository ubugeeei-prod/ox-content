"""Measure native CLI wall time, including discovery, reads, and JSON stdout."""
import json
import pathlib
import statistics
import subprocess
import sys
import tempfile
import time

binary = str(pathlib.Path(sys.argv[1]).resolve())
with tempfile.TemporaryDirectory(prefix="ox-lint-perf-") as directory:
    root = pathlib.Path(directory)
    paragraph = "This is clear prose with a [visible link](https://example.com) and `code`.\n\n"
    for index in range(1024):
        (root / f"page-{index:04d}.md").write_text("# Guide\n\n" + paragraph * 100)
    (root / ".oxlint.json").write_text(json.dumps({"textRules": {"sentenceLength": 100, "maxTen": 3, "noTodo": True}}))
    rows = []
    expected = None
    for threads in (1, 8):
        command = [binary, "lint", "--threads", str(threads), "--strict", "--format", "json"]
        sample = subprocess.run(command, cwd=root, check=True, capture_output=True)
        report = json.loads(sample.stdout)
        assert report["checkedFileCount"] == 1024
        assert report["diagnostics"] == []
        stable = {key: value for key, value in report.items() if key != "durationMs"}
        if expected is None:
            expected = stable
        assert stable == expected, "parallel output must match serial output"
        elapsed = []
        for _ in range(7):
            start = time.perf_counter()
            # A real pipe exercises stdout serialization/buffering as well as lint.
            subprocess.run(command, cwd=root, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            elapsed.append(time.perf_counter() - start)
        rows.append({"threads": threads, "medianMs": statistics.median(elapsed) * 1000})
    print(json.dumps({"documents": 1024, "sourceBytes": sum(path.stat().st_size for path in root.glob("*.md")), "runs": rows, "speedup": rows[0]["medianMs"] / rows[1]["medianMs"]}))
