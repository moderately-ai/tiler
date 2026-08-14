---
id: decide-the-selected-lowering-capability-subject-rust-surface
title: Decide the selected lowering capability subject Rust surface
status: done
priority: p1
dependencies: [reconcile-the-operation-identity-and-governed-key-grammars]
related: [replace-flat-selected-lowering-capability-keys-with-structured-subjects]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [identity, schema, public-boundary, decision, needs-tom]
---
## User-visible outcome

The already-accepted structured selected-lowering subject has one exact Rust surface at the compiler and neutral artifact boundaries. The implementation therefore does not invent a source-breaking public signature, a second family-name authority, or a layer-crossing type dependency.

## Authority and exact-base audit

[`reconcile-the-operation-identity-and-governed-key-grammars`](reconcile-the-operation-identity-and-governed-key-grammars.md) is accepted semantic and identity authority: compiler subject = closed lowering family plus exact `OpKey`; provider and capability revision remain separate; artifact subject = governed family key plus exact `OpKey`; no flattened parser, case fold, truncation, digest substitution, default, or fallback. It permits but does not require a diagnostic display. This ticket decides only the Rust spelling and cross-layer projection that decision did not name.

The source audit was run before production edits at exact implementation base `72b1357f892335e4883494c0e1906be89998258b`. Ticket-only audit commits after that base do not change these source Facts.

