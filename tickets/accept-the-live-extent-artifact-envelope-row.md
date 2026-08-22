---
id: accept-the-live-extent-artifact-envelope-row
title: Accept the live-extent artifact envelope row
status: in-progress
priority: p1
dependencies: [associate-live-extent-operands-with-symbolic-semantic-interface-axes]
related: [carry-live-extent-operands-through-the-artifact-envelope, bind-frozen-live-extent-bytes-at-declared-backend-transports]
scopes: [contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
claimed_from: todo
assignee: worker-envelope
lease_expires_at: 1787425502
---
## User-visible outcome

Tom accepts or revises the exact included and excluded public/schema surface of the live input-extent artifact envelope row, so the labelled draft on `DecodedExtentOperand` / `DecodedEntry::extent_operands` / `EntryRef::extent_operands` can stop being a draft.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes every concrete public surface to Tom. [`carry-live-extent-operands-through-the-artifact-envelope`](carry-live-extent-operands-through-the-artifact-envelope.md) added the envelope row that [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md) deliberately excluded. This node is not implementation work. Only Tom closes it.

## The surface, as drafted

**Included — decoded dispatch record.** `tiler_artifact::program::DecodedExtentOperand` with `key()`, `axis()`, and `value_type()`. `DecodedEntry::extent_operands` returns those rows in canonical `(key, axis)` order. Empty for every entry whose kernel declares no live extent. The live *value* is not on the row.

**Included — verified artifact view.** `EntryRef::extent_operands` returns the same `DecodedExtentOperand` rows. Construction derives them from the bound kernel's `InputExtentParameter` list through the stage access that names the program-interface key. Callers do not supply a second list.

**Included — encode / decode / validate.** A nonempty list is appended after the backend entry key under presence tag `0xfe`. Empty writes nothing, so previously encodable artifact bytes do not move and `tiler.artifact-program.v16` does not step. Validate refuses a missing interface key, an axis outside the named input's rank, a non-unsigned type, a non-canonical or duplicated list, a transport count other than `bindings + extents`, and an extent transport that is not `binding_count + ordinal`.

**Excluded, each by a stated reason rather than by omission.** The live extent *value*. A second caller-supplied scalar list. Baking the bound value into artifact, payload, library, or pipeline identity. Consuming reserved schedule tag `0x36`. `N = 14` / `N = 15` payload and pipeline execution. Schedule-verified `LiveContraction` end-to-end.

## The questions that are genuinely Tom's

1. **Accept `DecodedExtentOperand { key, axis, value_type }` as the public envelope row?** The alternative is folding the operand into `DecodedBinding`, which would force a scalar extent through buffer storage, encoding, and range fields that do not apply.
2. **Accept empty-writes-nothing on `tiler.artifact-program.v16`?** The alternative is an unconditional length that steps the domain to move every previously encodable artifact.
3. **Accept the extra payload transports after the tensor table as the backend placement?** That is the Metal `eN` ABI Tom already accepted, packaged so a decoder can bind without reconstructing the kernel.

## Recommendation

Accept all three as drafted. **Strongest counterpoint:** publishing `value_type` on a row that only admits `Unsigned` today freezes a field that a later signed or narrower extent might want to mean something else.

## Options eliminated before ranking

Inventing a second caller-supplied scalar list, baking the live value into artifact identity, or self-accepting this draft, can silently give one `S` two meanings or release dependents against an unaccepted boundary. Those are defects, not candidates.

## Closes when

Tom accepts, accepts with named exclusions, or revises.

## Decision hold — semantic source unresolved 2026-08-13

Do not answer the three questions above yet. Exact-base review found that the draft row is derived while artifact construction still rejects symbolic semantic interfaces, and the passing two-N fixture attaches the row to a fixed `[2,3]` semantic axis before executing extents 14 and 15. `{ key, axis, value_type }` may remain sufficient once the semantic source is carried, or the row/schema may need to name additional source identity; that is not yet established.

This packet now depends on [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md). Its release trigger is that ticket's independent derivation of the minimum complete row, including coverage, identity, schema, and unsupported-population consequences. Re-run the decision-packet readiness gate and replace this hold with the exact reviewed surface before presenting it to Tom.

## Released to ready — 2026-08-22, both trigger conditions fired

Verified by the coordinator at `f61c0786`. This packet's hold named two conditions and **both are now met**:

1. The release trigger was `associate-live-extent-operands-with-symbolic-semantic-interface-axes`'s independent derivation of the minimum complete row. That ticket is `status: done` and its verdict is already transcribed below: `{ key, axis, value_type }` remains minimal once the semantic source is carried.
2. The note below states the drafted surface's population stays empty "until `package-the-admitted-live-schedule-into-a-symbolic-kernel-program` lands the packaged symbolic interface representation", and that this packet "should be presented after or beside it". **That ticket is `status: done`** — merged and gated on 2026-08-22 — so the population exists.

**This gates a p0.** `bind-frozen-live-extent-bytes-at-declared-backend-transports` (p0) depends on exactly two tickets; the other is `done`, so this packet is its only remaining blocker.

**Facts below are stale and must be re-audited before the readiness gate is re-run.** The note records "question 2's premise moved with the domain (`tiler.artifact-program.v20` / manifest 20.0 now)". At this base those are **`tiler.artifact-program.v21` / manifest `(21, 0)`**, stepped by the packaging landing. Re-derive every domain, byte count, and pin at your own base; do not carry a figure from this ticket.

**Also carry forward, verified at source:** the accepted ADR 0108 packet states the schedule bounds-proof tags are `0x01`/`0x02` and assigns a new one `0x03`. All three are false — the real values are `TAG_LINEAR_RANGE = 0x11` / `TAG_REDUCTION_DOMAIN = 0x12`, and `0x03` is `TAG_SCALAR_BROADCAST` in the nibble-partitioned access-map space. If this packet reasons about tag space, use `0x13` and the reserved `0x0C`, and do not restate `0x03`.

**Scheduling.** Declares `contracts/decisions`, held by the live gather-remainder lane. **Release trigger: that lane merges or stops at a gated boundary.**

## Release-trigger input — semantic-source association delivered 2026-08-18

The association ticket's derivation is in its delivery record (section "Envelope-row sufficiency re-audit"), and its verdict is: **`{ key, axis, value_type }` remains the minimum complete row once the semantic source is carried, and no additional public or schema field is required.** The artifact carries exactly one shape-environment subject, and construction now proves a row valid only when the named interface axis is symbolic with its symbol rooted at exactly that `(key, axis)` in that one environment — so the carried environment subject plus the named axis determine the governing symbol, and a symbol-identity field on the row would be a second authority over one fact. The two-N fixture this hold cited no longer exists: a fixed semantic axis with a live operand refuses at construction (`ExtentOperandStaticAxis`) and at decode, so the drafted surface's population is empty until [`package-the-admitted-live-schedule-into-a-symbolic-kernel-program`](package-the-admitted-live-schedule-into-a-symbolic-kernel-program.md) lands the packaged symbolic interface representation. Before re-running the readiness gate for Tom, note that question 2's premise moved with the domain (`tiler.artifact-program.v20` / manifest 20.0 now; empty-writes-nothing still holds and no step landed with the association) and that the packaged-interface representation — the one place a genuinely new public/schema surface can arise for this family — is the packaging packet's own question, so this packet should be presented after or beside it.
