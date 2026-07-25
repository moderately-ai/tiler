---
id: prototype-artifact-family-delivery
title: Implement Apple artifact-family delivery selection
status: in-progress
priority: p1
dependencies: [prototype-neutral-artifact-codec, prototype-apple-aot-driver]
related: [generate-cfg-gated-artifact-family-delivery]
scopes: [implementation/frontend, implementation/metal-aot, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, apple-targets, inline-dx]
claimed_from: todo
assignee: agent-targets
lease_expires_at: 1784995760
---
Implement explicit family selection and generated routing for supported Apple artifact families, initially macOS, iOS device, iOS simulator, and explicit fallback-only behavior as contracted. Nonmatching targets must not silently select incompatible bytes; target facts belong in identity and diagnostics.

## Outcome

**Status: partially delivered; this ticket stays open.** The *selection* half landed and is tested. The *generated routing* half did not, because it belongs to a crate an accepted packaging profile places outside this ticket's scopes and which the frontend axis is review-gated from creating. `generate-cfg-gated-artifact-family-delivery` carries it with the full reasoning. Delivered on branch `tkt/prototype-artifact-family-delivery` from base `43f685f968699ca8a8edb01e384a00d7104565c8`.

### What landed (Fact)

`crates/tiler-metal-aot/src/family.rs` is a crate-private draft authority under ADR 0074 convention 7 — a private `mod family` whose items are `pub(crate)`, with a module-level `#![allow(dead_code, reason = "…")]` naming what it reserves and which slice consumes it. It holds:

- **`ArtifactFamilySelection`**, the canonical typed request field ADR 0049 requires every inline AOT compilation request to carry. A verified product: private field, fallible constructor, no `pub` fields (ADR 0074 conventions 4 and 6).
- **`ArtifactDeliveryPolicy`**, exactly the grammar `docs/integration/frontends.md` states — `SelectedFamilies([AppleArtifactFamily], RequiredWhenTargetMatches) | FallbackOnly`. `FamilyRequirement` is a one-variant enum rather than an implied constant, so the requirement mode reaches canonical bytes and the second mode `frontends.md` reserves becomes an explicit identity change instead of a silent behavioural one.
- **`SelectedFamily`**, carrying a deployment minimum and MSL standard **per family**. The macro-environment research contract requires it, and the facts genuinely differ: a macOS 13.0 floor and an iOS 13.0 floor are unrelated version lines, so one shared field would silently apply one family's minimum to another.
- **`compile_targets()`**, the fan-out to one `MetalTarget` per selected family, in canonical family order.
- **`canonical_bytes()`**, the identity subject ADR 0049's "fully participates in explain output and content identity" is over.

Sixteen tests; `cargo nextest run -p tiler-metal-aot --lib` runs 33 and passes 33.

### Three decisions, and why each went the way it did

**The family-to-SDK relation is resolved by search, not by a second table.** `sdk_for` scans the governed `AppleSdk` set for the one whose existing `AppleSdk::platform()` equals the requested family. A hand-written family-to-SDK match would have been a third copy of a relation `AppleSdk::platform` and `prototypes/serial-sum-compile/src/target.rs::sdk_for` already spell, and `prototype-metal-bundle-assembly` is explicit that a wildcard arm there "can only invent an `AppleSdk`… and the resulting artifact's provenance header and its actual compilation would disagree about what it is". The search returns `Option`, so a family no SDK produces is `FamilySelectionError::UnselectableFamily` rather than an invented selector. **This is a stated duplication that remains:** the prototype's own `sdk_for` still exists and is separately tested. It was not consolidated here because `prototypes/serial-sum-compile/**` was being edited concurrently by `carry-the-metal-payload-in-an-artifact-envelope`; see the collision note below.

**An empty `SelectedFamilies` list is rejected rather than treated as `FallbackOnly`.** ADR 0053 makes `FallbackOnly` "an explicit valid policy" that "invokes no backend compiler". Collapsing the two would make a producer's omission indistinguishable from a deliberate decision to perform no AOT work, and the contract requires the second to be explicit. `FamilySelectionError::EmptySelection` is what leaves `FallbackOnly` the only spelling of it.

**A repeated family is rejected, not deduplicated.** Two entries for one family disagree about that family's deployment minimum or standard whenever they differ, so silently keeping one would drop a stated compilation input.

### Identity is bytes, not a digest, and that is a refusal to invent one (Fact)

`canonical_bytes()` returns the domain-separated, length-prefixed, ordinal-free, exhaustively matched subject (ADR 0074 convention 3) and **no digest**. This crate's dependency closure is empty by decision (ADR 0077 item 2), so it owns no hash function, and the governed artifact digest is `tiler.digest.sha-256.v1` in `tiler-artifact`. Adding a local digest here would be a second identity authority over one subject; a caller that already owns the governed algorithm digests these bytes. Tested: declaration order does not change the bytes, every facet (family, major, minor, standard, family count) does change them, `FallbackOnly` has its own subject, and the length prefixes keep `ios-device`/`ios-simulator` runs unambiguous.

### What did not land, and why it is a boundary rather than a descope (Fact)