- **Fact — current compiler ownership.** `tiler-compiler` depends on `tiler-ir`, exports the already-public `capability` and `session` modules, and does not depend on `tiler-artifact`. `capability::LoweringFamily` is a public `#[non_exhaustive]` one-variant enum with public authoritative `key_token()`; `ResolvedLoweringCapability` retains exact family, `OpKey`, provider, and capability revision. Private `request::LoweringProviderIdentity` instead retains provider, a flattened `String`, and capability revision; public `session::SelectedCapability` is a borrowed wrapper exposing `provider()`, `capability_key() -> &str`, and `capability_revision()`. Read `crates/tiler-compiler/src/{lib,capability,request,lowering,session}.rs`; locate the definitions with `rg -n 'pub enum LoweringFamily|struct ResolvedLoweringCapability|struct LoweringProviderIdentity|struct SelectedCapability|capability_key' crates/tiler-compiler/src`.
- **Fact — exact operation ownership.** `OpKey` is defined in `crates/tiler-ir/src/semantic/operation.rs`; its `new` and `from_owned` delegate to the private structured key in `semantic/types.rs`. Each namespace and name is nonempty, at most 255 bytes, and admits ASCII alphanumeric bytes plus non-leading `.`, `_`, and `-`; the semantic version is nonzero. Reproduce the two-layer definition with `rg -n 'pub struct OpKey|pub fn from_owned|pub fn new' crates/tiler-ir/src/semantic/operation.rs` and `rg -n 'MAX_IDENTITY_COMPONENT_BYTES|fn validate_component|fn from_owned' crates/tiler-ir/src/semantic/types.rs`, then read both definitions.
- **Fact — current artifact ownership.** `tiler-artifact` depends on `tiler-ir` and not `tiler-compiler`; its public `program` module re-exports `CapabilityKey` and `SelectedProvider`. `CapabilityKey` is currently a private-`String` governed key under the shared lowercase 256-byte grammar. Public `SelectedProvider` is a caller-constructed leaf input record with three public fields and no `#[non_exhaustive]`; its `capability: CapabilityKey` is the only production model use of that type. Its canonical key, manifest encoder, allocation-budget census, and decoder all consume one capability text field. Read `crates/tiler-artifact/src/program/{keys,model,builder}.rs` and `crates/tiler-artifact/src/program/codec/{budget,encode,decode,error,model}.rs`; locate with `rg -n '\bCapabilityKey\b|provider\.capability|SelectedProvider' crates/tiler-artifact/src --glob '*.rs'`.
- **Fact — exact adapter population.** Exactly eight compiler-to-artifact adapters reconstruct the text key: `crates/tiler-build/src/plan_artifact.rs`, `prototypes/serial-sum-run/src/proof.rs`, `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs`, `spikes/cache/build-tool-exercise/envelope/src/lib.rs`, `spikes/cache/envelope-digest-coverage/harness/src/envelope.rs`, `spikes/cache/hot-path-efficiency/harness/src/envelope.rs`, `spikes/artifacts/decoder-allocation/harness/src/envelope.rs`, and `spikes/runtime/backend-provider-portfolio/src/portfolio.rs`. The first and last already propagate `ArtifactBuildError`; the scalar vertical erases the cause as `VerticalError::HostProfile`; the other five assert the conversion inside fixtures. The exact eight-hit reproducer is `rg -n 'CapabilityKey::new\(selected\.capability_key\(\)\)' crates prototypes spikes --glob '*.rs'`. Direct artifact tests, the runtime fixture, and artifact/proof doctests construct `SelectedProvider` without a compiler view and are a separate population found by `rg -n 'SelectedProvider \{' crates prototypes spikes --glob '*.rs'`.
- **Fact — adapter dependency census.** All eight adapters already depend directly on `tiler-compiler`, `tiler-artifact`, and `tiler-ir`. Only `tiler-build` itself, the serial-sum prototype, and the runtime portfolio already have `tiler-build`; centralizing projection there would add five high-level build-orchestration dependency edges to otherwise narrower target-profile, cache, artifact-allocation, and build-exercise crates. Reproduce from the eight owning manifests with `rg -n '^tiler-(artifact|build|compiler|ir)' crates/tiler-build/Cargo.toml prototypes/serial-sum-run/Cargo.toml spikes/target-profiles/scalar-cpu-vertical/Cargo.toml spikes/cache/build-tool-exercise/envelope/Cargo.toml spikes/cache/envelope-digest-coverage/harness/Cargo.toml spikes/cache/hot-path-efficiency/harness/Cargo.toml spikes/artifacts/decoder-allocation/harness/Cargo.toml spikes/runtime/backend-provider-portfolio/Cargo.toml`.
- **Fact — construction invariants.** There is no invariant relating a valid governed family text under the artifact key grammar to a valid `OpKey`; their pair is leaf value data that becomes provenance only when `ArtifactProgramBuilder::select_provider` binds it to an offered provider. `LoweringFamily::key_token()` is already the compiler's exhaustive family-to-governed-token authority. The existing artifact `governed_key!` macro generates the receiving `new`, `from_owned`, `as_str`, and `Display` surface for each governed key type declared through it.
- **Proposal — family-key projection consequence.** Declare `CapabilityFamilyKey` through that existing artifact macro. Its generated `new` becomes the sole receiving grammar check; an adapter that forwards `LoweringFamily::key_token()` to it and clones the exact `OpKey` makes no naming or validation decision of its own.
- **Fact — owned decode path.** `Cursor::text()` returns an owned `String`, and `OpKey::from_owned(namespace, name, version)` validates and retains two owned components. Calling `OpKey::new` on the cursor strings would copy both before dropping the originals. Read `Cursor::{text,u32}` in `crates/tiler-artifact/src/program/codec/decode.rs` and `OpKey::from_owned` in `crates/tiler-ir/src/semantic/operation.rs`.
- **Fact — no live subject-display consumer.** The current flat key has `Display` only because the governed-key macro gives every text key declared through it that surface. No accepted contract or live consumer requires a formatted composite structured subject. Existing `LoweringFamily` and `OpKey` already retain their own displays for component diagnostics. A new composite `Display` would therefore be a new attractive string spelling with no demonstrated use.
- **Proposal — future family-key display consequence.** The proposed macro-declared `CapabilityFamilyKey` receives the same component-level `Display` as existing governed artifact keys; neither proposed composite subject receives `Display`.
- **Fact — compatibility is not authority here.** ADR 0075's “The compatibility premise, considered and rejected” records that Tiler is version `0.0.0`, not publishable, and has no external consumer; Tom rejected compatibility framing for this phase because `cargo check` enumerates every in-workspace call site. Both artifact naming candidates change the type from one text field to two structured fields and therefore require complete workspace construction/codec migration regardless of whether the import name remains. Preserving `CapabilityKey` saves no supported consumer contract.
- **Fact — public convention.** ADR 0074 convention 5a explicitly excludes caller-constructed input records from `#[non_exhaustive]`; convention 6 permits public fields on leaf value-data descriptors; convention 3 requires separately framed identity fields and exhaustive encoders. ADR 0075 permits a new conforming public provenance/identity record without separate approval, but always routes a source-breaking existing public signature to Tom. Replacing `SelectedCapability::capability_key()` and either repurposing or retiring public `CapabilityKey` are therefore Tom's.
- **Fact — dependency direction.** Putting the compiler subject in `tiler-artifact` would add a compiler-to-artifact dependency; putting it in `tiler-compiler` would make the neutral artifact depend on the compiler; putting a lowering-family subject in `tiler-ir` would make public semantic IR own physical-lowering vocabulary. A new shared bottom crate adds a public namespace and dependency edges while still needing the closed-family-to-governed-family projection. None fits current ownership.
- **Fact — identity/schema consequence.** `PROVIDER_KEY_DOMAIN` remains `tiler.artifact-program.provider.v2`, while `ARTIFACT_DOMAIN` and `MANIFEST_SCHEMA` are already `tiler.artifact-program.v17` and `17.0`. Any correct structured provider row moves exactly those to provider `v3`, artifact `v18`, and manifest `18.0`; compiler lowering-registry, semantic-graph, refinement, kernel-program, and unrelated artifact domains do not change merely because this carrier changes.
- **Fact — public codec classification is intentionally coarse.** `ArtifactCodecFailure::from` in `crates/tiler-artifact/src/program/codec/view.rs` converts each internal `ArtifactCodecError` to a public action class after capturing `error.to_string()` as `detail`. Invalid governed/interface/provider/shape/alignment component constructors are classified as `ArtifactCodecFailure::Malformed { detail }`; public `ArtifactCodecFailure` implements `Error` without a `source`. A new invalid `OpKey` is the same readable-framing/invalid-component class. Its `TypeIdentityError` can remain `ArtifactCodecError::InvalidOperationKey`'s crate-private `Error::source`, but that typed cause does not and must not be claimed to survive the public classifier.

