---
id: reconcile-the-shared-object-claim-with-the-subject-check-in-the-artifact-contract
title: Reconcile the shared-object claim with the subject check in the artifact contract
status: done
priority: p3
dependencies: []
related: []
scopes: [contracts/artifacts, implementation/artifact]
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

## Outcome

**The doc gave way, and it already had — the check is the ground.** The elimination is not a preference: the artifact layer cannot decide which consumer target a shared object was built for, so admitting one object under two declared profiles would leave a loader inferring a payload's contract from the variant it happened to route to, which is the inference this layer exists to forbid. The measured evidence points the same way — two `BoundMetalCompileDeclaration`s differing only in `MetalTargetFacts::platform` share a profile key and a byte-identical descriptor, so several artifact families are one profile and N objects, and the case the withdrawn sentence described was never the case the multi-family work needed.

**Fact — the v13 delivery-position landing had already rewritten the contradicting sentence, and `check_subject` is unchanged.** `docs/artifact-abi.md:365` (line 327 at filing) now records the sentence as withdrawn rather than asserting it, and `crates/tiler-artifact/src/program/builder.rs:1253` still refuses a second variant whose declared profile differs from its siblings' as `ArtifactBuildError::TargetProfileMismatch`. Verified on the current tree at artifact `v14`, not from v13 memory.

**Fact — three residuals survived that verification, and all three are corrected here.**

1. `docs/artifact-abi.md:365` read "was, until the `v13` step, unreachable", whose plain reading is that the v13 step made it reachable. It did not: v13 widened what an *entry* may name and never touched the cross-variant agreement, so the shape is exactly as unreachable now as before. The sentence now says so and names the step's actual effect.
2. `crates/tiler-artifact/src/program/codec/tests.rs:2564` still carried the retired claim as live justification — "so a program that shares one compiled object across variants declaring different profiles has an honest encoding" — which is the second site the ticket asked to have the ground recorded at. It now justifies the per-payload contract by the reachable delivery-position case and names the refusal.
3. **No test anywhere pinned the refusal.** Exact check, reproducible in one line: `grep -rn "TargetProfileMismatch" . --exclude-dir=target --exclude-dir=.git` returned only `builder.rs:1254`, `error.rs:296`, `error.rs:547`, four ticket files, and the contract line — no test site. `InterfaceMismatch` is unpinned by the same measure, so the whole `check_subject` sibling-agreement branch was load-bearing and unwitnessed; the artifact would have kept building while the contract sentence quietly became wrong again.

**The pin, and the failure that proves it can say no.** `crates/tiler-artifact/src/program/tests.rs::refuses_a_second_variant_declaring_a_different_target_profile` pushes two variants over two distinct programs and asserts all three outcomes: agreeing siblings accepted, a differing profile key refused, and a moved descriptor digest under an unchanged key refused. Perturbation run: with the three-line `TargetProfileMismatch` branch deleted from `check_subject` — literally the other reading — the test failed with `left: Ok(())`, `right: Err(TargetProfileMismatch)`; the branch was restored and `git status` confirms `builder.rs` unmodified. The reachable half was already pinned by `packages_one_payload_per_delivery_position`, and the contract now names both tests so a later reader can reach the evidence from the sentence.

**Scope note (2026-08-01).** `implementation/artifact` was added to this ticket, held by nothing at the time, because residuals 2 and 3 are inside `crates/tiler-artifact` and the ticket's own instruction was to record the ground at both sites. No behaviour changed in that crate: the diff there is one corrected doc comment and one added test.
