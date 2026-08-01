---
id: correct-the-roadmap-frontier-and-opaque-call-status
title: Correct the roadmap's frontier and opaque-call status claims
status: done
priority: p2
dependencies: []
related: [correct-stale-public-compiler-boundary-authorities, integrate-opaque-calls-into-the-physical-frontier, enumerate-the-split-reduction-on-the-planning-frontier, implement-opaque-physical-call-providers]
scopes: [contracts/navigation]
shared_scopes: []
paths: [docs/roadmap.md]
tags: [documentation, correctness]
---
## User-visible outcome

`docs/roadmap.md` describes the physical frontier's actual admission set and the delivered opaque-call contract, instead of deferring both behind a ticket that closed.

## Why this is a correctness ticket

- **Fact:** `correct-stale-public-compiler-boundary-authorities` corrected the same claim in `docs/compiler/optimizer.md`, `docs/operation-extensions.md`, ADR 0078, and `crates/tiler-compiler/src/frontier.rs`, and could not reach `docs/roadmap.md`, which is outside its declared scopes. `docs/compiler/fusion-and-scheduling.md` and `docs/architecture.md` were already correct.
- **Fact:** two roadmap passages are falsified. Around line 106, "Opaque physical calls are not part of this bounded compiler path. Their reviewed provider ticket is deferred behind the optimizer conformance gate and the mature boundary-property and analytical-cost authorities." Around line 279, a Milestone 6 **Fact** quotes the now-corrected optimizer sentence "the bounded P0 frontier admits only checked `ScheduledKernel` proposals and rejects opaque-call proposals explicitly", and concludes that "GEMM recognition and library-call alternatives" is unreachable because opaque calls are deferred.
- **Fact:** `implement-opaque-physical-call-providers` and `integrate-opaque-calls-into-the-physical-frontier` are both `done`. The frontier admits checked `ScheduledKernel` and `KernelSubprogram` bodies and registered `OpaqueCall` proposals, and rejects only the reserved `View` variant. `enumerate-the-split-reduction-on-the-planning-frontier` promoted `KernelSubprogram`.
- **Inference:** the Milestone 6 inference is the load-bearing part and is not simply reversed by this. Opaque *admission* exists; opaque *providers supplied from outside the crate* do not, because `OpaqueCallDeclaration` and `OpaqueCallRegistry` are crate-private. A worker reading either passage would either believe a delivered contract is missing, or over-correct into believing a public provider seam exists.

## Implementation keys

- Read `docs/roadmap.md` in full and both cited passages in context before editing; the corrected wording already exists in `docs/compiler/fusion-and-scheduling.md` and `docs/architecture.md` and must not be contradicted.
- Keep the three claims apart: admitted body variants, delivered declaration contracts, and out-of-crate registration. Only the first two moved.
- Restate the Milestone 6 inference on the correct premise rather than deleting it, and name the exact remaining gap for a library GEMM.
- Preserve the numerical-evidence argument in the passage after it, which nothing here falsifies.

## Closes when

Both passages agree with the frontier's actual admission set and with the sibling contracts, the out-of-crate registration gap is stated rather than implied, and `tkt lint` and `make full` pass.

## Graph maintenance

- Link the corrections to the exact completed tickets that falsified them.
- Check the rest of `docs/roadmap.md` for the same claim family while the file is open, and correct or split whatever else it finds.

## Outcome (2026-07-31)

**Fact.** Both falsified passages now state the frontier's actual admission set — checked `ScheduledKernel` and `KernelSubprogram` bodies and registered `OpaqueCall` proposals, with only the reserved `View` rejected — and link the two completed tickets that falsified the old text (`implement-opaque-physical-call-providers`, `integrate-opaque-calls-into-the-physical-frontier`). The three claims are kept apart as required: admitted body variants and delivered declaration contracts moved; out-of-crate registration did not, and both passages state that `OpaqueCallDeclaration` and `OpaqueCallRegistry` are crate-private so no external provider can supply an opaque call.

**Fact — the Milestone 6 inference is restated on the corrected premise rather than deleted.** The exact remaining gap for a library GEMM is named as two independent things: the absent out-of-crate provider seam, and the per-shape numerical guarantee the L3 record measured no library supplying (`MPSMatrixMultiplication` refuted against all twenty-two named topologies). The numerical-evidence argument in the following passage is preserved and now explicitly load-bearing: the library alternative is inadmissible on those grounds today regardless of the seam.

**Fact — the same-claim-family sweep.** `grep -n "rejects opaque\|admits only checked\|opaque.*deferred" docs/roadmap.md` returns only the two corrected passages. One adjacent staleness found while the file was open and corrected: the Milestone 6 provider count still read "four governed index-access providers registered" in the present tense; it now records four at that landing and six since the structural families, matching the dated corrections at the support-matrix boundary note and `docs/open-questions.md`.
