---
id: widen-numerical-vocabulary-and-complete-identity
title: Widen the numerical vocabulary and complete its identity encoding
status: in-progress
priority: p1
dependencies: [accept-adr-0076-numerical-realizations]
related: [draft-target-honourable-numerical-contract-adr, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [implementation/ir, contracts/foundation, implementation/metal, implementation/artifact, implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity]
claimed_from: todo
assignee: agent-widen-numerical-vocabulary-and-complete-identity
lease_expires_at: 1784929884
---
ADR 0076 items 1 and 6. This is the first of four ordered tickets and the only one that carries a latent correctness defect, so the two halves must land in one change.

## Vocabulary

`tiler_ir::schedule::SubnormalMode` has exactly one variant, `Preserve`, and `NumericalPermission` has exactly one, `Forbidden`. Two `NumericalRealization` values therefore cannot differ on any subnormal or permission dimension. Widen both to the behaviours accepted ADR 0019 and ADR 0011 already name — preservation and explicit flush-to-zero on each subnormal dimension, and a permitted form alongside `Forbidden`.

Keep `NumericalRealization`'s `input_subnormals` and `result_subnormals` independent. Coupling them is forbidden even when a target couples them; that is ADR 0019's operative sentence. Do not add a coupled "fast" spelling and do not express any dimension as a single boolean.

**A flush behaviour must state which zero it produces.** The measured Apple flush is sign-preserving: `0x80400000 * 2.0f` returns `0x80000000`, not `0x00000000`. `docs/numerical-semantics.md` already requires that "the zero sign follows the resolved signed-zero and subnormal contract rather than an ambient target mode", so a flush behaviour that leaves the sign unstated cannot be checked against measured hardware or reference-evaluated. Whether the sign is a field of the flush behaviour or resolved from the contract's signed-zero dimension is yours to choose; leaving it unstated is not.

## Identity — the defect this ticket must close

Three sibling encoders treat the realization three different ways, and only one of them fails closed when the vocabulary widens. Verify each by reading the file; do not conclude anything about them from a substring search.

- **`tiler_ir::kernel::model::push_numerical` is correct.** It encodes the profile key, canonical NaN bits, both subnormal modes, and both permissions, through `push_subnormal` and `push_permission` whose `match` arms are exhaustive over non-`#[non_exhaustive]` enums. Adding a variant is a build error there.
- **`tiler_ir::schedule::model::push_numerical` is the defect.** It encodes the profile key, the NaN bits, and the two `permits_*` booleans, and encodes **neither subnormal field**. Because those accessors are `!matches!(…)` expressions rather than matches, adding a variant compiles silently, and two regions that differ only in subnormal treatment receive the same `CanonicalScheduledRegionIdentity`. That is a cache-and-artifact correctness failure of exactly the class `AGENTS.md` singles out for special scrutiny, and it would not warn.
- **`crates/tiler-metal/src/emit.rs` guards a third way**, with irrefutable `let SubnormalMode::Preserve = mode;` bindings in `realization_requirements` and `record_subnormal_obligation` that become compile errors on widening. Those are outside this ticket's scope and will break your build; that is the guard working. Coordinate with `declare-metal-numerical-honourability` or add the scope.

The omission is masked today only because `profile_key` is encoded and every distinct contract has so far carried a distinct key. Relying on a key to stand in for the field values it names is an unstated invariant. Make `schedule::model::push_numerical` complete over every field and exhaustive per field, matching the kernel encoder, and update whatever scheduled-region identity fixtures that shifts.

## The second defect: `derive_requirements`

`tiler_ir::schedule::model::derive_requirements` computes `requires_strict_f32: !permits_reassociation() && !permits_contraction()` — reading contraction and reassociation and ignoring both subnormal fields entirely. Once the vocabulary widens, a subnormal-preserving contract that permits contraction and reassociation derives `requires_strict_f32 == false` and would be **admitted** on a target declaring no strict-`f32` support, because the obligation it needs was never in the predicate. Stop collapsing the realization into one summary bit; carry the region's declared realization forward per dimension so `select-numerical-contract-and-compose-feasibility` can compose it.

## The contract half

`docs/ir.md` states that "`IndexRegion` identity commits only to the canonical structural program: iteration and reduction domains, typed tensor boundaries, access maps, scalar operations and values, constraints, and ordered outputs", and its layered summary reads "`IndexRegion` commits to canonical iteration/scalar/access content." The numerical realization appears in neither enumeration — even though the implemented `IndexRegion` carries a `numerical: NumericalRealization` field and its encoder partially encodes it, and even though `ScalarProgram`'s own variants separately carry the canonical NaN bits and a contraction flag. So part of the realization sits inside "scalar content" while the region-level declaration sits outside it, and the contract and the implementation disagree about where it belongs in the identity layering. That disagreement is *why* the encoder can omit the field without contradicting anything written down.

State the answer in `docs/ir.md`. Repairing the encoder without stating where the realization sits in the layering would leave the next encoder free to make the same omission.

## Boundaries

Convention 5 applies: do not blanket-apply `#[non_exhaustive]`. These enums are matched to decide *support* and are encoded into identity across crate boundaries, so they must stay exhaustive — an unhandled variant must be a build error, which is the entire mechanism this ticket relies on.

Run the full gate. A widened enum with unchanged identity fixtures is expected to shift those fixtures; that shift is the evidence the encoding is now complete, so record the before and after rather than only asserting the new values.

## Outcome

ADR 0076 items 1 and 6 are implemented. Both halves landed together, because the vocabulary widening is what makes the identity omission observable.

### Vocabulary

`SubnormalMode` is now `Preserve | FlushToZero { zero_sign: FlushedZeroSign }`, and `FlushedZeroSign` is `PreservesSign | AlwaysPositive`. `NumericalPermission` is now `Forbidden | Permitted`. `NumericalRealization`'s `input_subnormals` and `result_subnormals` stay two independent fields; nothing couples them and no dimension is a boolean.

**The flushed zero's sign is a field of the flush behaviour, not a resolution from a signed-zero dimension.** ADR 0019 says zero-sign behaviour "is resolved with the signed-zero contract" and ADR 0076 item 1 explicitly opened the choice. The deciding argument is that a signed-zero dimension is permission-shaped: a contract that *permits* ignoring the sign of a zero leaves the flush result's sign unspecified, and an unspecified flush result is exactly the under-specification ADR 0076 says a flush must not have — it cannot be checked against measured hardware and cannot be reference-evaluated. Carrying the sign on the behaviour makes every `SubnormalMode` value answer "which zero" on its own, without the rest of the realization in hand, which is what the Metal emitter, the artifact codec, and a future reference evaluator each need. `FlushedZeroSign` has two variants rather than one: a sign vocabulary with a single inhabitant cannot express a mismatch, which is the same defect this ADR exists to remove, and the two variants are not speculative hardware — they enumerate the two zeros binary32 has.

`permits_contraction` and `permits_reassociation` were `!matches!(…)` expressions; they are now exhaustive matches through a shared `permits` helper, so a third permission stops the build there too.

### Identity

`schedule::model::push_numerical` encoded the profile key, the NaN bits, and the two derived `permits_*` booleans, and encoded neither subnormal field. It now encodes every field, each through an exhaustive per-field match: `push_subnormal` (`Preserve` 0x01, sign-preserving flush 0x02, positive-zero flush 0x03) and `push_permission` (`Forbidden` 0x01, `Permitted` 0x02), the same tag values `kernel::model` uses. A derived boolean is no longer encoded anywhere in that identity, because a projection cannot fail closed when its source grows.

`docs/ir.md` now states where the realization sits. Its `IndexRegion` identity enumeration and its layered identity summary both name the declared numerical realization, and a new paragraph gives the reason it is inside the structural program rather than beside it — it says what the region's scalar operations mean, so two regions identical in domains, accesses, and scalar content but differing in a numerical resolution compute different values. The same paragraph states the obligation the encoder must meet (complete over every dimension, exhaustive per dimension, no contract key or derived predicate standing in for a field) and reconciles the region-level declaration with the numerical fields `ScalarProgram` variants carry: both are encoded, and the structural verifier requires them to agree, so they are one authority and a refinement rather than two.

### `derive_requirements`

`ResourceRequirements::requires_strict_f32` is gone. The record now carries `input_subnormals`, `result_subnormals`, `contraction`, and `reassociation`, copied from the region's realization. `profile_key` and the canonical NaN bits are deliberately not repeated: they name the governing contract and a produced value rather than a behaviour a target declares honourability for — and `profile_key` is a `&'static str`, which the artifact decoder cannot reconstruct from bytes (`own-the-numerical-realization-profile-key` owns that).

The single `CapabilityAxis::StrictF32Arithmetic` requirement is still needed until `select-numerical-contract-and-compose-feasibility` retires it, so the collapse moved to `physical::requires_strict_f32`, where the axis lives, and changed shape: it is a **disjunction** over all four dimensions, matched exhaustively per dimension. The predicate it replaced was `!permits_reassociation() && !permits_contraction()` — a conjunction over two of four — so a subnormal-preserving contract permitting both transforms derived `false` and would have been admitted on a target declaring no strict-`f32` support. `a_relaxed_transform_contract_still_carries_its_subnormal_obligation` in `crates/tiler-ir/src/schedule/builder.rs` pins exactly that case.

### Build breaks, all repaired without a wildcard

Four scopes were added before touching each area: `implementation/metal`, `implementation/artifact`, `implementation/compiler`, `contracts/numerics`. Every break was a guard working.

- `crates/tiler-metal/src/emit.rs` — both irrefutable `let SubnormalMode::Preserve = mode;` bindings, plus both `NumericalPermission` matches. `realization_requirements` now answers the flag question for all four dimensions: a granted permission names no flag, and neither subnormal behaviour names one, for two different measured reasons recorded on the function. `record_subnormal_obligation` delegates to a new total `subnormal_gap(declared, target)`; `MetalSubnormalArithmetic` is `#[non_exhaustive]` but is defined in the same crate, so the match is wildcard-free.
- `crates/tiler-metal/src/record.rs` — `MetalNumericalGap` gained `SubnormalPreservationInArithmetic` and `UndeclaredFlushedZeroSign` so the two newly expressible divergences are nameable rather than silently admitted.
- `crates/tiler-artifact/src/program/model.rs` — `subnormal_tag`/`permission_tag` and their `*_from_tag` inverses, all four exhaustive in the encoding direction and each new variant given its own tag. The decoders keep their `_ => None` arm, which is a `u8`-to-enum recognizer over untrusted bytes and not a vocabulary wildcard. `push_resources` and `codec::decode::parse_entry` follow the `ResourceRequirements` change.
- `crates/tiler-compiler/src/fusion.rs` — the explain-evidence encoder's `match permission { Forbidden => 1 }` now calls a shared `request::permission_tag`; the encoded value for `Forbidden` is unchanged.
- `crates/tiler-compiler/src/request.rs` — **a break the ticket did not predict, and the worst of the four sites.** `VerifiedRequestSubject::canonical_explain_subject_bytes` encoded all four numerical fields with `as u8` **discriminant casts**. That is strictly worse than a wildcard: a wildcard at least survives a variant reorder, whereas a discriminant cast silently re-encodes every request subject when variants move, with no diagnostic anywhere. It only became a build error because `FlushToZero` is a struct variant and `as` rejects those. Replaced with the exhaustive `subnormal_tag`/`permission_tag` helpers, now shared with `fusion.rs`.
- `crates/tiler-compiler/src/physical.rs` — `region_proposal`'s use of the retired boolean, repaired as described above.

No wildcard arm was introduced at any site. No enum gained `#[non_exhaustive]`.

### Identity fixtures, before and after

Every shift below is the encoding becoming complete, not drift.

| subject | before | after |
| --- | --- | --- |
| scheduled-region identity (strict-`f32` pointwise fixture) | 192 bytes, `sha256 d900fe4a759cc25e40b4c88dfdc4f411c0effee4d8c07df56ab2c53ca0cf65d4` | 194 bytes, `sha256 d221e1a36e9912b6eac694bab0d19590317d4364c0463711064a88ead02e89d2`, pinned as the exact hex in `builder.rs` |
| kernel identity (same fixture) | 607 bytes, `sha256 39804fc0bdb3b66fbf3526cfdd43bba63aac65556908aeb68295993948ddce65` | 612 bytes, `sha256 75181a5c5ae85e038e049be6d6807051f1f12b1be6cc528374c36d2f09b3efc1` |
| artifact-program identity (`default_artifact`) | 12833 bytes, `sha256 3a622133cc096c88d00840ce55a99aae293e26091f6dfe90356e6c2dcfacc966` | 12866 bytes, `sha256 271e9e359733be1626b4e985e8fa9e2d7e2dd257405ee4e32c4fcbc8dbf4cb1b` |
| explain request subject digest | `47bfe7ba37961bc3` | `be70237691f8f507` |

The scheduled-region encoding grew by exactly two bytes: the two subnormal tags. The permissions changed from derived booleans to tags of the same width. Golden MSL digests moved with them, and nothing else in the four goldens changed — in particular each still records `subnormal-flush-in-arithmetic` and only that, because the registered contract is still `Preserve`/`Preserve`:

| golden | kernel digest | scheduled-region digest |
| --- | --- | --- |
| `pointwise_scale_bias.metal` | `80528feef1f7070b` -> `56c4136874313b48` | `80c5c077ba0c383f` -> `8747500aa18bd2fb` |
| `reduction_single_axis.metal` | `7cc43171ae13bcce` -> `00634cefd2e0d8df` | `c48427542a02afb5` -> `30a8c423c1663849` |
| `reduction_multi_axis.metal` | `375c766b3db5a012` -> `cc845f33d21e62b1` | `f41ee169d570d82d` -> `c5420224b5719911` |
| `reduction_fused_multiply_add.metal` | `4e75d6dcce52e254` -> `b7b499964dd388f1` | `ea4d348d458bfdf1` -> `39820b13aedee425` |

### Tests

`every_numerical_dimension_separates_scheduled_region_identity` walks all four dimensions plus the flushed zero's sign, holding `profile_key` fixed so the key cannot stand in for the fields it names. Its subject is `encode_identity` rather than the builder, because the schedule verifier separately requires the scalar program's contraction flag to agree with the permission, and varying both would stop isolating the numerical field. `the_strict_f32_region_has_its_recorded_canonical_identity` pins the exact 194 bytes, so a later reordering or omission cannot pass the distinctness test alone. The artifact codec's tag-table round trip enumerates both flush behaviours and both permissions.

### Deferred, with the reason

- **The Metal profile's own flushed-zero declaration** goes to `declare-metal-numerical-honourability`, which already owns replacing `MetalSubnormalArithmetic` with a per-dimension honourability declaration. Refining it here would have pre-empted that restructuring. Until then a declared flush fails closed as `UndeclaredFlushedZeroSign`, and that ticket carries a note saying to retire the variant rather than keep it. The *flag* question is settled and recorded, so nothing about it is deferred.
- **Retiring `supports_strict_f32` and `CapabilityAxis::StrictF32Arithmetic`** is `select-numerical-contract-and-compose-feasibility`'s, per ADR 0076's ordering. This change stopped the collapse at its source and moved the surviving summary to the axis, where the next ticket deletes it.
- **`fusion_legality::effect_tag` and the two tag-form deviations** stay with `extend-canonical-identity-encodings-for-reserved-variants`, whose first bullet this change closes. That ticket now records what closed and that its remaining edits are a second deliberate re-baseline, with the shifts above so the two can be told apart.
- **`docs/decisions/**` and `docs/compiler/optimizer.md`** carry claims this change falsified, and both scopes were held by live siblings for the whole of the work. Filed as `reconcile-adr-records-with-the-widened-numerical-vocabulary` (p1, `contracts/decisions`) and `correct-the-optimizer-one-variant-permission-claim` (p2, `contracts/optimizer`). The first also asks ADR 0074's convention 5b to name the `as`-cast form, which it does not currently cover and which was the most dangerous site found. `docs/numerical-semantics.md` carried the same claim and was corrected here, because `contracts/numerics` was free.
