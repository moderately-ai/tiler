---
schema: "tiler-doc/v1"
id: "ADR-0034"
kind: "decision"
title: "Govern admitted built-in dtype keys in Tiler"
topics: ["numerics","dtypes","governance"]
catalog_group: "dtypes-quantization"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.dtype-identity-admission-policy"]
ticket: "define-dtype-namespace-admission-policy"
---

# 0034: Govern admitted built-in dtype keys in Tiler

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md).
- **Work record:** [define-dtype-namespace-admission-policy](../../tickets/define-dtype-namespace-admission-policy.md).
- **Preserved primary sources:** [dtype primary-source record](../research/numerics/sources/README.md),
  which pins the exact edition behind each mandatory normative-definition
  reference this decision requires, and names those with no local copy.


## Context

Many tensor scalar formats are defined by standards or external ecosystems.
Canonical keys could place those authorities directly in the namespace, such
as `ieee::binary32@2019`, or Tiler could own the IR key while normatively
referencing the external definition, such as `tiler::f32@1`.

Standards organizations generally do not publish or govern Tiler-compatible IR
key registries. Document revisions also do not necessarily correspond to
semantic compatibility versions, and formats such as bfloat16 have no single
unambiguous namespace owner. Conversely, Tiler must not appropriate or rename a
project/vendor identity that is already published and deployed.

## Decision

Formats deliberately admitted into Tiler's built-in vocabulary use
Tiler-governed canonical keys. Each immutable canonical descriptor contains a
mandatory normative-definition reference including authority, document,
revision/profile, and exact format where applicable.

The Tiler key owns IR identity and compatibility; the external reference owns
the cited numerical definition. Public aliases such as `f32`, frontend enum
values, and source-format spellings resolve to the canonical key before
semantic admission and do not create additional identities.

Published key meanings are immutable:

- an incompatible semantic change requires a new key semantic version;
- a later standards revision proven semantically identical may be recorded as
  additional non-semantic provenance/equivalence evidence;
- canonical serialization records the key and validates its registered
  descriptor fingerprint;
- key identity never uses Rust discriminants, `TypeId`, provider addresses, or
  insertion order.

This policy applies only at initial built-in admission. An already-published
external project/vendor canonical identity remains external when Tiler later
recognizes or bundles support for it. Tiler does not mint an equivalent built-in
key or migrate graphs. External equivalence is explicit, versioned, and backed
by bit/value and conversion conformance; spelling or structural similarity is
not sufficient.

Before minting a built-in key, admission checks the registry and catalog for an
existing canonical owner of the same exact format. Exact Rust structures,
display syntax, and the external namespace registration API remain evolvable
implementation details.

## Consequences

- Tiler controls the stability of its portable built-in vocabulary.
- Normative provenance remains machine-readable without pretending standards
  bodies govern Tiler's namespace.
- Standards revisions can be evaluated for semantic compatibility rather than
  mechanically changing every graph key.
- External identities never change merely because Tiler's support level grows.
- Importers need explicit alias/equivalence and source-provenance handling for
  faithful round trips.
- Admission and external providers require collision, ownership, descriptor-
  fingerprint, and equivalence governance.

## Implementation boundary

Added 2026-08-01 by [`re-audit-adr-implementation-status-after-the-runtime-and-metal-landings`](../../tickets/re-audit-adr-implementation-status-after-the-runtime-and-metal-landings.md), which moved `implementation_status` from `not-started` to `partial`. It supersedes the 2026-07 audit's `undetermined` finding, whose two stated grounds are re-examined below: one is answered by the catalog that has since landed and the other still holds. This section states which clauses the value rests on and which it does not, read at `2aa0824`. It is a status record and adds no decision.

**Realized — one governed catalog is the only construction site for a built-in identity.** `crates/tiler-ir/src/semantic/catalog.rs` states every governed nominal scalar, the parameterized complex family, and the OCP microscaling schemes as table rows, "so a row is the only place a canonical key, its descriptor, and its normative reference can be stated. A second construction site would let two spellings of one identity drift apart, which is the failure ADR 0034's immutable-descriptor rule exists to prevent." Every key is `tiler`-namespaced and versioned (`:543`), and `register_builtin_dtype_catalog` at `:826` installs each row once, so a duplicate is a registration failure rather than a silent shadow.

