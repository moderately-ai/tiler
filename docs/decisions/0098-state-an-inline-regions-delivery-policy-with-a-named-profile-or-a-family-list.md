---
schema: "tiler-doc/v1"
id: "ADR-0098"
kind: "decision"
title: "State an inline region's delivery policy with a named profile or a family list"
topics: ["frontends", "proc-macros", "apple-targets", "artifacts", "public-boundary"]
catalog_group: "artifacts-build-toolchains"
decision_status: "proposed"
implementation_status: "partial"
applies_to: ["tiler.contract.frontend-integration"]
evidence: ["tiler.research.macro-environment.build-environment", "tiler.research.apple-targets.compatibility"]
depends_on: ["ADR-0004", "ADR-0049", "ADR-0053", "ADR-0075", "ADR-0088"]
ticket: "draft-an-adr-for-the-inline-delivery-statement"
---

# 0098: State an inline region's delivery policy with a named profile or a family list

**Status:** proposed. The *spelling* this record describes is not proposed — Tom accepted it on 2026-07-31 under [`accept-the-inline-artifact-family-profile-syntax`](../../tickets/accept-the-inline-artifact-family-profile-syntax.md), whose Decision section is the source of record, and it is implemented and tested. What is proposed is this record: lifting that decision out of a ticket outcome, where it is unindexed, unreachable from a contract's frontmatter, and hard to find once the ticket is terminal, into the decision catalog. **Nothing here may change what a consumer writes**, and nothing here re-derives the grounds — the eliminations below are quoted from the deciding ticket rather than reconstructed, because a re-derivation risks recording different grounds than the ones Tom decided against. [`accept-adr-0098-inline-delivery-statement`](../../tickets/accept-adr-0098-inline-delivery-statement.md) is where this record becomes accepted; the acceptance is a relayed fact and that ticket keeps its own rollback cheap.

This record is the delivery half of one consumer-visible surface decided on one day. The other half — the expansion cache root — is [ADR 0089](0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md), accepted 2026-07-31, and this record matches its shape deliberately.

## Context

**Fact — the macro cannot infer a delivery policy, so a region must state one.** A proc macro executes for the *host*, and [the proc-macro build environment research](../research/macro-environment/proc-macro-build-environment.md) measured `HOST`, `TARGET`, `CARGO_BUILD_TARGET`, every probed `CARGO_CFG_TARGET_*`, `SDKROOT`, `MACOSX_DEPLOYMENT_TARGET`, and `IPHONEOS_DEPLOYMENT_TARGET` all absent from the proc-macro process. [ADR 0049](0049-explicit-artifact-family-selection.md) turned that measurement into a rule — an invocation resolves an explicit `ArtifactFamilySelection` and never infers the consumer family from the proc-macro host — and left open the one thing a measurement cannot decide: what a consumer *writes* to state it.

**Fact — everything except the spelling already existed.** `generate-cfg-gated-artifact-family-delivery` landed the four named profiles, their expansion to a canonical `ArtifactFamilySelection`, the versioned consumer-`cfg` map, and the gated payload selector. None of it was reachable: `delivery::stated_policy` returned `FallbackOnly` unconditionally, and the approved region grammar — a declaration block of `sym` and `in` statements followed by one `out` expression, approved by Tom on 2026-07-30 under `prototype-inline-proc-macro-frontend` — had no production for a profile name, an artifact family, a deployment minimum, or a language standard.

**Fact — the surface is Tom's rather than a derivation.** [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) reserves consumer-visible surface to Tom, and this one is consequential in both directions: the spelling is what every consumer writes and cannot be changed silently afterwards, and the vocabulary it publishes decides how much Apple backend policy appears on a consumer-neutral frontend boundary. [The frontend contract](../integration/frontends.md) had already reserved a second axis on the same surface — "a frontend may expose a separate explicit 'acceleration required' policy" — so a syntax chosen for profiles alone had to leave room for it.

**Fact — `ios` names two driver families, and that is a measurement rather than a convenience.** [The Apple Metal artifact compatibility research](../research/apple-targets/artifact-compatibility.md) records that the simulator "is not an iOS-device artifact selected by CPU architecture. Its `simulator` target environment makes it a separate artifact family", measured as a distinct final output. The driver's own vocabulary therefore has three Apple families where a consumer thinks of two platforms.

