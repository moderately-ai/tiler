---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.delivered-numerical-realization-record"
kind: "research"
title: "The delivered numerical realization record"
topics: ["numerics", "artifacts", "provenance", "feasibility"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "adopted"
implementation_status: "implemented"
evidence_classes: ["executable-model", "exhaustive-finite"]
informs: ["tiler.contract.numerical-semantics", "tiler.contract.artifact-abi"]
adopted_by: ["ADR-0076"]
ticket: "redesign-the-delivered-realization-record-from-typed-evidence"
---

# The delivered numerical realization record

**Status:** research complete and adopted; ADR 0076 item 4 carries the wording this record's boundary establishes, and all four proposed layers are in production.

**Why this record exists, and why it was written after the work it describes.** The experiment behind it — [the delivered-realization design packet](../../../spikes/numerics/delivered-realization-record/README.md) — was built and its design was adopted without a research record ever stating the bounded universe its claims hold over. The packet's `supports` edge named two contracts instead, which [the metadata contract](../../document-metadata.md) types as invalid: `supports` runs experiment to research. That mistyped edge is the symptom this record repairs, and the repair is not a re-pointed link because no existing record stated the boundary. Extracted by [`repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row`](../../../tickets/repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row.md); the research it states is [`redesign-the-delivered-realization-record-from-typed-evidence`](../../../tickets/redesign-the-delivered-realization-record-from-typed-evidence.md)'s.

The absence was itself the finding, and it is the exact absence the metadata contract predicts. Its section *A decision does not cite an experiment in metadata* refuses the analogous relaxation on the ground that a document able to name a harness directly "is never pushed to say what bounded universe, environment, and procedure make the measurement carry the weight put on it — naming that boundary being the research record's job." An experiment pointed straight at two contracts skipped that step in the same way and for the same reason.

## Outcome

A produced artifact must carry the numerical realization it delivered as a readable typed record keyed by the compiler-produced scalar-arithmetic policy subject, never inferred from compiler flags, target names, neighbouring dtypes, profile digests, or outer value shape. Four separated layers carry it, and the separation is the result rather than an implementation convenience:

1. one shared dimension and provenance vocabulary, so the dimension set has a single authority;
2. a borrowed compiler evidence view over a checked plan, so no consumer can forge a compiler-verified fact;
3. a required artifact record with its own canonical codec and failure vocabulary; and
4. one build-side translation, in the only crate that can see both the compiler's and the artifact's authorities.

The record states, per subject, the resolved contract complete over every governed dimension, and for each dimension an explicit **assessment disposition**: compiler-produced `NotRequired` for every packaged route, or `Required` naming a non-empty canonical range of locus-specific obligations. A dimension no packaged route consumes carries no target fact at all rather than a fabricated one.

## Evidence boundary

**This is the boundary that makes the evidence classes honest, and it is narrow in one direction that matters.** The packet's two-dtype fixture uses *checked synthetic* evidence. It proves a property of the **record**, not of any measured target. The divergence it exhibits — `f32` flushing and `f16` preserving input subnormals — is real in `tiler-metal`'s `MetalSubnormalArithmeticFacts`, but no target profile in this tree declares it: every `ScalarHonourabilityDeclaration` in the workspace is over governed `f32`, or over `f32` and `bf16` in `crates/tiler-build/src/metal_declaration.rs`, which states that F16 is deliberately absent. Nothing here is a measurement of Apple hardware, and no claim in it generalizes to one. [Apple GPU numerical behaviour](../apple-targets/numerical-behaviour.md) and [the first macOS Metal compile-profile authority ledger](../target-profiles/first-macos-metal-compile-profile-authority-ledger.md) are where measured target facts live.

`exhaustive-finite` is claimed over three explicitly named finite universes and nothing wider: the eleven governed numerical dimensions, the twenty-five distinct rule identifiers the two proposed error vocabularies define, and the thirty recognized `(type, arithmetic)` subject pairs the production validator was asked to judge. `executable-model` is claimed because the packet compiles against the real production vocabulary rather than a mock of it; it was never the production implementation, and the wiring that became one is separate work.

**Measurement — re-run at `fac629a7` on 2026-08-05, macOS 27.0.0, from `spikes/numerics/delivered-realization-record` with `CARGO_TARGET_DIR=./target cargo run`.** All ten stages pass and the process exits zero. The stage counters it prints are its own: 28 recognized `(type, arithmetic)` pairs refused and 2 governed pairs admitted; 11 dimensions covered with 3 required and 8 `NotRequired`, carrying 3 distinct loci on one `(type, dimension)`; 2 subjects and 2 evidence rows in one two-dtype record with no collision; two declaration orders producing identical bytes over 4 obligations citing 2 deduplicated evidence rows; a zero-obligation subject written explicitly across 258 canonical bytes; 3 distinct resolved-type identity families; a 1498-byte exact round trip whose 8 overlapping resolutions agree with the entry; and **38 perturbations tripping 25 distinct rules**, which is every rule in both `ALL_RULES` inventories.

Each perturbation asserts the exact rule identifier rather than merely that something failed, and the harness counts its coverage against the named inventories, so a rule added without a perturbation fails the run instead of quietly shrinking what has been watched. That property is what lets the rule count above be read as coverage rather than as a tally.

## Facts

- **The dimension set had three disagreeing authorities before this work.** The staged artifact draft declared four cases, `tiler_ir::schedule::NumericalRealization` carried eight, and the compiler's honourability authority carried eleven. The draft's own doc comment asserted the four were what the schedule realization carries, which was false in the direction that made the draft look complete.
- **Honourability is not keyed by dimension alone.** `NumericalRequirement::subject` resolves to `(NumericalDimension, ArithmeticType, resolved-type bytes)`, and feasibility filters on all of dimension, arithmetic, resolved type, and behaviour. A dtype-free `honoured(dimension)` cannot return one correct answer.
- **`HonouringMeans`'s presentation key is not injective.** Every `SupportedOnlyUnderDeclaredRelaxation` value returns one constant string whatever relaxation it names, while the encoder does write the relaxation. A record carrying that key could not distinguish two artifacts honouring one contract under different relaxations — which is the question the record exists to answer.
- **A dtype-wide ceiling cannot express two loci with different legal requirements.** ADR 0011's per-operation restrictions attach to a position, so one `f32` operation's accumulator and its observable materialization boundary can carry different legal requirements, and a record keyed by type alone keeps whichever was written last.

## Inferences

1. The means must be carried structurally with its relaxation payload, and the non-injective spelling demoted to a presentation label. This follows from the non-injectivity fact alone and does not depend on any measurement.
2. The obligation key must carry a policy locus above the dtype-wide ceiling, and the ceiling and the obligations must remain separate statements with neither derived from the other.
3. Dispositions must be derived at build time from the canonical obligation slice rather than declared, because two declared copies of the same claim can disagree, and a derived `Required` range is contiguous by construction and cannot name a row that is not there.
4. The record must be required rather than optional. An `Option` or an `UnrecordedRealization` reader is migration state, and a terminal required record contradicts it.

## The honest limit of what the artifact can prove

**An untrusted producer can write a wholly self-consistent record, including a false `NotRequired`, and every artifact-side check passes.** The artifact builder validates internal consistency, canonical order, references, coverage ranges, tags, and provenance completeness; it cannot supply authenticity and cannot re-run the compiler's consumption analysis. Decode verifies integrity and associations; it does not upgrade producer assertions into independently proved semantics. Ordinary checked production goes through `tiler-build`, and any retained low-level seam accepts typed producer assertions and must be named as such. Recording this limit is part of the result, because a reader who mistakes decode success for authenticity draws exactly the wrong conclusion from a passing artifact.

## What is implemented, and the producer gap that is not

`implementation_status` is `implemented` because every layer this record decides exists in production: `crates/tiler-ir/src/numerics.rs`, `crates/tiler-compiler/src/session/realization.rs`, `crates/tiler-artifact/src/program/realization.rs` with its `codec.rs`, and `crates/tiler-build/src/realization.rs`. [`wire-the-delivered-realization-record-into-the-artifact`](../../../tickets/wire-the-delivered-realization-record-into-the-artifact.md) landed them on 2026-08-05, and ADR 0076's status line records item 4 as implemented on that date.

**Do not read that as loci being populated.** The record's shape admits locus-keyed obligations and the packet exhibits three distinct loci on one `(type, dimension)`, but the compiler cannot yet *derive* them: `StrictF32NumericalContract` is one flat record for one arithmetic type, projected into whole-program requirements. Until that changes, a conforming producer emits one obligation per consumable dimension at the computation locus of the occurrence that consumes it, which is as much as the compiler can honestly say. [`derive-per-locus-numerical-obligations`](../../../tickets/derive-per-locus-numerical-obligations.md) owns the remainder and is open. Separately, [`key-numerical-requirements-by-the-contract-s-own-resolved-type`](../../../tickets/key-numerical-requirements-by-the-contract-s-own-resolved-type.md) owns a hard-coded `f32` resolved type in requirement construction; it fails closed to `Unknown` rather than returning a wrong answer, but it means no non-`f32` contract can be honoured whatever a profile declares.

One question this record does not answer: the cost of subject lookup. The artifact binary-searches the subject slice, and at today's one-subject scale a linear scan would probably win. That belongs with a real multi-subject portfolio and is recorded here as unmeasured rather than settled by assertion.

## Traceability

Adopted by [ADR 0076 item 4](../../decisions/0076-declare-target-honourable-numerical-realizations.md), whose refined wording is the packet's drafted replacement text landed by the wiring ticket. It informs [numerical semantics](../../numerical-semantics.md) on how the delivered record refines the honourability key by a policy locus, and [the artifact ABI](../../artifact-abi.md) on how the record sits beside an entry's own numerical facts, overlaps them on eight dimensions, and is cross-checked against them. Reproduced by [the delivered-realization design packet](../../../spikes/numerics/delivered-realization-record/README.md).
