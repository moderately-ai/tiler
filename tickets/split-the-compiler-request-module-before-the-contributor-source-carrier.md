---
id: split-the-compiler-request-module-before-the-contributor-source-carrier
title: Split the compiler request module before the contributor-source carrier
status: in-progress
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue, replace-the-serial-sum-contributor-fields-with-the-exhaustive-source]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, maintainability, compiler, identity-adjacent]
claimed_from: todo
assignee: worker-split-compiler-request
lease_expires_at: 1787147268
---
## User-visible outcome

`crates/tiler-compiler/src/request.rs` — the workspace's largest source file (13,746 lines at filing) — becomes a `request/` directory of cohesive submodules (recognition walk, normalized-output types, subject encoding, refusal vocabulary, control-population tests), so the accepted serial-sum contributor-source carrier migrates a set of small readable modules instead of one monolith.

## Why this exists

Filed 2026-08-19 from Tom's module-size directive. The file owns compiler recognition, every `Normalized*` type, the request-subject encoders (`tiler.compiler.request-subject.v6` content), and their control populations at once. Two accepted work items are about to churn it: the contraction migration (in integration at filing) rewrote its contraction arms, and the accepted materialized-producer carrier will force ~23 of its sites. Splitting first makes both cheaper to review; splitting concurrently would make the diffs unreadable.

## Sequencing — why blocked

Hard-held until the contraction-replacement migration (`e61fbc60` under review at filing) is merged, gated, and pushed; this file is inside that migration's diff. On release, this split should land **before** `replace-the-serial-sum-contributor-fields-with-the-exhaustive-source` dispatches, and that carrier then migrates the split tree. Coordinator flips this to `todo` at the release trigger.

## Required work

Same discipline as the sibling split tickets: full read first; seams from cohesion; directory-module conversion keeping the declaring `mod request;` untouched; pure code motion with zero public-surface movement; minimal recorded visibility widenings; **zero identity/pin/test movement** — the subject encoders are identity-bearing and every pinned qualifier must pass byte-identically with no test edits. The `#[cfg(test)]` control populations may move into submodule-local test modules but keep their names and assertions byte-for-byte.

## Evidence and checks

The sibling tickets' full package (`check`/`nextest`/doc-tests/clippy/fmt/rustdoc for `tiler-compiler`, `tkt lint`, `git diff --check`, `tkt guard`), plus explicit confirmation that the explain qualifier golden and program-alternative id pins are untouched.

## Closes when

The split lands from the post-migration base with all gates green and zero test edits, before the contributor-source carrier dispatches.
