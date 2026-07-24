---
id: choose-one-owner-for-apple-target-vocabulary
title: Choose one owner for the shared Apple target vocabulary
status: in-progress
priority: p2
dependencies: []
related: [prototype-metal-kir-lowering, prototype-apple-aot-driver, compile-golden-msl-through-the-aot-driver-in-the-gate]
scopes: [implementation/metal, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, api-hardening]
claimed_from: todo
assignee: agent-choose-one-owner-for-apple-target-vocabulary
lease_expires_at: 1784917695
---
`tiler-metal` and `tiler-metal-aot` now each define their own MSL language
version, Apple platform family, and deployment minimum:

- `tiler_metal_aot::input::{MslVersion, AppleSdk/ApplePlatform, DeploymentMinimum}`
- `tiler_metal::target::{MslLanguageVersion, MetalPlatform, MetalDeploymentMinimum}`

These describe the same facts about the same targets. Two independent
vocabularies for one domain is how a "3.1" in one crate and a "3.1" in the other
eventually disagree — most likely when one gains a version or platform the other
does not, and a caller translates between them by hand.

The duplication was **forced, not careless**. `tiler-metal-aot` is deliberately
dependency-free (it shells out to `xcrun` and must not drag in the lowering
stack), so it cannot depend on `tiler-metal`. And an MSL language version is not
target-neutral enough to belong in `tiler-artifact` alongside genuinely
backend-agnostic artifact vocabulary. So neither existing crate is an obviously
correct owner, which is why this needs a decision rather than a refactor.

Options to weigh, none obviously right:

- **`tiler-metal` owns it and `tiler-metal-aot` depends on it** — natural
  direction (source emission knows the language), but it breaks the driver's
  dependency-free property, which exists so the driver stays usable and auditable
  in isolation.
- **A small shared crate** owned by neither — clean, but admits a new workspace
  member for three enums, and `AGENTS.md` warns against scaffolding crates ahead
  of need.
- **Keep both and add a checked correspondence** — a test asserting the two
  vocabularies stay in step. Cheapest, keeps both crates' properties, but it is a
  guard rather than a fix and grows with every added variant.
- **Accept the duplication explicitly** and record why, so the next reader does
  not "fix" it into a worse shape.

Whichever is chosen, record the reasoning where a future reader meets the
duplication, not only in this ticket. If the decision is to keep both, that
outcome is legitimate — an unrecorded accidental duplication is the failure, not
duplication itself.

## Outcome

**Decision — both crates keep their own vocabulary, and the correspondence between them is enforced rather than shared.** That is the ticket's fourth option carried by its third: the duplication is accepted and recorded on the types, and a total checked correspondence is what makes accepting it safe. Neither type moved, no signature changed, and no crate was added, so nothing here needs Tom's approval under ADR 0075.

