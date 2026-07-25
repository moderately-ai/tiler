---
id: make-resolved-value-type-ordering-explicit
title: Make resolved value type ordering explicit
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, determinism, diagnostics]
---
Sibling of the `ValueTypeDefinitionKey` ordering fix, found and correctly scoped out during that work. `ResolvedValueType` also derives `Ord`, so its ordering follows Rust variant declaration order and a semantically neutral variant reorder would silently change behaviour.

The exposure is narrower than the key's was and must not be overstated: concrete-instance closure sets are never serialized, so this ordering reaches no durable identity encoding. What it does decide is which missing-authority error `freeze` reports first when several are candidates — diagnostic determinism, not identity stability. AGENTS.md requires stable diagnostics, so it is still worth closing.

Replace the derived `Ord`/`PartialOrd` with explicit implementations that preserve the current order exactly, following the pattern established for `ValueTypeDefinitionKey` (an explicit family discriminant shared between ordering and any encoding, so rank and tag cannot drift apart). Add a test that pins the family order and fails under a variant reorder. Confirm and state in the outcome whether this ordering reaches any durable encoding; if it turns out it does, that is a stronger finding than this ticket assumes and should be reported rather than quietly fixed.

## Outcome

**The ticket asked whether this ordering reaches a durable encoding. It does. The premise that it does not is false, and this is the identity-stability finding the ticket asked to have reported rather than quietly fixed.**

**Fact (inspected source, base `f286289`).** `crates/tiler-reference/src/lib.rs::compute_reference_identity` takes `value_validators: &BTreeMap<ResolvedValueType, RegisteredReferenceValueValidator>` and iterates it — `for (resolved_type, validator) in value_validators` — encoding each entry into `CanonicalReferenceRegistryIdentity` behind the `tiler.reference-registry.v2` domain separator. A `BTreeMap` iterates in key order, so the order in which `ResolvedValueType` ranks its families is the order in which those entries are encoded. That is a durable identity, not a diagnostic.

The consequence: reordering the variants of `ResolvedValueTypeData` — a change with no semantic content — would have changed `CanonicalReferenceRegistryIdentity` for any registry holding value validators of two different families, and nothing would have failed to say so.

**What the ticket got right.** Within `tiler-ir` the claim holds exactly. `SemanticAuthorityClosure::type_instances` is the concrete-instance closure set the ticket had in mind, and reading its three uses in `crates/tiler-ir/src/semantic/registry.rs` confirms it is never iterated: line 2242 uses `insert`'s boolean for cycle-breaking and line 2309 uses `len()` for a bound. Reproducible as `grep -n type_instances crates/tiler-ir/src/semantic/registry.rs`, which returns exactly those three lines. The ticket's error was scoping the search to the crate that defines the type rather than the crates that order it.

**Fixed at the definition, which covers the consumer.** `Ord`/`PartialOrd` on `ResolvedValueTypeData` are now written out against a `family_discriminant` of 1/2/3, preserving the previous order exactly (`Nominal < Parameterized < EncodedNumeric`, then by the family's own key). Because the ordering is defined on the type, the `tiler-reference` exposure is closed without editing `tiler-reference`.

`ResolvedValueType` itself keeps its derived `Ord`: it is a single-field newtype, so the derive delegates to the explicit impl and carries no variant-order hazard of its own. The hazard is the enum, which is where the ticket's `ValueTypeDefinitionKey` precedent puts the explicit impl too.

`encode` now writes `self.0.family_discriminant()` instead of three literal `output.push(1|2|3)` calls, so the rank and the encoded tag are the same value rather than two constants that happen to agree. The test asserts the tag is the byte immediately after the domain separator, so the two are pinned together from the outside as well.

**Sibling check — one enum shares the shape, and it is clean.** `crates/tiler-ir/src/semantic/types.rs` has exactly two enums deriving `Ord`: `ResolvedValueTypeData` and `CanonicalValueData`. The sibling's ordering reaches no encoding: there is no `BTreeSet<CanonicalValue>` or `BTreeMap<CanonicalValue, _>` in the workspace, and the one place record fields are ordered, `canonical_fields`, sorts by `field.id` alone (`fields.sort_unstable_by_key(|field| field.id)`) and rejects duplicate IDs, so a `CanonicalValue` comparison never breaks a tie. Reproducible as `grep -rn "BTreeSet<CanonicalValue>\|BTreeMap<CanonicalValue" crates/`, which returns nothing. `CanonicalValueData` therefore keeps its derive; converting it would add ceremony without closing an exposure.

**Measurement — no identity moved.** All 676 workspace tests pass unchanged (macOS arm64, pinned nightly `nightly-2026-07-19`). That is the intended result and is itself the evidence the refactor is order-preserving: `CanonicalReferenceRegistryIdentity`, `CanonicalResolvedValueType`, and every identity derived from them are byte-identical, so the explicit implementations reproduce the derived order exactly rather than silently rebaselining it.

**Not split out.** The fix at the definition closes the `tiler-reference` exposure, so no follow-up in that scope is needed. Nothing about the reference crate's own encoder is wrong — it correctly encodes in key order; it was relying on an ordering contract that had not been written down, and now it is.
