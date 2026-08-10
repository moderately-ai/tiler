---
id: redesign-the-delivered-realization-record-from-typed-evidence
title: Redesign the delivered-realization record from typed evidence
status: done
priority: p1
dependencies: [carry-the-honourability-fact-provenance-into-the-artifact-record, express-metal-honourability-in-the-shared-form]
related: [record-delivered-numerical-realization, drive-the-build-orchestrator-from-a-checked-compiler-plan, widen-the-region-realization-to-consumable-dimensions, accept-the-delivered-realization-artifact-surface, wire-the-delivered-realization-record-into-the-artifact, derive-per-locus-numerical-obligations, key-numerical-requirements-by-the-contract-s-own-resolved-type]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [design, artifact, numerics, provenance, api]
---
## User-visible outcome

Tom receives a compile-checked private, review-ready design packet for a required delivered-realization record. The draft record family demonstrates representation of the compiler-produced scalar-arithmetic policy subject completely over its eleven governed dimensions, preserves canonical full resolved-type identity and structured provenance, and reserves a versioned seam for future operation- or scheme-owned contract families without manufacturing scalar-arithmetic claims for integers, booleans, quantized values, complex values, decimal values, MX values, or owner-namespaced types. Production `crates/tiler-artifact/src/program/realization.rs` remains untouched until acceptance and downstream wiring.

## Why the staged draft must be replaced

**Correction — 2026-08-10.** The Facts below are the filing-time problem statement about the staged draft at base `6544d4f` (Outcome pins each defect there). They are **not** live claims about production. Downstream [`accept-the-delivered-realization-artifact-surface`](accept-the-delivered-realization-artifact-surface.md) and [`wire-the-delivered-realization-record-into-the-artifact`](wire-the-delivered-realization-record-into-the-artifact.md) (both `status: done`) landed the redesigned production surface: shared eleven-dimension `NumericalDimension` in `tiler_ir::numerics`, typed `declare_scalar_arithmetic` / `require`, structured provenance, and domain `delivered-realization.v2`. Reproduce: `rg -n "declare_scalar_arithmetic|delivered-realization.v2" crates/tiler-artifact/src/program/realization.rs`; `rg -n "DIMENSION_COUNT|pub enum PolicyLocus" crates/tiler-ir/src/numerics.rs`.

- **~~Fact~~ — historical at `6544d4f` (struck as a live claim 2026-08-10):** `crates/tiler-artifact/src/program/realization.rs` then declared a second `NumericalDimension` with four cases and fixed fields. The compiler authority had eleven cases, while the widened scheduled realization and artifact codec carried the eight dimensions then consumable by admitted operations.
- **~~Fact~~ — historical at filing (struck as a live claim 2026-08-10):** compiler honourability was keyed by `(NumericalDimension, ArithmeticType)`. The measured Apple profile preserves `f16` input subnormals and flushes `f32` input subnormals, so `honoured(dimension)` cannot return one correct answer. Outcome already upgrades the key to complete resolved type and bounds the two-dtype fixture: no in-tree profile declares an `f16` honourability row (`F16 is deliberately absent` in `crates/tiler-build/src/metal_declaration.rs`).
- **~~Fact~~ — historical at `6544d4f` (struck as a live claim 2026-08-10):** `HonouringMeans::SupportedOnlyUnderDeclaredRelaxation` carries a specific relaxation subject, but the then-named `HonouringMeans::key()` collapsed every such value to the same text. Opaque key bytes therefore cannot tell a reader which relaxation made a requirement honourable. **Correction — 2026-08-10.** Production renames that non-injective presentation API to `HonouringMeans::label`; identity and wire carry the relaxation via `encode` / `canonical_key`. Reproduce: `rg -n "fn label|fn encode|fn canonical_key" crates/tiler-ir/src/numerics.rs` on the `HonouringMeans` impl.
- **~~Fact~~ — historical at `6544d4f` (struck as a live claim 2026-08-10):** ADR 0076 requires availability phase, authority, validity scope, declaring profile, compiler build, and execution environment. The staged draft carried only phase and one record-level profile. Production provenance types (`FactSourceProvenance`, …) live in shared `tiler_ir::numerics`.
- **~~Fact~~ — historical at `6544d4f` (struck as a live claim 2026-08-10):** `DeliveredRealizationBuilder::declare` then accepted arbitrary bytes and a caller-selected phase, validating framing not that a checked compiler plan selected the claim. Production API is `declare_scalar_arithmetic` / `require` with typed subjects; ordinary production path is `tiler_build::realization::translate` from `DeliveredRealizationView`.
- **Inference (filing-time design consequence; not a live inventory claim):** publishing the staged draft would have frozen an incomplete and already-contradicted contract. The dtype key, structured means, provenance, dimension authority, constructor, readers, canonical encoding, and identity all required a coherent replacement — the packet and later accept/wire landings.

