---
id: prototype-runtime-artifact-validation
title: Implement runtime artifact validation
status: blocked
priority: p0
dependencies: [prototype-neutral-artifact-codec, admit-the-device-free-runtime-validation-crate, carry-reconstructable-kernel-programs-in-the-neutral-envelope]
related: []
scopes: [implementation/runtime, implementation/artifact, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, runtime, artifact]
claimed_from: todo
assignee: agent-runtime
lease_expires_at: 1784997351
---
Implement runtime-owned device-free decoding, integrity/program/ABI validation, checked expression evaluation, and typed compatibility classification. The runtime path must not import semantic IR, optimizer state, backend internals, or proof-sidecar semantics.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

## Blocked 2026-07-25 — two independent blockers, both established by reading

Attempted from `implementation/runtime`, `implementation/artifact`, `implementation/workspace`, with `project/tickets` and `implementation/cargo-lock` shared. One in-scope defect was found and fixed on the way; the ticket's own deliverable is unreachable from these scopes.

### Blocker 1 — the crate admission is not a workspace edit

Split into [`admit-the-device-free-runtime-validation-crate`](admit-the-device-free-runtime-validation-crate.md), which carries the full evidence. In short: `docs/architecture.md:352-354`'s accepted packaging profile "deliberately omits … reusable Metal-runtime crates until the proof reaches those boundaries", `docs/decisions/0077-…:80` says "A reader must not cite this admission as precedent for admitting one", and no doc, ADR, or ticket names a `tiler-runtime` crate — the string exists only in `ticketsplease.toml`'s scope glob. Admitting the crate needs `contracts/foundation` and `contracts/decisions` alongside `implementation/workspace`; landing only the workspace half would leave `scripts/check_workspace.py` disagreeing with the accepted architecture text, which is the exact state ADR 0077 exists to end.

Whether the *device-free* crate this ticket describes falls under the withheld clause at all is a genuine question — by ADR 0077's own test ("never touches a live device, an `MTLDevice`, or a pipeline state") it does not — and that is why it was split rather than decided here.

### Blocker 2 — "checked expression evaluation" is not reachable from decoded bytes

**Fact.** `DecodedArtifact` in `crates/tiler-artifact/src/program/codec/view.rs` exposes exactly `identity()`, `features()`, `routing()`, `payloads()`, `sections()`, `variant_count()`, and `re_encode()`. There is no accessor for a variant, an entry, a binding, an ABI expression, or a launch contract.

**Fact.** The envelope holds them. `codec/model.rs:341-360`: `EntryRow` carries `stage`, `resources`, `numerical`, `bindings`, `launch`, `payload`, `entry_key`, and `VariantRow` carries `guard`, `profile`, `feasibility_rules`, `deferred`, and `entries`. All are `pub(crate)`; nothing projects them.

**Inference.** The gap is a promotion boundary, not an encoding gap — a decode validates every one of those rows and `re_encode()` writes them all back — but it means no out-of-crate consumer can evaluate a guard, a binding's accessible byte range, or a launch formula from bytes. Every expression accessor that exists (`VariantRef::applicability_guard`, `EntryRef::bindings`, `BindingRef::accessible_bytes`, `EntryRef::launch_threads`, `AbiExprRef::evaluate`) hangs off `VerifiedArtifactProgram`, which no decode produces and which — per `carry-reconstructable-kernel-programs-in-the-neutral-envelope` — cannot be reconstructed.

**Consequence for this ticket.** Three of the four stated deliverables *are* reachable today: device-free decoding, integrity validation, and typed compatibility classification all fall out of `decode_artifact` plus `identity()`/`features()`/`payloads()`. The fourth, checked expression evaluation, needs either a promoted dispatch-record projection on `DecodedArtifact` or the binding-by-identity design in which the runtime holds the `VerifiedArtifactProgram` it compiled and uses the decoded identity to prove the loaded bytes name it. Choosing between those is `carry-reconstructable-kernel-programs-in-the-neutral-envelope`'s decision, recorded there; this ticket must not pre-empt it by promoting a surface.

### Landed under this ticket

`crates/tiler-artifact/src/program/codec/view.rs`'s module documentation claimed a reader gets "everything the envelope actually holds: … and each variant's entries with their ABI and launch expressions". That is false about the read surface and was the reason this ticket read as implementable. It now states what `DecodedArtifact` exposes, that the entry/binding/launch rows are held but unreachable, that this is a promotion boundary rather than an encoding gap, and what that implies for how a runtime is written. No public item, dependency, or contract changed.
