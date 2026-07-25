---
id: record-the-case-by-case-unsafe-boundary
title: Record the case-by-case unsafe boundary as an accepted decision
status: done
priority: p1
dependencies: []
related: [prototype-metal-runtime-execution]
scopes: [contracts/decisions, implementation/workspace]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [documentation, decisions, rust-api]
claimed_from: todo
assignee: agent-decisions
lease_expires_at: 1784996298
---
`AGENTS.md` states that unsafe code "remains forbidden unless an accepted decision changes that boundary". Tom changed it on 2026-07-25 and no accepted record says so, which is exactly the drift that sentence exists to prevent.

**Fact — what was decided.** Unsafe is permitted **case by case**, isolated to specific functions or modules, explicitly **not** whole crates and **not** a workspace relaxation.

**Fact — how it is realized at `a56bff8`.** `prototypes/serial-sum-run` does not inherit `[workspace.lints]` and declares `unsafe_code = "deny"` instead of the workspace's `forbid`, because `forbid` cannot be relaxed by an inner attribute at any scope. Two functions in `buffer.rs` carry `#[allow(unsafe_code, reason = ...)]`; they are the complete extent of unsafe code in the workspace. The necessity is structural rather than convenient: `MTLBuffer` storage is reachable only through the raw pointer `Buffer::contents` returns, and no Metal binding exposes it safely.

**Fact — the exception is pinned, not merely permitted.** `scripts/check_workspace.py` carries `UNINHERITED_LINT_MEMBERS`, naming the one member allowed to diverge and its exact table. A second crate dropping inheritance fails the gate, and so does widening that member's `deny` to `allow`.

## What this ticket produces

An accepted ADR recording the decision, its scope, and the mechanism, plus the `AGENTS.md` amendment its own sentence requires. The record should state the three properties that make the exception safe — narrowest-scope opt-in, `deny` rather than `allow` at crate level, and a pinned gate check — and the rule for future sites: an unsafe block is admitted only where a foreign API makes it structurally unavoidable, carries a `reason`, is preceded by an assertion bounding what it touches, and has a `SAFETY` comment naming the invariant it relies on.

**Do not let this become a general licence.** The decision is case-by-case, and the record must make a future reader ask again rather than cite this as precedent for a third site.

## Outcome

[ADR 0079](../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md) is accepted and `AGENTS.md` no longer states a rule the tree has departed from. One gap the ticket did not name is stated in the ADR and split into a follow-up rather than left implicit.

**Fact — every premise this ticket asserted was re-verified by reading, and all held.** The root `Cargo.toml` sets `[workspace.lints.rust] unsafe_code = "forbid"`; `prototypes/serial-sum-run/Cargo.toml` omits `[lints] workspace = true` and declares `unsafe_code = "deny"` with an otherwise byte-identical table, so the divergence is exactly one lint level; `scripts/check_workspace.py`'s `UNINHERITED_LINT_MEMBERS` is a single-entry map consulted twice, once through `expected_member_manifest`'s full-manifest comparison and once as an explicit lint-table comparison. The exact completeness check is `grep -rn 'unsafe' --include='*.rs' crates prototypes` at `43f685f`: twelve lines, of which two are `unsafe` blocks (`prototypes/serial-sum-run/src/buffer.rs` lines 52 and 85), two are the lint name inside the admitting attributes, and eight are the word in prose.

**Fact — the necessity is exhaustively established, not asserted.** `metal-0.33.0/src/buffer.rs` declares ten public buffer methods — `length`, `contents`, `did_modify_range`, `new_texture_with_descriptor`, `remote_storage_buffer`, `new_remote_buffer_view_for_device`, `add_debug_marker`, `remove_all_debug_markers`, `gpu_address`, `gpu_resource_id`. Exactly one reaches storage and it returns `*mut std::ffi::c_void`. None returns a slice or borrowed view. That is a finite named universe checked completely, so "no Metal binding exposes it safely" is an exhaustive-finite claim about this binding at this version rather than a general one about Metal bindings.

