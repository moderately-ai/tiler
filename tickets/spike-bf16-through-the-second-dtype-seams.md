---
id: spike-bf16-through-the-second-dtype-seams
title: Spike BF16 through the second-dtype seams
status: in-progress
priority: p1
dependencies: [register-the-accepted-built-in-dtype-catalog, construct-and-bind-the-first-authoritative-metal-compile-profile]
related: [preserve-primary-dtype-standards-evidence, own-the-dtype-support-maturity-matrix, own-operation-family-support-matrix, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, widen-the-f16-operation-vocabulary-to-contraction-and-reassociation, admit-a-caller-declared-target-profile, decide-per-dtype-dispatchability-as-a-target-capability, redesign-the-delivered-realization-record-from-typed-evidence, measure-apple-numerics-on-physical-ios-device]
scopes: [research/numerics, research/apple-targets, contracts/navigation]
shared_scopes: [project/tickets]
paths: [spikes/dtypes/bf16-second-dtype/**, docs/dtype-support.md, docs/roadmap.md]
tags: [dtype, bf16, spike, vertical-slice, planning]
claimed_from: todo
assignee: worker-spike-bf16-t
lease_expires_at: 1785565712
---
## User-visible outcome

A checked-in bounded BF16 proof crosses every second-dtype seam without promoting the spike into production support: canonical semantic identity, pure-BF16 constant/multiply/add semantics and exact reference vectors, a macOS Metal success route, and an iOS-Simulator typed pre-routing dtype refusal from the same target-profile mechanism. The spike records what held and what required redesign, updates the dtype and operation maturity ledgers at exact cells, creates or refines the dependency-ordered production tickets that would carry BF16 end to end, and leaves a reusable dtype-addition recipe that makes the next ordinary, integer/boolean, compound quantized, or vendor dtype explicit without pretending those families have identical requirements.

## Why BF16 is the architecture-driving widening

**Fact:** the dtype maturity ledger says BF16 has retained Apple conformance evidence but no standard semantic registration, operation signature, reference evaluation, numerical profile, physical ABI, optimizer legality, structured-kernel vocabulary, lowering, runtime validation, target dispatch fact, or backend vertical. The existing BF16 tickets measure numerical behavior and carry a Metal subnormal fact; they do not implement a Tiler BF16 program.

**Fact:** macOS accepts and runs the retained BF16 probes, while the iOS Simulator compiles and links the same BF16 modules and then refuses pipeline creation on the same physical GPU. The refusal includes an arithmetic-free materialization kernel, so the measured discriminator is target-family dtype dispatchability rather than one unsupported arithmetic operation. The retained probe also shows that Metal's `fma(bfloat, bfloat, bfloat)` promotes through F32 instead of providing a pure-BF16 fused operation.

**Inference:** BF16 is the strongest pattern-setting proof because it forces both positive execution and negative routing through one exact resolved dtype, and it prevents logical value dtype from being silently equated with computation or accumulator dtype. F16 is the cheaper positive-only route and remains an important later production vertical, but its retained probe has already spent the second-dtype-through-emission uncertainty and does not exercise negative dispatchability. Integer and boolean programs require different semantic operation families, while the completed U4/F32 vertical owns compound encoded values and does not prove a second ordinary arithmetic dtype.

## Scope keys

- Implement only a bounded pure-BF16 constant/multiply/add tensor signature with round-to-nearest-ties-to-even at every observable materialization and an explicit contract for subnormals, signed zero, NaN, infinity, overflow, and underflow. FMA, contraction, reassociation, mixed precision, and implicit promotion reject by typed reason rather than inheriting the observed F32 promotion.
- State logical input, constant, computation, accumulator, intermediate-materialization, and result types independently even where every selected field is BF16. A future F32 accumulator or BF16/F32 conversion requires an explicit operation with rounding and exceptional-value semantics; this spike must not reserve it as an ambient default.
- Reuse the accepted BF16 identity and primary standards evidence. Recognition is supplied by `register-the-accepted-built-in-dtype-catalog`; this ticket must not invent another key or make catalog registration imply arithmetic support.
- Derive the numerical contract from the retained macOS BF16 evidence and label its exact host/toolchain/family boundary. Do not generalize the measured flush behavior to iOS families or other devices.
- Drive the complete resolved BF16 identity through the caller target-profile and Metal honourability adapter. macOS supplies the measured positive fact; the iOS Simulator supplies a typed pre-routing `dtype unavailable` outcome; unknown families and absent facts reject; no neighboring F32 or F16 fact is inherited.
- Specify the exact physical carrier, storage encoding, alignment, binding ABI, host representation, structured-kernel scalar operations, Metal spelling, and runtime validation. Do not add caller-declared ABI fields to `BindingSpec`.
- Select or design a deterministic bit-level BF16 reference oracle and conformance corpus covering zeros, subnormals, normals, infinities, NaNs, ties, overflow, underflow, and operation-specific witnesses. Host-native arithmetic alone is not normative evidence.
- Audit every F32-named enum, match, constructor, identity encoder, cache key, explain record, artifact field, and test helper reached by the vertical. Classify each as legitimately F32-specific, scalar-float-generic, or a missing typed extension point; do not mechanically rename F32 into a universal abstraction.
- Add a durable dtype-addition recipe beside the maturity ledger. It must start from a named workload and walk recognition, semantic operation signatures, reference semantics, numerical policy, computation and accumulator types, optimizer legality, physical storage and ABI, artifact identity, target dispatch, lowering, runtime validation, and conformance. Every rung names its typed unsupported state, evidence class, identity consequence, and owner rather than reducing support to an enum variant.
- Validate that recipe against four dry-run cases: an ordinary float after BF16, an integer or boolean dtype, a compound quantized or MX dtype, and an external vendor dtype. The dry runs need not create implementation tickets, but they must identify which steps genuinely reuse the BF16 pattern and which require family-specific semantic, encoding, namespace, or target work. A recipe that assumes scalar floating arithmetic, one physical buffer per logical value, or built-in ownership fails this check.
- File dependency-ordered implementation tickets with non-overlapping scopes for identity/registration, semantic operations and validation, reference evaluation, numerical profile and target honourability, optimizer/schedule/KIR, storage/ABI/artifact identity, Metal lowering, runtime routing/validation, target dispatch, and conformance. Merge layers only when one fixture must change atomically to remain correct.
- File a separate computation/accumulator/conversion design ticket for BF16 FMA or mixed precision rather than widening this pure-BF16 proof. Make each child state its maturity claim and fail-closed boundary, and keep later F16, F64, integer/boolean, quantized/MX, OCP reduced-format, and vendor-format triggers explicit without creating a generic “support every dtype” implementation ticket.

## Required evidence

- A table maps every dtype-maturity and operation-maturity column to its current authority, required BF16 producer and consumer, child ticket, dependency, success fixture, and typed unsupported case.
- The dtype-addition recipe maps each maturity rung to its public or internal authority, required evidence, versioned identity domains, negative test, and ticket owner; the four family dry runs demonstrate that omissions and non-applicable steps remain explicit rather than silently inherited.
- The selected BF16 program is represented end to end with ordered inputs/outputs, operation keys, resolved value types, numerical contract, physical types, ABI bindings, target facts, runtime predicates, and expected result bits.
- The checked-in spike runs the exact reference corpus, demonstrates the macOS success route where the environment resolves, and demonstrates the iOS-Simulator target-profile refusal without submitting BF16 program work. A host without the measured environments must still run deterministic reference and structural checks while reporting the unavailable measurement boundary.
- At least one F32-only assumption is reproduced by a one-line source check and assigned to a child; at least one existing generic seam is shown to accept BF16 identity without modification.
- The plan distinguishes measured host/toolchain facts from portable guarantees and identifies every identity domain or pinned fixture expected to move.
- `docs/dtype-support.md`, the operation-family matrix, roadmap trigger, and related ticket graph agree with the selected vertical and do not claim implementation before the children land.

## Closes when

The pure-BF16 workload, exact semantics, reference corpus, macOS success route, and iOS-Simulator pre-routing refusal are implemented or precisely bounded; the complete per-layer table and end-to-end example are recorded; the reusable dtype-addition recipe survives the ordinary-float, integer/boolean, compound quantized/MX, and vendor-format dry runs; all required production child tickets exist with correct dependencies, scopes, tests, mutation proofs, identity consequences, and fail-closed cases; the separate BF16 computation/accumulator/conversion question is filed; later F16 and non-float triggers are explicit; the dtype and operation ledgers are updated without claiming production support; the spike's own harness passes; `tkt lint` and `git diff --check` pass; and no unresolved correctness or public-boundary choice is hidden inside an implementation child.

## Graph maintenance

- Consume BF16 identity only after `register-the-accepted-built-in-dtype-catalog`, whose own dependency on `preserve-primary-dtype-standards-evidence` keeps the normative source reproducible. Do not copy BF16 format tables into implementation tickets.
- Consume production target facts only after `construct-and-bind-the-first-authoritative-metal-compile-profile`; the F32 projection ticket provides a low-level caller-vouched seam rather than the authoritative construction and runtime-applicability mechanism this spike must widen. The spike remains the first non-F32 downstream proof and must not add a parallel backend dtype list.
- Relate every child to `own-the-dtype-support-maturity-matrix` and update only the cells its delivered evidence advances.
- Connect target-profile, dispatchability, numerical-honourability, delivered-realization, artifact, runtime, and Metal children to their existing owners instead of creating parallel vocabularies.
- Close this spike once its bounded evidence and child graph are complete; production implementation continues in those children rather than keeping the spike in review.
