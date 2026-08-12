---
id: decide-the-safe-cross-crate-composed-reference-boundary
title: Decide the safe cross-crate composed-reference boundary
status: awaiting-decision
priority: p2
dependencies: [retain-the-selected-semantic-candidate-for-the-conformance-oracle]
related: [define-the-composed-realization-driver-subject-bridge, implement-the-composed-realization-evaluation-driver, accept-the-composed-realization-evaluation-surface, accept-the-realization-witness-surface]
scopes: [implementation/conformance, implementation/reference, implementation/compiler, implementation/ir, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, conformance, reference, correctness]
---
## User-visible outcome

The composed conformance oracle can evaluate a plan-selected grouping without exposing a raw API that lets a caller pin device-produced tensors into the expected-value computation, without making `tiler-reference` depend on compiler plans, and without silently answering a permissive numerical contract as the strict one.

## Source-first Fact audit — exact main `1ff5e90b`

- **Verified — the accepted visibility/home pair is not implementable as later narrowed.** [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md), anchor `crate-private or #[doc(hidden)]`, left two possible visibility spellings. [`retain-the-selected-semantic-candidate-for-the-conformance-oracle`](retain-the-selected-semantic-candidate-for-the-conformance-oracle.md), anchor `stays crate-private`, later chose the narrower one while also choosing `tiler-conformance` as the driver home. The manifests show sibling normal dependencies and Rust has no friend-crate visibility; `tiler-conformance` cannot call a crate-private `tiler-reference` item.
- **Verified — making the raw primitive hidden-public is the wrong repair.** The accepted rationale, anchor `a public pinning primitive is a hole where an oracle should be`, is about language reachability rather than rustdoc presentation. `#[doc(hidden)] pub` remains callable and still accepts caller-supplied tensors.
- **Verified — the current reference bridge refuses the contract this driver is for.** `ReferenceNumericalConformance::from_realization`, anchor `ReassociationPermitted`, rejects any realization that grants regrouping because its ordinary evaluator answers one strict value. `ReferenceNumericalConformance::new` produces `ConformanceSubject::Unstated`; using it would lose the arithmetic subject rather than discharge the permission.
- **Verified — the safe primitive has to own every intermediate tensor.** The research record, anchor `Every pinned tensor is produced by a reference path`, makes this the property separating the survivor from the already-existing vacuous composition. A callback or request carrying a `Tensor` for an internal `ValueId` does not satisfy it.
- **Verified — current conformance consumers are test-only and private.** `crates/tiler-conformance/src/lib.rs`, anchors `There is none` and `Every module is #[cfg(test)]`, states the current public-surface and build population. Its manifest says nothing depends or may depend on the crate; the workspace is `publish = false`. The serial-sum tests are the first named consumers of the composed driver.
- **Verified — reference registry authority must stay explicit.** `ReferenceEvaluator` holds a caller-chosen `FrozenReferenceRegistry` and work allowance. A driver that silently calls `ReferenceEvaluator::standard()` would reject otherwise valid installed reference capabilities and turn a narrow first implementation into an implicit governed-only fallback.

## Decision required

Choose the smallest boundary satisfying all of the following together:

1. The raw ValueId pin/observe mechanism remains private inside `tiler-reference`.
2. The cross-crate operation accepts only the original declared inputs and typed semantic/realization descriptors; it computes every observed and pinned intermediate itself and accepts no caller-supplied internal tensor.
3. It validates the descriptor against `P'`, carries the exact arithmetic subject and subnormal modes, and treats a permitted freedom as discharged only when the corresponding exact witness is present. Unsupported or mixed realization populations refuse by typed cause.
4. The caller supplies the frozen reference registry/work authority explicitly; there is no standard-registry default or fallback.
5. The plan-bound conformance entry accepts one `PlanAlternative`, never free program/witness parts. Any lower-level language-public SPI must be named honestly and must be incapable of device-tensor injection.
6. Decide whether the first implementation remains `pub(crate)`/test-only in `tiler-conformance` until a non-test consumer exists, or supersedes that crate's accepted no-public-surface contract now.

Compare at least:

- a reference-owned safe composed-evaluation operation over typed descriptors, called by a plan-binding driver in `tiler-conformance`;
- relocating the whole driver beside the private primitive, including the resulting compiler/reference dependency and plan-ownership consequences; and
- exposing the raw pin primitive publicly or through a callback (the burden is to show how either prevents caller-originated tensors rather than merely documenting that callers should not use them).

## Audited nondominated frontier

### Recommended narrow first pass

