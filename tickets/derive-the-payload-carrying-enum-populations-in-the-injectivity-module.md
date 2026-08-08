---
id: derive-the-payload-carrying-enum-populations-in-the-injectivity-module
title: Derive the payload-carrying enum populations in the injectivity module
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Two populations are hand-written literals in a module that says none are

`crates/tiler-ir/src/exhaustive_injectivity.rs` declares `SUBNORMAL_MODES: [SubnormalMode; 3]` and `EXCEPTIONAL_ASSUMPTIONS: [ExceptionalValueAssumption; 4]` as hand-written literals, sitting directly beside `PERMISSIONS: [NumericalPermission; variant_count::<NumericalPermission>()]`. Coordinator-verified by reading the declarations.

The module header states: *"every array over a plain enum is sized by `variant_count`, so a widened vocabulary is a build error here rather than a population that quietly shrinks."*

**The assertions over these two are tautologies.** `assert_eq!(ARRAY.len(), <the same literal>)` — `.len()` on `[T; N]` *is* `N` — at `schedule/model.rs` and `kernel/model.rs`, in `the_subnormal_encoding_is_injective_over_its_whole_domain` and `the_exceptional_assumption_encoding_is_injective_over_its_whole_domain`. No input makes them fail.

The concrete failure: adding `FlushedZeroSign::NegativeZero` breaks `push_subnormal` and `subnormal_tag` — both exhaustive matches, so the build stops and an author adds arms — but **the array stays at 3**. The injectivity check then walks 3 of 4 inhabitants while the crate continues to claim *exhaustive finite evidence, a proof over the whole domain*. The same defect propagates to `numerics/tests.rs`'s `BEHAVIOUR_POPULATION`, whose own doc names precisely this scenario as what it prevents.

## The nuance that changes the fix — the audit missed this

**`variant_count` alone cannot fix these.** `SubnormalMode` and `ExceptionalValueAssumption` are **not plain enums**: they carry payloads (`FlushToZero { zero_sign }`, `AssumeAbsent { provenance }`), so their inhabitant counts are products, not variant counts. The module header's claim is therefore *technically consistent* — these are not "arrays over a plain enum" — and a worker who mechanically substitutes `variant_count` will produce a **wrong smaller number** and a green test.

So the fix is a **derived** population, not a borrowed idiom. Something whose arithmetic names the payload — for example a count composed from `variant_count::<SubnormalMode>()` and `variant_count::<FlushedZeroSign>()` with the relationship written down — or a generator over exhaustive matches so that a widened payload enum is a build error at the population itself.

**Whatever you choose, the property to demonstrate is the same:** widening `FlushedZeroSign` (or `ValueDomainProvenance`) must fail the build or fail a test, *at the population*, not merely at the encoder's match arms. Perturb by adding a variant and show it.

Then correct the module header so it states what is actually guaranteed for payload-carrying vocabularies, rather than a rule that reads as covering them.
