---
id: preserve-present-subgroup-requirements-in-the-artifact-resource-record
title: Preserve present subgroup requirements in the artifact resource record
status: done
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

## Exact-base Fact re-audit — 2026-08-16 at `7ad48e73c13f3953e67d1c3b95de252bce401498`

This re-audit supersedes the 2026-08-13 classifications for implementation at
the current base. The ticket's purpose is unchanged.

1. **Verified — the resource carrier still has the optional field and supported
   schedule derivation still produces absence.** `ResourceRequirements` declares
   `pub subgroup: Option<SubgroupRealizationSubject>`, while
   `derive_requirements` constructs `subgroup: None`. The old anchor remains in
   `schedule/model.rs`, but its claim that the public surface is still a labelled
   draft is stale after Tom accepted the exact surface recorded by
   [`accept-the-atomic-subgroup-realization-surface`](accept-the-atomic-subgroup-realization-surface.md).
2. **Imprecise — the kernel encoding mechanism remains true, but its domain
   version drifted.** `push_subgroup_requirement` still writes nothing for
   `None` and appends `0x01` plus `SubgroupRealizationSubject::encode` for
   `Some`. `KERNEL_DOMAIN` is now `tiler.kernel.v8`, not the old audit's `v7`,
   after unrelated accepted work.
3. **Verified — artifact resource encoding still drops the field.**
   `push_resources` still destructures `subgroup: _` and emits no subgroup
   bytes.
4. **Verified — artifact decode still invents absence.** The entry decoder still
   constructs `ResourceRequirements { subgroup: None, .. }` unconditionally.
5. **Verified for the supported production path — the loss remains unreachable,
   not safe.** `derive_requirements` still produces `None`, and kernel
   verification compares the complete declared and derived
   `ResourceRequirements` values for equality before an artifact can be built.
6. **Verified as a correction — canonical kernel identity is still insufficient
   by itself, and the KIR ticket now records that correction.**
   [`admit-subgroup-coordinates-and-xor-transfer-into-kernel-ir`](admit-subgroup-coordinates-and-xor-transfer-into-kernel-ir.md)
   now explicitly names this carrier and the dependency ordering; the accepted
   KIR operation boundary remains unchanged.
7. **Verified — no governed transfer decoder is live.** `SubgroupTransfer::tag`
   remains private and the type has no inverse; `ArithmeticType` already owns
   its public `tag` / `from_tag` pair. The accepted subgroup surface explicitly
   excludes a public raw tag and decoder, so this ticket must either derive a
   private exhaustive inverse from the accepted encoder at the real artifact
   consumer or stop rather than widen that surface.

## Required work

- Re-read the full artifact resource model, encoder, decoder, version/domain rules, limits, kernel/resource equality checks, build producer, and runtime consumers at the landing base. Repair any Fact that drifted before editing.
- Decide the minimum injective conditional encoding for `Option<SubgroupRealizationSubject>`. Preserve byte identity for every existing `None` record; a present row must encode width, arithmetic, and transfer under one framed/versioned grammar and must not be confused with trailing resource fields or EOF.
- Delegate forward subject bytes to the accepted public
  `SubgroupRealizationSubject::encode` authority, and keep the minimum transfer
  inverse private to the artifact decoder that consumes it. Couple that inverse
  to the public encoder with a `variant_count`-sized vocabulary and an exhaustive
  all-byte test so a widened transfer vocabulary fails loudly without adding the
  explicitly excluded public raw tag or decoder. Unknown transfer, arithmetic,
  truncated, duplicate, and malformed presence bytes must be typed artifact
  diagnostics; no default, guessed transfer, or `None` fallback.
- Decode the exact subject and preserve the producer's exact resource projection
  through publication and routing. `project_entries` derives the row directly
  from `stage.kernel().requirements()`, after kernel verification has proved the
  complete declared and schedule-derived records equal. The decoder then
  re-derives the whole artifact identity, and runtime compares that identity to
  the caller's independently recorded expected identity before routing. A
  resource mutation that retains the original declaration must refuse as
  `ArtifactIdentityMismatch`; a self-consistently re-stamped mutation is a
  different artifact and must refuse as `ProgramMismatch` against the original
  expected identity.
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

## Requirement repair — 2026-08-16 at `7ad48e73c13f3953e67d1c3b95de252bce401498`

