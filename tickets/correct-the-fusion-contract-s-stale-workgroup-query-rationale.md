---
id: correct-the-fusion-contract-s-stale-workgroup-query-rationale
title: Correct the fusion contract's stale workgroup-query rationale
status: todo
priority: p3
dependencies: []
related:
  - correct-the-capped-tree-partition-s-false-declared-workgroup-width-claim
  - bound-the-tree-cap-s-unmeasured-downward-direction
  - restate-the-tree-width-rule-outside-the-compiler-crate
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, scheduling, target-profiles]
---

`docs/compiler/fusion-and-scheduling.md` repeats the capped tree rule's
workgroup-width authority incorrectly after the rule's source comment was found
to do the same.

## Facts, coordinator-verified at `55621aef`

**Fact — one claimed repository-wide declared fact is false.** The document's
anchor `inside the widest workgroup any profile in this repository declares`
says the rule's maximum width 509 fits a declared repository-wide workgroup
bound. The authoritative Apple9 profile instead declares a
`PreparedKernelPreflight` query; it does not declare a compile-profile
workgroup-width fact. The profile builder rejects a quantitative fact beside
that query.

**Fact — the calibration still observed the relevant entry-specific bound.**
The retained partition calibration's anchor `The workgroup bound is not a
constant this spike asserts` says its prepared pipeline reported
`maxTotalThreadsPerThreadgroup = 1,024`. That supports statements about that
prepared entry, not a profile-wide declared-fact inventory.

**Fact — two neighbouring claims share the authority defect.** The stale
declared-inventory assertion is followed by the unqualified conclusion that no
tree is lost. Separately, the residue paragraph says 521 is `inside the
qualified entry's workgroup width`. The latter is accurate only when read as
the calibration entry's prepared-kernel result; it omits both the query
authority and its phase. No other target-specific workgroup-width claim occurs
in this document.

**Fact — the intrinsic bound survives independently.**
`capped_tree_partition`'s arithmetic establishes `s <= 509`; the compiler test
anchor `the widest reachable width moved` pins 509. Since 509 is below
`MAX_COOPERATIVE_PARTICIPANTS`'s 4,096, the rule introduces no intrinsic
`Unrepresentable` decline. Whether a particular target admits that width remains
a separately assessed workgroup query under ADR 0043.

## What this ticket owes

- In the downward-arithmetic bullet, remove the repository-wide declared-width
  claim. State instead that 509 is below the intrinsic 4,096 representation
  bound, hence introduces no `Unrepresentable` decline; state that target
  workgroup feasibility is separately answered by each prepared entry's query.
- Preserve the calibration fact by saying its prepared entry admitted 1,024
  threads per workgroup at `PreparedKernelPreflight`; do not turn that
  observation into a compile-profile fact or a repository-wide inventory.
- Apply the same phase-qualified wording to the nearby 521 residue sentence.
- Preserve every numerical figure, evidence rung, ticket link, and all
  implementation source unchanged.
- Add a fresh per-Fact audit at the worker's exact base before editing.

## Explicit non-goals

Do not edit `crates/**`, target profiles, the calibration spike, the numerical
rule, its tests, or any identity-bearing artifact. Do not make a new
measurement. Do not repair the same compiler-source comment; the related
source ticket owns that scope.

## Closing checks

Run the following source-safe anchors against the exact edited file:

```sh
rg -n -F 'inside the widest workgroup any profile in this repository declares' \
  docs/compiler/fusion-and-scheduling.md
rg -n -F "inside the qualified entry's workgroup width" \
  docs/compiler/fusion-and-scheduling.md
rg -n -F 'PreparedKernelPreflight' docs/compiler/fusion-and-scheduling.md
rg -n -F 'the rule introduces no `Unrepresentable` decline' \
  docs/compiler/fusion-and-scheduling.md
make citations
tkt lint
git diff --check
tkt guard tkt/correct-the-fusion-contract-s-stale-workgroup-query-rationale --format json
```

The first two searches must produce no matches after the edit. The next two
must find the new phase-qualified rationale. For the required subject
perturbation, temporarily restore the stale phrase in the document; the first
search must print its location and violate the closure condition. Restore the
correct text before commit.
