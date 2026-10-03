#!/usr/bin/env python3
"""Exercise the real reader terminal, including input and mode restoration."""
import fcntl
import os
import pty
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time
from pathlib import Path


def wait_for(master, marker, timeout=5):
    output = b""
    deadline = time.monotonic() + timeout
    while marker not in output:
        if time.monotonic() >= deadline:
            raise AssertionError(f"Missing terminal marker {marker!r}: {output[-2000:]!r}")
        if select.select([master], [], [], 0.1)[0]:
            output += os.read(master, 65536)
    return output


with tempfile.TemporaryDirectory(prefix="oxct-pty-") as directory:
    Path(directory, "a.md").write_text("# First\n\nSearch target\n\n" + "Content line\n\n" * 40)
    Path(directory, "b.md").write_text("# Second\n\nDifferent document\n")
    for exit_mode in ["quit", "signal"]:
        master, slave = pty.openpty()
        original = termios.tcgetattr(slave)
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
        process = subprocess.Popen(
            [sys.argv[1], sys.argv[2], "tui", directory],
            stdin=slave, stdout=slave, stderr=slave,
        )
        try:
            wait_for(master, b"OX CONTENT")
            if exit_mode == "quit":
                temporary = Path(directory, "saved.tmp")
                temporary.write_text("# First\n\nLive saved\n")
                os.replace(temporary, Path(directory, "a.md"))
                wait_for(master, b"Live saved")
                os.write(master, b"\t\x1b[B\r")
                wait_for(master, b"Different document")
                os.write(master, b"/Different\r")
                wait_for(master, b"Different")
                os.write(master, b"?")
                wait_for(master, b"READER KEYS")
                os.write(master, b"\x1b")
                time.sleep(0.1)
                os.write(master, b"q")
            else:
                process.send_signal(signal.SIGTERM)
            wait_for(master, b"\x1b[?1049l")
            assert process.wait(timeout=5) == 0
            assert termios.tcgetattr(slave) == original, "Raw terminal mode was not restored"
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            os.close(master)
            os.close(slave)
print("PTY navigation and terminal restoration passed")