## Implementation keys

### Compiler-produced policy subjects, never inferred applicability

Define one shared, exhaustive numerical-dimension authority for the eleven governed scalar-arithmetic dimensions. The dimension is numerical-contract vocabulary; the target-aware means of honouring it remains outside semantic IR. Remove the artifact-local and compiler-local lists as independent authorities, and keep exhaustive tag mappings so a new dimension breaks every total encoder and consumer at compile time.

Key the implemented contract by a checked compiler-produced numerical-policy subject. Its first and only implemented subject kind is `ScalarArithmetic`: it carries the canonical full `ResolvedValueType` identity of one dtype-wide arithmetic contract plus the eleven-dimension schema. `TypeKey` alone is insufficient because parameters and ordered encoded components distinguish resolved types within one definition family. The subject cannot be inferred by scanning graph value types, recursively walking encoded components, or guessing from storage width.

Keep operation-specific requirements distinct from that dtype-wide ceiling. A canonical `NumericalObligationKey` identifies the program occurrence and policy locus—input, computation, accumulator, result, component, or materialization—that produced the requirement. One `(subject, dimension)` disposition is therefore `NotRequired` or references a non-empty canonical range of obligation rows, not one evidence row. Each obligation row carries its own required behaviour and evidence reference, so two f32 loci with different legal requirements never collapse merely because they share a type.

Reserve a versioned/tagged record-family seam rather than a universal dtype enum. A future integer, boolean, complex, decimal, quantized, MX, conversion, or owner-defined contract family arrives only with its first producer, consumer, behaviour schema, validation, identity, and lowering evidence. Subject existence follows the checked request's selected governed contract, not the number of obligations: a selected scalar-arithmetic contract always produces one complete subject, even when every dimension is `NotRequired`. A recognized semantic type not governed by any selected scalar contract does not create an additional subject merely by appearing in the program.

### Complete contract and complete assessment coverage

For every scalar-arithmetic subject emitted by the checked compiler plan, carry the resolved contract complete over all eleven dimensions. Separately carry the canonical union of honourability obligations relied upon by every packaged executable variant and stage that routing may select. Never use runtime language such as “actually exercised”: the artifact exists before a route executes.

Every dimension has an explicit assessment disposition: either compiler-produced `NotRequired` for every packaged route or `Required` with a canonical non-empty obligation range. This refines ADR 0076 item 4's earlier “each dimension's means” wording without turning an unconsumed dimension into a fabricated target fact. The review packet includes exact proposed ADR and numerical/artifact contract text; it does not make that proposal operative before ratification.

Each obligation row carries the exact policy subject, dimension, locus key, required behaviour, and evidence reference. Each referenced evidence row carries the declared target behaviour, structured honouring means including any declared-relaxation payload, availability phase, measured-fact authority, validity scope, declaring profile, compiler-build identity, and execution-environment identity. The target/profile declarer produces the fact; the compiler validates and carries the selected evidence rather than becoming its authority.

### Compact canonical representation

Store a canonical sorted slice of versioned subject records. The `ScalarArithmetic` record stores the resolved type identity once and its eleven resolutions and assessment dispositions in dense dimension-indexed arrays whose index conversion is one exhaustive shared match. Store obligation rows as a canonical sorted sparse slice keyed by exact subject, dimension, and locus key; store deduplicated target evidence separately and reference it canonically.

