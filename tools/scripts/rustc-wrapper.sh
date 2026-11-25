#!/bin/sh
# Cargo's RUSTC_WRAPPER for the Nix dev shell and the Rust CI jobs.
# Cargo runs: rustc-wrapper.sh <rustc or clippy-driver> [rustc] <arguments>
#
# Workspace crates compile with rustc's parallel frontend
# (`-Zunstable-options --jobs-frontend=N`) on the stable compiler that
# rust-toolchain.toml selects. Unstable options are enabled for that one crate
# only, with RUSTC_BOOTSTRAP=<crate name>:
# - RUSTFLAGS would change every crate's `-C metadata` and miss every target
#   cache; flags added here are invisible to Cargo's fingerprints.
# - A workspace-wide RUSTC_BOOTSTRAP=1 lets dependency build scripts detect a
#   "nightly" compiler and turn on unstable features (proc-macro2 enables
#   `proc_macro_span` and fails with E0554). A build-script probe inherits its
#   package's CARGO_CRATE_NAME but passes its own --crate-name, so it never
#   qualifies here.
# When OX_CONTENT_KACHE names a kache binary, every call goes through it.
case "${OX_CONTENT_RUSTC_FRONTEND_JOBS:-1}" in
  '' | 0 | 1) ;;
  *)
    workspace="${0%/tools/scripts/rustc-wrapper.sh}"
    case "${CARGO_MANIFEST_DIR:-}" in
      "$workspace"/*)
        previous=
        for argument in "$@"; do
          if [ "$previous" = --crate-name ]; then
            if [ "$argument" = "${CARGO_CRATE_NAME:-}" ]; then
              RUSTC_BOOTSTRAP="$argument"
              export RUSTC_BOOTSTRAP
              set -- "$@" -Zunstable-options "--jobs-frontend=$OX_CONTENT_RUSTC_FRONTEND_JOBS"
            fi
            break
          fi
          previous="$argument"
        done
        ;;
    esac
    ;;
esac
if [ -n "${OX_CONTENT_KACHE:-}" ]; then
  exec "$OX_CONTENT_KACHE" "$@"
fi
exec "$@"