The earlier requirement that bare decode prove equality directly against the
verified kernel requirement was **false under the accepted dispatch-record
boundary**. `StageSubject` and the kernel-program section carry opaque canonical
identity bytes; the envelope intentionally does not reconstruct a
`VerifiedKernelProgram`, so it has no local kernel resource value to compare.
Reading a possible seven-byte subgroup suffix is not a substitute: an absent
kernel's variable tail can coincidentally end in that pattern. A sound local
check would require a complete canonical-kernel decoder or a changed kernel
subject, both consequential changes excluded from this carrier ticket.

The governing guarantee is identity-bound instead. Construction derives the
entry row from the same verified kernel, decode preserves the row and re-derives
the artifact identity, and routing compares that identity to the caller's
recorded expectation before any commitment. Bare decode therefore does not
claim to re-prove opaque kernel semantics; it proves that the received row is
the row belonging to the artifact identity it derived. This repair preserves
the ticket's purpose: always restoring `None` is still eliminated because it
erases the resource projection itself and changes that artifact on the way
through the codec.

## Implementation evidence — 2026-08-16

The artifact resource encoder now writes the subgroup block last, immediately
before the bounded numerical-profile text length. `None` appends no byte;
`Some(width=32, F32, InRangeXorShuffle)` appends exactly
`01 00 00 00 20 03 01`. The legacy default resource row remains exactly
`00 00 00 02 00 00 00 01 00 00 00 00 00 00 00 00 01 01 00 01 01 01 01 01 01 01 01`,
and the standard Metal artifact/cache/fixed-content pin test remains green.
The first present carrier fixture's governed identity digest is pinned at
`00072a197382895dd4f044ed30c7df7d35ed6e557b12e8a1355461abcda94ac7`.

The decoder peeks the following zero without consuming it for `None`, consumes
the seven-byte block for `Some`, and constructs through `SubgroupWidth::new`
and `SubgroupRealizationSubject::new`. The public decoded entry exposes the
exact `Some` record and re-encodes byte-identically for all four
`ArithmeticType` inhabitants. The private transfer inverse is sized from the
one-member `SubgroupTransfer` type, derives its claimed byte through the public
subject encoder, and refuses the other 255 raw transfer bytes. The presence
grammar admits two states and refuses all 254 other raw leading bytes.

Re-sealed full-envelope perturbations reach the following exact internal
diagnostics before identity comparison:

- width zero: `InvalidSubgroupRealization { cause: ZeroWidth }`;
- width three: `InvalidSubgroupRealization { cause: UnsupportedWidth }`;
- arithmetic `0x7e`: `UnknownTag { subject: SubgroupArithmetic, tag: 126 }`;
- transfer `0x7d`: `UnknownTag { subject: SubgroupTransfer, tag: 125 }`;
- presence `0x7f`: `UnknownTag { subject: SubgroupPresence, tag: 127 }`;
- an incomplete present block: `Truncated { needed: 4, available: 0 }`;
- a second block: `DuplicateSubgroupRequirement`;
- a trailing nonzero neighbour `0x7c`:
  `UnknownTag { subject: SubgroupPresence, tag: 124 }`.

An old reader interprets the present marker as text length
`72057594574995712` and refuses
`Limit { resource: TextBytes, actual: 72057594574995712, limit: 4096 }` before
allocation. Erasing `Some` while retaining its declaration refuses exactly as
`ArtifactIdentityMismatch`; re-stamping the erasure produces a different
artifact identity. The existing runtime identity-join test then proves the
generic next boundary: every valid self-consistent artifact differing from the
caller's recorded identity refuses as `ProgramMismatch` before adapter work.
A subgroup-specific runtime fixture remains intentionally unconstructible until
the schedule/KIR producer ticket makes `Some` public-production reachable; no
runtime or public-construction widening was added merely to specialize that
generic identity join.

Perturbing the public IR transfer encoder from `0x01` to `0xff` makes the
artifact coupling test fail with
`artifact subgroup transfer tag 0xff does not round-trip`, showing the private
inverse cannot silently survive an upstream vocabulary move. The restored
source passes the IR and artifact population of 1,459 tests, the standard build
pin, and the runtime identity join. The ABI ledger remains
`tiler.artifact-program.v18` / manifest 18.0: all previously publishable rows
were `None` and retain exact bytes, while an old reader fails closed on the new
`Some` block.