**Realized — the mandatory normative-definition reference carries authority, document edition, exact format, and preserved source.** Each row states one: `IEEE 754-2019 binary32; source ieee-754-2019; tiler::f32@1` (`:360`), `RISC-V Unprivileged ISA version 20260120, chapter 25, BF16 extensions version 1.0, operand format; source riscv-unprivileged-isa-20260120; tiler::bf16@1` (`:388`), `OCP Microscaling Formats (MX) version 1.0, E8M0 scale data; source ocp-mx-v1.0; tiler::f8e8m0fnu@1` (`:471`). The reference is required by `ValueTypeDefinition` rather than optional, and the source ids resolve to the preserved primary-source record this decision's traceability names.

**Realized — a published key resolves to one immutable descriptor, and the descriptor is the identity subject.** `ValueTypeDefinition::canonical_descriptor` (`crates/tiler-ir/src/semantic/registry.rs:516`) encodes the family key, the normative reference, and the complete canonical facts under their own versioned domain separator, and deliberately excludes the provider, because "two providers registering byte-identical descriptors have registered the same meaning". A published version the catalog never minted, an alias spelling, a lookalike width, and an owner-namespaced identity of the same name each fail as `RegistryError::UnregisteredTypeAuthority` rather than resolving to a neighbouring row (`crates/tiler-ir/src/semantic/catalog/tests.rs:914`).

**Realized — identity never uses a Rust discriminant, `TypeId`, provider address, or insertion order.** A key is namespace, name, and semantic version; the catalog is iterated in canonical key order, and the frozen registry's snapshot identity is computed over an ordered map rather than the registration call sequence, which `crates/tiler-ir/src/semantic/registry.rs:2121` states where a reader would otherwise infer meaning from registration order.

**Realized — the alias and equivalence rule is a descriptor field rather than prose.** Every governed row carries `aliases-resolve-to-this-key-before-admission; external-equivalence-requires-versioned-conformance-evidence` (`crates/tiler-ir/src/semantic/catalog.rs:56`), stated once for the whole catalog because "a per-row restatement would be the same sentence copied thirty times, which is how one copy silently becomes wrong".

**Unrealized — the reference cannot separately represent its four parts.** `NormativeDefinitionRef` is a newtype over `String` validated only for non-emptiness and a byte bound (`crates/tiler-ir/src/semantic/registry.rs:203`, `:235`). Authority, document edition, revision or profile, and exact format are a documented semicolon convention that nothing parses or checks, so a reference naming no authority at all is admissible, and no consumer can ask which standard revision a format was pinned to without splitting a string. This half of the 2026-07 `undetermined` finding is unchanged.

**Unrealized — no same-format owner check runs before minting a built-in key.** The decision requires admission to "check the registry and catalog for an existing canonical owner of the same exact format". Nothing does: the formats that were correctly left external — `f8e4m3fnuz`, `f8e5m2fnuz`, `f8e4m3b11fnuz`, `f8e3m4`, `f8e4m3` — are recorded by a test asserting they are *not* registered (`crates/tiler-ir/src/semantic/catalog/tests.rs:932`), which preserves a judgement already made by hand rather than performing the check on the next row somebody adds.

**Unrealized — no external identity, alias table, or equivalence evidence exists to exercise the policy.** The catalog registers no external owner-namespaced identity and no versioned equivalence record, so "an already-published external canonical identity remains external" and "external equivalence is explicit, versioned, and backed by bit/value and conversion conformance" are declared policy with nothing yet governed by them.

## Alternatives considered

Authority-qualified keys make provenance visible but place compatibility in
namespaces Tiler does not control and overfit document revision to semantic
versioning. URI-style authorities have the same governance problem with more
serialization complexity. Renaming external identities when they become
officially supported breaks equality, artifacts, caches, and round trips and is
already rejected by ADR 0027.
