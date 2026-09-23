#!/usr/bin/env bash

set -uo pipefail

log_file="$(mktemp)"
trap 'rm -f "$log_file"' EXIT

for attempt in 1 2 3; do
  "$@" 2>&1 | tee "$log_file"
  result=${PIPESTATUS[0]}

  if (( result == 0 )); then
    exit 0
  fi

  # This message comes from pkg-pr-new's initial /check request, before it
  # packs or publishes anything. Do not retry authorization or publish errors.
  if (( attempt == 3 )) || ! grep -Fq 'Failed to connect to server: TypeError: fetch failed' "$log_file"; then
    exit "$result"
  fi

  delay=$((attempt * 5))
  echo "::warning::Preview service connection failed; retrying in ${delay}s (${attempt}/3)."
  sleep "$delay"
done