**Retracted — one stated reason in `prototype-metal-runtime-execution`, not its conclusion.** That ticket attributes the necessity partly to `Device::new_buffer_with_data` "taking a `*const c_void`". Read at `metal-0.33.0/src/device.rs:1956`, that function is a safe `fn`; calling it requires no `unsafe`, because the binding encapsulates its own `msg_send!`. The runner does not call it at all — `prototypes/serial-sum-run/src/main.rs:227` and `:231` allocate through the safe `Device::new_buffer`. The accurate reason is the one above: the binding exposes storage only as a raw pointer and dereferencing it is Tiler's own `unsafe`. The conclusion the ticket drew is unchanged; ADR 0079 records the narrowing under "Correction to the record that stated the blocker" rather than quietly restating it.

**Fact — Tom chose a third option, narrower than either form put to him.** The blocker ticket offered a narrow form (`unsafe_code = "allow"` on the one crate) and a broad form (relax the workspace default). What `a56bff8` landed is `deny` plus per-function opt-in, which is strictly narrower than the narrow form: under `allow`, a later unsafe block in that crate compiles with no attribute and no reason. ADR 0079 records this in Alternatives rather than presenting the landed shape as the only one considered.

**Decision — `decision_status: accepted`, not `proposed`.** ADR 0077 is `proposed` because the workspace ran ahead of a decision Tom had not made. Here Tom made the decision and authored `a56bff8` himself; the only thing missing was the record. Marking it `proposed` would tell a reader the divergence in the tree is unauthorized, which is false.

**Fact — the gap this ticket did not name, now stated and owned.** Nothing counts, locates, or constrains `#[allow(unsafe_code)]` attributes inside the crate permitted to have them. A third site added to `prototypes/serial-sum-run` compiles and passes the complete gate unchanged. The ticket's "three properties that make the exception safe" holds as written only because its third property is a claim about which *crates* may diverge; it is not a claim about sites, and the per-site half of the decision is enforced by review alone. ADR 0079 says so in its Consequences and in its Implementation boundary rather than letting "a pinned gate check" imply more than it covers. [`pin-the-admitted-unsafe-sites-in-the-workspace-gate`](pin-the-admitted-unsafe-sites-in-the-workspace-gate.md) owns closing it, carrying the three candidate predicates and their distinct failure modes; it is `implementation/workspace` engineering rather than decision custody, and this ticket held no evidence for choosing among them.

**Fact — the `AGENTS.md` edit is two bullets where there was one, because two claims were falsified.** "Unsafe code remains forbidden unless an accepted decision changes that boundary" is the one the ticket named. Its neighbour, "Keep workspace Rust and Clippy lints inherited by every crate", was falsified by the same commit and is corrected in the same edit: it now names `UNINHERITED_LINT_MEMBERS` as the pin and states that a second member dropping inheritance fails the gate. The unsafe bullet states ADR 0079's four conditions inline so a reviewer can apply them without opening the ADR, and states that citing ADR 0079 is not sufficient to admit a new site. The touched bullets are written as single lines per `docs/document-metadata.md`'s prose source form; their hard-wrapped neighbours are left alone, which that contract names as the accepted transitional state.

**Scope.** `implementation/workspace` was added exclusively for `AGENTS.md` and `contracts/navigation` as a shared scope for the regenerated `docs/decisions/README.md` catalog blocks. Both were uncontended: no `in-progress` ticket held either at claim time. No file outside `docs/decisions/`, `AGENTS.md`, and `tickets/` was touched.

**Measurement.** `uv run --locked python scripts/docs.py render` reported "documentation render passed (182 records)". `uv run --locked python scripts/check_repository.py` exited 0 with "complete repository validation passed". Host macOS arm64, toolchain `nightly-2026-07-19` as pinned.
