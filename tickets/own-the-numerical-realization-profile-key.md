---
id: own-the-numerical-realization-profile-key
title: Decide whether numerical profile keys cross into runtime-owned values
status: done
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [ir, numerics, serialization]
---
**Fact.** `tiler_ir::schedule::NumericalRealization::profile_key` is `&'static str` (`crates/tiler-ir/src/schedule/numerics.rs`). It therefore names a compile-time constant of the producing build and cannot represent a key read from bytes.

**Consequence.** The decoded dispatch record carries an owned numerical-facts
key rather than reconstructing the schedule-layer `NumericalRealization`.
That split may now be intentional: accepted artifact policy says decoding
produces a dispatch record, not reconstructed compiler IR.

**User-visible outcome.** Each layer has one clearly owned numerical profile
identity, and callers do not translate between duplicate records without an
explicit boundary.

**What closes this.** Either document that schedule construction uses a
compile-time key while decoded dispatch data owns its bytes, or establish that
one shared record must cross both boundaries and adopt an owned representation.
The latter must account for loss of `Copy`, `const fn` construction, and current
value-semantic call sites. A decoder's inability to rebuild compiler IR is not
by itself a defect.

**Scope note.** The ticket owns both existing spellings so the selected outcome
cannot change the IR record while leaving an unnecessary artifact duplicate, or
declare the split intentional without documenting both sides.

## Outcome — the split is intentional, documented on both sides (2026-07-27)

The ticket's first branch, taken with the derivation rather than the preference. Both spellings are documented, which the scope note required.

**Fact: the two records sit on opposite sides of a serialization boundary and own different things.** `tiler_ir::schedule::NumericalRealization` is compiler IR, and the only thing that mints one is a compiling build whose contract keys are its own compile-time constants — `&'static str` is what that key *is* on that side, not a limitation of it. `tiler-artifact`'s `NumericalFacts` is a decoded dispatch record whose key arrived as bytes, which is the definition of the boundary rather than a narrowing of the other.

**Fact: the asymmetry is already the shape of the code.** The artifact *builder* takes the shared-IR record directly — `realization.rs` records that "a packaged artifact holds one `tiler_ir::schedule::NumericalRealization` for the whole portfolio" — and only *decoding* produces the owned record. That matches the accepted policy that decoding yields a dispatch record rather than reconstructed compiler IR, so a decoder's inability to rebuild the IR record is not a defect, exactly as this ticket allowed for.

**Inference: unifying them would pay a real cost for a use the policy excludes.** One owned record crossing both boundaries costs `NumericalRealization` its `Copy` and its `const fn new` across the schedule layer — 105 references, 23 of them by-value signatures. `Cow<'static, str>` would keep `const fn` and still lose `Copy`, and would buy the ability to construct an IR realization with a runtime key, which nothing needs because nothing turns a decoded artifact back into schedulable IR.

**What would reopen it, recorded at both sites:** something needing to rebuild schedulable IR from a decoded artifact. Then one owned record must cross both boundaries and the `Copy`/`const fn` cost is the price of it.

Neither doc comment now points at this ticket as an open question; both state the decision and its trigger for reconsideration.
