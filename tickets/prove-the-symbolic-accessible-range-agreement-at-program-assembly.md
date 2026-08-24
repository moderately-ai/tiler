---
id: prove-the-symbolic-accessible-range-agreement-at-program-assembly
title: Prove the symbolic accessible-range agreement at program assembly
status: todo
priority: p0
dependencies: []
related: [decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary, replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n, package-the-admitted-live-schedule-into-a-symbolic-kernel-program]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, fail-closed, abi, shapes, implementation]
---
## User-visible outcome

A packaged symbolic program's published accessible byte range is checked against the boundary it claims to cover, instead of passing a check that compares zero against zero.

## Why this exists — filed 2026-08-23 from the live-bounds witness re-derivation

Filed by `worker-witness` while re-deriving [`decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary`](decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary.md) at `069873f4fff46f446bfece8e68cc12fa5a04cc93`. It is the one genuine correctness gap that re-derivation found, and it is deliberately **not** a schedule-vocabulary question, which is why it is a ticket of its own rather than an option in that packet.

**Fact — the agreement check is vacuous on a symbolic boundary, and the source says so.** `crates/tiler-ir/src/program/builder.rs`, anchor `agreement check is vacuous`: "it compares zero against zero". The mechanism is that `interface_extent_shape` maps a symbolic axis to a zero static extent (anchor `a symbolic axis occupies a zero static extent`), `static_facts` binds exactly those zeroed extents (anchor `Binds the bound semantic program's declared input extents as ABI facts.`), and `push_stage` then requires `evaluate_static_abi(accessible_bytes)` to equal `view.window.length` — both of which are 0.

**Fact — the compiler's own binding check collapses the same way.** `crates/tiler-compiler/src/program.rs` compares each stage access's evaluated `accessible_bytes` against `access.view().value().required_bytes()`, which is derived through `sourced_element_count` and is likewise 0 on a symbolic boundary.

**Fact — nothing recovers the check at decode.** `crates/tiler-artifact/src/program/codec/validate.rs` re-proves the *launch* geometry against static facts (`ArtifactBuildError::LaunchDisagreement`), but for a binding it only proves `accessible_bytes` is unsigned, interface-only, and phase-legal (anchor `AbiExprUse::AccessibleBytes`). No value agreement is re-derived on either the static or the symbolic path.

**Inference — what is actually unchecked.** The published expression is built by one authority, `crates/tiler-compiler/src/program.rs` anchor `fn declare_element_count(`, from the value's own `SourcedExtent`s with a `root_of` that refuses rather than invents. So a producer-side defect requires that one function to be wrong. What has no independent check at all is a **forged or hand-built** program whose `accessible_bytes` expression names the wrong `InputExtent` root, drops an axis, or scales by the wrong carrier width: the runtime will evaluate it against live facts (`crates/tiler-runtime/src/load.rs`, anchor `fn place_bindings`) and hand a backend a range that no layer proved corresponds to the access. On a static boundary the `push_stage` comparison is the check that catches this; on a symbolic one it is not present.

**Fact — the population is currently empty of *executing* subjects, which is why this is not p0 or p1.** No live-extent artifact reaches a backend at this base; [`prove-one-live-extent-artifact-payload-and-pipeline-at-two-n`](prove-one-live-extent-artifact-payload-and-pipeline-at-two-n.md) owns making one. This ticket should land before, or with, that one — a proof of the pipeline over an unchecked range would pin the gap rather than close it.

**Every Fact above must be re-audited at the worker's own base before any edit**, per `AGENTS.md`. The anchors were each run with `grep -cF` against the file the citation names on 2026-08-23.

## Exact-current Fact audit and selected repair — 2026-08-24, `6e713e12`

