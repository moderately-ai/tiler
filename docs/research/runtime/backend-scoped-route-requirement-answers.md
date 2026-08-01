---
schema: "tiler-doc/v1"
id: "tiler.research.runtime.backend-scoped-route-requirement-answers"
kind: "research"
title: "Backend-scoped route-requirement answers"
topics: ["runtime", "routing", "backends", "metal", "extensions", "public-boundary", "feasibility"]
catalog_group: "runtime-integration-placement"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.architecture", "tiler.contract.artifact-abi", "tiler.contract.metal-backend"]
depends_on: ["tiler.research.extensions.backend-provider-composition", "tiler.research.runtime.execution-contract"]
ticket: "design-the-adapter-owned-route-requirement-answer-channel"
---

# Backend-scoped route-requirement answers

Every line number here is read at base commit `6f7caf3`, and every claim labelled **Fact** is inspected source or a primary specification at that commit with its reproduction printed beside it. Line numbers in cited prior records have drifted — [ADR 0090](../../decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 4 cites `crates/tiler-runtime/src/load.rs:586-595` for a refusal now at `load.rs:758-782`, and its item 9 cites `load.rs:309-310` for calls now at `load.rs:410-411` — so symbols are the durable citation and the numbers are a convenience.

**Two of the cited files moved on 2026-08-01, and the paragraphs corrected for it read at `8252312` instead.** `662d9be` rewrote `crates/tiler-metal/src/applicability.rs` and `prototypes/candle-metal-adapter/src/adapter.rs`. Every citation into the adapter below is re-read at `8252312` and says so, because the code at those positions changed and not merely its offset. The `applicability.rs` numbers are deliberately left at `6f7caf3`: each was re-checked by symbol and every claim resting on one still holds — `MetalGpuFamilySupport`'s deliberate exhaustiveness is at `applicability.rs:354`, the exact-equality family comparison at `1025`, `every_outcome_is_a_refusal` at `1082` — so re-pointing them would buy a reader nothing that the durable-symbol convention does not already buy, and would go stale again at the next edit to that file. The one `applicability.rs` citation whose *text* changed rather than its position is corrected in place, under "A defect found while deriving this".

## The finding that reframes the question

**Fact — the answer channel already exists, is already neutral, and already works end to end.** `LiveDeviceObservation` (`crates/tiler-runtime/src/load/route.rs:348`) carries three answers: `Quantity(u64)`, `Feature(bool)`, and `Unrecognized`. What crosses the seam for a backend-scoped row is a **`bool`**. `LiveDeviceQualification::resolve_live_device_requirements` (`route.rs:439-475`) matches the row kind against the answer shape on both axes with no wildcard, and the loader — not the adapter — turns the answer into satisfaction or one of three distinct refusals. `crates/tiler-runtime/tests/adapter_route` exercises the complete path for a backend feature row inside the ordinary gate: `fixture.rs:129-133` declares `tiler.test.scalar-host.route-requirement.strict-f32` at version 1 with payload `b"subnormals-preserved"`, `adapter.rs:413-438` answers it by matching owner, key, version, and payload exactly and reporting a *measurement* rather than a constant, and three perturbations — `UnrecognizeLiveDevice`, `MisanswerLiveDevice`, `RefuseLiveDeviceFeature` — drive the three refusal classes.

**Inference — the ticket's framing names a channel; the actual gap is a decoder.** Nothing needs to be added to the seam for a consumer to answer `tiler.metal.route-requirement.minimum-gpu-family`. What a consumer cannot do is work out *which bool to report*, because deciding that means decoding the row's canonical payload into a family and comparing it under that vocabulary's own ordering — and the vocabulary both halves need is `tiler_metal::applicability::MetalGpuFamily`, which a consumer may not name. The whole of this design is therefore about **payload-decode reachability**, and every candidate below is a different answer to "where does the decoder live and who may name it".

This reframing is load-bearing rather than pedantic. A design that added an answer variant, a payload accessor on the neutral seam, or a second observation shape would be widening a boundary that is already correct, and would have to be undone.

## What is true of the Metal row today

**Fact — no production path mints the row.** `grep -rn 'BackendFeatureRequirement::new' --include='*.rs' .` returns nine sites at `6f7caf3`: one decoder (`crates/tiler-artifact/src/program/codec/decode.rs:871`), six artifact-layer or runtime tests, one runtime test fixture, and one prototype (`prototypes/serial-sum-run/src/proof.rs:3451`). `grep -rn 'RouteRequirement' --include='*.rs' crates/tiler-build/src/` returns nothing. `tiler-build` — the build-time orchestrator that assembles every Metal artifact the facade embeds — declares no route requirement of any kind.

**Inference — the ticket's stated end-state hazard is prospective, not current.** The ticket records that fail-closed-forever "leaves the compiler minting requirements nothing on the primary consumer path can answer". At `6f7caf3` the compiler mints none, so no consumer is refused today for this reason; `spikes/runtime/inline-dispatch`'s transcript shows `observe-live-device` absent from the stage list entirely, which is what zero rows looks like. The hazard is real and arrives the moment a Metal plan states the row — which the first authoritative Metal profile will need, because family is exactly the kind of qualitative device fact no floor expresses. **This does not weaken the case for the design; it changes the landing order.** The producer half and the consumer half are one codec, and there is no reason to land them apart.

**Fact — the payload codec is written twice, in two prototypes, and `tiler-metal` owns neither half.** The producing side is `prototypes/serial-sum-run/src/proof.rs:3460-3468`, which encodes `family.as_str().as_bytes()`. The consuming side is `prototypes/candle-metal-adapter/src/adapter.rs:605-609` at `8252312` — `603-607` at `6f7caf3`, and the body is unchanged between them:

```rust
fn gpu_family_from_payload(payload: &[u8]) -> Option<MetalGpuFamily> {
    MetalGpuFamily::ALL
        .into_iter()
        .find(|family| family.as_str().as_bytes() == payload)
}
```

Both route through `MetalGpuFamily::as_str` (`crates/tiler-metal/src/applicability.rs:148-156`), so there is **one spelling authority and two independently written codecs**, and no round-trip test spans them. `grep -rn '"tiler\.metal"' crates/tiler-metal/src/` returns nothing and `grep -rn 'BackendKey' crates/tiler-metal/src/` returns nothing: the Metal backend crate does not name the governed backend key that identifies it. That key is `pub(crate) const BACKEND: &str = "tiler.metal"` in `crates/tiler-build/src/metal_assembly.rs:27`, and is separately spelled as a bare literal in `prototypes/candle-metal-adapter/src/proof.rs:71`, `prototypes/serial-sum-run/src/proof.rs:234`, `crates/tiler-runtime/tests/adapter_route/fixture.rs:114`, and `crates/tiler-runtime/src/load/host.rs:122`. The requirement key and its version are duplicated bare `const`s in the two prototypes (`candle-metal-adapter/src/adapter.rs:94,100` and `serial-sum-run/src/proof.rs:1765,1772`).

**Inference — the surface this design must publish is not primarily an enum.** It is the *codec and its governed identity*, which today exist only as duplicated constants and two hand-written scans. Publishing `MetalGpuFamily` alone would make the duplication reachable rather than remove it.

**Fact — a working implementation of the surviving design already exists in-workspace, and only its reachability is missing.** `prototypes/candle-metal-adapter` is a workspace member, so it may name `tiler-metal`; `adapter.rs:715-743` at `8252312` — `713-742` at `6f7caf3` — decodes the payload, observes the device through `observed_apple_family`, compares, and returns `Feature(bool)`. Nothing in the loader, the artifact layer, or the seam had to change for it. `spikes/runtime/inline-dispatch` cannot do the same thing for exactly one reason, stated in its own source at `src/adapter.rs:659-666`: it is an out-of-workspace consumer written against `tiler` alone, and "a consumer may not name an internal crate".

*Corrected on 2026-08-01 — the observation half of that implementation is no longer the one this record read.* At `6f7caf3`, `observed_apple_family` (`adapter.rs:582-596`) was a call-site walk of one `supportsFamily` call per family, driven by the pair table the next section records as a defect. Since `662d9be` it is two lines that name no family (`adapter.rs:595-598` at `8252312`), forwarding an opaque raw enumerator to `supportsFamily` and delegating the walk to `tiler_metal::applicability::observe_highest_gpu_family`. The claim the paragraph makes is unaffected — the decision logic works in-workspace and its reachability is still what is missing — but the citation now names a different implementation of the same observation, and the shape this record goes on to *propose* is the shape that site already has.

## A defect found while deriving this, which the design must not multiply — since closed in one of its two sites

**Fact at `6f7caf3` — `MetalGpuFamily` is a convention 5b type by [ADR 0074](../../decisions/0074-use-explicit-public-api-conventions.md)'s own test, and both its attribute and its stated reason were wrong.** Its declaration (`applicability.rs:116-120`) read: "it is `#[non_exhaustive]` because a later Apple family lands additively and **no consumer outside this crate classifies it by exhaustive match**." One did. `prototypes/candle-metal-adapter/src/adapter.rs:584-590` mapped every variant onto its Apple counterpart:

```rust
[
    (MTLGPUFamily::Apple9, MetalGpuFamily::Apple9),
    (MTLGPUFamily::Apple8, MetalGpuFamily::Apple8),
    (MTLGPUFamily::Apple7, MetalGpuFamily::Apple7),
    (MTLGPUFamily::Apple6, MetalGpuFamily::Apple6),
    (MTLGPUFamily::Apple5, MetalGpuFamily::Apple5),
]
```

Convention 5b's test is "a match in which every variant must contribute its own correct result and no wildcard value is derivable from the variant it would cover", and its 2026-07-24 amendment extends the clause to total maps "whose arms are all implied rather than written". There is no Apple constant a wildcard could return for an unrecognized Tiler family, so that is a total map and `MetalGpuFamily` must not be `#[non_exhaustive]` — **except** that written as a *table* rather than a match, the incompleteness does not even produce the compile error the attribute would otherwise force. Adding `Apple10` to `MetalGpuFamily::ALL` compiled cleanly at that site, the device was never probed for it, `observed_apple_family` reported a lower family or `NoneNamed`, and every route requiring Apple10 was refused on a device that satisfies it. That is convention 5c's named failure mode exactly — "fail-closed but silently incomplete, which is the harder failure to notice" — reached without the attribute's help.

**Fact — closed at `662d9be` in the site this record cites, and the derivation above is what closed it.** [`close-the-metal-gpu-family-out-of-crate-total-map`](../../../tickets/close-the-metal-gpu-family-out-of-crate-total-map.md) landed on 2026-08-01 and took both halves rather than one. The attribute is gone, and the declaration now states the opposite reason: `MetalGpuFamily` is exhaustive *because* consumers classified it by total map, and the attribute was the cause of the table form rather than a guard against it — written as a match, the correspondence is `E0004` across a crate boundary, so its author wrote a table instead, and a table cannot fail closed. The pair table quoted above is gone from `prototypes/candle-metal-adapter`, because the walk moved into `tiler-metal` as `observe_highest_gpu_family` over `MetalGpuFamily::ALL`, with `MetalGpuFamily::apple_constant` as the Apple-side authority — an in-crate wildcard-free match. The check that can say no is `ALL`'s declared length `core::mem::variant_count::<Self>()`, and it was watched failing: the same `Apple10` perturbation that left `cargo check -p tiler-prototype-candle -p tiler-metal --all-targets` at exit 0 at `cb5d86a` now produces `E0308` on `ALL` and `E0004` at both `as_str` and `apple_constant`. **A public-boundary consequence goes with it and is flagged rather than self-accepted:** removing `#[non_exhaustive]` makes a future family a source-breaking change for any out-of-crate exhaustive match.

**Fact — the identical table still stands in the second prototype, which this record did not cite.** `prototypes/serial-sum-run/src/proof.rs:703-716` carries the same five-element `(MTLGPUFamily, MetalGpuFamily)` pairing, unchanged at `8252312`. It is genuinely a different fix rather than the same edit twice: `metal` 0.33.0 models `MTLGPUFamily` as a `#[repr(i64)]` Rust enum with no safe constructor from a raw value, so that consumer must name the enumerator back by hand and needs a decision about one its binding does not know. Filed as [`close-the-serial-sum-run-gpu-family-probe-table`](../../../tickets/close-the-serial-sum-run-gpu-family-probe-table.md). The observation above is therefore live evidence and not a historical note, and its reasoning is preserved rather than deleted because it is what makes the remaining site a defect too.

**Inference — b2 makes this worse before it makes it better, unless the design removes the map.** Both prototypes that answered the row wrote that table independently — which is now measured rather than predicted — so publishing the vocabulary multiplies the sites at which a family can be silently unprobed. **The design below therefore removes the out-of-crate total map rather than accepting it**, which is the reason its public surface is smaller than "re-export the vocabulary" would be. The defect was filed separately and remains independent of whether this design lands: one of its two sites closed on 2026-08-01 by adopting exactly the shape proposed below, and the other is filed and open.

## The elimination: b1 against b2

Candidates (a) and (c)-as-terminus were eliminated on the record by Tom on 2026-08-01 and are restated in the ticket rather than re-derived here. What follows is the b1/b2 derivation the ticket asks for, stated so a reader can refute the elimination rather than only the conclusion.

### b1 — neutralize the vocabulary into the runtime or artifact layer

b1 is not one candidate. It splits into three, and two of them collapse.

**b1a — a neutral enum naming Apple GPU families, held by `tiler-artifact` or `tiler-runtime`.** Eliminated on the architectural contract, and the contract sentence is already written against it. `crates/tiler-artifact/src/program/requirement.rs:45-47` states: "This layer deliberately does not interpret that payload. It is bytes minted by the backend that emitted the payload, and reading them here would put a backend's vocabulary — **an Apple GPU family, say** — inside the neutral core." Adopting b1a means overturning a sentence that names this exact case. The independent objection is dependency direction: `tiler-runtime` is backend-neutral by charter under [ADR 0081](../../decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md) and its whole closure is `[tiler-artifact]`; a neutral layer that named Apple families would have to grow a second vocabulary for every backend that mints a row, which is the "disguised backend registry" the ticket names.

**b1b — a neutral *ordered token* the runtime compares itself.** This is the candidate that looks like it works, and it is the one worth refuting concretely rather than by principle. The shape: the row carries a token, the adapter reports a token, and the loader compares them with a neutral relation — no backend vocabulary anywhere.

*Eliminated on correctness, with a counterexample a reader can check by hand.* A neutral comparator has exactly one ordering available to it: lexicographic over the payload bytes. `MetalGpuFamily`'s own ordering is its declaration order (`applicability.rs:118-131`, `#[derive(… Ord …)]` over `Apple5 … Apple9`), and the two agree on all five current members because the spellings are equal-length and differ in one digit. They diverge at the first two-digit member: `"Apple10"` compares **less** than `"Apple9"` byte-for-byte, because `'1'` (0x31) precedes `'9'` (0x39) at the sixth byte. A neutral lexicographic comparator would therefore report an Apple10 device as failing an Apple9 floor — a wrong answer that routes a plan away from hardware satisfying it, and one that appears only when Apple ships the member. The general statement is stronger than the instance: **a vocabulary's ordering is a fact about the backend that mints it**, and a neutral layer supplying an ordering asserts a backend fact it has no authority for. Making the ordering data rather than code — a carried rank integer — does not help: the producer would mint the rank, the adapter would mint its own, and two independently minted rank tables are precisely the second authority b1 exists to avoid.

*What would refute this elimination:* a backend-scoped row whose satisfaction relation is genuinely equality over opaque bytes, with no ordering at all. `tiler.test.scalar-host.route-requirement.strict-f32` is exactly that row — `adapter_route/adapter.rs:422` compares `feature.payload() != HOST_ARITHMETIC_PAYLOAD` — and for that shape a neutral comparator would be correct. It does not generalize, because the row this design exists for is a *minimum*, and a minimum needs an order.

**b1c — a neutral carrier of opaque bytes that the owning backend validates.** Not eliminated: *already implemented, and it is what `BackendFeatureRequirement` is.* The owner is a governed `BackendKey`, the key and version are validated, the payload is bounded and non-empty and opaque, and `LiveDeviceObservation::Feature(bool)` is the neutral answer. b1c and b2 are the same design viewed from the two sides of one seam, so b1c is not an alternative to b2 — it is b2's neutral half, and it needs no change.

### b2 — adapter-owned answers, with a deliberately public backend vocabulary

**Survives.** The dependency arrow stays consumer→backend and never core→backend; the neutral layer keeps carrying opaque bytes and neutral answers; the ordering, the spelling, and the satisfaction relation stay with the backend that mints them. Its cost is the one Tom's lean already priced: a *dispatching* consumer names a second crate.

**The elimination reduces to one line of a test, with opposite signs, which is the sharpest available statement of the difference between (a) and b2.** `crates/tiler/tests/dependency_direction.rs:38` reads:

```rust
const FACADE_FORBIDDEN_DEPENDENCIES: [&str; 1] = ["tiler-metal-aot"];
```

Candidate (a) — re-export through the facade — requires `tiler` to acquire an edge to `tiler-metal`, and its enforcement change is that this list must *not* grow. b2 requires that `tiler` never acquire that edge, and its enforcement change is that this list grows to include `tiler-metal`. One line, two directions.

**One elimination of (a) in the ticket is weaker than stated, and saying so is what keeps the record refutable.** The ticket records that (a) "would put a backend crate in every consumer's closure including fallback-only ones". *Fact:* `crates/tiler-metal/Cargo.toml` depends on `tiler-artifact` and `tiler-ir` and nothing else, with `tiler-metal-aot` a development dependency — and both are already in the facade's closure (`crates/tiler/Cargo.toml`). So (a) adds **no new transitive crate**; it adds `tiler-metal` itself, which is 4,076 non-test source lines including the whole MSL emitter (`emit.rs`, 1,369 lines), the target-fact vocabulary (`target.rs`, 798), and the correspondence table (`target_correspondence.rs`, 354). The surviving grounds for (a)'s elimination are the two that do not depend on closure size: the facade would name a *backend* in a boundary whose whole contract is backend neutrality, and a second backend would require a second such edge — `tiler::metal` and `tiler::cpu` beside `tiler::runtime`, in a crate whose documentation says the compiler's internals are not part of the consumer contract. Those hold. The closure-cost argument does not, and a future reader should not rest on it.

## The design

Three parties, three jobs, and the split is the one `tiler_metal::applicability` already uses for host applicability — "a platform adapter observes the host, and this decides" (`applicability.rs:5-8`).

| Party | Owns | Does not own |
|---|---|---|
| The consumer's adapter | Asking the live device a question the backend hands it, and passing the backend's answer on | Decoding the payload, the family vocabulary, the ordering, the satisfaction relation |
| `tiler-metal` | The governed key, version, payload codec, the probe order, the family ordering, and the decision for its own rows | Which device answered, and whether the route may proceed |
| `tiler-runtime` loader | The owner check, the answer-shape check, and turning the answer into satisfaction or one of three refusals | Anything requiring a backend vocabulary |

*Proposal.* `tiler-metal` gains two functions, and a consumer's whole participation is one closure and one three-arm match.

```rust
// in tiler_metal; module placement is a public-boundary item (see below).

/// One Apple `MTLGPUFamily` constant, as an opaque raw value the caller passes
/// to its own Metal binding. Deliberately not `MTLGPUFamily`: this crate must
/// not name a Metal runtime type, and the two live bindings spell it differently.
pub struct AppleGpuFamilyConstant(i64); // superseded: landed as `isize` — see property 2.

/// Asks the caller's device about each family this vocabulary names, highest
/// first, and reports the highest it supports. Cumulative families make the
/// highest supported one the most specific true statement a device makes.
pub fn observe_highest_gpu_family(
    supports: impl FnMut(AppleGpuFamilyConstant) -> bool,
) -> MetalGpuFamilySupport;

/// Decides one live-device route requirement for the Metal backend.
pub fn decide_metal_route_requirement(
    requirement: &RouteRequirement,          // tiler_artifact::program
    observed: MetalGpuFamilySupport,
) -> MetalRouteRequirementAnswer;            // Supported | Unsupported | Unrecognized
```

An adapter's `observe_live_device` becomes:

```rust
let observed = tiler_metal::observe_highest_gpu_family(|family| {
    device.supportsFamily(MTLGPUFamily(family.value()))
});
match tiler_metal::decide_metal_route_requirement(request.requirement(), observed) {
    MetalRouteRequirementAnswer::Supported => LiveDeviceObservation::Feature(true),
    MetalRouteRequirementAnswer::Unsupported => LiveDeviceObservation::Feature(false),
    MetalRouteRequirementAnswer::Unrecognized => LiveDeviceObservation::Unrecognized,
}
```

Six properties are the design rather than the signature, and each is a place a plausible simplification is wrong.

1. **The consumer never names the family vocabulary, and that is the point rather than a convenience.** The probe loop is `tiler-metal`'s, so the map from a Tiler family to Apple's constant is an *in-crate* exhaustive match that convention 3 already requires to be wildcard-free — a family added to the vocabulary is a build error inside `tiler-metal` instead of a silently unprobed family at every consumer. This is what removes the convention 5b/5c defect recorded above rather than replicating it.

2. **Fact — the raw constant is a primary-source value and a stable one, which is what makes it passable as data.** `MTLDevice.h` in the installed macOS 26.5 SDK (build `25F70`) declares `MTLGPUFamilyApple1 = 1001` through `MTLGPUFamilyApple10 = 1010` (`$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h:233-242`); `metal` 0.33.0 transcribes `Apple1` through `Apple9` as `#[repr(i64)]` (`src/device.rs:74-82`) and names no `Apple10`, so the binding is one family behind the header it transcribes. Passing the constant rather than a typed family is also binding-agnostic: `metal` 0.33.0 models `MTLGPUFamily` as a Rust enum and `objc2-metal` models it as a newtype over `NSInteger` with associated constants, and one raw integer crosses both.

    *Corrected on 2026-08-01, twice, and both corrections are re-runnable in one line.* **The range.** This paragraph read "through `MTLGPUFamilyApple9 = 1009` (`…MTLDevice.h:233-241`)" — a window that stops one line before line 242, which is `MTLGPUFamilyApple10 = 1010` in the same header of the same installed SDK. Reproduce with `grep -n MTLGPUFamilyApple "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h"` rather than trusting either reading. Nothing derived here depended on the omission — the `"Apple10" < "Apple9"` elimination is arithmetic on ASCII and holds whether or not Apple ships the member — but the measurement boundary below claimed the member's existence was unknown, and it was readable all along. Whether Tiler's own vocabulary should name `Apple10` is a separate question owned by [`widen-the-metal-gpu-family-vocabulary-to-apple10`](../../../tickets/widen-the-metal-gpu-family-vocabulary-to-apple10.md); this record owns only the recorded header fact. **The width.** The sketch above spells the carried integer `i64`; the landed `AppleGpuFamilyConstant` spells it `isize`, and that supersedes the sketch. `MTLDevice.h` declares the enumeration as `NS_ENUM(NSInteger, MTLGPUFamily)` and `NSInteger` is pointer-sized, so `objc2-metal`'s `MTLGPUFamily(pub NSInteger)` takes an `isize` directly while an `i64` forces a fallible conversion at the one binding that names Apple's type exactly. `i64` would have been correct on every target Tiler supports, which is why this is a supersession rather than a defect.

3. **It takes the whole `RouteRequirement`, not a payload slice.** The key and version match happens inside, so the governed constants stay private to `tiler-metal` and cease to be duplicated at every call site. The consumer performs no string comparison of its own.

4. **`tiler-metal` cannot and must not return a `LiveDeviceObservation`.** It depends on `tiler-artifact` and `tiler-ir`; adding `tiler-runtime` would give the Metal backend an edge to the neutral loader and invert the direction this design exists to protect. The three-valued Metal answer is a distinct type the consumer maps, and — because that mapping is a total map across a crate boundary — `MetalRouteRequirementAnswer` is **not** `#[non_exhaustive]`, under convention 5b and for the same reason `LiveDeviceObservation` is not.

5. **An undecodable payload is `Unrecognized` and never `Unsupported`.** They produce different loader refusals — `UnownedRouteRequirement` versus `UnsatisfiedRouteRequirement` (`load.rs:1328-1361`) — and the repairs differ: "this adapter and this producer disagree about a payload" sends a reader to the codec, "this device is too old" sends them to different hardware. The prototypes already get this right (`candle-metal-adapter/src/adapter.rs:730-732` at `8252312`); the design makes it normative rather than incidental.

6. **The `ResourceFloor` arm is answered too, and answered `Unrecognized` for a stated Metal reason.** `RouteResourceDimension::SubgroupThreads` has no device-scoped answer on Metal: `threadExecutionWidth` lives on `MTLComputePipelineState`, a prepared-kernel object, so answering it from a family table would report a documentation constant as a device observation. Both prototypes and the runtime test adapter already reason this way. Putting the arm inside the decision function makes "Metal's answer to every route requirement" one item with one exhaustive match, so a dimension added to the neutral vocabulary is a build error in `tiler-metal` rather than a wildcard in three adapters.

The producing half is the same codec read the other way: `tiler-metal` gains the constructor that mints the row, so encode and decode are a pair in one module with a round-trip test, replacing the two independently written scans.

## The five questions

### 1. Which items go public, under what versioned identity, and what the compatibility contract is

**Answer — four items, and neither `MetalGpuFamily` nor the governed constants are among them.**

- `observe_highest_gpu_family` and `AppleGpuFamilyConstant` — the observation half. The constant is a raw-value newtype with one reader; it is not an identity type under convention 2 and carries no canonical bytes, because it is a foreign API's argument rather than a Tiler subject.
- `decide_metal_route_requirement` and `MetalRouteRequirementAnswer` — the decision half. The answer type is exhaustive under convention 5b.
- `MetalGpuFamilySupport` — **passed through, not matched**, in the shape above. It stays public because it is the observation's type; it remains deliberately exhaustive, as its own documentation already argues (`applicability.rs:172-177`).
- The minting constructor, for the producer half, so encode and decode are one pair.

**Not public:** the governed key and version, matched inside the decision function — publishing them invites a consumer to re-implement the match, which is the duplication being removed. **Not public:** `MetalGpuFamily` itself, which is what keeps the out-of-crate total map from existing at all. A consumer needing to *render* which family was required gets it from `Display` on the answer or the support type, which is presentation-only under convention 2 and not a match.

*Versioned identity.* The row already carries `version: u32` matched exactly (`requirement.rs:257-265`: "An adapter matches it exactly. A version it does not know is not a requirement it may approximate"). That is the versioning mechanism and needs no addition. What the design newly fixes is the *Rust* compatibility contract: `MetalGpuFamilySupport`'s exhaustiveness and `MetalRouteRequirementAnswer`'s become promises to out-of-crate code rather than crate-local choices, and both are 5b types that must never gain `#[non_exhaustive]`.

*Elimination.* **Re-export the whole `applicability` module** — eliminated, because the module's other half is the ADR 0086 host-applicability policy, whose public items are a *reviewed draft* (`applicability.rs:107`) whose whole content is one measured row and an uninhabited authority type. Publishing them would make an unreachable receipt part of a compatibility surface and would put `MetalHostApplicabilityPolicy`'s exact-equality family comparison beside the route row's minimum comparison with nothing separating them (see question 3). **Publish `MetalGpuFamily` and let consumers observe the device themselves** — eliminated by the 5b/5c defect above: it is what both prototypes did at `6f7caf3` and what `prototypes/serial-sum-run` still does, it puts a silently-incompletable total map at every consumer, and it makes the vocabulary's growth a hazard rather than an addition. **Publish the raw payload spelling (`as_str`) and let consumers write their own codec** — eliminated on the two-authorities ground: two hand-written scans that happen to agree are not one authority. **Mint a fresh consumer-facing family enum in a new crate** — eliminated on the same ground, whether the second enum is neutral or backend-owned.

### 2. How the neutral runtime validates a constructed answer without depending on the backend

**Answer — the validation the neutral layer performs is of *owner and shape*, never of content, and that is sufficient because the loader's decision is total over the answer.**

*What travels typed:* the owner (`BackendKey`), the key (`RouteFeatureKey`), and the version (`u32`) — all validated by the artifact layer's governed key grammar, all comparable byte-for-byte across producers that never met. And the answer: `LiveDeviceObservation`, three values, not `#[non_exhaustive]` (`route.rs:344-346`) so that an answer added later stops each host's build.

*What travels as canonical bytes with backend validation:* the payload alone. The artifact layer validates it for non-emptiness, a 1,024-byte ceiling (`requirement.rs:77`), inclusion in `canonical_bytes` and hence in artifact identity (`requirement.rs:325-341`), and subject uniqueness. It validates nothing about its meaning.

*Where the validation authority lives, in the order it runs:*

1. **Owner, in the loader, without a device and without consulting anything.** `route_requirements` (`load.rs:758-782`) refuses `ForeignRouteRequirementOwner` when the row's owner is not the host's own stated `BackendKey`. This is decidable from the host's own declaration, and the reason it runs first is stated at `load.rs:427-431`: "asking an adapter about another backend's namespace would invite it to answer."
2. **Payload meaning, in the owning backend.** Only `tiler-metal` can say whether these bytes name a family it knows at a version it knows.
3. **Shape, in the loader.** The `(kind, answer)` match at `route.rs:450-466` is exhaustive on both axes with no wildcard; a quantity for a qualitative row and a verdict for a floor are `MisansweredRouteRequirement` rather than coerced.
4. **Satisfaction, in the loader.** `Feature(false)` is `UnsatisfiedRouteRequirement`; `Unrecognized` is `UnownedRouteRequirement`.

*Why this needs no backend dependency:* the loader never inspects the payload, so it never needs the vocabulary. Its decision is a total function of `(row kind, answer)` and every cell of that 2×3 table is written out. **There is no answer a backend can construct that the loader cannot rule on**, which is the property that makes "validate without depending on the backend" achievable at all.

*Elimination.* **Give the neutral layer a payload validator the backend registers** — eliminated: a registered validator is a Rust object that cannot survive the process boundary an artifact crosses, which ADR 0090's alternatives already eliminate for the producer-to-adapter join, and it would make the loader's refusal depend on whether a registration happened rather than on what the artifact says. **Have the adapter return a richer answer the loader compares** — eliminated on ADR 0090 item 4's rule read the other way: a richer answer would need the loader to hold a backend-specific comparison, which is b1 by another route. **Have the loader re-check the backend's decision** — eliminated as vacuous: any check the loader could perform on a `bool` it did not derive is a check of nothing.

*One apparent conflict, resolved explicitly because it will be raised.* ADR 0090 item 4 says "a runtime adapter reports facts and never adjudicates them", and this design has the backend deciding a qualitative row. There is no conflict. `LiveDeviceObservation::Feature`'s own documentation (`route.rs:357-359`) reads "The owning adapter **decided** this qualitative row for the bound device", and `requirement.rs:41-43` says the payload is one "the **owning adapter** validates". What item 4 forbids is the adapter reversing a *capacity comparison* — the quantitative half, where the loader holds the threshold and the direction (`floor.is_satisfied_by(observed)`, `requirement.rs:186-188`). The qualitative half was always the backend's, and the seam is shaped that way on purpose: three answers rather than a boolean exist so that "I do not own this row" stays distinct from "this device does not satisfy it".

### 3. How ADR 0086's eligibility gate composes with an answered row

**Answer — a satisfied family row licenses continuation of *this route* and nothing about *host eligibility for a profile*, and the two comparisons over the one vocabulary run in opposite directions.**

**Fact — the two comparisons differ, and a reader who assumed one answer serves both would get the direction wrong.** `evaluate_metal_host_applicability` compares for **exact equality** (`applicability.rs:843`):

```rust
if gpu_family != MetalGpuFamilySupport::Highest(policy.gpu_family) {
    return Err(MetalHostApplicabilityRefusal::GpuFamilyMismatch { … });
}
```

with the reason stated at `applicability.rs:600-603`: "Exact rather than 'at least': the policy's whole validity is the measured row, and admitting a higher family would extend a bounded measurement to hardware nobody ran it on." The route requirement compares for **at-least** (`candle-metal-adapter/src/adapter.rs:736-739` at `8252312`): `MetalGpuFamilySupport::Highest(highest) => highest >= required`. An Apple9 device satisfies a `minimum-gpu-family = Apple8` route row and *fails* a policy naming Apple8. One vocabulary, two relations, two authorities.

**What a satisfied row licenses.** Exactly this: the bound device reports supporting at least the named Apple family, so this route's requirement at that position holds and `resolve_live_device_requirements` may proceed to `RoutePreparation`. It is a *live-device capability* fact in ADR 0043's phase vocabulary, scoped to this device and this route.

**What it must never license.** That the host may offer the target profile the artifact declares. `MetalHostPredicate::GpuFamily` is **one of seven** predicates (`applicability.rs:220-228`), and the seventh — `NativeTranslationAuthority` — is `Unknown` on every macOS row currently observable under ADR 0086 item 1. The refusal is structural rather than conditional: `MetalHostEligibility` holds a `NativeTranslationAuthority` whose one field is a private empty enum, so no value of the receipt exists anywhere including inside `tiler-metal`, and `structural_unreachability::every_outcome_is_a_refusal` (`applicability.rs:900-906`) is a match with no `Ok` arm that stops compiling the moment that changes.

**Inference — the composition is safe by construction, and the reason is worth naming because it is not obvious.** Eligibility and the route row are not merely separate; the route row is *downstream* of everything eligibility touches. `variant_eligibility` (`load.rs:660-710`) decides host eligibility from the stated `ExecutionEnvironment` alone — profile classification and the backend/representation pair — and runs during `select_variant`, **before** `route_requirements` is even built. A route requirement therefore cannot influence variant eligibility, because it is read after it. And `evaluate_metal_host_applicability` cannot be reached from a route answer at all: its second parameter is a `MetalHostObservation` and nothing else, with two compile-fail doc-tests pinning that a declared `TargetProfileRef` and raw artifact bytes are both `E0308` (`applicability.rs:771-798`).

**The hazard this design would have created, and how the surface above removes it.** If `MetalGpuFamily` were published, its derived `Ord` would be published with it, and a consumer could write `observed >= required` by hand — correct for the route row, wrong for applicability. The surface in question 1 does not publish the family type or its ordering; the comparison is reachable only through `decide_metal_route_requirement`, which fixes the direction. That is a second, independent reason to prefer the narrow surface over re-exporting the vocabulary.

*Elimination.* **Let a satisfied family row contribute to the applicability policy's `GpuFamily` predicate** — eliminated, and it is the cheap option that is wrong exactly where it saves: it would satisfy one of seven predicates while the receipt needs all seven, and the six-of-seven progress would appear in explain output as though eligibility were nearly earned, when the missing predicate is the one ADR 0086 item 3 says the measured row *cannot* stand in for. **Route the family observation through one shared function serving both** — eliminated on the two relations above. One function returning "the highest family this device claims" is correct and is what `observe_highest_gpu_family` is; one function returning "does this device qualify" would have to pick a direction and be wrong for one caller. The prototypes already share the *observation* deliberately (`candle-metal-adapter/src/adapter.rs:592-594` at `8252312`: "two spellings of 'what family does this device claim' would let the two answers drift") and deliberately do not share the *comparisons*.

### 4. What a second backend does with the same channel

**Answer — the channel generalizes structurally; the pattern costs each backend a consumer-nameable crate; and two named costs make the CPU case worse than the Metal case without making it a special case.**

*Structurally it generalizes without change.* A CPU row — `tiler.cpu.route-requirement.minimum-isa-level`, say — is the same shape: governed owner `tiler.cpu.scalar`, governed key, exact version, canonical payload, `Feature(bool)` answer. ADR 0090 item 14 records that the CPU vertical's "vector width, mask and tail support, scalable-vector length, cache levels, and thread count are not merely undeclared but *inexpressible*" as capability axes, and item 10 records that `tiler.cpu.scalar` and `tiler.cpu.scalar-image-v1` were minted without touching a registry. The qualitative half of that gap is exactly what this channel carries.

*Cost one — the availability phase is borrowed.* ADR 0090 item 14: "no availability phase names a bound host *process*, so a process-bound fact must borrow `LiveDevicePreflight`". An ISA feature is readable at process start with no device of any kind, so a CPU backend answering this row reports a fact whose phase the vocabulary cannot name. The channel works; the phase label is wrong, and `name-a-host-process-availability-phase` is the existing ticket that owns it.

*Cost two — a CPU row forces the `prepare` path.* ADR 0090 item 9 states, and the CPU vertical measured, that `preflight` alone is sufficient exactly when the selected variant has zero deferred predicates **and** zero route requirements; the vertical "carried zero of both and completed correctly on `preflight` alone". *Fact — the spike takes that path and implements no adapter at all:* `grep -rn 'RuntimeAdapter\|observe_live_device\|RouteRequirement' spikes/target-profiles/scalar-cpu-vertical/src/` returns nothing, and `src/vertical.rs:849-895` goes `preflight` → its own preparation → `commit`. One ISA row would make `preflight` return `UnansweredRouteRequirements` (`load.rs:742-751`) and force the caller onto `prepare`, where **both** device stages then run unconditionally. A CPU backend gains a `prepare_entries` call it has nothing to do in and an `observe_prepared_entry` path it has no property for, discharged by returning `Ok` and declaring no deferred predicate. That is the contract working as designed — the stage sequence is what makes a route requirement unskippable — and it is a real cost to state rather than discover.

*Structural precondition — a consumer-nameable crate per row-minting backend.* b2's mechanism is "the consumer's adapter names the backend crate that owns the vocabulary". For Metal that crate is `tiler-metal` and it exists. For CPU it does not: the scalar CPU vertical is a spike, and ADR 0090 item 10 notes it "declared its own governed backend-family and representation keys from a spike rather than from a crate at all". So the generalization is honest but conditional: **the pattern generalizes; each backend that mints a backend-scoped row must first pay for a crate a consumer may name.** A backend whose rows are all neutral `ResourceFloor`s pays nothing, because the loader compares those itself.

*What the CPU case does **not** need.* The raw-constant indirection of property 2 above is Metal-specific: it exists because the probe is a foreign API call whose argument is a foreign constant. An ISA probe is `std::arch::is_aarch64_feature_detected!`, a macro over a string literal the backend can hold itself, so a CPU backend's observation function takes no callback at all and the consumer's participation shrinks further. **The shape generalizes; its exact spelling is per-backend, and a contract that fixed one spelling for both would be wrong for one of them.**

*Elimination.* **Make the channel Metal-only and say so** — eliminated, because nothing in the neutral half is Metal-shaped: `BackendFeatureRequirement` is keyed by an arbitrary governed `BackendKey`, and the runtime's own in-gate evidence uses a fictional `tiler.test.scalar-host` backend. Declaring it Metal-only would be a claim contradicted by the test that proves it works. **Give the CPU backend a neutral floor instead of a feature row** — eliminated for ISA level on the same ordering ground that eliminates b1b: "at least AVX2" is not a number, and the numeric encodings that look like one are a rank table someone has to mint. It is *not* eliminated for genuinely quantitative CPU facts, and those should be floors; `RouteResourceDimension` is deliberately not `#[non_exhaustive]` so that adding one is a build error at every adapter.

### 5. What the fallback-only consumer's contract remains

**Answer — naming `tiler` alone stays sufficient for every non-dispatching use, unchanged.**

*Fact — the facade's own statement of the rule.* `crates/tiler/src/lib.rs:3-7`: "A consumer writes `tiler = { … }` in its manifest and reaches the inline frontend through [`tensor!`]. Nothing else in the workspace is part of that contract." `docs/architecture.md:389` states it as contract: "`tiler` is the one crate a consumer names."

*Fact — nothing on the fallback path touches a route requirement.* A `tensor!` region without `deliver` never routes at all; the facade constructs the declared result through `TensorAdapter::build` and evaluates nothing. A region that *does* deliver but whose route refuses takes the same path. `LiveDeviceObservation` and `RouteRequirement` are reachable through the facade's existing whole-module re-exports (`lib.rs:166-178`) and no fallback-only consumer names either.

*What changes, precisely.* The sentence "a consumer names `tiler` alone" becomes a property of the **fallback path** and of the **non-dispatching consumer**; a *dispatching* consumer names `tiler` plus the backend crate whose device it drives. This is not a weakening the design smuggles in — it is already true in every observable form except the vocabulary: `spikes/runtime/inline-dispatch` links the third-party `metal` crate, opens a `Device`, builds pipelines, and encodes command buffers. A consumer that does all of that and cannot name the family vocabulary is not backend-neutral; it is backend-specific and short one name.

*Enforcement, and a defect in the obvious one-line change.* `FACADE_FORBIDDEN_DEPENDENCIES` (`dependency_direction.rs:38`) should grow to include `tiler-metal`. **But the test's anti-vacuity guard does not cover a new entry.** `dependency_direction.rs:121-128` asserts, for *each* forbidden name, that `tiler-macros` really holds that edge before concluding the facade does not — so a name that stopped resolving fails the test instead of passing it vacuously. `tiler-macros` does **not** depend on `tiler-metal`, so adding `tiler-metal` to the list as it stands makes the test fail on its own guard. The enforcement change is therefore two parts: a second list for forbidden edges no frontend package holds, or a per-entry witness naming which package is expected to hold each. *Found by reading the guard rather than the assertion, and it is exactly the class of thing that otherwise lands as a green test checking nothing.*

*Elimination.* **Feature-gate the vocabulary onto the facade (`tiler = { features = ["metal"] }`)** — eliminated, and this is the cheap option that looks like the best of both. It fails on two counts. Cargo features are additive and unify across a build graph, so one crate anywhere in a consumer's tree enabling `metal` puts `tiler-metal` in every crate's closure — the property (a)'s elimination rests on, restored by a mechanism that hides it. And it puts the backend name in the *facade's* manifest and feature namespace, so the facade enumerates backends after all; a second backend is a second feature on a crate whose contract is neutrality. **Publish a separate thin `tiler-metal-consumer` crate holding only the answer surface** — not eliminated on correctness, and deferred rather than adopted: it is strictly more surface for the same reachability while `tiler-metal`'s closure is already `[tiler-artifact, tiler-ir]`, both already in the facade's closure. It becomes the right answer if `tiler-metal` ever acquires an edge a consumer should not pay for, and that is its reconsideration trigger.

## Worked example — the GPU-family row end to end

A region delivering `macos` compiles to a plan whose emitted kernel uses an Apple8-and-above capability. Every stage names its exact site, and the last two name what the route does and does not license.

**Minted.** `tiler-build`'s Metal assembly declares one row on the variant: owner `BackendKey("tiler.metal")`, key `RouteFeatureKey("tiler.metal.route-requirement.minimum-gpu-family")`, version `1`, payload `b"Apple8"` — the spelling `MetalGpuFamily::as_str` fixes — minted through `tiler-metal`'s constructor rather than by a call site assembling bytes. *Proposal: no such call exists at `6f7caf3`; `tiler-build` declares no route requirement.* `BackendFeatureRequirement::new` (`requirement.rs:214-243`) refuses a zero version, an empty payload, and one over 1,024 bytes.

**Carried.** The row enters `canonical_bytes` (`requirement.rs:325-341`) with the subject leading, so two rows naming one subject sort adjacent and the builder's duplicate-subject check is a scan of neighbours. The payload is inside artifact identity, so an artifact whose row was edited is a different artifact and the envelope's re-derived identity refuses it before any route exists.

**Loaded and owner-checked.** The consumer's `DecodedProgram::prepare` selects the variant — note that `variant_eligibility` has already run, so profile and representation are settled before any requirement is read. `route_requirements` (`load.rs:758-782`) compares `feature.owner()` against the host's stated `environment.backend`. A host stating `tiler.metal` passes; one stating `tiler.cpu.scalar` is refused `ForeignRouteRequirementOwner` here, with no adapter consulted and no device bound. The row becomes `LiveDeviceRequest { variant: 0, position: 0, requirement }`.

**Decoded and answered.** `route_with_adapter` (`crates/tiler-runtime/src/adapter.rs:453-456`) hands each request to `RuntimeAdapter::observe_live_device`. The adapter calls `observe_highest_gpu_family` with a closure that forwards each raw constant to `supportsFamily`; `tiler-metal` walks its own vocabulary highest first and returns `MetalGpuFamilySupport::Highest(Apple9)` on an M4 Max. It then calls `decide_metal_route_requirement`, which matches key and version exactly, decodes `b"Apple8"` against its own vocabulary, finds `Apple9 >= Apple8`, and returns `Supported`. The adapter maps that to `LiveDeviceObservation::Feature(true)`. **The consumer named no family, wrote no table, and compared nothing.**

**Validated.** `resolve_live_device_requirements` (`route.rs:450-466`) matches `(RouteRequirement::BackendFeature(_), LiveDeviceObservation::Feature(true))` and the row is satisfied. The loader never saw `b"Apple8"` as anything but bytes.

**What this licenses.** The route proceeds to `RoutePreparation`, then pipeline preparation, prepared-entry properties, `plan_dispatch`, and `Preflight::commit`. The requirement at position 0 of variant 0 holds on this bound device for this attempt.

**What it must not license — the ADR 0086 boundary, stated as the negative.** It does **not** license the claim that this host may offer `tiler.metal.macos-apple9.msl4-0.f32.v1`. `evaluate_metal_host_applicability` on the very same M4 Max, with every one of its six environment predicates matching the retained measured row byte for byte, returns `UnknownNativeTranslationAuthority` — the doc-test at `applicability.rs:725-759` asserts exactly that. The dispatch that follows a satisfied family row is settled on producer-declared equality, not host-earned eligibility, and `spikes/runtime/inline-dispatch` prints that distinction as a labelled diagnostic on every successful run. **A satisfied `minimum-gpu-family` row answers one of seven predicates and says nothing about the seventh.**

**The three refusals, each reachable.** A device reporting `Highest(Apple7)` → `Unsupported` → `Feature(false)` → `UnsatisfiedRouteRequirement`. A payload of `b"Apple99"` or a version of `2` → `Unrecognized` → `UnownedRouteRequirement`. An adapter answering `Quantity(9)` → `MisansweredRouteRequirement`. All three are already exercised for a backend feature row by `prototypes/serial-sum-run/src/proof.rs:4738-4800` and by `crates/tiler-runtime/tests/adapter_route`'s three perturbations; what is new here is only which vocabulary decodes the payload.

## Worked example — the CPU backend, and where it diverges

The same row shape on `tiler.cpu.scalar`: key `tiler.cpu.route-requirement.minimum-isa-level`, version `1`, payload `b"neon"`.

**Identical through four stages.** Minted by the CPU backend's own constructor; carried in `canonical_bytes` and identity; owner-checked by `route_requirements` against a host stating `tiler.cpu.scalar`, which refuses the Metal row and admits this one; decided by the crate owning the CPU vocabulary; validated by the same `(kind, answer)` match. Nothing in the neutral half distinguishes the two backends, which is the generalization claim discharged.

**Four divergences, all costs rather than blockers.**

*The observation needs no device, and no callback.* `is_aarch64_feature_detected!("neon")` takes a string literal the backend can hold itself, so the CPU decision function needs no consumer closure and the consumer's participation is a single call. Metal's callback exists only because its probe is a foreign API call with a foreign constant argument.

*The phase is borrowed.* The fact is a property of the bound *process*, reported through a method whose contract is "what the bound device is". ADR 0090 item 14 names this vocabulary gap; `name-a-host-process-availability-phase` owns it.

*The route is pushed onto `prepare`.* Before this row, the CPU vertical completed on `preflight` alone. One row forces `prepare`, where both device stages run unconditionally, and the backend discharges two of them by doing nothing.

*The vocabulary has no crate.* There is no `tiler-cpu`. Until one exists, a CPU consumer is in precisely the position `spikes/runtime/inline-dispatch` is in for Metal — able to observe, unable to reach what decides — and the correct interim is the same: `Unrecognized`, fail-closed.

**Inference — the channel does not deliberately decline to generalize, and the honest statement is narrower than "it generalizes".** The *neutral* half generalizes with no change and is already proven against a fictional backend in the ordinary gate. The *backend* half generalizes only as far as each backend owns a consumer-nameable crate, and that is a per-backend packaging decision rather than a property of this channel.

## Public-boundary items, enumerated for Tom and not self-accepted

[ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md) decides approval by change category read from the diff. **Its mechanical categories fire for only part of this**, and saying so is more useful than asserting the whole thing is Tom's: AGENTS.md's broader clause — "acceptance of a public crate, module, trait, type, or consequential call-site boundary" — carries the rest.

**Unchanged by the 2026-08-01 landing, and this is the sentence most likely to be misread in the other direction.** [ADR 0092](../../decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md) is `proposed`, so it accepts neither the model nor any of the seven items below; it restates them so that a reader arriving at the decision index sees the same list rather than a decision that looks settled. Tom's recorded acts are narrower still: the design-ticket route with a b2 lean, and the eliminations of candidate (a) and of fail-closed-forever as a terminus, all on 2026-08-01. A lean toward a candidate is not an acceptance of the model the candidate produced, and the model would not be an acceptance of the surfaces under it — the distinction ADR 0075 exists to hold, where the model says what a surface must mean and Tom says what it is called and what shape it takes.

**Fires under ADR 0075's stated categories.**

1. **A new `pub mod` in `tiler-metal`'s crate root**, if the answer surface lands as its own module rather than inside `applicability`. The namespace category is mechanical and fires regardless of surface size, and ADR 0075 records that cost as accepted rather than overlooked. *This record recommends a separate module*, because `applicability` is scoped to the ADR 0086 host policy and mixing the two comparisons in one module is the confusion question 3 exists to prevent.

**Does not fire mechanically; Tom's under AGENTS.md's clause.**

2. **Reclassifying `tiler-metal` as a crate a consumer may name.** No diff line adds a workspace member or a facade `pub mod`, so the mechanical test is silent — but `docs/architecture.md:389` states "`tiler` is the one crate a consumer names" as contract, and this changes it for the dispatching consumer. Amending that sentence is the acceptance act, and it lands in `contracts/foundation`.
3. **The exact shapes of `observe_highest_gpu_family`, `decide_metal_route_requirement`, `AppleGpuFamilyConstant`, and `MetalRouteRequirementAnswer`.** A tested implementation is a concrete draft, not implicit approval of its interface. **Two of the four have since landed as exactly that — concrete drafts, unaccepted.** `662d9be` published `observe_highest_gpu_family` and `AppleGpuFamilyConstant` in `tiler_metal::applicability` to close the total-map defect, which needed the observation half and not the decision half. They are inside the module whose every public item is already declared a reviewed draft under ADR 0074 §7, and the ticket that landed them flagged them for Tom rather than self-accepting. Their existence changes what item 3 costs to decide — the observation half is now a shape Tom can read and run rather than a sketch — and changes nothing about whether it is decided. `decide_metal_route_requirement` and `MetalRouteRequirementAnswer` do not exist.
4. **Whether the observation crosses as a raw Apple constant at all.** The alternative — publishing `MetalGpuFamily` and letting consumers map it — is what this record eliminates, and the elimination rests on a convention-5b reading Tom may want to check. `662d9be` made the raw-constant crossing real in-workspace for the defect's own sake, so the reading is now checkable against a running site rather than only against this argument; it remains the reading, not a decision.
5. **The minting constructor's shape**, which fixes how every producer states this row.
6. **`MetalGpuFamilySupport` becoming a compatibility surface**, and its deliberate exhaustiveness becoming a promise to out-of-crate code rather than a crate-local choice. `MetalGpuFamily`'s exhaustiveness became the same kind of promise at `662d9be`, ahead of this design and for the defect's reason rather than this one: a future Apple family is now a source-breaking change for any out-of-crate exhaustive match over it, which ADR 0075 admits while nothing is publishable and no external consumer exists, and which is Tom's to accept rather than the landing ticket's.
7. **Whether the governed key and version stay private to `tiler-metal`.** This record recommends they do; the opposite is defensible if a consumer should ever enumerate the rows a backend owns.

**No approval required.**

8. The `dependency_direction.rs` change and its guard repair — tests.
9. This record and the spike documentation restatement — documentation.

## Measurement boundary and unsupported cases

- **Nothing here was compiled or measured at `6f7caf3`.** Every interface shape above was a *type-system reservation* in ADR 0090's own sense: none compiled, none had an out-of-crate fixture. The working implementation cited (`prototypes/candle-metal-adapter`) is an in-workspace crate reaching the vocabulary through an ordinary dependency, so it proves the *decision logic*, not the *reachability* this design is about — and at `6f7caf3` it used the map shape this record eliminates. **Two of the shapes stopped being reservations at `662d9be`,** and the distinction between reservation, implementation, and accepted guarantee is exactly the one not to collapse: `observe_highest_gpu_family` and `AppleGpuFamilyConstant` are *implemented and tested in-crate*, with doc-tests and a probe-recording test that pins the queried population against `MetalGpuFamily::ALL`. They are still not *reachable* out of workspace and still not *accepted* as a boundary — which is the whole subject of this record and is unchanged. `decide_metal_route_requirement`, `MetalRouteRequirementAnswer`, and the minting constructor remain reservations that compile nowhere.
- **The SDK constant is primary-source but host-bound.** `MTLGPUFamilyApple1 = 1001 … Apple10 = 1010` is read from `MTLDevice.h` in the macOS 26.5 SDK (build `25F70`) installed on this host; a different SDK is a different read, and the reproduction is printed above so it can be re-run rather than trusted. *Corrected on 2026-08-01:* this bullet read `Apple5 = 1005 … Apple9 = 1009` from a window that stopped at line 241, one line before `Apple10`.
- **The Apple10 divergence is arithmetic on ASCII, not an Apple roadmap claim, and the member is no longer hypothetical.** `"Apple10" < "Apple9"` byte-for-byte is checkable by hand, and the elimination it supports does not depend on the member existing, because the general statement — a vocabulary's ordering is a backend fact — stands without the instance. *Corrected on 2026-08-01:* this bullet said "whether Apple ships an `MTLGPUFamilyApple10` is unknown here", which was never a measurement boundary but an unread line of a file this record already had open. It is `MTLDevice.h:242` in the same SDK. What that changes is the *status* of the counterexample rather than the argument: the b1b elimination now rests on a member Apple has shipped rather than on one it might, and it would have stood either way. What Tiler's own `MetalGpuFamily` should name remains open under [`widen-the-metal-gpu-family-vocabulary-to-apple10`](../../../tickets/widen-the-metal-gpu-family-vocabulary-to-apple10.md), and `metal` 0.33.0 not naming it is one of that question's inputs.
- **The interim is unchanged and stays fail-closed.** `spikes/runtime/inline-dispatch` answers `Unrecognized` for both arms and the loader refuses the route. That is correct while this design is unimplemented and is deliberate rather than a gap.
- **No claim about a three-backend portfolio.** ADR 0090's reopening trigger has not fired, and every statement here about a mixed portfolio is inference.
- **The `ResourceFloor` half is untouched.** `RouteResourceDimension::SubgroupThreads` remains unanswerable on Metal and correctly `Unrecognized`; a route genuinely needing subgroup width on Metal must state it as a `PreparedEntryTargetRequirement` against the prepared pipeline, which is the authority that has it.

## Drafted ADR body — landed as [ADR 0092](../../decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md) on 2026-08-01, `decision_status: proposed`

**The record of the decision is [ADR 0092](../../decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md); the span below the rule is retained as the drafted text it landed from and is not a second authority over the same subject.** *And neither document is an authority yet*: the ADR is `proposed`, which under [the decisions index](../../decisions/README.md)'s own preamble means it remains a non-decision until Tom accepts it, so nothing in this record's **Proposal** labels weakened when it landed and none of them was rewritten after the fact. The transfer was byte-identical — context, the nine numbered decisions, consequences, the six alternatives-considered entries, and the traceability paragraph — with the section headings demoted one level, from `###` nested under this heading to `##` under the ADR's own title, and nothing else changed. A reader who wants the decision should read the ADR, which additionally states what a lean is not, the seven boundary items that stay Tom's, the implementation boundary, and the deferrals. The span is kept rather than replaced by a bare pointer because it is the exact text the ADR was cut from, and a record that hatched a decision is evidence about that decision as well as about the derivation.

**One consequence of retaining the exact bytes, stated because it costs a reader who does not know it.** The span was written with its relative links resolved from `docs/decisions/`, which is where it now lives — so eight of them do not resolve from *this* file: `0074`, `0075`, `0081`, `0086`, and `0090` are bare siblings, `../architecture.md` and `../artifact-abi.md` are one level short, and the self-citation opening the traceability paragraph points at `../research/runtime/…`. All eight resolve correctly inside [ADR 0092](../../decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md), which is the document to follow them from; all eight were already broken here at `cb5d86a`, before the ADR existed, and repointing them now would trade a reader's inconvenience for the byte-identity that makes this span quotable at all. Every link in this record *outside* the span resolves from here.

The paragraph this replaces recorded why the drafted body could not become a file from here, and the reason is preserved because it is the reusable part: `ticketsplease.toml` routes `docs/decisions/[0-9]*.md` to `contracts/decisions` and both catalogs to `contracts/navigation`, and the ticket that produced this record holds `research/runtime`, `research/extensions`, and shared `project/tickets` only, so writing the ADR or either catalog row from that branch would have been a guard escape. [`land-the-backend-scoped-route-requirement-answer-adr`](../../../tickets/land-the-backend-scoped-route-requirement-answer-adr.md) held both catalog scopes and carried it. It also carried this record's own catalog row, which was absent from [the research catalog](../README.md) entirely rather than merely stale — the same scope split, found one document further out. The number was taken by reading the directory as instructed: `0091` had landed since `6f7caf3` and `0092` was free.

**A second consequence of retaining the exact bytes, found on 2026-08-01 and deliberately not repaired inside the span.** One span sentence drifted the same day the span landed, and the correct handling is the one the link note above already models: record the discrepancy beside the span, because editing inside it forks the transfer and spends the byte-identity that makes the span quotable at all. The alternatives-considered entry *Publish the family vocabulary and let each consumer observe the device itself* reads "written as a table rather than a match — which is what the existing prototype does". At drafting, "the existing prototype" was `prototypes/candle-metal-adapter`; `662d9be` removed its table that evening. The sentence is not false — `prototypes/serial-sum-run/src/proof.rs:703-716` carries the identical table and is open under [`close-the-serial-sum-run-gpu-family-probe-table`](../../../tickets/close-the-serial-sum-run-gpu-family-probe-table.md) — but its singular referent now names a different prototype than the one its author had in view, and a reader who resolves it to the candle adapter will find nothing there. [ADR 0092](../../decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md) carries the same sentence, byte-identically, in its own alternatives-considered section. **Flagged for the ADR 0092 acceptance sweep**, because the ADR is the authority over this subject and the span follows it rather than leading it: the sentence should become "which is what a prototype still does" in the ADR, and this span should be re-transferred from the ADR at that point rather than corrected here first.

---

**Title:** Answer backend-scoped route requirements in the owning backend's vocabulary

**Frontmatter:** `decision_status: proposed`, `implementation_status: not-started`, `catalog_group: "runtime-integration-placement"`, `applies_to: ["tiler.contract.architecture", "tiler.contract.artifact-abi", "tiler.contract.metal-backend"]`, `evidence: ["tiler.research.runtime.backend-scoped-route-requirement-answers"]`, `depends_on: ["ADR-0074", "ADR-0075", "ADR-0081", "ADR-0086", "ADR-0090"]`, `ticket: "land-the-backend-scoped-route-requirement-answer-adr"`.

### Context

A `RouteRequirement::BackendFeature` carries an owner, a governed key, an exact version, and a canonical payload the artifact layer deliberately does not interpret. The neutral answer channel is complete: `LiveDeviceObservation::Feature(bool)` crosses the seam, the loader owns the owner check, the shape check, and the satisfaction decision, and `crates/tiler-runtime/tests/adapter_route` exercises all three refusal classes in the ordinary gate against a fictional backend.

What no out-of-workspace consumer can do is decide which `bool` to report for `tiler.metal.route-requirement.minimum-gpu-family`, because deciding it means decoding the payload into a family and comparing it under that vocabulary's own ordering, and `tiler-metal` is an internal crate a consumer may not name. `spikes/runtime/inline-dispatch` therefore answers `Unrecognized` for every row, which is correct and fail-closed and is not a viable end state once a Metal plan states the row.

### Decision

1. **The channel is not extended.** No answer variant, payload accessor, or observation shape is added to `tiler-runtime` or `tiler-artifact`. The neutral layer keeps carrying opaque bytes and a `bool`, and the loader keeps every comparison it holds today.

2. **The owning backend owns the decode, the probe order, the ordering, and the qualitative decision**, published as one item with an exhaustive match over the neutral requirement vocabulary. An undecodable payload, an unknown key, and an unknown version are `Unrecognized` and never a negative verdict, because the two produce different loader refusals with different repairs.

3. **The published surface must not require an out-of-crate total map over the backend's own vocabulary.** Where observing the device means naming a foreign constant, the backend publishes an observation function that walks its own vocabulary and takes the caller's probe as a callback over an opaque raw constant. This is convention 5b and 5c applied rather than amended: a variant added to a backend vocabulary must be a compile error inside the defining crate, never a silently unprobed case at each consumer.

4. **The consumer's adapter observes and never decodes.** It supplies the probe, passes the backend's observation back unexamined, and maps the backend's three-valued answer onto `LiveDeviceObservation` — a total map, so that answer type is not `#[non_exhaustive]`. It performs no string comparison of its own, which removes the governed key and version from every call site.

5. **`tiler-metal` becomes a crate a *dispatching* consumer may name, and the facade does not.** The dependency arrow is consumer→backend, never core→backend or facade→backend. `crates/tiler/tests/dependency_direction.rs` is extended to assert the facade holds no edge to `tiler-metal`, with an anti-vacuity witness valid for an edge no frontend package holds.

6. **"A consumer names `tiler` alone" is a property of the non-dispatching consumer and of the fallback path**, and is restated that way in `docs/architecture.md`. Every non-dispatching use is unaffected.

7. **A satisfied route requirement is a live-device capability fact and never host-applicability eligibility.** A satisfied `minimum-gpu-family` row licenses continuation of that route and answers exactly one of `MetalHostPredicate`'s seven predicates. It contributes nothing to `evaluate_metal_host_applicability`, which remains structurally unreachable under ADR 0086 while `NativeTranslationAuthority` is uninhabited. The two comparisons over the one vocabulary run in opposite directions — the policy requires an exact family, the route row a minimum — and are never served by one function, nor published in one module.

8. **The pattern is available to every backend and free to none.** A backend minting a backend-scoped row must own a crate a consumer may name; a backend whose rows are all neutral `ResourceFloor`s owns nothing. For a process-bound fact the availability phase is borrowed and the `prepare` path is forced, and both costs are stated rather than absorbed. The *shape* is normative; its exact spelling is per-backend, because a probe over a foreign constant and a probe over a compiler intrinsic do not need the same signature.

9. **Fail-closed stays the interim.** Until a backend publishes its answer surface, `Unrecognized` is the only correct answer and the loader's refusal is the correct outcome.

### Consequences

- A dispatching consumer declares two crates and a fallback-only consumer declares one. That asymmetry becomes contract rather than an artifact of what happens to be reachable.
- The governed key, version, and payload spelling for a backend-scoped row acquire exactly one authority, replacing duplicated `const`s and two independently written scans in two prototypes.
- `MetalGpuFamilySupport`'s deliberate exhaustiveness becomes a promise to out-of-crate code. `MetalGpuFamily` itself stays unreachable from outside `tiler-metal`, which is what keeps its growth additive rather than hazardous.
- `tiler-build` gains the ability to mint the row it currently declares none of, and the producing and consuming halves land as one codec.
- The exact public shapes remain Tom's under ADR 0075 and AGENTS.md; this record decides the model.

### Alternatives considered

**Re-export the backend vocabulary through the facade.** Eliminated by Tom on 2026-08-01: no facade-reachable signature names the vocabulary, so the `tiler::runtime` whole-module re-export precedent does not apply — a consumer must *produce* a family, not read one. It would also make the facade enumerate backends, and a second backend a second edge. The closure-cost argument sometimes offered alongside is weak and should not be relied on: `tiler-metal`'s dependencies are `tiler-artifact` and `tiler-ir`, both already in the facade's closure.

**Publish the family vocabulary and let each consumer observe the device itself.** Eliminated on ADR 0074 conventions 5b and 5c. Mapping a Tiler family onto its Apple constant is a total map with no derivable wildcard value, so it is a 5b site; and written as a table rather than a match — which is what the existing prototype does — a variant added to the vocabulary compiles cleanly, is never probed, and silently under-reports the device. That is 5c's named failure: fail-closed but silently incomplete.

**Feature-gate the vocabulary onto the facade.** Eliminated on Cargo semantics. Features are additive and unify across a build graph, so one crate enabling `metal` puts the backend in every crate's closure — restoring the property the elimination above rests on, by a mechanism that hides it — and it puts backend names in the facade's own feature namespace.

**Neutralize the vocabulary into the runtime or artifact layer.** Eliminated in three parts. A neutral *enum* of Apple families is refused by the artifact layer's own contract sentence, which names this exact case. A neutral *ordered token* is refused by correctness: the only ordering a neutral layer has is lexicographic, `"Apple10"` sorts below `"Apple9"` byte-for-byte, and a vocabulary's ordering is a fact about the backend that mints it; carrying a rank instead moves the same authority problem into data minted twice. A neutral *carrier of opaque bytes validated by the owning backend* is not an alternative at all — it is what `BackendFeatureRequirement` already is and what this decision keeps.

**Fail-closed forever.** Rejected as a terminus and accepted as the interim by Tom on the same date. It leaves a producer able to mint rows nothing on the primary consumer path can answer.

**Let a satisfied family row contribute to the ADR 0086 eligibility policy.** Eliminated because it satisfies one of seven predicates while the receipt needs all seven, and would show near-eligibility in explain output for a host whose missing predicate is the one ADR 0086 item 3 says the measured environment row cannot stand in for.

### Traceability

[Backend-scoped route-requirement answers](../research/runtime/backend-scoped-route-requirement-answers.md) is the derivation, the eliminations, both worked examples, the public-boundary list, and the measurement boundary. [ADR 0090](0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 4 owns the report-versus-adjudicate division and the independent adapter selection this record extends; [ADR 0086](0086-require-attributable-or-attested-native-translation.md) owns the eligibility gate item 7 composes with; [ADR 0081](0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md) owns the loader's backend neutrality; [ADR 0074](0074-use-explicit-public-api-conventions.md) conventions 5b and 5c decide item 3; [ADR 0075](0075-scope-public-boundary-approval-by-change-category.md) routes the public boundaries to Tom. [The architecture contract](../architecture.md) owns the packaging profile and the sentence item 6 amends; [the artifact contract](../artifact-abi.md) owns the route-requirement family and its governed keys.

---

## Deferrals, each with its closing evidence and trigger

- **The convention 5b/5c defect on `MetalGpuFamily` was live at `6f7caf3` and independent of this design; it is now closed in one site and open in one.** `prototypes/candle-metal-adapter`'s family table was an out-of-crate total map that could not fail to compile when the vocabulary grew, and the type's doc comment asserted no such consumer existed. [`close-the-metal-gpu-family-out-of-crate-total-map`](../../../tickets/close-the-metal-gpu-family-out-of-crate-total-map.md) closed both halves at `662d9be` — the attribute is gone, the doc comment now states the opposite reason, the probe walk moved into `tiler-metal`, and `MetalGpuFamily::ALL`'s declared length makes a widened vocabulary a build error. The identical table survives at `prototypes/serial-sum-run/src/proof.rs:703-716` and needs a different fix, because `metal` 0.33.0's `MTLGPUFamily` has no safe constructor from a raw value; filed as [`close-the-serial-sum-run-gpu-family-probe-table`](../../../tickets/close-the-serial-sum-run-gpu-family-probe-table.md). Trigger: already fired for both; neither waits on this design.
- **Does `tiler-metal` become the owner of the governed backend key `"tiler.metal"`?** It does not name it today; `tiler-build` does, as `pub(crate)`. ADR 0090 item 11's promoted orchestration closure would have the backend supply its own payload declaration, which implies the key moves. Not decided here because it is a `tiler-build` and `tiler-metal` change under other scopes. Trigger: the item-11 promotion, or the first second backend needing `tiler-build`.
- **Does `dependency_direction.rs` need a second forbidden list or a per-entry witness?** Question 5 establishes that the obvious one-line change makes the test fail on its own anti-vacuity guard. Closes with the implementation ticket that lands the edge assertion. Trigger: implementing decision item 5.
- **Does a `tiler-cpu` crate exist before a CPU backend-scoped row does?** The CPU generalization is conditional on it. Closes with the CPU backend's own admission record. Trigger: the first CPU plan needing a qualitative live-host fact.
- **Is `LiveDevicePreflight` the right phase for a process-bound fact?** Named by ADR 0090 item 14 and owned by `name-a-host-process-availability-phase`; this design borrows the phase and does not resolve it. Trigger: the CPU ISA row.
