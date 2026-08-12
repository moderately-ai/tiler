---
id: frame-provider-identities-before-using-them-as-explain-keys
title: Frame provider identities before using them as explain keys
status: todo
priority: p1
dependencies: []
related: [reconcile-the-operation-identity-and-governed-key-grammars, replace-flat-selected-lowering-capability-keys-with-structured-subjects, emit-typed-opaque-call-frontier-rejection-records]
scopes: [implementation/ir, implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [identity, explain, diagnostics, correctness, implementation]
---
## User-visible outcome

Explain evidence names every registered provider injectively and accepts the full provider-identity vocabulary rather than collapsing dotted component boundaries or refusing a legal provider only because its rendered key exceeds an unrelated display bound.

## Why this slice exists

**Fact at `b03d1e7699d4f7cfbfb6ee7a903e2d2fbe16af18`.** `ProviderIdentity` validates and canonically encodes namespace and name as distinct length-framed components. `ProviderRef::registered` in `crates/tiler-compiler/src/explain.rs` instead joins them with `format!("{}.{}", namespace, name)` and validates the result against the explain key's 255-byte ceiling.

**Inference.** Legal providers `("a.b", "c")` and `("a", "b.c")` therefore become one explain key. Two maximum legal components can also be refused after registration. Explain text is not artifact identity, but retained evidence must still name the authority it claims without collision or late refusal.

## Required delivery

- Perform the repository-required source-first per-Fact audit at the implementation base; the Fact above is stale until then.
- Replace the delimiter-composed provider key with a structured or opaque received-identity carrier whose canonical bytes preserve the namespace/name boundary and revision.
- Keep human-readable rendering separate from equality and canonical explain evidence.
- Reconcile explain encoding/version/pins and every provider-ref consumer without narrowing `ProviderIdentity` or introducing a default/fallback provider.
- Perturb the dotted boundary pair, maximum component sizes, and revision independently with assertions unchanged, and quote each failure.

## Non-goals

Changing artifact provider provenance, provider installation/selection policy, provider precedence, or the selected lowering capability subject owned by the related implementation ticket.

## Closes when

Every legal registered provider can be represented in explain evidence, distinct structured identities stay distinct, all encoded evidence and pins reconcile, and independent review confirms presentation text is not authority.

## Source-first Fact audit — 2026-08-12

Audited at exact main `65b0a9be1869c1cf0185381387b65adb8042ebee` after reading the complete ticket, the provider identity owner, every `ProviderRef` construction and comparison site, all provider-bearing trace encoders, the renderer, the composite explain identity, the pipeline consumers, ADRs 0072–0074, and the governed optimizer contract.

1. **Verified — the source Fact remains live.** `ProviderIdentity`, anchor `pub struct ProviderIdentity`, validates namespace and name independently, each against `MAX_IDENTITY_COMPONENT_BYTES = 255`, and its private `encode` length-frames both components before the provider revision. `ProviderRef::registered`, anchor `References a registered provider by its governed namespaced identity`, still formats `namespace.name` and then subjects the joined string to the unrelated explain-wide `MAX_KEY_BYTES = 255` bound.
2. **Verified — delimiter collision is reachable.** The shared semantic component grammar admits `.` after the first byte. Legal identities `("a.b", "c", 1)` and `("a", "b.c", 1)` therefore become equal `ProviderRef { key: "a.b.c", revision: 1 }` values. `allowed_providers.contains`, rule-provider equality, sound-proof receipt equality, canonical trace bytes, and rendered evidence all lose the distinction.
3. **Verified — maximum legal identities refuse late.** Two legal 255-byte components render to 511 bytes before revision and fail `ProviderKey::new`. The failure occurs while constructing `ExplainWriter` or recording a provider-attributed rule, after the provider has already been admitted elsewhere. The complete structured identity fits the trace's aggregate byte bound; a separate 255-byte display-key limit is not an authority bound.
4. **New verified defect — registered and builtin authority classes collide.** `ProviderRef::builtin()` is the flat key `tiler.compiler` at revision 1, and `is_builtin()` recognizes only those two flattened fields. The legal registered identity `ProviderIdentity::new("tiler", "compiler", 1)` produces the same value and is consequently classified as the compiler's own authority. Reserving that semantic provider spelling now would narrow an already-public identity grammar and would still leave the general dotted-boundary defect.
5. **Verified — presentation currently participates in authority.** `push_record` and `encode_basis` write `provider.key` and revision into canonical evidence; membership and equality compare the same `ProviderRef`; `VerifiedExplainTrace::render` prints it. The display spelling is therefore not merely reused for convenience: it is the current equality and identity subject.
6. **False — this ticket owns a consequential public Rust boundary.** ADR 0073 keeps the explain module private; `ProviderRef`, `ProviderKey`, `RuleRef`, writer, records, and trace readers are all `pub(crate)`. The accepted reconsideration trigger—another crate reading canonical traces—has not fired. Reusing the existing public `tiler_ir::semantic::ProviderIdentity` changes no public constructor or provider grammar. The `public-boundary` tag was removed; this is an internal identity repair with deliberate canonical-evidence versioning.
7. **Verified — the encoding population is small and total.** Provider authority reaches canonical trace bytes in two places: each rule head in `push_record`, and a `SoundProof` receipt in `encode_basis`. Rendering has one provider site. Construction is centralized through builtin, lowering, and registered constructors. A single total `encode_provider_ref` and a single renderer can cover the population; no artifact, cache, request-subject, provider-selection, or precedence grammar consumes `ProviderRef`.
8. **Verified — the existing trace schema must step.** Replacing a previously encodable registered provider's flat payload with tagged structured components changes records that schema 9 can already encode, so `EXPLAIN_SCHEMA_VERSION` must move from 9 to 10. The composite explain identity length-frames nested trace identities, so its grammar and schema remain unchanged while its values move transitively. It must not duplicate the provider fields.
9. **Verified — unambiguous rendering moves existing output.** The current renderer prints `provider=<flat>@<revision>`. Rendering the authority class and the existing `ProviderIdentity` display form, for example `provider=registered:namespace::name@revision`, separates presentation from equality and distinguishes it from `provider=compiler:tiler.compiler@1`. `EXPLAIN_RENDERER_VERSION` must step from 7 to 8; because the composite renderer embeds nested rendered traces, its renderer version must also step if its contract promises visible changes under its own header. No canonical identity may be derived by parsing either spelling.

## Corrected implementation boundary — 2026-08-12

One design dominates the string and opaque-byte alternatives:

- Replace the private product with a closed private sum, conceptually `ProviderRef::Compiler | ProviderRef::Registered(ProviderIdentity)`. The compiler arm has no caller-set revision or key; its governed revision stays one named constant used by the encoder and renderer. The registered arm retains the already-validated identity whole.
- Remove `ProviderKey` and `KeyKind::Provider` if their complete current census remains provider-reference-only. Explain's generic free-text keys continue to use their own bound; a provider identity no longer pretends to be one.
- Canonically encode an explicit authority-class tag. The registered arm then length-frames namespace and name and writes revision. Both `push_record` and `encode_basis` must call the same exhaustive helper. The enum and helper remain same-crate total maps, so a future authority class stops the build at every owning match.
- Render through a separate exhaustive helper with an explicit class prefix and the existing unambiguous `namespace::name@revision` provider display. Rendering never feeds equality, membership, ordering, or canonical bytes.
- Store allowed registered authorities as an exact structured set rather than a flattened vector when touching the field. This removes duplicate identities and changes membership from linear to logarithmic without creating provider precedence; it does not impose a provider count or outcome budget.
- Step trace schema 9→10 and trace renderer 7→8, reassess the composite renderer header as described above, update the version ledger, and rebaseline only explain identities/render pins. Artifact, cache, request, semantic, lowering, and selected-provider identities do not move.

## Ranked alternatives

1. **Closed `Compiler | Registered(ProviderIdentity)` sum with structured encoding and separate rendering.** Best correctness: it is injective over both identity components and authority class. Best maintainability: one existing authority type, exhaustive matches, no duplicated grammar. Host cost is neutral to better in membership checks; retained records may clone one additional small string allocation, bounded by the existing complete-trace budget.
2. **Closed authority sum carrying copied namespace/name/revision fields.** It can be equally correct and has the same asymptotics, but duplicates `ProviderIdentity` validation and getters or relies on an unchecked internal reconstruction. It offers no boundary benefit while explain and IR already share the type.
3. **Opaque canonical provider bytes plus a display sidecar.** Injective if minted by `ProviderIdentity`, but its private encoder is not a public received-identity API and adding one solely for an internal same-process consumer widens the public surface. It also makes diagnostics reparse or duplicate fields. Use only if ADR 0073's second-reader trigger later requires an opaque cross-crate carrier.
4. **Injectively escape or length-prefix into one string.** It can avoid collision, but it preserves the false premise that a typed authority is a free-text explain key, duplicates encoding inside text, and still needs a separate class tag for builtin versus registered. Schema and renderer still move, so it buys no compatibility.
5. **Reserve `tiler.compiler`, forbid dots, raise the key limit, hash, truncate, or retain the status quo.** Reject. Each either narrows legal public provider identities, leaves a collision class, introduces probabilistic/opaque authority, or continues late refusal.

## Strongest counterpoint and reversal evidence

Retaining `ProviderIdentity` directly makes a private explain record depend on a public IR type and may clone two component strings where the flat key cloned one. That dependency already exists in the constructor, ADR 0073 explicitly permits compiler-internal subjects, and the trace budget makes the allocation difference bounded. Reverse to an opaque received identity only if a second crate must read canonical traces and the ADR 0073 trigger relocates the vocabulary into `tiler-ir`; at that point a cross-crate opaque carrier may reduce coupling. No such consumer exists now.

## Accepted direction — 2026-08-12

Tom accepted the corrected closed `Compiler | Registered(ProviderIdentity)` authority representation and its explain-only schema/renderer migration in the direct coordination thread by replying `okay agreeed, next decision`. This acceptance confirms the implementation boundary above: structured equality and encoding, separate unambiguous rendering, no reserved provider spelling, no opaque-byte public API, and no artifact or provider-selection change. The ticket remains `todo` for implementation and independent review.
