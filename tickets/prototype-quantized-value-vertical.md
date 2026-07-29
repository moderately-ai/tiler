---
id: prototype-quantized-value-vertical
title: Prove a quantized compound-value vertical
status: review
priority: p2
dependencies: [implement-first-profile-numerical-policies]
related: [implement-workload-selected-quantized-parameter-maps, implement-first-runtime-semantic-value-precondition-enforcement, admit-a-dtype-dispatchability-capability-axis, scope-first-quantized-lm-profile, implement-first-quantized-backend-profile, group-internal-compound-materializations-by-logical-value]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/artifact, contracts/numerics, contracts/artifacts, implementation/metal, implementation/build, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, dtype, vertical-slice]
claimed_from: todo
assignee: codex
lease_expires_at: 1785296825
---
## User-visible outcome

A quantized value is a typed compound contract — storage plus inseparable metadata, with layout/conversion/materialization stated as three separate contracts — proven end to end on one scheme, with every unsupported scheme refused by name. This is the vertical that decides whether quantization is a dtype or a contract, before any LM work depends on the answer.

Prove quantized values as typed compound storage/metadata contracts rather than integer dtypes alone. Cover metadata association, validation, reference semantics, layout/access lowering, conversion/materialization, identity and ABI binding while explicitly rejecting unsupported schemes and preserving future block/group formats.

## Closes when (2026-07-28)

1. **A quantized value is a typed compound contract, not an integer dtype with a convention attached.** Storage and its metadata — scale, zero point, whatever the scheme requires — are one value type, and the metadata cannot be separated from the storage it describes at any layer. A value whose metadata is carried alongside by caller discipline does not close this.
2. **The numerical contract is stated separately for each of three things, because they are three different contracts.** *Layout and access lowering:* how a packed element is addressed, and what a partial or unaligned access means. *Conversion:* the exact rounding and saturation behaviour quantizing into and dequantizing out of the scheme, including what happens at the representable boundary and on exceptional inputs. *Observable materialization:* the rounding applied when a quantized value becomes a result a caller reads. Folding these into one "quantization contract" is the failure mode — they are separately observable and they can disagree.
3. **Reference semantics exist and are compared against.** The reference oracle computes the same program over the same scheme, and the comparison's tolerance is derived from the scheme's stated conversion contract rather than chosen to make the test pass.
4. **Identity and ABI binding fold the scheme into artifact identity.** Two artifacts differing only in quantization scheme, or only in a scale that changes what the bytes mean, must have different identities. A scheme that reaches the ABI without reaching identity is a silently wrong cache hit waiting to happen.
5. **Every unsupported scheme is refused with a typed error naming it**, at the earliest layer that can name it, and never approximated by the nearest supported scheme. A test drives at least one unsupported scheme and watches the refusal fire.
6. **Block and group formats are reserved as an architectural seam with an explicit record of what broadening would require** — not implemented, not silently excluded. `AGENTS.md` distinguishes four maturity claims: a type-system reservation, an architectural seam, implemented support, and a tested guarantee. Say which of the four this ticket delivers for block/group formats, and record what a fifth reader would need to move it to the next one.
7. **`make full` passes.**

**No field is reserved before its producer exists.** `carry-the-honourability-fact-provenance-into-the-artifact-record.md:27` states the rule and the reason: "a field a producer cannot fill is the producer-less placeholder this repository has repeatedly had to retract, which is exactly why the draft omits them rather than defaulting them." That applies directly here, because a quantized-value vertical is exactly the kind of work that invites reserving a `block_size` or a `group_metadata` field ahead of any code that fills one. Omit it and record what would be needed, rather than defaulting it.

## Dependency note (2026-07-28)

`implement-first-profile-numerical-policies` is `status: in-progress` with completed but **uncommitted** work in the harness worktree `.claude/worktrees/agent-ad2893b1fba4d7f5b`. Its Outcome already states this ticket's seam explicitly, and states it as an *absence* rather than a stub: "Preserved by absence rather than by a placeholder. `ArithmeticType` names scalar float formats; a compound or quantized value is a scheme-typed `ResolvedValueType::encoded_numeric` whose conversion behaviour is its own typed contract, and `operation_capabilities` enumerates only the scalar `f32` operations this build admits, so an operation outside that table has no capability entry and therefore no effective permission to compute."

