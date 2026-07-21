---
id: enforce-repository-validation-gate-integrity
title: Enforce repository validation gate integrity
status: todo
priority: p0
dependencies: [repair-macro-and-embedding-harness-integrity, repair-numerical-witness-integrity]
related: []
scopes: [implementation/workspace, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [tooling, correctness, developer-experience]
---

Make the repository gates prove the contributor contract instead of trusting
self-disableable Cargo, Python, shell, environment, and CI configuration. The
fixed-point audit at `ad6e9f463de6eabad44af47eaddad9317e0935fd` demonstrated
green gates after concrete disabling mutations.

## Required outcome

- Validate the complete workspace package and dependency boundary, including
  forbidden external dependencies, dependency kind/optionality/target/rename
  semantics, resolver 3, required library versus proof-binary target roles,
  expected test targets, and package/target `test`, `doctest`, and `doc`
  enablement.
- Enforce root workspace lint policy and member inheritance, including the
  no-unsafe and public-documentation contract. Govern `rustfmt.toml` so
  disabling or ignoring formatting cannot pass.
- Select and verify the exact dated Rust compiler for every Cargo command;
  eliminate duplicated drifting pins. Reject or sanitize ambient
  `RUSTUP_TOOLCHAIN`, cap-lints, runner, rustfmt/Clippy/rustdoc executable
  overrides, Cargo config, and related environment that can turn checks into
  successful no-ops.
- Use locked Cargo operation and prove `Cargo.lock` remains unchanged. Run the
  strict Rust gate on every supported target-independent host profile, not only
  macOS, and define the supported CPU/atomic-width/endian boundary explicitly.
- Exercise exact numerical behavior under optimized release code generation as
  well as the development profile; a debug-only exactness check is not a
  compiler conformance result.
- Govern pytest and Ruff discovery/configuration and sanitize
  `PYTEST_ADDOPTS`. Verify that `uv run --locked` resolves this project, this
  lock, the synchronized environment, and the pinned tool versions despite
  ambient `UV_*` redirection controls.
- Govern ShellCheck severity/configuration and cover every supported shell
  entrypoint while handling the intentional zsh-only Apple probe explicitly.
- Extend documentation validation to reference-style links, local images,
  malformed question headings, lexical canonicality of every present metadata
  field, exact `YYYY-MM-DD`, and executable mode for directly invoked
  entrypoints—including `deps.sh`.
- Make `deps.sh` repair or correctly reject old standalone `uv`, non-exported
  managed `tkt`, stale managed aliases, and genuine user-owned collisions.
- Add `tkt lint` to CI and keep the supported Debian bootstrap prerequisites
  complete, including measurement tools used by documented Linux procedures.

## Acceptance

Create mutation tests for every bypass above. Each mutation must fail the gate
for the intended typed reason; the unmodified repository must pass on macOS
and the supported Debian-family CI profile. Do not rely on arbitrary ambient
user configuration as part of the proof.
