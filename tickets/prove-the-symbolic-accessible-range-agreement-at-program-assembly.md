---
id: prove-the-symbolic-accessible-range-agreement-at-program-assembly
title: Prove the symbolic accessible-range agreement at program assembly
status: todo
priority: p2
dependencies: []
related: [decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary, replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n, package-the-admitted-live-schedule-into-a-symbolic-kernel-program]
scopes: [implementation/ir, implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [research, correctness, fail-closed, abi, shapes]
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

## Required work — research first, then the narrowest change

Answer, with source read in full, which of these the repository wants, and take it or escalate one concrete question:

- **Structural agreement.** Require the `accessible_bytes` expression of a symbolic boundary to be structurally the canonical chain derived from the view's value's own sourced extents and carrier. Needs the kernel-program builder to hold the *sourced* boundary rather than the zeroed `Shape` it holds today — establish whether that is a builder input change or a subject change.
- **Symbolic window length.** Make `ByteWindow::length` ABI-expression-valued so the existing comparison becomes real rather than vacuous. This is a `tiler.kernel-program` grammar and identity question; if it survives the readiness gate it goes to Tom with its exact domain step.
- **Decode-side re-proof.** Re-derive the agreement at load from the decoded interface and the decoded expression arena, so a forged envelope fails closed independently of the producer.
- **Typed refusal.** Refuse a symbolic boundary whose `accessible_bytes` this layer cannot check, and record the population as unsupported until one of the above lands.

Do not present a spelling until the population, the forged-input threat model, and the identity consequence of each are established. If the answer is a `tiler.kernel-program` step, it is Tom's.

## Required evidence

Perturb the subject rather than the assertion, and quote the failure text for each independently: name the wrong `InputExtent` root; drop one axis from the chain; scale by the wrong carrier width; and substitute a literal for the symbolic factor. Each must fail with an unchanged assertion under whatever check lands, and each must be shown to **pass** today, which is the demonstration that the gap is real. Add the negative control that a wholly static boundary keeps its existing check and its existing bytes.

## Non-goals

The schedule-layer spelling of a live access's bounds obligation — that is [`decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary`](decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary.md)'s, and this ticket is independent of which option Tom accepts there. Executing a live-extent artifact, which is [`prove-one-live-extent-artifact-payload-and-pipeline-at-two-n`](prove-one-live-extent-artifact-payload-and-pipeline-at-two-n.md)'s.

## Closes when

A symbolic boundary's published accessible range is proven against the access it describes by some layer that can decide it, or the population is refused by name with the refusal's reachability demonstrated — and no forged expression can put a range in front of a backend that no layer checked.
