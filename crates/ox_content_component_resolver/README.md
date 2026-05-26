# ox_content_component_resolver

Resolves Vue / React component prop metadata for Ox Content
Markdown/MDC documents via [typescript-go](https://github.com/microsoft/typescript-go)
through the [`corsa_client`](https://crates.io/crates/corsa_client) crate.

## Status

This crate ships in stages. The first PR establishes the public API
surface and proves the dependency wires through the workspace. The
follow-up PRs land:

1. The real resolver implementation against `tsgo`.
2. The LSP integration (component-aware attribute completion, hover,
   go-to-definition, type-check diagnostics for `<MyComponent prop={…}>`).
3. The `ox-content-component-check` CLI for CI.

Today the resolver returns `Error::NotImplemented` for every
`resolve_component_props` call. The types and the spawn/path-validation
contract are stable so editor integrations can develop against them
before the implementation lands.

## Why a Rust-side resolver

The LSP already runs in Rust. Pushing through a Node sidecar to reach
`tsgo` would (a) double the process count per workspace and (b) add an
extra IPC hop on every prop hover. `corsa_client` talks to `tsgo`
directly over stdio, so the LSP can share a single long-lived worker
per workspace.

## Opt-in

The resolver is **disabled by default**. Users opt in by setting one
of:

- `oxContent.components.tsgoPath` (VS Code, future PR)
- `componentsTsgoPath` LSP initialization option (future PR)
- `OX_CONTENT_TSGO_PATH` environment variable

When `tsgo` is not configured, the LSP and CLI surface the rest of the
feature set without props completion — no errors, no panics, no
diagnostic spam.

## Public surface (today)

```rust
use ox_content_component_resolver::{Resolver, ResolverConfig};

let resolver = Resolver::spawn(
    ResolverConfig::with_tsgo_path("/usr/local/bin/tsgo")
        .with_tsconfig("/repo/tsconfig.json"),
).await?;

let component = resolver.resolve_component_props(
    std::path::Path::new("/repo/src/Alert.tsx"),
).await?;

for prop in &component.props {
    println!("{}: {} ({} required)",
        prop.name,
        prop.type_text,
        if prop.optional { "not" } else { "is" },
    );
}
```

## Out of scope

- TSX / Vue SFC handling specifics. The first real implementation
  targets plain `.tsx` / `.jsx` components; Vue SFC support is a
  tracked follow-up.
- HTTP type resolution (e.g. types fetched from a registry). Local
  file system only.
