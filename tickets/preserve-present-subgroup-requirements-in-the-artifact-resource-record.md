---
id: preserve-present-subgroup-requirements-in-the-artifact-resource-record
title: Preserve present subgroup requirements in the artifact resource record
status: todo
priority: p1
dependencies: [accept-the-atomic-subgroup-realization-surface]
related: [admit-subgroup-bindings-into-the-schedule-vocabulary, admit-subgroup-coordinates-and-xor-transfer-into-kernel-ir]
scopes: [implementation/ir, implementation/artifact, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [subgroup, artifact, schema, identity, public-boundary, correctness]
---
## User-visible outcome

An artifact carrying a real subgroup requirement preserves that complete subject through encode, decode, re-encode, feasibility, and routing instead of silently restoring `None`.

## Exact-base Fact audit — 2026-08-13 at `4fb0427319b1504e1549e03ba023ac486343a743`

1. **Verified — the schedule/resource carrier already has the optional field.** `ResourceRequirements.subgroup` is `Option<SubgroupRealizationSubject>`; `None` is currently derived for every schedule. Anchor: `This ticket does not derive a Some from any admitted topology` in `schedule/model.rs`.
2. **Verified — kernel identity already distinguishes a present subject.** `push_subgroup_requirement` appends `0x01` plus `subject.encode(bytes)` for `Some` and writes nothing for `None`. `KERNEL_DOMAIN` stays `tiler.kernel.v7` because no previously constructible kernel carried `Some`.
3. **Verified — artifact resource encoding drops the field.** `push_resources` exhaustively destructures `ResourceRequirements { ..., subgroup: _, ... }` and writes no subgroup bytes.
4. **Verified — artifact decode invents absence.** Resource decode constructs `subgroup: None` unconditionally. An encoded present requirement would therefore return as absence even though the kernel identity still names it.
5. **Verified — the loss is currently unreachable, not safe.** `derive_requirements` always returns `subgroup: None`, and kernel construction checks exact derived requirements, so no supported program can yet publish `Some`. [`admit-subgroup-bindings-into-the-schedule-vocabulary`](admit-subgroup-bindings-into-the-schedule-vocabulary.md) is the first path that will derive one and now depends on this ticket.
6. **False — the KIR ticket's statement that no artifact schema changes because artifacts carry canonical kernel identity is complete.** Artifact resources are a separate decoded authority used after identity verification. A correct identity cannot make a resource record that erased a requirement truthful. This correction changes dependency ordering, not the accepted KIR operation boundary.
7. **Verified — no governed subgroup decoder is live today.** `SubgroupTransfer::from_tag` has no production consumer. The atomic-surface repair removes it; this ticket must derive the minimum decode authority with a real artifact consumer rather than inheriting the speculative helper by accident.

## Required work

- Re-read the full artifact resource model, encoder, decoder, version/domain rules, limits, kernel/resource equality checks, build producer, and runtime consumers at the landing base. Repair any Fact that drifted before editing.
- Decide the minimum injective conditional encoding for `Option<SubgroupRealizationSubject>`. Preserve byte identity for every existing `None` record; a present row must encode width, arithmetic, and transfer under one framed/versioned grammar and must not be confused with trailing resource fields or EOF.
- Put the tag inverse beside the defining vocabulary in `tiler-ir` only when this decoder consumes it. Unknown transfer, arithmetic, truncated, duplicate, and malformed presence bytes must be typed artifact diagnostics; no default, guessed transfer, or `None` fallback.
- Decode the exact subject and prove resource equality against the verified kernel requirement before publication/routing. A kernel saying `Some` while resources say `None`, or the reverse, must refuse rather than trusting identity alone.
- Preserve format/cardinality bounds and advance the owning artifact codec/schema identity only if the existing conditional grammar cannot add the row injectively. Recompute all nested pins from the exact tree; do not infer that silence-as-absence automatically avoids every version move.
- Add encode/decode/re-encode tests for `None` byte stability and `Some` preservation. Independently perturb width, arithmetic, transfer tag, presence, truncation, and late erasure. Show the failure text for each subject perturbation.
- Keep schedule derivation and KIR emission out of scope. This ticket proves the carrier with a constructed resource record so later work cannot make `Some` reachable first.

## Option gate

- **Status quo until `Some` is reachable:** fail-closed only accidentally; once the dependency lands it silently erases a correctness requirement. Not a terminal option.
- **Conditional append-only resource row before the producer:** preferred if the codec's enclosing framing makes it injective. Preserves old bytes and makes the future producer safe.
- **Unconditional option tag / schema step:** survives only if reading proves conditional absence is ambiguous or violates the codec grammar. It moves all existing resource bytes and must rederive every pin.
- **Rely on kernel identity and keep decoded resources at `None`:** eliminated. It leaves two representations of one kernel disagreeing after successful decode.
- **Land producer and carrier atomically:** correct in principle but worse scheduling and review isolation; the carrier is independently constructible and can be proved first.

## Closure

Close when `Some` round-trips without authority loss, every malformed/unknown neighbour refuses by name, existing `None` bytes and pins move only where the exact grammar requires, and the schedule ticket can derive a present requirement without crossing an unproved artifact boundary.
