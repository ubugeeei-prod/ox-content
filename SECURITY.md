# Security Policy

## Supported Versions

Ox Content is developed as a fast-moving project. Security fixes are provided for
the latest released version of each published crate, npm package, editor
extension, and documentation site deployment. Older versions are not routinely
patched unless a fix can be safely backported with low risk.

Please upgrade to the latest available release before reporting an issue that may
already be fixed.

## Reporting a Vulnerability

Please report suspected security vulnerabilities privately through
[GitHub Security Advisories](https://github.com/ubugeeei/ox-content/security/advisories/new).
Do not open a public issue, discussion, or pull request for a vulnerability until
it has been coordinated.

Include as much of the following as you can:

- A clear description of the vulnerability and expected impact.
- Affected package, crate, extension, command, or docs site feature.
- A minimal reproduction, proof of concept, or triggering input.
- Affected versions, commit hashes, operating system, runtime, and relevant
  dependency versions.
- Whether the issue is already public or known to be exploited.
- Any suggested mitigation or patch, if available.

## Response Cadence

This is a personal open source project, but security reports are prioritized.
You can generally expect:

- Initial acknowledgement within 7 days.
- Triage and scope confirmation within 14 days when enough detail is provided.
- Status updates at least every 14 days until the report is resolved or closed.

If a report needs more information, coordination may pause until the requested
details are available.

## Scope

Security reports are in scope when they affect this repository or official Ox
Content release artifacts, including:

- Rust crates and native bindings.
- npm packages, including Node.js, Vite, framework, CLI, and WebAssembly
  packages.
- Editor tooling for VS Code, Zed, Neovim, and language server integrations.
- The documentation site, playground, generated documentation pipeline, and
  examples maintained in this repository.
- Build, release, and dependency configuration that could affect distributed
  artifacts.

Out of scope reports include unsupported forks, third-party integrations not
maintained here, social engineering, denial-of-service against public hosting
providers, and findings that require access to a reporter-controlled environment
without affecting other users.

## Responsible Disclosure

Please give maintainers a reasonable opportunity to investigate and release a
fix before public disclosure. Avoid accessing, modifying, or deleting data that
does not belong to you, and avoid testing techniques that could disrupt services
or other users.

After a fix is available, public disclosure may happen through a GitHub Security
Advisory, release notes, or another coordinated channel appropriate to the issue.