## Decision

### 1. Two productions, in the declaration block, at most once

**Decided.** A region states its artifact-family delivery policy with a `deliver` statement in the declaration block beside `sym`, `in`, and `contract`, at most once, in either of two productions:

```text
deliver <name>;                                             // a named profile
deliver <name> <major>.<minor>[, <name> <major>.<minor>]*;  // a family list
```

The profile is the ergonomic default and the family list is the escape hatch. This is option 3 of the four the deciding ticket put to Tom, and its recorded grounds are that "it is the shape `docs/integration/frontends.md` already reserves — an ergonomic literal default profile whose resolved selection is still explicit compiler input — and it keeps the floor-override case expressible without publishing Apple vocabulary on the mandatory path."

A second `deliver` statement is a refusal at the second keyword rather than an overwrite, because two statements would be two answers to one question and nothing in the text says which one delivers.

**Inference — the two productions are told apart by one token of lookahead and nothing else.** The token *after* the first name decides: `;` ends a profile, and a literal opens a family list. That is decidable without knowing which names exist, which is what keeps the vocabulary out of the grammar — widening the profile list changes no parsing rule.

### 2. The exact vocabularies, and which layer owns each

**Decided.** The profile vocabulary is `fallback-only`, `macos`, `ios`, and `macos-and-ios`. The family-list vocabulary is `macos` and `ios`, each carrying a `<major>.<minor>` deployment minimum.

Matching is exact in both — no case folding, no prefixes, no underscored near misses — because a name that is *nearly* a profile decides which targets a consumer's build compiles for, and guessing there is not a convenience but a wrong answer.

**Decided — they are one vocabulary, not two.** A profile names its families through the same family values a list names, so `deliver ios;` and `deliver ios 26.0;` cannot come to disagree about which driver families `ios` means. A list stating the governed floors resolves to the *identical* selection as the profile naming the same families, which is asserted rather than assumed.

**Decided, and this is the part a reader must not mistake for a constant: the numbers are the driver's, not this surface's.** A profile publishes no version because it fixes every family it names to that family's governed floor for the Metal language standard Tiler compiles with. **Fact — at this commit both families sit at 26.0, because the profile standard is MSL 4.0.** Under MSL 3.1 the same two rows read 14.0 and 17.0, which is what the deciding ticket's own examples show; a standard change moves those numbers without touching one line of this decision. A minimum below the governed floor is the driver's own typed refusal, carried rather than flattened and reported at the version token that stated it. Any example anywhere in the corpus that spells a floor is therefore dated evidence about a standard, never part of the accepted spelling.

### 3. The statement's absence is `fallback-only`, and that is a decision rather than a gap

**Decided.** A region that states no `deliver` statement resolves to the same explicit `FallbackOnly` policy a region stating `deliver fallback-only;` resolves to. No consumer is required to state a policy in order to have one.

The property this buys is exact and is worth more than the ergonomics: `FallbackOnly` is the policy every region stated before the statement existed, so **a region written without a `deliver` statement expands to token-identical output before and after this decision landed**. An absence read as "unstated" would have to be filled in later by something, and every candidate filler is a policy nobody wrote.

**Inference — the asymmetry with the `contract` statement is deliberate and points the same way.** An absent *delivery* policy builds nothing, which is a complete and harmless meaning; an absent *numerical* contract would have to be filled in with a meaning that decides what results a program may return, which is why that statement is mandatory and this one is not. Two statements in one block with opposite defaults is not an inconsistency — it is the difference between omitting an optimization and omitting a meaning.

### 4. `ios` names the device and the simulator together

**Decided.** The consumer-facing `ios` covers the iOS device *and* the iOS simulator, and the driver's own `ios-device` and `ios-simulator` identifiers are deliberately not published on the region surface. `deliver ios-device 17.0;` is refused at the name — a driver identifier is not a consumer spelling.

The ground is behavioural rather than aesthetic: a developer building for iOS builds for the simulator too, so a name covering only the device would leave every simulator build silently on the fallback path — the exact silent downgrade this contract exists to forbid. Publishing both identifiers instead would put two driver names on the region surface to express the case that always wants both.

