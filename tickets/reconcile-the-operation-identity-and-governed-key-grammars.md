---
id: reconcile-the-operation-identity-and-governed-key-grammars
title: Reconcile the operation-identity and governed-key grammars
status: awaiting-decision
priority: p2
dependencies: []
related: [reconcile-the-two-target-profile-key-grammars]
scopes: [implementation/ir, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [identity, validation, extensions, decision, needs-tom, public-boundary]
---
## User-visible outcome

A capability key composed from a legally registered operation is either always a legal governed key, or the registration that would compose an illegal one is refused where it is made rather than at packaging time.

## Why this slice exists

**Fact, measured at `c142991` plus this branch's grammar change.** `crates/tiler-compiler/src/lowering.rs`'s `governed_capability_key` composes `tiler.capability.{family}.{namespace}.{name}.v{version}` from an `OpKey`. `OpKey`'s components are validated by `validate_component` at `crates/tiler-ir/src/semantic/types.rs:1378-1399`, which admits `byte.is_ascii_alphanumeric()` — **uppercase included** — and `MAX_IDENTITY_COMPONENT_BYTES` = 255 per component. `tiler_artifact::program`'s governed keys admit ASCII lowercase, digits, `.`, `-`, `_` within `MAX_GOVERNED_KEY_BYTES` = 256. Three grammars, not the two `reconcile-the-two-target-profile-key-grammars` named.

**Measurement**, a throwaway integration test in `tiler-build` (which depends on both crates), run once and deleted:

- `OpKey::new("Acme", "MyOp", 1)` succeeds. The composed key is `tiler.capability.scalar.Acme.MyOp.v1`, and `CapabilityKey::new` returns `Err(NoncanonicalKeyByte { kind: Capability, index: 24, value: 65 })`.
- `OpKey::new(&"a".repeat(255), &"a".repeat(255), 1)` succeeds. The composed key is 538 bytes, and `CapabilityKey::new` returns `Err(KeyTooLong { kind: Capability, bytes: 538, limit: 256 })`. **This half predates the grammar change** and is reachable today.

`register_scalar_lowering` and `register_index_access` (`crates/tiler-compiler/src/capability.rs:888`, `:918`) are `pub` and take an `OpKey`, so both are reachable through a public boundary. `governed_capability_key` returns `String` and is infallible, so neither can be reported where it is caused; the refusal lands at `crates/tiler-build/src/metal_plan.rs:283` — and as a **panic** at `prototypes/serial-sum-run/src/proof.rs:3235` and `spikes/cache/build-tool-exercise/envelope/src/lib.rs:126`, which `.expect()` the wrap.

**Inference.** Refusing an uppercase capability key is correct: it would compare unequal to the key every reader sees, which is what the governed alphabet exists to prevent. What is wrong is the site. A derivation that turns a legal input into an identity its consumer refuses has to fail at the derivation, or the input grammar has to be the one that says no.

Every in-tree operation key is lowercase and at most 66 bytes, so nothing in the tree is affected today; this is reachable only by an out-of-crate registration.

## Implementation keys

- Decide between the two shapes before changing a byte. Narrowing `validate_component` to lowercase reconciles all three grammars at the source, but it is a wider change than capability keys — it governs `TypeKey`, `OpKey`, `ScalarOpKey`, provider identities, and every canonical type identity, and it is a **public boundary narrowing** in `tiler-ir` that refuses inputs legal today. Making `governed_capability_key` fallible keeps the identity grammar as it is but adds a `LoweringRegistryError` variant, which is a public boundary widening in `tiler-compiler`. Both are Tom's under ADR 0075.
- A lowercase *fold* is not a third option and must be eliminated explicitly: folding makes `Acme` and `acme` mint one capability key for two operations, which is a silent identity collision — strictly worse than the refusal.
- The length half is independent of the alphabet half and is live today. Nothing between the composition and the wrap bounds the composed length, and two of the three wrap sites panic.
- Whichever shape wins, the `.expect()` at the two spike/prototype wrap sites is a panic on a producer-side input and should become a typed refusal.

## Decision packet — 2026-08-09

- **Option A — narrow every operation/type identity component to the governed lowercase grammar.** This prevents invalid composition at the source but rejects public inputs legal today across more identities than capability keys.
- **Option B — make capability-key composition fallible (recommended).** Preserve the broader operation grammar, add a typed `LoweringRegistryError` for noncanonical or overlong composed keys, and remove downstream `.expect()` panics. This localizes the restriction to the consumer that actually has it.

Lowercasing is excluded because it collides distinct operation identities. Tom must select the public-boundary change before implementation.

## Closes when

The composition cannot produce a key its consumer refuses — either because the input grammar no longer admits one, or because the composition reports it — with the decision recorded and the failing case watched failing.