The typed producer builder accepts declarations in arbitrary order, validates them, sorts once, and rejects duplicates. Canonical decode rejects out-of-order or duplicate wire rows; producer call sites do not have to reproduce wire ordering. Decoded views borrow contiguous rows without per-read allocation. Lookup remains allocation-free; choose linear, partitioned, or binary sparse lookup from a targeted benchmark rather than assuming asymptotic performance dominates at the current one-subject scale.

### Honest producer and trust boundary

Specify a borrowed, typed compiler view of the resolved subjects, assessment coverage, and selected evidence. Specify one exhaustive `tiler-build` translation into the artifact representation. The translation matches every subject, dimension, disposition, structured means, and provenance variant; it never reconstructs evidence from flags, target names, neighbouring dtypes, profile digests, or outer value shape.

The artifact builder validates internal consistency and provenance but cannot provide authenticity or re-run compiler consumption analysis: an untrusted producer can write a self-consistent assertion, including a false `NotRequired`. Decode verifies integrity, canonical coverage, references, and associations; it does not upgrade producer assertions into independently proved semantics. Document that boundary. Ordinary checked production consumes the compiler-selected obligations and evidence through `tiler-build`; any retained low-level construction seam accepts typed producer assertions and is named accordingly.

The design must state every cross-check and its proving layer. The compiler proves the policy subject, obligation loci, required behaviours, and `NotRequired` claims from the checked plan. `tiler-build` proves the translated subject and obligation references agree with that compiler view. Artifact construction and decode prove the record's profile equals the artifact subject, every packaged entry/variant references an existing policy subject, and the record's eight overlapping scalar resolutions equal every entry's existing widened `NumericalFacts`; a mismatch rejects. Where the neutral artifact cannot independently derive the arithmetic type from its dispatch record, it validates the encoded association and documents that the compiler/build producer proved its semantic meaning.

Unknown record-family, family-schema, subject-kind, dimension, disposition, means, provenance, or locus tags reject fail-closed. Extensibility means a newer reader implements and validates a new family; an older reader never skips an unknown numerical family while still calling the executable artifact validated.

### Review workflow without crossing the boundary

This ticket owns a compile-checked design packet or bounded spike with exact proposed public signatures, canonical ordering, validation rules, failure vocabulary, and representative call sites. It does not promote production visibility, wire the production builder/decoder, advance schema or identity domains, or rebaseline production pins.

Place the private draft under `spikes/numerics/delivered-realization-record/` with its own README and exact invocation. The spike must compile against the repository toolchain, contain the adversarial fixtures below, and document one deliberate validation mutation that was observed failing. It is a design artifact, not production support.

`accept-the-delivered-realization-artifact-surface` reviews the exact proposed IR/compiler/artifact/build boundary after this ticket is done. `wire-the-delivered-realization-record-into-the-artifact` implements the ratified boundary, required terminal state, compiler-to-build-to-artifact path, codec, readers, identity/schema changes, and merged-tree rebaselines.

## Required evidence

- One scalar-arithmetic subject fixture covers all eleven resolved dimensions, `NotRequired`, one required obligation, and multiple distinct required loci on one `(type, dimension)`.
- One profile carries different `f16` and `f32` subnormal evidence without collision.
- Two declared-relaxation means differing only in relaxation payload remain distinct.
- The producer builder accepts shuffled declarations and canonicalizes them; duplicate declarations reject.
- Canonical decode rejects shuffled, duplicate, missing, malformed, profile-mismatched, overlapping-realization-mismatched, dangling-obligation/evidence, unknown-tag, behaviour-mismatched, and incomplete-provenance rows with typed causes.
- Recognized bool, integer, complex, decimal, strict-affine encoded, and MX values do not manufacture additional scalar-arithmetic subjects merely by appearing in a program.
- A selected scalar contract with zero obligations still produces a complete eleven-dimension subject whose dispositions are all `NotRequired`; an unsupported owner-namespaced type cannot create another subject except through checked producer evidence.
- The identity codec distinguishes nominal, parameterized, and encoded full resolved-type identities without claiming those types inhabit the scalar-arithmetic schema.
- Every proposed validation check is perturbed once and observed failing before restoration.
- The design/spike's exact invocation passes together with `tkt lint` and `git diff --check`.