- **Verified, and stronger than the original priority claim.** IR program construction, compiler entry verification, and artifact construction all evaluate the producer-supplied expression over the same zero-narrowed symbolic facts they compare with a zero window. Artifact decode checks expression type, phase, and target names but never proves that the expression denotes the target's sourced boundary.
- **False — the population is no longer empty in the sense that determines correctness priority.** Symbolic artifact delivery is production-reachable, artifact identity authenticates the supplied expression rather than proving its meaning, and runtime `place_bindings` evaluates it over live facts. A forged undersized input expression can admit short caller storage, and a forged output/internal expression already controls the storage length requested from an adapter. The loader separately freezes and publishes the correct live extent parameter; no in-tree backend binds those parameter bytes yet, but [`bind-frozen-live-extent-bytes-at-declared-backend-transports`](bind-frozen-live-extent-bytes-at-declared-backend-transports.md) would make that independently correct extent drive indexing over the forged short allocation. This unchecked authority therefore gates the P0 execution path rather than already demonstrating backend execution.
- **Verified — no public, identity, or schema decision is required for the admitted slice.** The valid kernel/artifact bytes can remain unchanged. The IR can privately retain sourced interface extents for proof while keeping the existing zero `Shape` for static storage bookkeeping, and artifact validation can independently rederive the same contract from the already-carried interface, environment, target, component, carrier, encoding, and expression data.

One repair dominates. Retain each interface axis's `SourcedExtent` privately in the IR semantic subject. After output/component association is known, require each unpacked whole-interface symbolic binding's accessible range to be the exact canonical tree: its scalar width multiplied by the left-associated product of every axis, with static axes written as exact literals, input symbols naming that input's exact `(InputKey, Axis)`, output symbols resolved through the retained shape environment to the exact rooted input axis, and zero offset. Re-derive that contract independently in artifact construction/decode; do not call the compiler expression producer. Refuse symbolic internal storage, partial live windows, encoded components, bit-packed carriers, inconsistent output aliases, static targets depending on `InputExtent`, and unknown future source forms by typed cause until their authority is represented.

Artifact-only validation is dominated because it leaves a falsely verified kernel program available to other consumers. Making `ByteWindow` expression-valued is a larger public grammar and identity change with no correctness benefit for this slice. Blanket refusal needlessly withdraws the whole-interface population whose proof inputs are already present. Further research is not needed; implementation must stop only if its source audit finds a required public boundary or identity step that this audit did not.

## Required work

- Preserve the sourced semantic interface privately through IR program validation and prove the canonical whole-interface expression structurally rather than by evaluating sample or zero facts.
- Add an independently derived artifact validator shared by artifact construction/envelope/decode, while keeping the IR derivation separate so a producer bug cannot validate itself.
- Add typed disagreement and unsupported-population diagnostics without changing valid program/artifact bytes, kernel-program v13, artifact-program v22, or manifest 22.0.
- Correct the IR comment that treats runtime rebinding as a substitute for compile-time proof and narrow the codec's blanket claim that the agreement cannot be reproved.
- Use an exhaustive private proof disposition for every access/binding: static with no live root, canonical symbolic interface, or typed unsupported. Exhaustively match binding target, storage encoding, storage scalar, and sourced-extent forms so a widened vocabulary is a build error rather than a silent skip.

## Required evidence

Perturb the subject rather than the assertion, and quote the failure text for each independently: name a sibling input's root; substitute the wrong axis; drop one axis from the chain; scale by the wrong carrier width; and substitute a nonzero literal for one symbolic factor while another keeps the old zero comparison green. Each must be shown to pass before the repair and fail under the unchanged post-repair assertion. At least one undersized mutation must target an output to demonstrate allocation/write exposure. Start codec controls from a valid symbolic artifact, mutate only the expression, and reseal its digest and canonical identity; decode must still refuse it. Independently remove the IR validation call and the artifact validation call so each guard proves it reaches its subject. Preserve positive controls for correct symbolic input/output, static exact-byte bindings, and computed accessible bytes, plus typed refusals for internal, encoded-component, and bit-packed cases.

## Non-goals

The schedule-layer spelling of a live access's bounds obligation — that is [`decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary`](decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary.md)'s, and this ticket is independent of which option Tom accepts there. Executing a live-extent artifact, which is [`prove-one-live-extent-artifact-payload-and-pipeline-at-two-n`](prove-one-live-extent-artifact-payload-and-pipeline-at-two-n.md)'s.

## Closes when

A symbolic boundary's published accessible range is proven against the access it describes by some layer that can decide it, or the population is refused by name with the refusal's reachability demonstrated — and no forged expression can put a range in front of a backend that no layer checked.