Two consequences for this ticket. The seam it must build on is `ResolvedValueType::encoded_numeric`, already named — do not introduce a second spelling for a compound value type. And the fail-closed property is currently supplied by `operation_capabilities` having no entry for a quantized operation, which means **admitting one is what removes the current protection**: every capability entry added here must arrive with its conversion contract, or the absence that was doing the work is gone and nothing replaces it.

## Graph maintenance

- **State which of the four maturity claims you delivered for block/group formats** (criterion 6) on this ticket at close — reservation, seam, implemented, or tested — and file the broadening ticket only if you can name its first consumer.
- **Scheme-into-identity** (criterion 4) advances the artifact identity domain: reason recorded at the site, and expect the determinism pins to move exactly once.
- **The compound/quantized seams in the numerical-policy work are preserved by absence** (its worktree Outcome says so explicitly) — when `rebase-and-land-the-stranded-numerical-policies-worktree` lands, check whether its eleven-dimension vocabulary gives your conversion contract a home before inventing one.

## Broader dtype audit and ownership (2026-07-28)

This vertical must not make `f32`, affine quantization, or three fixed metadata roles the foundation of the value model. The accepted taxonomy and ADRs require the implementation and its tests to preserve these separations:

- Boolean is a two-valued logical type, not `i1`; packed bits, bytes, and target-native representations are physical encodings.
- Signed and unsigned integers have exact widths and operation-specific overflow, division, and conversion semantics; a packed `u4` integer and a strict-affine value whose codes are `u4` are different logical values.
- Complex is a parameterized logical type; planar and interleaved component storage are physical choices.
- Binary, decimal, bfloat, FP8/FP6/FP4, fixed-point, normalized, posit, and extension formats retain namespaced versioned identity without implying operation, reference, storage, lowering, dispatch, or native-arithmetic support.
- Quantized and block-scaled values have scheme-defined ordered component roles. Future codebook, hierarchical-scale, mask, outlier, nested-metadata, MX, NVFP, GGML, and vendor formats must be able to add different role sets without modifying a universal affine struct or a core enum of scheme combinations.
- Sparse and ragged values remain unsupported value families until their logical cardinality, component contracts, views, and validation semantics are admitted; they must never be approximated as dense compound values.

The reusable boundary delivered here is therefore generic ordered component declarations on `ResolvedValueType::encoded_numeric`, a typed parameter-index-map seam with only a real per-tensor producer, and a separate physical storage encoding. Recognition, structural validation, reference evaluation, semantic operations, physical storage, ABI expansion, lowering, runtime enforcement, target dispatchability, and native execution remain separately admitted capabilities.

Existing owners cover the next concrete consumers:

- `implement-first-runtime-semantic-value-precondition-enforcement` owns logical-view reconstruction and validation across packed boolean/sub-byte, complex, quantized, extension, sparse, and ragged representations, with unsupported cases refused by name.
- `scope-first-quantized-lm-profile` is the first named consumer that may justify per-axis/per-block maps, codebooks, hierarchical metadata, or a native packed contraction profile; it must file dependency-ordered implementation tickets from measured workload evidence.
- `implement-first-quantized-backend-profile` remains deferred until that workload-backed selection names the exact scheme, operation set, target, storage layout, numerical contract, and conformance corpus.
- `admit-a-dtype-dispatchability-capability-axis` owns target-family dispatchability keyed by the semantic dtype vocabulary; this vertical must not create another dtype list inside artifact or backend code.

At close, record exactly which rows are type-system reservations, architectural seams, implementations, and tested guarantees. A test using a nominal bool, integer, complex, or opaque encoded type proves dtype-neutrality of a generic mechanism only; it does not claim arithmetic or backend support for that type.

## Cross-layer audit findings that must close with this ticket (2026-07-28)

The first implementation draft exposed correctness gaps that a local green test suite could not detect. They are part of this ticket rather than follow-ups because leaving any one of them would make the claimed vertical internally inconsistent:

