"""Attach the exact head benchmark harness to a historical base checkout."""
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
if "rayon =" not in text:
    text = text.replace("[dependencies]", "[dependencies]\nrayon = { workspace = true }")
if 'name = "throughput"' not in text:
    text += '\n[[bench]]\nname = "throughput"\nharness = false\n'
path.write_text(text)
