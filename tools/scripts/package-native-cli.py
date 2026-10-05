"""Verify and package the standalone Rust runner, with reproducible archives."""
import gzip
import hashlib
import io
import json
import pathlib
import re
import subprocess
import sys
import tarfile
import zipfile

root = pathlib.Path(__file__).resolve().parents[2]
binary = pathlib.Path(sys.argv[1]).resolve()
target = sys.argv[2]
version = re.search(r'^version = "([^"]+)"', (root / "Cargo.toml").read_text(), re.M)[1]
assert re.fullmatch(r"\d+\.\d+\.\d+(?:-[\w.-]+)?", version)
assert target in {"x86_64-unknown-linux-gnu", "x86_64-apple-darwin",
                  "aarch64-apple-darwin", "x86_64-pc-windows-msvc"}
actual_version = subprocess.check_output([str(binary), "--version"], text=True).strip()
assert actual_version == version, f"Expected {version}, received {actual_version!r}"
rules = json.loads(subprocess.check_output([str(binary), "lint", "--list-rules", "--format", "json"]))
assert len(rules) == 53 and rules[0]["id"] == "MD001" and rules[-1]["id"] == "MD060"
result = subprocess.run([str(binary), "lint", "--stdin", "--format", "json"],
                        input="#  Title!\n\nText.\n", text=True, capture_output=True, cwd=binary.parent)
assert result.returncode == 1, result.stderr
diagnostics = json.loads(result.stdout)["diagnostics"]
assert [value["ruleId"] for value in diagnostics] == ["MD019", "MD026"], diagnostics

files = [(binary.name, binary.read_bytes(), 0o755),
         ("LICENSE", (root / "LICENSE").read_bytes(), 0o644),
         ("markdownlint-LICENSE", (root / "crates/ox_content_markdown_lint/markdownlint-LICENSE").read_bytes(), 0o644),
         ("README.md", b"# Ox Content native CLI\n\nRun `./oxct lint . --threads 8` (Windows: `oxct.exe`).\nUse `oxct lint --help` or `oxct lint --list-rules` for options.\nDocumentation: https://github.com/ubugeeei-prod/ox-content/tree/main/docs\n", 0o644)]
output = root / "dist/native"
output.mkdir(parents=True, exist_ok=True)
name = f"oxct-v{version}-{target}"
if binary.suffix == ".exe":
    archive = output / f"{name}.zip"
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as package:
        for filename, content, mode in files:
            entry = zipfile.ZipInfo(filename, date_time=(1980, 1, 1, 0, 0, 0))
            entry.external_attr = (mode | 0o100000) << 16
            entry.compress_type = zipfile.ZIP_DEFLATED
            package.writestr(entry, content)
else:
    archive = output / f"{name}.tar.gz"
    with archive.open("wb") as file, gzip.GzipFile(fileobj=file, filename="", mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode="w") as package:
            for filename, content, mode in files:
                entry = tarfile.TarInfo(filename)
                entry.mode = mode
                entry.size = len(content)
                package.addfile(entry, io.BytesIO(content))
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_suffix(archive.suffix + ".sha256").write_text(f"{digest}  {archive.name}\n")
print(f"Verified 53 rules and packaged {archive.name} ({archive.stat().st_size} bytes)")
