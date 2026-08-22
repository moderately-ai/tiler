---
id: package-selected-physical-implementation-provenance-in-artifact-identity
title: Package selected physical implementation provenance in artifact identity
status: in-progress
priority: p1
dependencies: [disclose-the-physical-provider-environment-a-compilation-was-offered, publish-occurrence-bound-selected-physical-implementation-evidence, replace-flat-selected-lowering-capability-keys-with-structured-subjects, decide-the-artifact-physical-selection-provenance-surface]
related: [disclose-offered-and-selected-physical-provider-sets-separately, reconcile-the-operation-identity-and-governed-key-grammars]
scopes: [implementation/artifact, implementation/build, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [backend-providers, provenance, artifact, identity, schema, public-boundary]
claimed_from: todo
assignee: worker-provenance
lease_expires_at: 1787429727
---
## Exact-base Fact audit and stop — 2026-08-16

Audited at exact base `4e02be6f4aed72209bb15019c43c247abf530e17`
before any production edit. Read in full: repository `AGENTS.md`; this ticket;
all three original dependencies; accepted ADRs 0072, 0075, and 0090;
`docs/artifact-abi.md`; and the relevant artifact/build/compiler construction,
validation, identity, envelope, codec, decode, equality, limit, refusal, test,
and production-assembly paths listed in
[`decide-the-artifact-physical-selection-provenance-surface`](decide-the-artifact-physical-selection-provenance-surface.md)'s
exact-base audit.

1. **Verified.** `CompilationEnvironment` has one `available` provider set,
   one-argument `new(providers)`, `available()`, and private `offers()`.
   File: `crates/tiler-artifact/src/program/builder.rs`. Anchors:
   `pub struct CompilationEnvironment`, `pub fn available`, `fn offers`.
2. **Verified.** Production `assemble_plan_artifact` constructs that set only
   from `compilation.offered_lowering_providers()` and forwards only
   `plan.selected_capabilities()` through `select_provider`. File:
   `crates/tiler-build/src/plan_artifact.rs`. Anchors:
   `CompilationEnvironment::new(` and `for selected in
   plan.selected_capabilities()`.
3. **Verified.** Compiler exposes the physical offered and selected halves
   separately through `Compilation::offered_physical_providers()` and
   `PlanAlternative::selected_physical_providers()`. File:
   `crates/tiler-compiler/src/session.rs`. Anchors are the accessor symbols.
4. **Verified.** `SelectedImplementation` exposes the whole occurrence
   identity, whole implementation-proposal identity, provider, and stable
   proposal-kind code, while remaining compiler-constructed. File:
   `crates/tiler-compiler/src/session.rs`. Anchors:
   `pub struct SelectedImplementation`, `region_occurrence_identity`,
   `implementation_proposal_identity`, and `proposal_kind`.
5. **Verified.** `assemble_plan` proves one selection per occurrence and sorts
   the retained run by whole occurrence bytes. File:
   `crates/tiler-compiler/src/selection.rs`. Anchors: `fn assemble_plan`,
   `duplicate-selection`, and `ordered.sort_by`.
6. **Verified.** Artifact `VariantSpec`, `VariantData`, `VariantRow`,
   `VariantRef`, and `DecodedVariant` carry no physical-selection run; the only
   packaged provider row is artifact-global lowering `SelectedProvider`.
   Files: artifact `builder.rs`, `model.rs`, and codec `model.rs`/`view.rs`.
   Anchors are those six type names.
7. **Verified.** Live owners are `tiler.artifact-program.v18`, lowering
   provider key `tiler.artifact-program.provider.v3`, and manifest schema
   18.0. Files: artifact `model.rs` and codec `encode.rs`. Anchors:
   `ARTIFACT_DOMAIN`, `PROVIDER_KEY_DOMAIN`, and `MANIFEST_SCHEMA`.
8. **Imprecise — repaired by a prerequisite.** The Required delivery fixes the
   semantic row but did not fix the consequential public Rust record,
   constructor/accessor/error surface, insertion topology, limits, or wire
   tags. The existing public artifact vocabulary cannot carry the row across
   the `tiler-build` crate boundary. Several valid spellings existed, so ADR
   0075 forbids choosing one as implementation detail.
9. **Imprecise packet limit — repaired before presentation.** The current
   compiler path proves selected physical rows no more numerous than its at
   most twelve scheduled stages, but artifact `push_variant` independently
   supports direct verified programs with up to 4,096 stages and requires
   `entries.len() == stages.len()`; shared IR's `MAX_PROGRAM_STAGES` is also
   4,096. A fixed twelve-row limit without
   `selected rows <= entries` both admitted a one-entry/twelve-row statement
   outside the accepted association and rejected a structurally consistent
   direct thirteen-entry/thirteen-row artifact. The decision prerequisite now
   derives `MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS` from
   `MAX_VARIANT_ENTRIES` and requires the relational cardinality rule at build,
   encode, and decode. Files: artifact `builder.rs`, `mod.rs`, codec `budget.rs`
   and `decode.rs`, plus shared-IR `program/mod.rs`. Anchors:
   `if stages != spec.entries.len()`,
   `limit(stages, MAX_VARIANT_ENTRIES`, `pub const MAX_VARIANT_ENTRIES`,
   `pub const MAX_PROGRAM_STAGES`, and `cursor.vec(MAX_VARIANT_ENTRIES`.
10. **False packet failure — repaired before presentation.** The envelope
    decoder bounds `manifest_bytes` by the existing 64 MiB
    `MAX_MANIFEST_BYTES` before `parse_manifest`; every physical-selection row
    and the complete physical run are strict framed subsets of that admitted
    manifest. A decoder-specific per-identity or aggregate physical-byte limit
    would therefore expose an error no input can reach. Direct construction can
    retain caller-owned rows before a manifest exists, so the prerequisite
    instead specifies an exact canonical physical-subset lower-bound check
    against the existing `MAX_ARTIFACT_IDENTITY_BYTES`, with no new byte budget,
    and leaves decode governed by the existing whole-manifest admission. Files:
    artifact codec `decode.rs`/`encode.rs`, artifact `model.rs`, and
    `builder.rs`. Anchors: `let manifest_bytes =
    cursor.count(MAX_MANIFEST_BYTES`, `pub(super) const MAX_MANIFEST_BYTES`,
    `fn encode_identity`, and `if bytes.len() >
    MAX_ARTIFACT_IDENTITY_BYTES`.

Reproduce the live anchors and bounded migration population:

```sh
git merge-base HEAD 4e02be6f4aed72209bb15019c43c247abf530e17
git diff --quiet 4e02be6f4aed72209bb15019c43c247abf530e17..HEAD -- crates prototypes spikes
rg -n 'pub struct SelectedImplementation|region_occurrence_identity|implementation_proposal_identity|proposal_kind\(' crates/tiler-compiler/src/session.rs
rg -n 'CompilationEnvironment::new\(|select_provider\(|selected_physical_providers\(' crates/tiler-build/src/plan_artifact.rs crates/tiler-artifact/src/program/builder.rs crates/tiler-compiler/src/session.rs
rg -n '^\s*VariantSpec \{' crates prototypes spikes --glob '*.rs'
rg -n 'VariantSpec \{' crates --glob '*.rs'
rg -n '\.push_variant\(' crates prototypes spikes --glob '*.rs'
rg -n 'CompilationEnvironment::new\(' crates prototypes spikes --glob '*.rs' | wc -l
rg -l 'CompilationEnvironment::new\(' crates prototypes spikes --glob '*.rs' | wc -l
rg -n 'pub fn available\(|\.available\(\)' crates/tiler-artifact crates/tiler-build prototypes spikes --glob '*.rs'
rg -n 'if stages != spec\.entries\.len\(\)|limit\(stages, MAX_VARIANT_ENTRIES|pub const MAX_VARIANT_ENTRIES|cursor\.vec\(MAX_VARIANT_ENTRIES' crates/tiler-artifact/src/program/{builder.rs,mod.rs,codec/decode.rs}
rg -n 'pub const MAX_PROGRAM_STAGES' crates/tiler-ir/src/program/mod.rs
```

The exact broader anchored `crates prototypes spikes` literal census is 13;
the root `crates` textual population is 11 but includes the struct declaration,
function signatures, and rustdoc examples rather than eleven literals. The
positional `push_variant` call population is 65 across 13 source files; constructor
census is 69 calls in 15 source files; the artifact environment's
`available()` has no consumer beyond its definition. These commands were run,
not merely proposed.

## Released to ready — 2026-08-22, the stop condition's decision landed four days ago

Found by the deferred/blocked sweep, verified by the coordinator at `56eecba1`. The stop condition below parks this on `decide-the-artifact-physical-selection-provenance-surface`. That ticket is **`status: done`**, and its `## Accepted decision — 2026-08-18` says verbatim: *"Implementation flows through `package-selected-physical-implementation-provenance-in-artifact-identity`, which this acceptance unblocks."* All four declared dependencies are `done`. It sat blocked for four days after being explicitly unblocked.

**This is high-leverage:** 13 non-terminal dependents, 7 of them p1 — including `promote-the-bounded-scalar-cpu-vertical-into-a-production-backend`, `decide-how-vector-requirements-cross-the-artifact-boundary`, `publish-the-backend-provider-conformance-suite`, and `prove-the-first-real-fixed-vector-cpu-execution-approach`.

**Facts below predate several landings** — `tiler.kernel-program` v12→v13, `tiler.artifact-program` v20→v21, manifest (20,0)→(21,0), the retired contraction key, and the index-layer gather. Re-audit every Fact at your own base per the stale-Facts rule; repair the ticket and report the repair.

**Scheduling.** Collides with the live gather lane on `contracts/decisions` **only**. **Release trigger: that lane merges or stops at a gated boundary.**

**Stop condition.** No production work may start until Tom accepts or replaces
the Pareto-complete exact surface in the new P1
[`decide-the-artifact-physical-selection-provenance-surface`](decide-the-artifact-physical-selection-provenance-surface.md)
prerequisite. This ticket is `blocked` on that parked `awaiting-decision` node.
The accepted semantic outcome below remains open and unchanged; the repair does
not mark implementation complete or reopen its three discharged dependencies.

## User-visible outcome

An artifact records exactly which physical authority produced each selected region while remaining invariant to every offered provider the selected plan did not use.

## Required delivery

- Replace the lowering-only construction authority with an explicit role-separated `CompilationEnvironment`: required canonical lowering and physical offered sets, no union, no default, and no inference from payload/backend/profile.
- Validate existing selected lowering rows only against the lowering set and new selected physical rows only against the physical set. A missing member is a typed artifact-build refusal; never substitute the governed provider or omit the row.
- Add a separately tagged occurrence-bound physical-selection run inside each artifact variant, carrying the compiler projection: occurrence binding, exact implementation-proposal identity, provider identity, and proposal kind. Preserve multiplicity and association; do not reduce it to an artifact-global provider set, an iterator position, a backend entry, or a payload association.
- Bound that run by the artifact's existing 4,096-entry ceiling and require its
  selected-row count not to exceed the variant's executable-entry count.
  Current compiler production remains independently bounded to twelve; do not
  turn that producer policy into an artifact-only direct-construction refusal.
- Add no physical-specific byte budget or decoder byte-limit kind. Decode is
  governed by the existing whole-manifest limit before parsing. Before a
  builder retains a candidate run, compute its exact canonical physical-run
  contribution and refuse only when that strict subset proves the complete
  artifact identity must exceed the existing whole-identity limit; keep the
  check transactional and expose no second budget authority.
- Encode the complete selected physical row population in artifact canonical identity and the manifest/envelope bytes. Step the owning artifact-program/schema domains coherently at the implementation base and recompute all derived pins and cache/envelope subjects. Do not step semantic, schedule, structured-kernel, payload-content, or unrelated wire domains.
- Keep both offered sets construction-only and discard them after validation. Perturb an unused lowering provider and an unused physical provider independently and prove artifact identity, bytes, envelope digest, and cache subject are unchanged.
- Perturb selected provider identity, implementation-proposal identity, occurrence association, and proposal kind independently and prove the artifact identity/bytes checks fail with assertions unchanged.
- Update `docs/artifact-abi.md`, crate rustdoc, identity ledgers, build translation, codec/decode, equality, limits, and all exhaustive consumers as one coherent identity step.

## Non-goals

Serializing the full offered environments, changing provider selection or cost policy, defining provider precedence, retrying another provider, inferring a missing selected provider, or changing executable kernel semantics.

## Closes when

The production assemble path forwards every selected physical row, construction rejects cross-role or absent authority, unused offered providers remain byte-invariant, selected authority is identity-bearing, all schema/domain pins reconcile, and full artifact/build gates plus independent exact-commit review pass.
