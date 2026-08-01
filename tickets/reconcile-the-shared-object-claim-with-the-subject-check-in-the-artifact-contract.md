---
id: reconcile-the-shared-object-claim-with-the-subject-check-in-the-artifact-contract
title: Reconcile the shared-object claim with the subject check in the artifact contract
status: todo
priority: p3
dependencies: []
related: []
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`docs/artifact-abi.md` and `check_subject` agree about whether one compiled object may be shared across variants declaring different profiles.

## Why

**Fact — found by `deliver-several-artifact-families-from-one-expansion` (2026-08-01).** `docs/artifact-abi.md:327` claims a program may "share one compiled object across variants declaring different profiles", while `check_subject`'s `TargetProfileMismatch` makes that unreachable. The same landing also measured that two `BoundMetalCompileDeclaration`s differing only in `MetalTargetFacts::platform` share a profile key and a byte-identical descriptor — evidence relevant to which side should give way. Either the contract sentence overstates the implementation's intent, or the check over-refuses a documented capability; decide which, correct the loser, and record the ground at both sites.

## Closes when

The sentence and the check agree, with the decision's derivation recorded, and any test that pins the chosen behaviour was watched failing against the other reading.