## Decision gate

### Compiler surface options

1. **Status quo or a narrower registration-time text check — eliminated.** The dotted-component pair remains equal after flattening, or legal uppercase/maximum-length `OpKey` values are refused for an artifact text grammar. Either silently aliases identity or narrows accepted semantic identity contrary to the accepted decision.
2. **Split `SelectedCapability::family()` and `operation()` accessors, with no named subject — eliminated.** Each value can be exact, but there is no single typed value for the accepted subject. A translator can combine the family from one selected row with the operation from another and still construct valid artifact input. That is weaker fail-closed strictness than an equally cheap borrowed subject.
3. **One compiler-owned stored subject returned by borrow — survives and is recommended.** Store `LoweringCapabilitySubject` whole inside `LoweringProviderIdentity` and expose `SelectedCapability::subject() -> &LoweringCapabilitySubject`. The same value participates in equality/order/dedup and crosses the public boundary; provider and capability revision remain sibling fields. It adds no per-read allocation and prevents accidental cross-pairing.
4. **Repurpose `SelectedCapability` itself as the subject or flatten family/operation into it — eliminated.** `SelectedCapability` also names provider and capability revision. Treating that whole selected row as the capability subject conflates subjects ADR 0072 and the accepted dependency keep separate; adding only component accessors reduces to option 2.
5. **Return an owned subject copy from each iterator item — eliminated.** It is equally correct but clones two operation strings on every read while the plan already retains the owned subject. A borrowed return is no worse on correctness or maintenance and strictly better in host work.
6. **Move/share the compiler subject across crates — eliminated.** Existing dependency direction rules out either owner crate; `tiler-ir` would acquire physical-lowering vocabulary; a new crate adds a public namespace and edges without removing the necessary layer projection.

### Artifact shape and naming options

