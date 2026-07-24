---
id: resolve-ticketsplease-version-authority-drift
title: Resolve the ticketsplease version-authority drift blocking the gate
status: todo
priority: p0
dependencies: []
related: [make-spike-process-group-cleanup-best-effort]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [toolchain, gate-reliability, blocked]
---
`scripts/check_repository.py` fails on this host with
`repository validation failed: ticketsplease must be 0.11.0`.

**Cause: host drift, not a repository defect.** The `ticketsplease` binary
auto-updated to **0.12.0** part-way through a working session — the binary at
`~/.local/bin/ticketsplease` is timestamped the same day — while
`tool-versions.toml` pins `ticketsplease = "0.11.0"` together with a 40-hex
`ticketsplease_rev`. The gate reads that authority and checks the version
**first**, exiting before any other stage, so a failure here says nothing about
the tree under test.

**The tree is healthy.** Verified on `main` at the time of the failure, bypassing
the version check: `scripts/docs.py validate` passed (177 records),
`scripts/check_rust.py` passed in full (fmt, strict Clippy, workspace tests,
optimized numerical tests, doctests, warning-free rustdoc, lock checks), and
`ticketsplease lint` reported no problems under 0.12.0. `ticketsplease doctor`
also passes every check and reports the canonical skill already refreshed to
0.12.0.

**Why this is blocking rather than cosmetic.** Accepted ADR 0075 makes the
coordinator's terminal-merge authority conditional on a green
`check_repository.py`. While the gate cannot run, that precondition is unmet by
construction, so no change may be merged on coordinator authority regardless of
category. The policy is behaving correctly; the gate is simply unavailable.

Two resolutions, both requiring Tom because `AGENTS.md` makes
`tool-versions.toml` the sole ticketsplease version authority and reserves host
toolchain mutation to him:

- **Restore 0.11.0** — reinstall the pinned binary. Keeps the recorded authority
  true. Requires fetching an older release; no 0.11.0 binary remains locally.
- **Adopt 0.12.0** — update `tool-versions.toml` to the new version *and* its
  matching 40-hex revision, which must be looked up rather than guessed, since
  the gate validates the rev's shape. Before adopting, confirm 0.12.0 does not
  change `guard`, `claim`, or `lint` semantics the repository depends on, and run
  `ticketsplease migrate` if the board reports drift.

Whichever is chosen, consider whether the pin should tolerate an unattended
auto-update at all. A tool that can silently change under a pinned repository
will do this again, and the failure mode — a gate that stops running rather than
reporting a real problem — is one that tempts a worker to bypass the check. If
`deps.sh` is the intended installer, it may be the right place to pin rather than
merely verify.
