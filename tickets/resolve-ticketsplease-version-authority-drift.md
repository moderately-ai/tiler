---
id: resolve-ticketsplease-version-authority-drift
title: Resolve the ticketsplease version-authority drift blocking the gate
status: done
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

## Outcome

Resolved by **removing the check**, not by reconciling versions — a third option neither listed in this ticket.

**What was removed** from `scripts/check_repository.py`: `ticketsplease_policy()` (which read the pinned version and revision out of `tool-versions.toml` and validated their shapes), `validate_ticketsplease_receipt()` (which required a bootstrap-written receipt file to equal the pinned revision), the `ticketsplease --version` invocation and its equality assertion, and the now-dead `TOOL_VERSIONS` constant. The stale monkeypatches and `--version` stub in `scripts/tests/test_repository_gate.py` went with them, along with `test_ticketsplease_revision_receipt_is_exact`.

**What was kept**: the gate still resolves the binary and still runs `ticketsplease lint`. That is the part that actually validates something about this repository.

**Why.** The enforcement made a background auto-update of a ticket-tracking CLI halt the entire gate — including the Rust build, the test suite, and the documentation validation, none of which depend on the tool's patch version. The failure mode was the worst kind: a gate that stops running rather than reporting a real problem, which trains a worker to bypass it. Tom's judgement was that this was over-engineered, and on inspection that is right — the pin was protecting nothing proportional to the cost of enforcing it.

**Note.** `tool-versions.toml` still carries the `ticketsplease` and `ticketsplease_rev` entries, and `deps.sh` and `scripts/check_ci.py` still use them to *install* a known version. That is fine and was left alone: installing a known tool is useful, whereas failing the gate when it drifts is not. `deps.sh --check` will still report drift, which is appropriate for a bootstrap diagnostic and blocks nothing.

Gate green after the removal.