1. **Keep or parse the text `CapabilityKey` — eliminated.** No injective legacy grammar exists, and the accepted decision forbids a parser/fallback.
2. **Put `capability_family` and `operation` directly beside provider and revision — eliminated.** The bytes can be framed correctly, but there is no typed artifact capability subject and callers can accidentally cross-pair components. Nesting the same fields costs no runtime and is stricter.
3. **Repurpose `CapabilityKey` as `CapabilityKey { family: CapabilityFamilyKey, operation: OpKey }` — eliminated as dominated.** This can be correct, and structured `OpKey`/`TypeKey` refute any claim that “key” itself implies display text. Its only surviving benefit is keeping an import/type name while every constructor, field access, codec, fixture, and identity pin must still migrate. ADR 0075 assigns that compatibility benefit near-zero authority at this phase. It is weaker for long-term maintenance because compiler and artifact would use different category nouns for the same accepted semantic subject. Reconsider only if Tiler acquires an external consumer or publishable crate before implementation, firing ADR 0075's compatibility trigger.
4. **Retire text `CapabilityKey` and add `LoweringCapabilitySubject { family: CapabilityFamilyKey, operation: OpKey }` — survives and is recommended.** Compiler and artifact then name the same semantic subject alike, while `tiler_compiler::capability::LoweringCapabilitySubject` versus `tiler_artifact::program::LoweringCapabilitySubject` makes layer ownership explicit and the differing `family` types fail loudly at the projection. It is equal to option 3 on correctness, strictness, host runtime/memory, and required migration, and strictly better on semantic maintenance under the accepted no-compatibility premise. Its strongest counterargument is same-named imports when a consumer uses both types; ordinary qualified imports or a local alias resolves that compile-time ambiguity without changing public vocabulary. Evidence of an external consumer, a publishable crate, or repeated call-site confusion that survives explicit imports would reverse the ranking. Follow-up is exactly the implementation ticket with old `CapabilityKey` removed and every artifact construction migrated.
5. **Hide either artifact pair behind a constructor and private fields — eliminated.** There is no cross-field invariant beyond the independently validated types. It adds accessors, weakens compile-time discovery of future identity-field growth, and conflicts with the existing caller-built `SelectedProvider` leaf posture without increasing strictness.
6. **Privatize `SelectedProvider` behind a new constructor — eliminated.** Provider availability is already checked by the builder, while provider and capability revision must remain independent. Privatization expands the breaking surface without adding an invariant.
7. **Share the compiler Rust type — eliminated by dependency direction.** The compiler family is a closed enum and the neutral artifact family is governed text; making either universal erases a layer distinction or adds a wrong dependency.

### Display options

1. **Add public composite `Display` — eliminated.** It contributes nothing to equality, validation, framing, or refusal; no live consumer or contract requires it; the three components already have diagnostic displays. It would create a parse-like spelling that later consumers could accidentally treat as authority and would add another public behavior to preserve.
2. **No composite `Display` — survives and is recommended.** Both subject types derive `Debug` and expose their typed components. Diagnostics may format those components locally without defining a public composite grammar. Reconsider only when a concrete public diagnostic consumer demonstrates why typed component access and `Debug` are insufficient.

### Cross-layer projection ownership

1. **Compiler-owned family token plus artifact validation at every adapter — survives and is recommended.** The one governed contract is exact: read the borrowed compiler subject; pass `subject.family().key_token()` only to `CapabilityFamilyKey::new`; clone `subject.operation()` whole; forward provider and capability revision unchanged. The compiler remains the sole family-token authority, the artifact remains the sole receiving-grammar authority, and an adapter does not format, parse, match a family, or validate an operation. The two public artifact fields make future subject growth a build error at all eight sites. All eight projection expressions propagate `ArtifactBuildError`; none wraps the family check in `expect` or maps it to an unrelated class. The five assertion-oriented fixtures instead return the typed error from their internal assembly helper and may assert only at their pre-existing outer fixture boundary. The scalar vertical adds an `ArtifactBuild(ArtifactBuildError)` cause-preserving error arm instead of `HostProfile`.
2. **Public helper in `tiler-build` used by all eight — eliminated as dominated at this population.** It can be correct, and its strongest argument is one function body rather than eight typed record literals. But five adapters do not depend on `tiler-build`; adding that high-level orchestration crate also pulls their otherwise unrelated cache/Metal/AOT dependency surface. Because the direct option contains no mapping choice—only an authoritative token, a checked family constructor, and an exact `OpKey` clone—the helper removes no second authority or refusal path. It adds one public function plus five dependency edges for mechanical field forwarding that compiler and artifact types already make exhaustive. Reconsider if another production crate needs this projection, the adapter count grows beyond these bounded fixtures/spikes, or drift survives the required source census and end-to-end tests.
3. **New neutral bridge crate — eliminated.** It would add a public crate and eight dependency edges to join two layer-owned leaves; it cannot own the compiler's closed-family token or the artifact's grammar, so it reduces no authority and is strictly more infrastructure than option 1.
4. **Artifact- or compiler-owned conversion from the other layer's type — eliminated.** Either direction introduces the forbidden compiler/artifact dependency. A shared conversion in `tiler-ir` would put physical-lowering vocabulary in semantic IR.
5. **Adapter-local formatting, family matches, parsing, defaults, or lossy error remaps — eliminated.** Each creates another identity or validation authority and can drift silently. The exact option-1 expression is field forwarding, not adapter-owned mapping.