**Inference — the vocabularies are consumer-facing and the driver's is not, and this is where that boundary is drawn.** The Context records the measurement that makes device and simulator genuinely distinct artifacts; nothing about that measurement obliges the *consumer* surface to enumerate them, and the frontend crate's edge to the Apple toolchain driver is exactly the seam where three driver families become two consumer names.

### 5. The escape hatch exists because a profile fixes a floor

**Decided.** The family list is not redundant with the profiles, and it is what makes the profile vocabulary affordable.

A profile fixes each family it names to that family's *governed* floor. A consumer whose own deployment minimum is higher — a perfectly ordinary situation, and one Tiler cannot enumerate in advance — would otherwise have no way to say so and would have to wait for a new profile to be minted for their number. The list lets them state it directly, and it does so without putting Apple deployment-minimum vocabulary on the *mandatory* path: a consumer who does not need a floor never writes one and never reads one.

**Inference — this is what keeps one surface from becoming a growing table.** Without the list, every distinct floor a consumer needs is a new accepted profile name, so the vocabulary would grow with the cross product of families and OS versions. With it, the vocabulary stays at four names and the open-ended axis lives in a production instead.

### 6. The surface leaves room for the reserved second axis

**Decided.** The reserved "acceleration required" policy — a separate explicit statement of whether a matching target *must* get an artifact — remains statable as its own statement in the same declaration block, and this decision deliberately does not consume it.

That is the direct consequence of choosing a statement over an attribute or a richer single expression: the declaration block is an open list of statements, so a second axis is a fifth statement rather than a renegotiation of this one. Nothing in the accepted grammar or in either production spends the room.

## Consequences

- [The frontend contract](../integration/frontends.md) is the normative home and states this spelling in its Target policy section; this record does not make a second copy of the productions' semantics, and where the two disagree about a *number* the driver's governed table is the authority over both.
- `delivery::stated_policy` became a function of the parsed region rather than a constant, which is what made the pre-existing delivery machinery reachable at all. Q-ART-008 closed with the deciding ticket.
- **`implementation_status` is `partial`, and the boundary is stated exactly rather than implied.** The *statement* is fully implemented: both productions parse to their exact spans, every profile and family name resolves, the equality of a governed-floor list with the profile naming the same families is asserted, and every statement-level refusal is covered with an accepting neighbour differing in one token. What is not reached is delivery for the whole vocabulary — see the measurement boundary below.
- **A `deliver` statement is not a plan.** It states which artifact families this invocation builds for. It does not state how many kernels the region becomes, which schedule is selected, or whether the selection splits into several entries; those remain the compiler's, exactly as the region grammar's separation requires.
- **Reopening trigger.** A second delivery axis reaching consumers — the reserved "acceleration required" policy is the named one — would be a new statement decided on its own merits, not a widening of this one. A non-Apple artifact family would need its own consumer vocabulary and would put pressure on the assumption that one `deliver` statement names one platform vendor's families; that is a decision to take then, not a generalization to pre-build now.

### Measurement boundary