- Replace the duplicate compiler/program packed-storage vocabularies with one shared realized `StorageEncoding`. Preserve compiler requirement/guarantee wrappers where they express planning, but require an explicit checked lowering from the selected boundary property into the program materialization.
- Restrict the current per-byte packed contract to widths that divide eight. Widths that cross bytes need a separately specified bitstream order and are unsupported until a real consumer supplies that contract.
- Separate the physical storage scalar/carrier from semantic component type and kernel SSA type. The ABI must truthfully name unpacked `u8` and the byte carrier of packed `u4`; `KernelType::Bool` is a control predicate and must not be reused as a byte spelling. Validate semantic component, storage carrier/encoding, and kernel access compatibility at their owning boundaries.
- Widen the verified structured-kernel profile far enough to produce the `u8` accesses the proof actually needs. Merely adding a `KernelType::U8` spelling or Metal type name is not executable support while `verify_signature` and canonical lowering still require exactly two f32 buffers. The successful artifact fixture must derive its component bindings from a verified kernel that implements the named strict-affine operation; an unrelated f32 stage, a no-op read, or a forged envelope is not evidence.
- Keep the target-neutral kernel proof distinct from Metal executability. The admitted strict-affine contract preserves f32 subnormals, while the governed Apple profiles are measured and documented to flush f32 arithmetic; native Metal multiplication therefore cannot implement this contract. Build and verify the exact target-neutral dequantization KIR and use it to prove producer-derived artifact ABI/identity, but make `tiler-metal` reject it by the typed numerical-contract boundary. `implement-first-quantized-backend-profile` owns any later software-preserving Metal implementation or a separately authorized FTZ profile.
- Permit physical encoding on ordinary nominal values independently of compound-component roles. Packed boolean and packed nominal integers are physical choices, not encoded-numeric schemes.
- Reject encoded contracts with no physical components when program materialization is requested, rather than accepting an omitted interface value that later fails artifact decoding.
- Reject nested encoded component types at this bounded program/ABI boundary until recursive role paths and flattening have an implemented producer. Generic ordered roles remain the extension seam; silent one-level flattening is forbidden.
- Carry enough packed access information into verified program structure to reject neighbor-clobbering writes. The bounded first proof may admit whole-component packed views only; arbitrary partial packed writes require a typed logical-element range and ownership contract.
- Expose the semantic component type symmetrically on verified and decoded artifact views, and publicly re-export every item type returned by a public iterator.
- Make the strict-affine normative widened subtraction and the reference implementation use the same integer width.
- Add a successful strict-affine program/artifact/codec fixture proving complete ordered roles, unpacked `u8`, packed `u4`, binding-role association, round-trip decoding, and typed corruption refusals. Fault-inject every new invariant.
- Add identity proofs that vary scheme, static contract, component role/order/type/map, embedded scale bits, and storage encoding one at a time. Separately prove that changing a runtime scale payload changes evaluation but not the program/artifact identity.
- Strengthen the reference proof to compare component roles and payload bits exactly, including scale, zero point, signed zero, and the smallest positive subnormal. The implemented strict-affine comparator has zero tolerance because its contract specifies exact `f32` evaluation.

## Outcome (2026-07-28)

The vertical is implemented as a target-neutral structural and exact-reference proof. `ResolvedValueType::encoded_numeric` now owns generic ordered component declarations and a typed parameter-index-map seam; the governed proof instances are per-tensor strict-affine U4/F32 and U8/F32. Exact reference association, quantization, dequantization, validation, exceptional-value handling, signed zero, subnormal preservation, saturation, and round-to-nearest-ties-to-even are exercised against component-exact fixtures.

The executable proof is intentionally narrower than the semantic proof. Strict-affine U4 dequantization lowers through role-addressed schedule accesses, packed-U4 extraction, widened I32 subtraction, I32-to-F32 conversion, F32 multiplication, verified kernel program construction, producer-derived artifact ABI expansion, neutral encoding, public decoded views, and byte-identical re-encoding. Quantization has exact semantic and reference support but no backend publication path because producing an internal compound value requires producer-derived logical grouping that did not exist and must not be inferred from roles or slots.