Deferral is required until Tom accepts the source-breaking public surface, but it is not another API candidate: source reading, ADR 0075, and the dependency census leave one nondominated surface.

## Nondominated frontier

Compiler option 3, artifact option 4, no-`Display` option 2, and projection option 1 form the sole nondominated surface.

| Exact surface | Correctness and fail-closed strictness | Long-term maintenance and compatibility | Tiler host runtime/memory | Identity, schema, and public consequences |
| --- | --- | --- | --- | --- |
| **Compiler and artifact layer-owned `LoweringCapabilitySubject` records** | Exact typed pair at each layer; no parser/display/fallback; public artifact fields expose only validated component types. | One semantic noun, explicit crate paths, and a fail-loud family representation boundary. Retiring the old import costs only enumerated in-workspace edits under ADR 0075. | One governed-family string and two operation strings per artifact row; borrowed compiler reads; no composite display work. | Provider `v3`, artifact `v18`, manifest `18.0`; artifact text `CapabilityKey` disappears and `CapabilityFamilyKey` is added. |

## Exact recommended surface

### Compiler — included

- Add public `tiler_compiler::capability::LoweringCapabilitySubject` with private owned fields `family: LoweringFamily` and `operation: OpKey`; derive `Clone`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, and `PartialOrd`. Do not mark this exact identity pair `#[non_exhaustive]`: its fields are private, and another subject component requires a new identity decision rather than additive output growth.
- Give it no public constructor and no `Display`. A crate-private constructor is used at capability resolution. Public `family(&self) -> LoweringFamily` and `operation(&self) -> &OpKey` are its only semantic accessors.
- Replace `SelectedCapability::capability_key() -> &str` with `pub fn subject(self) -> &'a LoweringCapabilitySubject`, borrowing from the selected plan for the view's existing `'a` lifetime. Keep the existing by-value view receivers and `provider() -> &'a ProviderIdentity` / `capability_revision() -> u32` surfaces unchanged.
- Store the subject whole in private `LoweringProviderIdentity`; use it directly for equality/order/dedup and public selection evidence.
- Keep existing public `LoweringFamily` `#[non_exhaustive]` and its authoritative `key_token()` unchanged.

### Compiler — excluded

- No direct `family()` or `operation()` convenience methods on `SelectedCapability`; no session-module re-export of the subject; no `Display`, `as_str`, parser, public constructor, signature, provider, capability revision, fold, escape, truncation, digest, or compatibility alias on the subject.
- Do not change registration, ambiguity resolution, provider selection, or the one-signature-per-provider/family/operation law.

### Neutral artifact — included

- Add governed `CapabilityFamilyKey` under the existing lowercase/digit/`.`/`-`/`_`, 256-byte grammar, with the governed-key macro's `new`, `from_owned`, `as_str`, and `Display` surface. Rename public `ArtifactKeyKind::Capability` to `CapabilityFamily` so typed key errors name the actual component.
- Retire artifact text `CapabilityKey`. Add `tiler_artifact::program::LoweringCapabilitySubject` with exactly `pub family: CapabilityFamilyKey` and `pub operation: tiler_ir::semantic::OpKey`; derive `Clone`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, and `PartialOrd`; and give it no `#[non_exhaustive]`, constructor, private fields, `Display`, parser, or string accessor.
- Keep `SelectedProvider` as the existing three-public-field caller input; change only the representation of its `capability` field. Provider and capability revision remain sibling fields.
- Frame family, operation namespace, operation name, and semantic version independently in provider canonical identity and manifest bytes.
- Decode family with `CapabilityFamilyKey::from_owned(cursor.text()?)`. Read operation namespace and name into owned strings plus the version, then call `OpKey::from_owned(namespace, name, semantic_version)`. Map only that failure to crate-private `ArtifactCodecError::InvalidOperationKey { cause: TypeIdentityError }` and include it in that internal error's `Error::source` as `Some(cause)`. Add the new variant exhaustively to `ArtifactCodecFailure::from` as the existing public `Malformed { detail }` class; the public classifier intentionally retains only rendered detail and no typed source. This retains cursor ownership, avoids the two copies `OpKey::new` would make per provider row, and does not expand the public rejection vocabulary.

