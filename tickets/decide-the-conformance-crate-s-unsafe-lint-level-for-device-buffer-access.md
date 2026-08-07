---
id: decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access
title: Decide the conformance crate's unsafe lint level for device buffer access
status: done
priority: p1
dependencies: []
related: [admit-the-conformance-crate-to-the-workspace, conform-the-bf16-vertical-end-to-end]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, unsafe]
---
## The decision

**Only Tom closes this.** `crates/tiler-conformance` inherits the workspace lint table, which sets `unsafe_code = "forbid"`. The device-reaching half of a conformance run cannot be written under it. What lint level should the crate carry, and under what named sites?

## Why it cannot be worked around

**Fact — `MTLBuffer` storage is reachable only through a raw pointer.** Moving operands into a device buffer and results back out goes through what `metal::Buffer::contents` returns, and there is no safe route. **Fact — `forbid` cannot be relaxed by an inner attribute at any scope**, so a `#[allow(unsafe_code)]` at a named site does not compile under it; the level itself has to move. **Fact — the repository already answered this once**: `prototypes/serial-sum-run` sets `deny` and carries two reasoned `#[allow(unsafe_code)]` sites in `src/buffer.rs`, which is the only construction in the tree that has actually crossed the device boundary.

Found by the worker on [`admit-the-conformance-crate-to-the-workspace`](admit-the-conformance-crate-to-the-workspace.md), which inherited the table whole and **deliberately did not pre-authorize a weaker level**, on the ground that admitting a named unsafe site is a decision under [ADR 0079](../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md) and belongs to the ticket that first needs one. The constraint is recorded in a comment above `[lints]` in the crate manifest.

## Why it is Tom's

`AGENTS.md`: unsafe code is admitted only at named sites under ADR 0079 — no safe foreign-API route, a reasoned `#[allow]`, a bounding assertion, and a `SAFETY` explanation — and **"broad unsafe or lint relaxations remain Tom's decision."** Moving a crate from `forbid` to `deny` is a lint relaxation, whatever the sites under it end up looking like.

## What each answer enables and prevents

- **`deny`, with named `#[allow(unsafe_code)]` sites carrying `SAFETY` and a bounding assertion.** The `serial-sum-run` precedent exactly. Enables the device half. Prevents nothing structurally, and each site stays individually reviewable. Counterpoint: it puts unsafe into a crate whose purpose is *evidence*, so a defect in the harness could in principle produce a wrong verdict rather than a crash — although the same is already true of the prototype the evidence is gathered in today.
- **Keep `forbid`, and put the buffer plumbing behind a safe wrapper in another crate.** Enables the device half without relaxing anything here. Counterpoint: no such wrapper exists, and the only crate that could host one is `tiler-metal` or `tiler-runtime` — so this is a real design question about where device-buffer access belongs, not a lint tweak, and it would widen a crate whose surface is otherwise about artifacts.
- **Keep `forbid` and leave the device half in `prototypes/serial-sum-run`.** Costs nothing now and forfeits the reason the conformance crate was admitted, which Tom decided on 2026-08-07 precisely because long-term holding evidence must not live in throwaway code.

**Recommendation: `deny` with named sites**, matching the precedent. The counterpoint is real but the alternative that avoids it — a safe device-buffer wrapper — is a genuine architectural question that should be decided on its own merits rather than forced by a lint level, and the third option reverses a decision made hours earlier.

## Closes when

Tom names a level; if it is `deny`, the sites that carry `#[allow(unsafe_code)]` are named here with what each does, and the crate manifest's comment above `[lints]` is replaced by the decision rather than left describing an open question.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration of the crate admission, from a constraint that ticket's worker found and declined to pre-empt. [`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) depends on it: that ticket's device half cannot be written until this is answered.

## Decided — 2026-08-07

**Tom answered on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator. The answer is narrower than the recommendation asked for, and the narrowing is the point.

### The rule

**`deny`, with named `#[allow(unsafe_code)]` exceptions at individual sites — never at the crate.** A crate-level allow is refused outright and is not a fallback if the sites prove awkward.

**The only justification admitted right now is FFI memory management with Metal**, and only where it is genuinely unavoidable. Concretely, that is the raw pointer `metal::Buffer::contents` returns, because `MTLBuffer` storage is reachable no other way. It is **not** a general licence for the crate: not for convenience, not for performance, not for a shape that would merely be tidier unsafely. A site that is not forced by the foreign API's memory management does not qualify, whatever else recommends it.

**The goal is isolation, and it is a design constraint rather than an aspiration.** Unsafe is to be concentrated into as few sites as possible, so the rest of the crate is written against a safe surface and the reviewable population stays small and enumerable. `prototypes/serial-sum-run` is the shape: two reasoned sites in a single `src/buffer.rs`, with everything else safe.

### What this obliges of the implementing ticket

- A **single narrow module** owning every unsafe site, exposing a safe API to the rest of the crate. Conformance logic — corpus, oracle comparison, evidence attribution — must contain no `unsafe` at all, and must not need to.
- Each site carries what ADR 0079 requires: no safe foreign-API route, a reasoned `#[allow(unsafe_code)]`, a bounding assertion, and a `SAFETY` explanation naming the invariant and why the foreign API forces it.
- The site population is **named and counted** where a reader can find it, so a later addition is visible rather than absorbed. A third site appearing without a stated reason is the failure this rule exists to catch.
- The crate manifest's comment above `[lints]` currently describes this as an open question. It must be replaced by the decision — the level, the single admitted justification, and the isolation requirement — rather than left describing a fork that is closed.

### What was rejected, and why it is not merely deferred

**A crate-level relaxation**, in any spelling. **The safe-wrapper-elsewhere option** was not chosen, but it is not refuted: whether device-buffer access eventually belongs behind a safe surface in `tiler-metal` or `tiler-runtime` remains a real architectural question, and this decision deliberately does not answer it. Isolating the unsafe into one module here is what keeps that option cheap — a single owning module is the thing a future wrapper would replace.

**Leaving the device half in `prototypes/serial-sum-run`** was rejected on the same ground as the crate admission itself: long-term holding evidence must not live in throwaway code.

## Released work

[`conform-the-bf16-vertical-end-to-end`](conform-the-bf16-vertical-end-to-end.md) depends on this node and is now unblocked on this axis. The lint change lands with the first site that needs it, under this rule, rather than as a standalone relaxation — a level moved with no site under it would be a relaxation for nothing.
