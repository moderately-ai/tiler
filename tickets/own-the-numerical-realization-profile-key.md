---
id: own-the-numerical-realization-profile-key
title: Decide whether numerical profile keys cross into runtime-owned values
status: todo
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