## Closes when

The review packet eliminates the stale draft, defines the exact signatures and call sites across the shared dimension, compiler evidence view, artifact record, and build translation, includes the exact proposed ADR 0076 and numerical/artifact contract corrections, demonstrates the compact representation and adversarial validation in a compile-checked bounded spike or equivalent private draft, and leaves applying those contract changes plus every production public item and wire/schema/identity change for acceptance and downstream implementation.

## Outcome — the packet exists and is reviewable

The compile-checked private design packet is `spikes/numerics/delivered-realization-record/`, its own workspace carrying no `rust-toolchain.toml` of its own. Exact invocation, from that directory: `CARGO_TARGET_DIR=./target cargo run`. Ten stages, ending in **38 perturbations covering all 25 distinct rule identifiers** the two proposed error vocabularies define; `cargo clippy` and `cargo fmt --check` are clean.

**Production is untouched.** `git diff --name-only <base>..HEAD` reaches nothing under `crates/`; `crates/tiler-artifact/src/program/realization.rs` is byte-identical. **Correction — 2026-08-10.** That claim is ticket-local (this redesign delivered only the spike packet). It is not a tree-global reading: [`accept-the-delivered-realization-artifact-surface`](accept-the-delivered-realization-artifact-surface.md) and [`wire-the-delivered-realization-record-into-the-artifact`](wire-the-delivered-realization-record-into-the-artifact.md) both `status: done` and landed the redesigned production surface.

### The five cited defects, each verified at source

All hold at base `6544d4f`. The packet README carries the line-level derivation:

1. The artifact's second `NumericalDimension` has four cases against the compiler's eleven — **and** its doc comment claims those four are what `NumericalRealization` carries, which has been **eight** since `widen-the-region-realization-to-consumable-dimensions`. The comment is false in the direction that makes the draft look complete.
2. Honourability is keyed by more than `(dimension, arithmetic)`: the **complete resolved type** is the third coordinate (`NumericalRequirement::subject`), and `tiler_compiler::target::ScalarArithmetic` is already the public validated subject.
3. `HonouringMeans::key` collapses every declared relaxation to one constant string while `encode` carries the payload. This is the decisive defect: opaque key bytes cannot carry the record even for comparison, which is what reopens the elimination `record-delivered-numerical-realization` recorded. **Correction — 2026-08-10.** Production presents that non-injective string as `HonouringMeans::label`; structured identity is `encode` / `canonical_key` (no `fn key` on `HonouringMeans` in `tiler_ir::numerics`).
4. Authority, validity scope, compiler build, and execution environment are absent; the vocabulary now exists in `tiler-compiler` and is `pub(crate)`.
5. `declare` validates framing, not that a checked plan selected the claim.

### One correction that bounds a fixture's claim

The ticket says "the measured Apple profile preserves `f16` input subnormals and flushes `f32`". The divergence is measured and real, but **no target profile in this tree declares an `f16` honourability row** — it lives in `tiler-metal`'s `MetalSubnormalArithmeticFacts`, every `ScalarHonourabilityDeclaration` is over `f32` or `bf16`, and `crates/tiler-build/src/metal_declaration.rs:616-620` states F16 is deliberately absent. The two-dtype fixture therefore uses checked synthetic evidence — which `express-metal-honourability-in-the-shared-form` explicitly admits for this ticket — and proves a property of the **record**, not of any measured target. Its README says so where the fixture is defined.

### Two defects found while verifying, filed rather than absorbed

- [`derive-per-locus-numerical-obligations`](derive-per-locus-numerical-obligations.md) — at filing, the compiler had no locus vocabulary at all (`grep -rni "locus" --include="*.rs" crates/` was empty), so it could not yet produce the obligation rows the record is shaped to carry. The shape is derived from ADR 0011 and was not blocked by this; the producer was, and a single-locus producer was admissible meanwhile. **Correction — 2026-08-10.** That ticket is `status: done`; `PolicyLocus` is public in `tiler_ir::numerics`. Reproduce: `rg -n "pub enum PolicyLocus" crates/tiler-ir/src/numerics.rs`.
- [`key-numerical-requirements-by-the-contract-s-own-resolved-type`](key-numerical-requirements-by-the-contract-s-own-resolved-type.md) — at filing, `policy::dimension_requirements` hard-coded `F32::resolved_type()` while reading `contract.arithmetic`, so no non-`f32` contract could ever be honoured. It failed closed, so it was a structural refusal rather than a wrong answer. **Correction — 2026-08-10.** That ticket is `status: closed` (defect already fixed); current `dimension_requirements` derives the subject from `contract.arithmetic` and uses `subject.resolved_type().clone()`.