The ticket's "generated routing" is generated Rust: ADR 0053's `#[cfg]`-gated payload-or-`compile_error!` with a semantic fallback for nonmatching targets, plus the versioned family-to-consumer-`cfg` predicate map. Three independent facts put it outside this ticket:

1. **An accepted packaging profile places it elsewhere.** ADR 0077 item 1: `tiler-metal-aot` "does not emit MSL, does not assemble the target-neutral artifact bundle, and does not implement the expansion cache or the **proc-macro layer**". `docs/architecture.md`'s crate table assigns "emit artifact plus runtime/fallback tokens" to the frontend proc-macro crate. A consumer-target `#[cfg]` predicate is a fact about a *Rust* target; this crate knows only about `xcrun`.
2. **The owning crate cannot be created under these scopes.** `implementation/frontend` maps to `crates/tiler-macros/**` and `crates/tiler-frontend-*/**`; neither exists. Admitting a workspace member needs the root `Cargo.toml` (`implementation/workspace`), `Cargo.lock` (`implementation/cargo-lock`), and `scripts/check_workspace.py`'s pinned tables. This ticket declares none of those three. `prototype-apple-aot-driver`, which did admit a crate, declared `implementation/workspace` and carried an explicit clause authorizing it.
3. **The axis is gated on a review.** `record-that-the-frontend-axis-is-review-gated` closes with "Do not close this by starting frontend work that routes around the unreviewed boundary — that would answer the review question by omission."

`generate-cfg-gated-artifact-family-delivery` is `blocked` on `prototype-inline-proc-macro-frontend` and carries the reasoning, the measured `cfg` distinctions, the normative test list from `docs/correctness-and-testing.md`, and the Catalyst treatment.

### The ticket's second clause was already satisfied (Fact)

"Target facts belong in identity and diagnostics" is delivered by `assemble-the-metal-payload-from-emission-and-compilation`, not by this ticket. `tiler_artifact::program::PayloadProvenance` carries `target`, `family`, `language`, and both deployment components; those bytes are the payload's identity subject, and `push_carried_payload` *derives* the descriptor digest from them so a payload cannot claim a subject it does not carry. Nothing was added here, and nothing needed to be. Two facets the ticket names are therefore already true, which is why the remaining gap is routing rather than provenance.

### The unresolved architectural question this work surfaced (Inference)

Nothing in the corpus says whether one selection naming N families produces **N separate envelopes** or **one envelope carrying N Apple payloads**. `docs/integration/frontends.md` implies the first ("Each family remains a distinct artifact with its own target manifest and content identity"); `docs/research/apple-targets/artifact-compatibility.md` permits the second ("One neutral envelope may carry several Apple payloads, but each payload retains its own descriptor and digest"). `docs/artifact-abi.md` deliberately leaves the bundle-identity seam open. `ArtifactFamilySelection` as landed is agnostic — it yields N compile targets and says nothing about packaging — so it does not pre-empt the choice, but the follow-up cannot emit tokens without it being settled. Recorded here rather than decided.

### A documentation divergence found while siting this work (Fact)

`docs/architecture.md`'s crate table gives `tiler-metal-aot` "Expansion-time Apple tool invocation, **cross-process content cache, atomic publication, byte embedding**, and the target facts a compiler invocation selects". ADR 0077 item 1 says the crate "does not implement the expansion cache or the proc-macro layer", and the crate's own `lib.rs` restates ADR 0077. The two disagree about the cache, atomic publication, and byte embedding. Both agree about the artifact family, which is what this ticket needed, so the divergence did not block the work; it is unowned and outside `contracts/foundation`, which this ticket does not declare.

### Collision (Fact)

`carry-the-metal-payload-in-an-artifact-envelope` was `in-progress` at this base, co-declaring `implementation/metal-aot` and `implementation/artifact`, and its recorded work is in `crates/tiler-artifact`, `crates/tiler-compiler`, and `prototypes/serial-sum-compile`. This change touches none of those: it adds one new file under `crates/tiler-metal-aot/src/` and four lines to that crate's `lib.rs`. `crates/tiler-artifact/**` was deliberately not edited.

### Verification (Measurement)

`uv run --locked python scripts/check_repository.py` passes (`complete repository validation passed`). `uv run --locked python scripts/docs.py render`, `cargo fmt --all -- --check`, strict Clippy, `git diff --check`, `tkt lint`, and `tkt guard tkt/prototype-artifact-family-delivery` all pass.

**An Apple toolchain was available, and that is asserted rather than assumed.** The gate's log contains zero `golden_compilation: skipped` lines, and `TILER_REQUIRE_METAL_TOOLCHAIN=1 cargo nextest run -p tiler-metal -E 'test(golden_compilation)'` — which converts a self-skip into a hard failure — runs 7 and passes 7. The resolved row is `metal` and `metallib` version `32023.883` (`metalfe-32023.883`, AIR-LLD linker) against macOS SDK `26.5` build `25F70`, which is the qualified compatibility row. The four goldens linked 3,683 / 3,715 / 3,747 / 3,859 bytes and the four-entry-point portfolio linked 14,716 bytes. Those counts are run evidence, not assertions: `golden_compilation`'s own module documentation records why a `metallib` byte count is never asserted.

`cargo nextest run -p tiler-metal-aot --lib` runs 33 and passes 33, of which 16 are the new `family::tests`.