**Fact — the two records are not one type in disguise, which is what decides it.** `tiler_metal::target::MetalTargetFacts` carries six fields and `tiler_metal_aot::input::MetalTarget` carries three; they overlap in exactly the language standard, the artifact family, and the deployment minimum. The emitter additionally owns a `LaunchIndexRealization`, a `MetalSubnormalArithmetic`, and a buffer binding capacity, none of which a compiler invocation has any use for. The driver additionally owns `AppleSdk`, which selects `xcrun --sdk` and builds the `air64-apple-*` triple — tool-discovery knowledge the emitter must never acquire, and which has no emitter counterpart at all. Neither record subsumes the other. A shared crate would therefore relocate three items and leave both crates still owning the rest of their target vocabulary, in exchange for a new workspace member (which `AGENTS.md` warns against ahead of need), a new publicly reachable namespace and three moved public types (both Tom's under ADR 0075), and two more scopes this ticket did not hold. It buys no invariant the checked correspondence does not already give.

**Fact — the dependency-direction objection to option 1 does not survive checking, and the real objection is a different one.** The dispatch brief stated that option 1 — `tiler-metal` owns the vocabulary and `tiler-metal-aot` depends on it — points opposite to the direction the development-only edge was chosen to keep open. It does not. `compile-golden-msl-through-the-aot-driver-in-the-gate` recorded that it kept `tiler-metal` → `tiler-metal-aot` out of the normal graph precisely to preserve the eventual `tiler-metal-aot` → `tiler-metal` production direction, and option 1's edge *is* `tiler-metal-aot` → `tiler-metal`. Same direction, not opposite. What actually rules option 1 out is the dependency *closure*: `tiler-metal` depends on `tiler-ir` and `tiler-artifact`, so the driver would acquire the whole lowering stack to obtain three enums, destroying the property that makes a crate whose job is spawning `xcrun` auditable in isolation.

The direction argument does bite — on a fifth option the ticket does not list. Making `tiler-metal` normally depend on `tiler-metal-aot` for the vocabulary would put Apple tool discovery into every consumer's build graph, which is the stated reason the edge is development-only, and Cargo's cycle rule would then forbid `tiler-metal-aot` → `tiler-metal` outright. That option is worse than option 1 on both counts and is likewise rejected.

**Fact — the existing correspondence guard is pointwise, so choosing option 3 was not ratification.** `every_golden_declares_the_target_the_driver_compiles_it_for` compares one hard-coded macOS 13.0 MSL 3.1 target in both spellings. It is green today and would stay green if either crate gained a language standard or an artifact family the other lacked, because it never looks at a variant the fixtures do not use. It proves the goldens are compiled for the target they declare — its actual and correct purpose — and nothing about the vocabularies. The divergence this ticket names is exactly what it does not cover.

**Fact — `crates/tiler-metal/src/target_correspondence.rs` closes that.** It maps each vocabulary onto the other *totally*, through four exhaustive index matches paired with count-declared tables, so a widened index function that is not accompanied by a new pair is an array-length error rather than a short table. Five tests assert that the family tables and the language tables each cover every variant of both vocabularies exactly once, that paired variants produce identical stable identifiers and `-std` tokens, and that both deployment minimums carry and render a version identically. `tiler-metal-aot` gains `every_artifact_family_names_the_sdk_that_selects_it`, whose exhaustive match over `ApplePlatform` keeps every family reachable through an SDK selector — the driver-internal half of the same property.

**Measurement — the guard has teeth in both directions.** On the pinned `nightly-2026-07-19` toolchain, adding a probe `ApplePlatform::MacCatalyst` to `tiler-metal-aot` fails `cargo nextest run -p tiler-metal --no-run` with `error[E0004]: non-exhaustive patterns: 'ApplePlatform::MacCatalyst' not covered` at `target_correspondence.rs:73`; adding a probe `MslLanguageVersion::Metal3_2` to `tiler-metal` fails the same command with `E0004` at `target_correspondence.rs:85`. Both probes were reverted. The out-of-crate half compiles only because the driver's enums are exhaustive, which is now stated on those types as a requirement rather than left as an accident.

**Fact — the check can only live in `tiler-metal`, and can only ever be a test.** `tiler-metal-aot` cannot see `tiler-metal`, so the development-only edge is the sole place in the workspace where both vocabularies are simultaneously visible. The same fact bounds what the correspondence can be: a production `MetalTargetFacts` → `MetalTarget` conversion needs a normal dependency in one direction or the other, so it belongs to whichever component eventually orchestrates emission and compilation together. `prototype-metal-bundle-assembly` now records that it inherits that obligation, that the translation must be total rather than wildcard-defaulted, and that writing it makes `MslLanguageVersion` and `MetalPlatform` ADR 0074 convention 5b types whose `#[non_exhaustive]` it then owns removing.

**Fact — where the reasoning now lives.** On the types, which is what the ticket asked for. `crates/tiler-metal/src/target.rs` and `crates/tiler-metal-aot/src/input.rs` each open with a section stating that the duplication is decided, why the crate keeps its own copy, why the other options were rejected, and what enforces the correspondence; each of the three duplicated types on both sides carries a note naming its counterpart and what it means differently. `crates/tiler-metal/src/lib.rs` records the second obligation the development dependency carries, and `golden_compilation.rs` now says explicitly that its target check is pointwise and does not cover the totality obligation.

**Fact — an `#[non_exhaustive]` premise in a sibling ticket did not survive checking.** `harden-public-enums-non-exhaustive` planned to mark `tiler_metal_aot::input::MslVersion` on the stated ground that the driver's types "have no consumer outside `tiler-metal-aot` at all". That has been false since the development edge landed, and `#[non_exhaustive]` binds every out-of-crate consumer regardless of dependency kind. Executing the list as written would have forced a wildcard arm into the correspondence and silently removed the guard this ticket exists to install. That ticket is corrected: `MslVersion` is removed from its mark list, `ApplePlatform` is recorded as needing to stay off it, and `DriverError` is added as a convention 5c recognizer — `golden_compilation::resolved_toolchain` matches it out of crate to separate an absent Apple toolchain from a defect, and a wildcard there would convert a future defect into a skipped test. `DriverError` now says so in its own doc comment. `AppleSdk` and `OptimizationLevel` stay on the mark list: `tiler-metal` constructs both and matches neither, which is 5a.

**Deliberately not changed: `#[non_exhaustive]` on `tiler_metal::target::{MslLanguageVersion, MetalPlatform}`.** The dispatch brief read these as the exhaustive case. Checked against the source, the attribute is inert for them today: the only total map is `target_correspondence`, which lives in the crate that defines them, so `#[non_exhaustive]` does not reach it and both directions of the guard already work. ADR 0074's amendment states that the 5a/5b/5c classification "is a property of the consumers that exist" and that the ticket adding an out-of-crate consumer owns re-checking it. Pre-emptively stripping an attribute for a consumer that does not exist would apply the rule ahead of its own stated trigger. What was done instead is to write the classification and its exact trigger onto both types, and to name the owner — `prototype-metal-bundle-assembly` — in that ticket. The two driver enums, where the attribute is not inert, are the ones now pinned exhaustive.

**Fact — no ADR was written, and the question is routed rather than dropped.** This decision changes no public surface: nothing moved, no signature changed, no namespace opened, so none of ADR 0075's always-ask categories applies. The reasoning lives on the types. Whether the ownership split additionally warrants an ADR in the genre of ADR 0065 and ADR 0070 is a real question and is now owned by `record-metal-aot-in-architecture-crate-profile`, which must answer it either way rather than leave it implicit.

**Follow-up filed.** `record-metal-aot-in-architecture-crate-profile` (`contracts/foundation`, a scope this ticket did not hold) owns two `docs/architecture.md` gaps found here: the "Accepted prototype packaging profile" block names five libraries and omits `tiler-metal-aot` entirely, though the workspace has had six library members since the driver landed; and the Component ownership table's `tiler-metal` row ("Metal target metadata") reads as if one crate owns all of it, which is the sentence a future reader would cite while consolidating this duplication into the shape this ticket rejects.

**Measurement boundary.** The correspondence is a compile-time and structural guarantee about two Rust vocabularies. It says nothing about whether any listed family or standard actually compiles: only macOS 13.0 under MSL 3.1 is compiler-validated, by the golden tests, on one host. It also cannot constrain a translation written outside `tiler-metal`, because no such translation can exist while neither crate normally depends on the other — which is why the obligation is written into the ticket that will first need one rather than assumed to be inherited.