### Boundaries held

No production edit, no identity or schema advance, no pinned rebaseline, no contract edit — the proposed ADR 0076, numerical-semantics, and artifact-abi text is drafted **inside** the packet. The packet is a proposal and is not accepted by having compiled.

### Remainder

~~The top-level `spikes/README.md` catalog entry is deliberately not made: that file maps to `contracts/navigation`, which the live ticket `cite-adr-0095-in-the-milestone-6-distributivity-framing` holds.~~ **Correction — 2026-08-10.** The catalog entry now exists (`spikes/README.md` links `numerics/delivered-realization-record/README.md`); [`cite-adr-0095-in-the-milestone-6-distributivity-framing`](cite-adr-0095-in-the-milestone-6-distributivity-framing.md) is `status: done` — this remainder was delivered outside this ticket. `spikes/numerics/README.md` — inside this ticket's `research/numerics` scope — lists the packet and also corrects a pre-existing omission of the BF16 spike from that portal.

## Graph maintenance

- `research/numerics` was added to this ticket's scopes: the packet lands under `spikes/numerics/**`, which `ticketsplease.toml` maps to that scope. Declaration and scheduling metadata for already-authorized work, not a product-scope expansion.
- **Scope-overlap verification.** `tkt guard` reports `severity: warn`, `conflict: false`, `under_declared: []`. Two tickets share `research/numerics` directly. `define-the-runtime-kv-state-boundary` is `closed`, and `git diff --name-only main...tkt/define-the-runtime-kv-state-boundary` touches only `docs/` and `tickets/` — file-level disjoint from `spikes/numerics/**`. `preserve-an-mlir-linalg-dialect-source-in-the-primary-source-record` is `in-progress` with **no `tkt/` branch**, so there is no diff to verify against; the check that can be reproduced is `git rev-parse --verify tkt/preserve-an-mlir-linalg-dialect-source-in-the-primary-source-record`, which fails. Every remaining collision is a `shared` claim on `project/tickets`, which every claimed ticket declares. `contracts/navigation` was deliberately not touched — see the remainder above.
- This ticket follows both structured provenance and `express-metal-honourability-in-the-shared-form`; the review fixture cannot claim compiler-selected Metal evidence until the producer path exists — and it does not: it says in its own README that its evidence is checked synthetic.
- `accept-the-delivered-realization-artifact-surface` depends on this review packet and owns Tom's exact public-boundary ratification.
- `wire-the-delivered-realization-record-into-the-artifact` follows acceptance and owns all production integration.
- Qualify `record-delivered-numerical-realization` as historical evidence: its staged four-dimension outcome was valid for its tree and is not the current candidate. **Done 2026-08-05** — its `## Outcome` now carries a confirmed-historical note re-verifying every qualifying claim at source, plus the two corrections to that ticket's own text (the false "four dimensions `NumericalRealization` carries" comment, and the overstated "measured `f16`/`f32` divergence" that no profile declares).
- `derive-per-locus-numerical-obligations` and `key-numerical-requirements-by-the-contract-s-own-resolved-type` were filed from this work; neither blocks acceptance.
- Keep the caller-profile declaration and checked Metal adapter visible as explicit upstream gates rather than relying on prose or a reversed provenance edge.
- Leave removing the stale four-dimension production module, applying accepted contract text, advancing schema/identity domains, and recomputing production goldens to `wire-the-delivered-realization-record-into-the-artifact` after public acceptance. **Correction — 2026-08-10.** Accept and wire both `status: done`; that production integration is no longer open work owned by this ticket.
