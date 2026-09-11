# Firesquid contributor instructions

Firesquid is a Linux microVM manager built around Firecracker. The project is
being modernized toward a 1.0 architecture, with support for multiple guest
architectures over time; breaking changes to the pre-1.0 CLI, API, state format,
and filesystem layout are allowed.

## Before changing code

- Read `PLAN.md` and the relevant documents in `docs/` before starting work.
- For substantial work, state assumptions, phases, and verification criteria
  before editing. Keep implementation scoped to the requested phase.
- Preserve unrelated user changes in the working tree.
- Do not commit, push, reset, or discard changes. Leave review and commits to the
  user.

## Project architecture

The intended 1.0 boundaries are:

- `cli`: command parsing, RPC calls, and human/machine-readable output.
- `api`: versioned request and response types for CLI-daemon communication.
- `daemon`: systemd entrypoint, request handling, reconciliation, and shutdown.
- `state`: SQLite schema, migrations, and transactional state changes.
- `runtime`: Firecracker process management and its private API socket.
- `network`: TAP devices, namespaces, bridges, routes, nftables, and cleanup.
- `image`: local Docker/OCI import and ext4 rootfs creation.
- `kernel`: local Linux source configuration and compilation.

Keep privileged Linux operations behind narrow, testable interfaces. The daemon
owns Firecracker processes, persistent state, and host networking resources; the
CLI is a client and must not silently start a daemon or download VM artifacts.

The public control surface should be Docker-familiar, but it is not required to
be Docker Engine API compatible. The daemon control socket must be restricted and
must never be world-writable.

## Rust standards

- Use the Rust edition and MSRV declared in `Cargo.toml`.
- Prefer a Cargo workspace with small crates or clearly separated modules when
  introducing the 1.0 boundaries.
- Use `Result` with structured error types and actionable context. Avoid
  `unwrap()` and `expect()` in runtime, daemon, CLI, and request-handling paths;
  use them only for genuine startup invariants that cannot fail meaningfully.
- Do not build shell commands from user input. Use structured process arguments,
  validate paths and identifiers, and make external tools explicit dependencies.
- Do not block the async runtime with synchronous filesystem, database, or build
  work. Use appropriate blocking tasks or dedicated workers.
- Add tests for domain rules, protocol serialization, state transitions, path and
  artifact validation, and cleanup behavior. Privileged KVM/network tests should
  be separated from unprivileged unit tests.
- Write comments only when they explain a non-obvious reason or safety constraint.

## Firecracker and Linux safety

- Assume Linux, KVM, systemd, TAP devices, network namespaces, and nftables for
  the initial supported platform. Keep architecture-specific behavior isolated
  so ARM and other future targets can be added without changing core models or
  the control protocol.
- Treat Firecracker's HTTP API as an internal runtime detail, not the public
  Firesquid API.
- Persist lifecycle transitions before and after resource allocation, and make
  start, stop, kill, remove, and cleanup operations idempotent where possible.
- Reconcile SQLite state with live processes and network resources after daemon
  startup or failure.
- Keep generated runtime data, sockets, logs, databases, and artifacts out of the
  source tree and installed read-only asset directories.
- Never silently download kernels, root filesystems, Linux sources, or packages.
  Local build/import commands must report missing tools, inputs, permissions, or
  architecture mismatches clearly.

## Documentation and planning

- Update the relevant file in `docs/` when an architectural, security, host
  requirement, CLI, networking, kernel, or image-build decision changes.
- Update `PLAN.md` checkboxes only for work actually completed and verified.
- Keep documentation aligned with the implementation; do not promise features
  that are still only design work.

## Verification

For Rust changes, run the narrowest useful checks during iteration and finish the
affected phase with:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run privileged integration tests only on a deliberately configured Linux host.
When changing packaging or CI, validate the relevant Debian/systemd/workflow
files in addition to the Rust checks.

## Generated and sensitive files

Do not commit build output, runtime state, downloaded assets, VM disks, logs,
databases, secrets, credentials, or machine-specific configuration. Keep large
kernel, rootfs, and Firecracker binaries out of source control unless the plan
explicitly establishes a versioned distribution artifact policy.
