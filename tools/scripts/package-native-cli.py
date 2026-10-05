"""Verify and package the standalone Rust runner, with reproducible archives."""
import gzip
import hashlib
import io
import json
import os
import pathlib
import re
import subprocess
import sys
import tarfile
import tempfile
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
plugin = root / "editors/neovim"
files.extend((f"neovim/{path.relative_to(plugin).as_posix()}", path.read_bytes(), 0o644)
             for path in sorted(plugin.rglob("*")) if path.is_file())
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

# Verify the actual archive contents in an isolated installation.
with tempfile.TemporaryDirectory(prefix="ox-native-cli-") as temporary:
    installed = pathlib.Path(temporary)
    with (zipfile.ZipFile(archive) if binary.suffix == ".exe" else tarfile.open(archive)) as package:
        for filename, content, mode in files:
            packed = (package.read(filename) if binary.suffix == ".exe"
                      else package.extractfile(filename).read())
            assert packed == content, filename
            destination = installed / filename
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(packed)
            destination.chmod(mode)
    runner = installed / binary.name
    installed_version = subprocess.check_output([str(runner), "--version"], text=True).strip()
    assert installed_version == version
    environment = os.environ.copy()
    environment["XDG_DATA_HOME"] = str(installed / "ide-data")
    subprocess.run([str(runner), "ide", "install", "--ide", "neovim", "--extensions-only", "--yes"],
                   cwd=installed, env=environment, check=True, capture_output=True)
    copied = installed / "ide-data/nvim/site/pack/ox-content/start/ox-content/plugin/ox-content.lua"
    assert copied.read_bytes() == (plugin / "plugin/ox-content.lua").read_bytes()
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_suffix(archive.suffix + ".sha256").write_text(f"{digest}  {archive.name}\n")
print(f"Verified 53 rules and packaged {archive.name} ({archive.stat().st_size} bytes)")
