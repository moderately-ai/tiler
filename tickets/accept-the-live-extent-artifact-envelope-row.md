---
id: accept-the-live-extent-artifact-envelope-row
title: Accept the live-extent artifact envelope row
status: awaiting-decision
priority: p1
dependencies: []
related: [carry-live-extent-operands-through-the-artifact-envelope]
scopes: [contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
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