Physical representation is independent of semantic type. One shared `StorageEncoding` authority describes unpacked storage and the exact `PackedU4LsbZeroTail` layout; `StorageScalar` names only truthful U8 and F32 carriers; `KernelType` independently names U8, I32, F32, Index, and Bool computation/access roles. Ordinary nominal values may be physically packed without becoming encoded-numeric schemes. Nested encoded components, empty encoded component sets, partial packed writes, incompatible carrier/encoding/access combinations, and ungrouped internal compound materializations fail with typed diagnostics.

The strict-affine schedule verifier requires preserved input and result subnormals, forbids contraction, reassociation, permutation, and signed-zero elimination, and rejects both NaN-absence and infinity-absence assumptions. The latter check was added after an independent final audit showed that finite valid inputs can still overflow to infinity; the negative fixture was run before the verifier change and failed by accepting the forged assumption, then passed after the fix.

Artifact identity advanced to `tiler.artifact-program.v9` and manifest schema 7.0. Its dependency ledger is `tiler.resolved-value-type.v3`, `tiler.schedule.v2`, `tiler.kernel.v3`, and `tiler.kernel-program.v5`. Identity tests independently perturb scheme, static fields, component roles/order/types/maps, embedded scale bits, storage scalar/encoding/access type, and binding association. Runtime scale payload changes alter evaluation without altering program identity, while embedded constants alter identity.

Metal execution remains correctly unsupported for this exact profile. The governed Apple target profiles flush F32 arithmetic subnormals, while the strict-affine contract requires preservation, so `tiler-metal` reaches the explicit U4/I32 syntax path and then refuses with `SubnormalFlushInArithmetic` before producing a falsely executable payload. Runtime preflight must additionally validate positive finite scale, in-range zero point, and canonical packed tail bits before routing commit; `implement-first-runtime-semantic-value-precondition-enforcement` owns that work.

### Maturity ledger

- **Tested guarantee:** exact per-tensor strict-affine U4/F32 and U8/F32 semantic/reference behavior, including exact component payloads and typed unsupported-contract refusal.
- **Tested guarantee:** target-neutral strict-affine U4 dequantization through role-addressed schedule, KIR, program, artifact identity, codec validation, decoded ABI views, and typed Metal numerical refusal.
- **Implemented mechanism:** generic ordered component declarations, one real per-tensor parameter map, independent storage scalar/encoding, component-aware access/binding identity, and fail-closed validation.
- **Architectural seam:** additional parameter maps and arbitrary ordered role sets can extend the same dependency direction without adding affine fields or a second dtype list.
- **Type-system reservation only:** per-axis, per-block, per-group, codebook, hierarchical-scale, MX, NVFP, GGML, mask/outlier, complex compound, sparse, and ragged families have no claimed reference, lowering, runtime, dispatch, or backend support.

### Filed follow-up ownership

- `implement-workload-selected-quantized-parameter-maps` implements only the first non-per-tensor map selected by measured workload evidence and closes as obsolete if the selected profile stays per-tensor.
- `group-internal-compound-materializations-by-logical-value` adds producer-derived grouping before any backend may publish an internally produced compound value.
- `scope-first-quantized-lm-profile` selects the first real scheme, operation, target, storage, map, and conformance corpus.
- `implement-first-quantized-backend-profile` consumes that selection, depends on internal grouping, conditionally depends on parameter-map and runtime-validation work, and must reject every unselected row by name.

### Verification

The final targeted runs passed 319/319 `tiler-ir` tests, 13/13 `tiler-build` tests, and 16/16 `tiler-prototype-run` tests. The exceptional-value verifier fixture was first observed failing against the permissive implementation and then passing after the verifier rejected forged NaN/infinity absence assumptions. Artifact tag/order and decoder validations, reference exactness checks, storage/access compatibility, and the Metal numerical refusal were likewise fault-injected during implementation.

The completed `make full` passed workspace check, crate Clippy with warnings denied, 1,264/1,264 workspace tests with four configured skips, all doc tests, rustdoc with warnings denied, 474/474 release-profile reference/compiler tests with one configured skip, `tkt lint`, and shellcheck.
