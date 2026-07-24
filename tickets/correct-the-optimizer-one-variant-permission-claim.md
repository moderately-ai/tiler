---
id: correct-the-optimizer-one-variant-permission-claim
title: Correct the optimizer contract's one-variant NumericalPermission claim
status: todo
priority: p2
dependencies: []
related: [widen-numerical-vocabulary-and-complete-identity, decide-whether-to-admit-a-distributivity-permission]
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, numerics]
---
`docs/compiler/optimizer.md`, in the distributivity paragraph, states:

> `StrictF32NumericalContract::governed` in `crates/tiler-compiler/src/request.rs` is the only numerical contract the compiler registers, and its `reassociation` is `NumericalPermission::Forbidden`; **the enum has exactly one variant, so no registrable contract permits reassociation either.**

The clause in bold is false as of `widen-numerical-vocabulary-and-complete-identity`: `NumericalPermission` now has `Forbidden` and `Permitted` (ADR 0076 item 1). The paragraph's *conclusion* is unaffected — `StrictF32NumericalContract::governed` is still the only contract the compiler registers and its `reassociation` is still `Forbidden`, so no registrable contract permits reassociation. Only the stated reason changed: it is now a property of the single registered contract rather than of the vocabulary.

`docs/numerical-semantics.md` carried the same claim in its own distributivity paragraph and was corrected in the same change; this file was left alone because `contracts/optimizer` was held by a live sibling. Match that wording so the two contracts do not diverge on the point.

`contracts/optimizer` is a small scope and this is one sentence; fold it into whatever next touches the file rather than opening a worktree for it alone.