### Neutral artifact — excluded

- No compiler-subject re-export, `tiler-compiler` dependency, shared subject in `tiler-ir`, `OpKey` re-export solely for this field, composite `Display`, subject parser/string accessor, legacy single-text decode arm, or builder/selection/capability-revision semantic change.
- No compatibility overload, deprecated alias, or old `CapabilityKey` constructor/accessor survives.

### Exact eight-adapter projection

Every adapter uses the artifact record in this exact checked projection, with no helper that owns a second token map:

```rust
let subject = selected.subject();
SelectedProvider {
    provider: selected.provider().clone(),
    capability: tiler_artifact::program::LoweringCapabilitySubject {
        family: CapabilityFamilyKey::new(subject.family().key_token())?,
        operation: subject.operation().clone(),
    },
    capability_revision: selected.capability_revision(),
}
```

`crates/tiler-build` and the runtime portfolio retain direct typed `ArtifactBuildError` propagation. The scalar vertical adds cause-preserving `VerticalError::ArtifactBuild(ArtifactBuildError)`. The five fixture/proof adapters make their internal assembly path return `Result` and move any intentional invariant assertion to the existing outer fixture boundary, after the full typed cause is retained. A source census must find exactly the eight `CapabilityFamilyKey::new(subject.family().key_token())` projections and no adapter-local family literal, `format!`, parse, `expect`, or unrelated error mapping around them.

## Negative controls and follow-up

The implementation ticket must perturb the subject, never its assertions:

- the `("a.b", "c", 1)` / `("a", "b.c", 1)` pair remains distinct through compiler census, borrowed public subject, all eight projections, artifact rows, codec round-trip, provider-key identity, artifact identity, proof subject, envelope digest, and cache subject;
- family, namespace/name boundary, operation version, provider, and capability revision move only identities whose exact subject contains them, with each failure naming the changed subject;
- uppercase and 255-byte legal operation components round-trip without a text conversion;
- corrupt family, namespace, name, and version fields independently; quote the governed-key refusal or internal `InvalidOperationKey` with its `TypeIdentityError` source, and separately require the public decode result to be `Malformed { detail }` with no source; a one-field legacy flattened row fails framing/schema rather than parsing;
- temporarily change only existing `LoweringFamily` or `OpKey` diagnostic display punctuation, prove compiler equality/order plus provider/artifact identity pins remain unchanged, then restore it; the new composite subjects have no `Display` to mutate;
- a compile-fail fixture attempts public construction of the compiler subject; a source census proves every artifact construction uses typed components and no equality/order/dedup/cache/receipt/artifact identity consumer formats them.

[`replace-flat-selected-lowering-capability-keys-with-structured-subjects`](replace-flat-selected-lowering-capability-keys-with-structured-subjects.md) is the complete implementation follow-up. It owns `LoweringCapabilitySubject`, the `v3`/`v18`/`18.0` steps, codecs, contracts, fixtures, all eight adapter migrations, typed internal error propagation and truthful public classification, derived pins, perturbations, and exact-base gates. No additional work is implicit.

## Accepted answer — 2026-08-14

**Accepted by Tom in the live Codex conversation, relayed by the coordinating agent:** the sole nondominated exact Rust surface under “Exact recommended surface” — layer-owned compiler and artifact `LoweringCapabilitySubject` records, no composite `Display`, the checked eight-adapter projection, owned `OpKey` decode, existing public `Malformed { detail }` classification, and all explicit exclusions.

Recommendation: **accept**. It gives one accepted semantic subject one name, keeps its different layer representations explicit in the type paths, and adds no compatibility, display, dependency, or public-error surface without authority.

## Outcome

The exact public surface is accepted. [`replace-flat-selected-lowering-capability-keys-with-structured-subjects`](replace-flat-selected-lowering-capability-keys-with-structured-subjects.md) owns the complete implementation, identity/schema migration, adapter conversion, typed internal failure, truthful public classification, and evidence population.
