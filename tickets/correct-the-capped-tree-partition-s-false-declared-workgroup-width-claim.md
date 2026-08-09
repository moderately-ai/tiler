---
id: correct-the-capped-tree-partition-s-false-declared-workgroup-width-claim
title: Correct the capped tree partition s false declared-workgroup-width claim
status: in-progress
priority: p2
dependencies: []
related: [carry-the-tree-participant-cap-as-a-target-profile-row, bound-the-tree-cap-s-unmeasured-downward-direction]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, compiler, profiles]
claimed_from: todo
assignee: terra-capped
lease_expires_at: 1786243732
---

`capped_tree_partition`'s doc comment claims something about every profile in the repository that is false. A sibling constant a few lines away avoids that declaration claim but does not state the query's phase, so the two comments need one current-authority spelling rather than competing conclusions.

## Facts, coordinator-verified at `df5d23fc`

**Fact.** The doc comment asserts that the widest workgroup any profile in this repository **declares** is the qualified Apple9 entry's 1,024.

**Fact — no production profile declares it.** The only `declare_max_threads_per_workgroup(1_024, …)` call in `crates/tiler-build/src/metal_declaration.rs` sits inside `#[cfg(test)] mod tests`, which opens at the `#[cfg(test)]` attribute immediately preceding it. `FIRST_MACOS_APPLE9` declares workgroup threads as a `PreparedKernelPreflight` **query**, not a fact — and `declare_max_threads_per_workgroup_query` rejects a coexisting fact, so the two cannot both be present.

**Fact — corrected at `c383e86d`.** `MEASURED_TREE_PARTICIPANT_CAP`'s doc avoids the false declaration claim: it says the prepared entry admits 1,024 threads per workgroup. It does not name the `PreparedKernelPreflight` query or its later resolution phase, so it is safe but incomplete rather than a complete correct spelling.

**Inference — corrected at `c383e86d`.** The false version is load-bearing in an argument: it is offered as the reason a widened participant count stays inside the workgroup width. The qualified prepared-entry query can resolve below a width this rule offers — the comment's 8,192-contributor case moves from 128 to 256, so a 128-thread entry would reject the latter — and therefore the conclusion that no previously offered tree is lost does **not** survive universally. The query decides that feasibility at preflight; this ticket corrects its authority description only and does not change selection or admission behaviour.

## What closes this

The claim restated to distinguish a **declared fact** from a **preflight query**, so a reader can tell which authority bounds the width and when it resolves. Preserve `MEASURED_TREE_PARTICIPANT_CAP`'s safe prepared-entry vocabulary, but name the query and phase rather than copying its incomplete wording.

**Cite by searchable anchor, not line number.** Note the failure mode `AGENTS.md` records and that bit this ticket's predecessor: an anchor spanning a line break greps as **absent**, and doc comments here wrap at 80 columns. The predecessor's durable fragment was `second target profile should carry its own row`; find an equivalent and **run its grep before committing to it**.

**Check the rest of this doc comment's inventory claims.** The worker that found this reported the comment leaks **three** downstream claims, of which this is the one it verified false — so the other two are unexamined, not clean. **Name the count you checked**, so a clean result is distinguishable from an unchecked one.

Do not change the rule, the constant, or any assertion — `bound-the-tree-cap-s-unmeasured-downward-direction` landed the selection logic and its evidence rungs, and this is a documentation repair on top of it. Do not edit `crates/tiler-build/**` (`implementation/build`, not this scope); read it to describe it correctly.

## Outcome

**Fact, 2026-08-08 — three downstream inventory claims checked.**

1. **Verified.** `509 participants stage 2,036 f32 bytes`: `capped_tree_partition`'s `s <= 509` anchor and the tree's `one \`f32\` slot per participant` anchor establish the count and storage shape.
2. **Verified.** The schedule authority is 4,096: `pub const MAX_COOPERATIVE_PARTICIPANTS: u64 = 4_096;` is the independent static representation limit.
3. **False and corrected.** The Apple9 1,024 value is not a declared compile-profile fact. `BoundMetalCompileDeclaration::declare` installs `declare_max_threads_per_workgroup_query` at `AvailabilityPhase::PreparedKernelPreflight`; test `a_prepared_kernel_row_cannot_be_declared_as_a_compile_profile_fact` rejects the 1,024 fact beside it with `ConflictingQuantitativeFactAndQuery` on `threads-per-workgroup`.

The repaired `capped_tree_partition` comment keeps the static 4,096 representation bound, identifies the qualified Apple9 workgroup capacity as the later prepared-kernel query, and leaves that query to decide whether its resolved value admits the selected tree width. It makes no rule, cap, assertion, API, identity, numerical, or build-profile change.

**Deliberate prose perturbation.** Temporarily restoring `the widest workgroup any profile in this repository declares is` made the source-only negative check fail with: `negative prose check failed as intended: retired declaration claim is present`. The final source-only check reported: `restored prose check passed: retired declaration claim absent`.

**Validation.** `cargo fmt --all --check`; `cargo nextest run -p tiler-build -E 'test(a_prepared_kernel_row_cannot_be_declared_as_a_compile_profile_fact)'`; `cargo check -p tiler-compiler`; `cargo clippy -p tiler-compiler -- -D warnings`; `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps -p tiler-compiler`; `tkt lint`; `make citations`; `git diff --check`; and `tkt guard tkt/correct-the-capped-tree-partition-s-false-declared-workgroup-width-claim --base c383e86dd7b59911d23bd8891df8c4fecff31403 --format json` passed (the guard reports no under-declaration or conflict; its non-gating warnings are declared-scope collisions).