Keep the raw `(ValueId, Tensor)` pin/observe machinery genuinely private in `tiler-reference`. Add a plan-neutral **safe composed-evaluation session** as the cross-crate mechanism. The session accepts an explicit frozen reference registry, an explicit work allowance, the retained semantic program, declared inputs, and typed semantic `ValueId`/fold descriptors. It accepts **no tensor for an internal value**: every observed value, declared-order fold result, and subsequent pin is produced and retained inside `tiler-reference`.

The session derives a subjectful numerical conformance from the full realization plus the exact fold descriptor. It may discharge reassociation only for the grouping that descriptor completely states; contraction, permutation, signed-zero elimination, exceptional-value assumptions, arithmetic/NaN disagreement, unsupported topology, and incomplete coverage remain distinct typed refusals. There is no strict, standard-registry, unsubjected, or baseline fallback.

The first `tiler-conformance` wrapper remains `pub(crate)` and test-only because every named consumer is currently in that crate's test population and the accepted crate contract forbids downstream dependencies. It accepts `PlanAlternative`, explicit reference authority, and declared inputs, obtains one compiler projection internally, and never accepts a free recipe. The earlier public driver remains a reserved future boundary rather than an orphan API. A separate activation ticket is required when a named non-test consumer appears; that decision must also decide whether the reusable home should be this dependency-heavy evidence crate or a narrower oracle crate.

This deliberately amends `sole public composition entry` to `sole supported plan-conformance entry`. The safe plan-neutral reference session is language-public only because a sibling crate must call it, and is not access control disguised as `#[doc(hidden)]`: its safety comes from having no caller-tensor parameter. The compiler subject projection is decided by the dependent bridge ticket; current evidence favors an on-demand compiler-owned visitor over public parallel slices or a detached snapshot.

### Other top-tier architecture

A new narrow reusable oracle crate depending only on compiler, IR, and reference can carry a genuinely public driver without turning the device-evidence crate into an API dependency. It matches the recommendation on correctness, strictness, and oracle runtime, and has the cleaner dependency closure for a real external consumer. It loses today on maintenance and compatibility because it adds a crate and namespace before any such consumer exists. A named consumer needing reusable composed evaluation is the reconsideration trigger.

### Dominated or unsafe options

- Moving the driver to `tiler-compiler` adds a normal compiler-to-reference dependency, couples production compiler users to the oracle, and still needs a safe cross-crate reference operation.
- Moving it to `tiler-reference` makes the reference name a compiler plan or requires a free caller-minted recipe as the plan-conformance claim.
- A raw public or `#[doc(hidden)]` pin/observe method is rejected: documentation visibility does not prevent a device tensor from becoming the expected value.
- A callback or feature gate around raw pins is rejected: Rust feature unification is not friend access, and a callback can still supply or retain the wrong tensor unless the reference owns the whole session.
- `ReferenceEvaluator::standard()`, strict conformance, `ReferenceNumericalConformance::new`, baseline reconstruction, and unsupported-topology fallback are all rejected silent defaults.

### Decision matrix

| Option | Correctness | Fail-closed contract | Tiler host cost | Long-term maintenance/compatibility |
| --- | --- | --- | --- | --- |
| Safe reference session + test-only plan-binding wrapper | Top: reference owns all intermediates and the wrapper revalidates one plan | Top: every authority is required; unsupported sites refuse | On-demand `O(stages + edges)` projection plus the unavoidable reference evaluations; no compile hot-path or device-kernel cost | Best now: no new crate or orphan public driver; honest future activation seam |
| New narrow public oracle crate + same safe session | Top | Top | Same oracle work; one additional crate in the build graph | Best only once a reusable consumer exists; premature namespace/dependency cost today |
| Public driver in current conformance crate + same safe session | Top | Top | Same oracle work, but consumers inherit the full evidence/device dependency closure | Worse than the two above while no consumer exists; contradicts the current top-of-graph/no-dependent contract |
| Raw pin API, strict/default evaluation, or caller-built free parts | Rejected | Rejected | Superficially cheap | Creates a silent-wrong or vacuous oracle and permanent authority ambiguity |

No option changes kernel execution, artifact/schema/cache identity, or ordinary compilation cost. An eager retained recipe is unnecessary; derive it only when conformance runs, and measure before caching it.

## Non-goals

Choosing the compiler's exact opaque plan projection; retaining `P'`; implementing the driver; serializing a semantic program; artifact-only replay; changing artifact/schema/cache identity; using device results as oracle inputs.

## Closes when

Tom accepts the exact safe reference-side operation and the first driver visibility, the earlier contradictory acceptance wording is corrected without losing its no-injection rationale, every registry/conformance input is explicit, and the compiler-subject and implementation tickets depend on the accepted boundary.
