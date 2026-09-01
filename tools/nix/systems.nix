# The systems this repository supports, shared by the flake's `systems` and by
# `meta.platforms` on the packages, so the two cannot disagree about what is
# buildable.
#
# x86_64-darwin is absent: nixpkgs 26.11 dropped it.
[
  "aarch64-darwin"
  "aarch64-linux"
  "x86_64-linux"
]
