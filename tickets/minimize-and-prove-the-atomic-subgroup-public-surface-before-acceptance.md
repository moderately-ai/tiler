---
id: minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance
title: Minimize and prove the atomic subgroup public surface before acceptance
status: in-progress
priority: p1
dependencies: [admit-an-atomic-subgroup-realization-subject-to-target-profiles]
related: [accept-the-atomic-subgroup-realization-surface]
scopes: [implementation/ir, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [subgroup, public-boundary, identity, correctness]
claimed_from: todo
assignee: worker-atomic-surface
lease_expires_at: 1786681121
---
## User-visible outcome

Tom receives the exact, smallest atomic-subgroup Rust surface that the accepted model needs today, with live identity consumers proved and speculative decoder/error vocabulary deferred until a real codec owns it.

## Exact-base Fact audit — 2026-08-13 at `b2ab50f278616a1ad8f171184a16d60ae7e608ff`

Read in full: the acceptance and implementation tickets; ADRs 0074 and 0075; `schedule/subgroup.rs`; the `ResourceRequirements` definition and derivation; target-profile construction, checked facts, complete and checked descriptor encoders, lookup, and tests; frontier and kernel identity consumers; artifact resource encoding/decoding; and the landed diff `5cd61fbe^..5cd61fbe` plus follow-up `eecc4002`.

1. **False — the acceptance ticket does not enumerate the landed public surface.** Its Included list omits `SubgroupTransfer::{tag, from_tag, key}`, the variants and `rule`/`Display`/`Error`/derived traits of `SubgroupRealizationError`, `SubgroupRealizationSubject::encode`, `SubgroupRealizationResolution`, `TargetProfile::subgroup_realization`, `TargetProfileBuildError::DuplicateSubgroupRealization`, and exact derived traits. Reproduce with `git diff 5cd61fbe^ 5cd61fbe -- '*.rs' | rg '^\+\s*pub'` and the symbol anchors in `schedule/subgroup.rs` and `target.rs`.
2. **Imprecise — `key` and `encode` have current cross-crate consumers, while raw `tag` does not.** Kernel identity, complete and checked target descriptors, and frontier identity call `subject.encode`; physical failure and explain construction call `transfer().key()`. `SubgroupTransfer::tag` is only the private implementation detail through which `SubgroupRealizationSubject::encode` writes the transfer byte, so its consumer proves the mapping must remain, not that the raw accessor must remain public. Anchor commands: `rg -n 'subject\.encode|transfer\(\)\.key' crates/tiler-ir crates/tiler-compiler --glob '*.rs'` and `rg -n '\.tag\(\)' crates/tiler-ir/src/schedule/subgroup.rs`.
3. **Verified — public `SubgroupTransfer::from_tag` has no production consumer.** The only call is the local `unknown_transfer_tag_is_undefined` test. `rg -n 'SubgroupTransfer::from_tag' crates --glob '*.rs'` names only `schedule/subgroup.rs`.
4. **Verified — `SubgroupRealizationError::UndefinedTransfer` is unreachable from every public constructor.** `SubgroupWidth::new` returns only `ZeroWidth`; `SubgroupRealizationSubject::new` returns only `UnsupportedWidth`; `from_tag` returns `Option`, not the error. The variant appears only in its own docs, `rule` match, and the same test. Anchor: `UndefinedTransfer` in `schedule/subgroup.rs`.
5. **Verified — a present artifact resource subject is explicitly deferred.** The artifact model destructures `subgroup: _` and decode constructs `subgroup: None`; the acceptance ticket excludes present-subject artifact encoding. There is therefore no governed decoder whose implementation currently needs `from_tag`.
6. **Verified — the landed identity evidence is incomplete.** Target complete/checked descriptor tests perturb width and arithmetic, but no test constructs a kernel with `ResourceRequirements.subgroup = Some(_)`. `rg -n 'subgroup.*identity|identity.*subgroup|subgroup_requirement' crates/tiler-ir/src/kernel crates/tiler-compiler/src/target.rs` locates the encoder and target-only tests.
7. **Imprecise — “perturb transfer independently” is not currently a realizable typed subject test.** `SubgroupTransfer` has exactly one variant. The unknown raw tag test proves an unrecognized tag is rejected by the speculative helper, not that whole-subject equality distinguishes two typed transfer values. Do not manufacture a test-only production variant or claim the raw-tag test as a subject perturbation.
8. **False — the draft public error does not conform to ADR 0074 convention 5a.** `SubgroupRealizationError` has no out-of-crate total recognizer and is an error/diagnostic vocabulary, yet it lacks `#[non_exhaustive]`. The accepted convention names errors as the ordinary 5a population. `ScalarArithmeticSubjectError` is the direct sibling precedent.
9. **Imprecise — public `SubgroupTransfer::tag` is not needed cross-crate.** Every live cross-crate identity consumer calls `SubgroupRealizationSubject::encode`; the raw tag is used only inside `schedule/subgroup.rs`. `key` remains genuinely cross-crate through physical and explain consumers.
10. **False — the original Required-work claim that `xor_shuffle_rejects_width_one` compares the governed rule token to itself.** At this ticket's stated exact base, the test already compares `SubgroupRealizationError::UnsupportedWidth.rule()` with the literal `"subgroup-width-unsupported"`. The token is pinned and no source repair is owed. Reproduce with `git show b2ab50f278616a1ad8f171184a16d60ae7e608ff:crates/tiler-ir/src/schedule/subgroup.rs | rg -n -A4 -B4 'subgroup-width-unsupported'` and read the whole test around the hit.
11. **False — `SubgroupTransfer` does not conform to ADR 0074 convention 5a.** The first exact-base audit missed this defect because it repeated the module's claim that the identity encoder and constructor made the enum a total-map vocabulary. Every exact-base exhaustive match — then-public `tag`, public `key`, and private `transfer_defines_width` — has its body in `tiler-ir`, the crate that defines the enum, so `#[non_exhaustive]` has no effect on them and widening still stops all three authorities. Every out-of-crate consumer constructs the known variant, reads it through `SubgroupRealizationSubject::transfer`, or calls `key`; none itself totally maps or recognizes the enum. It is therefore a growing 5a public enum and must be `#[non_exhaustive]`. Reproduce with `rg -n 'match .*SubgroupTransfer|SubgroupTransfer::|transfer_defines_width|\.transfer\(\)' crates --glob '*.rs'` and read each hit rather than treating construction as recognition.

These repairs narrow the exact spelling presented for acceptance; they do not change the already accepted whole-subject model.

## Required work

- Remove `SubgroupTransfer::from_tag` and the unreachable `SubgroupRealizationError::UndefinedTransfer` from the public draft. Reintroduce a decoder only with the first schema that consumes it, where unknown-tag refusal and byte ownership can be tested end to end.
- Make `SubgroupTransfer::tag` private; preserve public `key` and `SubgroupRealizationSubject::encode`, whose current cross-crate explanation and identity consumers need one defining authority rather than locally reconstructed mappings.
- Mark `SubgroupRealizationError` `#[non_exhaustive]` under ADR 0074 convention 5a. Keep its typed `ZeroWidth` and `UnsupportedWidth` variants, `rule`, `Display`, and `Error` surface.
- Mark `SubgroupTransfer` `#[non_exhaustive]` under convention 5a while retaining same-crate exhaustive matches as the compile-time widening guards. Add an out-of-crate API test whose required wildcard becomes an `unreachable_patterns` error if the attribute is removed.
- Add a kernel-identity test with a real `Some(SubgroupRealizationSubject)`. Prove absent subjects preserve the existing pin and independently perturb every presently constructible dimension (width and arithmetic). Assert the transfer tag is encoded at its governed position without claiming that a second typed transfer exists.
- Re-run the target complete/checked descriptor tests and subgroup feasibility tests. Record the transfer-perturbation evidence boundary explicitly for the later ticket that introduces a second transfer or the first artifact decoder.
- Rewrite `accept-the-atomic-subgroup-realization-surface` against the repaired exact commit: enumerate every public type, variant, method, field, relevant trait implementation, observed identity consequence, and exclusion. Apply the complete decision-packet readiness gate rather than presenting the current abbreviated recommendation.
- Perturb the source, not an assertion: remove or corrupt the subgroup requirement passed to the kernel identity encoder and show the new `Some` test's failure text; separately change a width/arithmetic subject and show the descriptor/identity distinction fires.

## Option gate

- **Status quo:** keep the speculative decoder and unreachable error. Correct but exposes more unowned public vocabulary and has no host/runtime benefit.
- **Narrow now:** retain only currently consumed encoding/explanation helpers; privatize the raw tag; remove the decoder/error reservation until a real schema owns it; make the growing transfer and error non-exhaustive. Same correctness and runtime, smaller conforming surface, clearer authority, and future decoder work gains an end-to-end negative. This dominates status quo.
- **Remove all tag/key/encode helpers:** rejected. Cross-crate consumers would duplicate canonical mappings or require a larger public trait/identity redesign.
- **Invent a second transfer only for perturbation:** rejected. It would widen the production vocabulary without an admitted semantic or backend realization and make the test prove a population the product does not support.

## Closure

Close when the dominated speculative surface is gone, present-subject kernel identity has a subject perturbation, targeted tests and package gates pass, and the dependent acceptance packet names the exact repaired surface and its honest evidence boundary.

## Delivery evidence — 2026-08-13

- The raw tag is private; `from_tag` and `UndefinedTransfer` are absent; `SubgroupTransfer` and `SubgroupRealizationError` are non-exhaustive under convention 5a. Same-crate exhaustive transfer matches remain the widening guards. `key` and subject `encode` remain the one public explanation and identity authorities their cross-crate consumers use.
- `kernel::tests::subgroup_requirement_is_append_only_and_identity_bearing` constructs a prospective pointwise kernel with the same verified schedule/body but a real `Some(SubgroupRealizationSubject)`, then runs the ordinary refinement verifier. The currently derivable `None` control retains an exact base-derived identity pin. The present suffix is exactly presence, big-endian width, arithmetic, transfer; width and arithmetic move independently, and the only transfer's tag is pinned at the final byte.
- The unsupported population remains explicit: no admitted schedule derives `Some`; no artifact record encodes or decodes a present subject; and no second typed transfer exists. Transfer equality cannot be independently perturbed until one of those semantic populations lands. A raw unknown byte is not evidence for a second typed transfer.

### Production-subject perturbations

Each perturbation changed production code temporarily, ran the named unmodified test, and was restored before the clean run.

1. Removing `subject.encode(bytes)` from `push_subgroup_requirement` made `cargo test -p tiler-ir --lib -- subgroup_requirement_is_append_only_and_identity_bearing` fail at the encoder's own reservation backstop: `assertion left == right failed: the reserved kernel-identity length must equal what the encoder wrote`, with `left: 825` and `right: 831`.
2. Replacing the encoded width with zero made the same kernel test report `presence, width, arithmetic, and transfer append in governed order`, with `left: [1, 0, 0, 0, 0, 3, 1]` and `right: [1, 0, 0, 0, 32, 3, 1]`. The independent compiler consumer `cargo test -p tiler-compiler --lib -- subgroup_subject_and_verdict_participate_in_identity_independently` also failed with `the width dimension does not reach the complete descriptor`.
3. Replacing the encoded arithmetic tag with `0xff` made the kernel test report `left: [1, 0, 0, 0, 32, 255, 1]` and `right: [1, 0, 0, 0, 32, 3, 1]`; the independent compiler consumer failed with `the arithmetic dimension does not reach the complete descriptor`.
4. Replacing `InRangeXorShuffle`'s private tag with `0xff` made the kernel test report `left: [1, 0, 0, 0, 32, 3, 255]` and `right: [1, 0, 0, 0, 32, 3, 1]`. This proves the encoded transfer position without claiming an uninhabited typed neighbour.
5. Removing `#[non_exhaustive]` from production `SubgroupTransfer` made `cargo test -p tiler-ir --test subgroup_public_surface` fail its out-of-crate API fixture under `#![deny(unreachable_patterns)]`: the required wildcard was `unreachable`, because `InRangeXorShuffle` then `matches all the relevant values`.
6. Temporarily widening production `SubgroupTransfer` with `FutureTransfer` made `cargo check -p tiler-ir --lib` fail with three `E0004` errors: `pattern SubgroupTransfer::FutureTransfer not covered` at private `tag`, public `key`, and private `transfer_defines_width`. This proves the external growth marker does not weaken the defining crate's exhaustive widening guards.

### Clean targeted results

- `cargo test -p tiler-ir --lib -- subgroup`: 7 passed, including the verified present-kernel subject and all construction/equality tests.
- `cargo test -p tiler-compiler --lib -- subgroup`: 22 passed, covering public target construction/lookup, complete and checked descriptor identity, feasibility admission/refusal/unknown, and neighbour non-composition without editing compiler source.
- `cargo test -p tiler-ir --test subgroup_public_surface`: 1 passed, proving a downstream crate can construct the known transfer while partial classification retains the wildcard required for future variants.
- `cargo nextest run -p tiler-ir`: 1,146 passed across 10 binaries, including the new out-of-crate public-surface subject.
- `cargo check -p tiler-ir --all-targets`, `cargo clippy -p tiler-ir --all-targets -- -D warnings`, `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-ir --no-deps`, and `cargo test -p tiler-ir --doc`: passed; doctests ran 9 ordinary passes, 1 ignored example, and 9 compile-fail passes.
- `cargo fmt --all -- --check`, `tkt lint`, `make citations`, and `git diff --check`: passed. Citations resolved 1,178 pinned citations and 6,527 local links.
- `tkt guard tkt/minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance --base b2ab50f278616a1ad8f171184a16d60ae7e608ff --ticket minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance --explain`: `WARN` only for declared/live claim overlap; no scope under-declaration. The direct files map to `implementation/ir` and `project/tickets`, with reverse dependencies reported transitively. The accepted-surface packet is within declared `contracts/decisions`.
- Worktree-local `target/` occupied 2.2 GB with 82 GB free when the package gates completed.
## Scope correction — 2026-08-13

The exact work census touches production/test source only in `tiler-ir` plus the decision ticket. `tiler-compiler` is a read-and-test consumer, not an edit owner, so its exclusive scope was removed before claim. If implementation proves a compiler edit necessary, stop and add the scope before touching it.