**Measurement — the tree at this commit, verified by reading the fixtures rather than inferred from the contract.** Exactly two spellings have ever carried a region to a completed expansion: `deliver macos;` and `deliver fallback-only;` (with the statement's absence, which is the same policy). [`crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs`](../../crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs) drives the first through compilation, identity, caching, embedding, and routing inside `rustc`, and [`spikes/runtime/inline-dispatch`](../../spikes/runtime/inline-dispatch) takes one such region to a completed dispatch on **one Apple M4 Max, macOS 27.0, `nightly-2026-07-19`, 2026-08-01**, checked bit for bit against the consumer's own arithmetic. That is one host on one date, not a portable guarantee.

**What that leaves unevidenced, precisely.** Two of the four profile names — `ios` and `macos-and-ios` — have never delivered anything, because no measured Metal compile-time declaration exists for an iOS family and one cannot be inherited from the macOS rows; both are refused at the `deliver` keyword, and [`first-authoritative-ios-metal-compile-declaration`](../../tickets/first-authoritative-ios-metal-compile-declaration.md) is the measurement that would change it. **The family-list production has never produced a payload at all**: every appearance of it in the checked-in fixtures is a refusal or a unit-test selection comparison, and the reproducible check is that no file under `crates/tiler/tests/facade/pass/` or `spikes/` contains a `deliver <name> <major>.<minor>` statement. So the list is evidenced as *parsed, resolved, and validated*, and not as *delivered* — which are, as the architectural contract insists, different maturity claims.

**A stale claim this record corrects rather than repeats.** The deciding ticket recorded that a stated selected family was refused outright, because no expansion ran the offline driver; `prototype-inline-aot-integration-proof` landed later the same day and made `deliver macos;` deliver. The refusal path it named, `DeliveryRefusal::BackendCompilationUnavailable`, no longer exists in the tree — reproduce with `grep -rn "BackendCompilationUnavailable" crates/`, which reports no match. What refuses today is narrower and better attributed: an iOS family for want of a declaration, a symbolic-extent region under a selected family, a floor below the governed minimum, and every vocabulary and syntax mistake, each at the token responsible for it.

## Alternatives considered

Four candidates were put; three were eliminated. **The grounds below are quoted from [the deciding ticket](../../tickets/accept-the-inline-artifact-family-profile-syntax.md) as recorded, not re-derived here.**

**A named profile alone, with no floor override.** "Cheapest to read and to parse, and it publishes no Apple version vocabulary. It also fixes each family's deployment minimum and language standard at whatever the profile says, so a consumer needing a different floor has no way to state one and must wait for" a richer option. Eliminated on that last clause: the case it cannot express is ordinary, and the remedy it forces is a new accepted vocabulary entry per consumer requirement.

**An explicit family list alone.** "Maximally explicit and matches `ArtifactFamilySelection` one-to-one, at the cost of publishing Apple family, deployment-minimum, and MSL vocabulary on the consumer-facing surface — and of making every consumer restate floors the driver already governs." Eliminated on both costs: it puts Apple backend policy on the *mandatory* path of a consumer-neutral frontend boundary, and it makes every consumer a second authority over numbers the driver owns.

**An attribute rather than a statement — `#[tiler::deliver(macos)]` above the invocation.** Recorded as "rejected here rather than offered", and it stays eliminated in the acceptance: "a `#[proc_macro]` cannot see attributes outside its own token stream, so this would require a second macro form and would break the accepted 'each invocation is a self-contained AOT and embedding unit'." The second clause is the load-bearing one — it is [ADR 0004](0004-inline-macro-aot-bundles.md)'s invariant and the inline developer experience the architectural contract preserves by name, so the elimination is a correctness consequence rather than a preference between two syntaxes.

## Traceability

[`accept-the-inline-artifact-family-profile-syntax`](../../tickets/accept-the-inline-artifact-family-profile-syntax.md) is the source of record: it carries the four candidates, Tom's acceptance of option 3 with the draft profile names on 2026-07-31, the implemented grammar, the complete refusal table with the token each refusal names, and the nine deliberate defects applied and watched failing. [`draft-an-adr-for-the-inline-delivery-statement`](../../tickets/draft-an-adr-for-the-inline-delivery-statement.md) is this record, and [`accept-adr-0098-inline-delivery-statement`](../../tickets/accept-adr-0098-inline-delivery-statement.md) is where it is accepted or rejected.

[The frontend and proc-macro integration contract](../integration/frontends.md) is the normative home. [The proc-macro build environment research](../research/macro-environment/proc-macro-build-environment.md) supplies the absent-variable measurements that make a *stated* policy necessary, and [the Apple Metal artifact compatibility research](../research/apple-targets/artifact-compatibility.md) supplies the device-versus-simulator distinctness that decision 4 reasons from.

[ADR 0049](0049-explicit-artifact-family-selection.md) requires the selection to be explicit compiler input and is what this statement is the consumer-facing spelling of; [ADR 0053](0053-gate-artifact-delivery-by-consumer-family.md) owns the rule that a selected family is required when the consumer target matches it, which is why every refusal above is a refusal rather than a quiet downgrade; [ADR 0004](0004-inline-macro-aot-bundles.md) owns the self-contained-invocation property that eliminates the attribute form; [ADR 0088](0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) admitted the crate pair that owns the grammar and the vocabulary; and [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) is why the spelling is Tom's acceptance rather than a worker's derivation. [ADR 0089](0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) is the other consumer-visible half of the same contract decided the same day.
