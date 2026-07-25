---
id: prototype-runtime-artifact-validation
title: Implement runtime artifact validation
status: done
priority: p0
dependencies: [prototype-neutral-artifact-codec, admit-the-device-free-runtime-validation-crate]
related: [carry-reconstructable-kernel-programs-in-the-neutral-envelope]
scopes: [implementation/runtime, implementation/artifact, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, runtime, artifact]
---
Implement runtime-owned device-free decoding, integrity/program/ABI validation, checked expression evaluation, and typed compatibility classification. The runtime path must not import semantic IR, optimizer state, backend internals, or proof-sidecar semantics.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

## Blocked 2026-07-25 — two independent blockers, both established by reading

Attempted from `implementation/runtime`, `implementation/artifact`, `implementation/workspace`, with `project/tickets` and `implementation/cargo-lock` shared. One in-scope defect was found and fixed on the way; the ticket's own deliverable is unreachable from these scopes.

### Blocker 1 — the crate admission is not a workspace edit

Split into [`admit-the-device-free-runtime-validation-crate`](admit-the-device-free-runtime-validation-crate.md), which carries the full evidence. In short: `docs/architecture.md:352-354`'s accepted packaging profile "deliberately omits … reusable Metal-runtime crates until the proof reaches those boundaries", `docs/decisions/0077-…:80` says "A reader must not cite this admission as precedent for admitting one", and no doc, ADR, or ticket names a `tiler-runtime` crate — the string exists only in `ticketsplease.toml`'s scope glob. Admitting the crate needs `contracts/foundation` and `contracts/decisions` alongside `implementation/workspace`; landing only the workspace half would leave `scripts/check_workspace.py` disagreeing with the accepted architecture text, which is the exact state ADR 0077 exists to end.

Whether the *device-free* crate this ticket describes falls under the withheld clause at all is a genuine question — by ADR 0077's own test ("never touches a live device, an `MTLDevice`, or a pipeline state") it does not — and that is why it was split rather than decided here.

### Blocker 2 — "checked expression evaluation" is not reachable from decoded bytes

**Fact.** `DecodedArtifact` in `crates/tiler-artifact/src/program/codec/view.rs` exposes exactly `identity()`, `features()`, `routing()`, `payloads()`, `sections()`, `variant_count()`, and `re_encode()`. There is no accessor for a variant, an entry, a binding, an ABI expression, or a launch contract.

**Fact.** The envelope holds them. `codec/model.rs:341-360`: `EntryRow` carries `stage`, `resources`, `numerical`, `bindings`, `launch`, `payload`, `entry_key`, and `VariantRow` carries `guard`, `profile`, `feasibility_rules`, `deferred`, and `entries`. All are `pub(crate)`; nothing projects them.

**Inference.** The gap is a promotion boundary, not an encoding gap — a decode validates every one of those rows and `re_encode()` writes them all back — but it means no out-of-crate consumer can evaluate a guard, a binding's accessible byte range, or a launch formula from bytes. Every expression accessor that exists (`VariantRef::applicability_guard`, `EntryRef::bindings`, `BindingRef::accessible_bytes`, `EntryRef::launch_threads`, `AbiExprRef::evaluate`) hangs off `VerifiedArtifactProgram`, which no decode produces and which — per `carry-reconstructable-kernel-programs-in-the-neutral-envelope` — cannot be reconstructed.

**Consequence for this ticket.** Three of the four stated deliverables *are* reachable today: device-free decoding, integrity validation, and typed compatibility classification all fall out of `decode_artifact` plus `identity()`/`features()`/`payloads()`. The fourth, checked expression evaluation, needs either a promoted dispatch-record projection on `DecodedArtifact` or the binding-by-identity design in which the runtime holds the `VerifiedArtifactProgram` it compiled and uses the decoded identity to prove the loaded bytes name it. Choosing between those is `carry-reconstructable-kernel-programs-in-the-neutral-envelope`'s decision, recorded there; this ticket must not pre-empt it by promoting a surface.

### Landed under this ticket

`crates/tiler-artifact/src/program/codec/view.rs`'s module documentation claimed a reader gets "everything the envelope actually holds: … and each variant's entries with their ABI and launch expressions". That is false about the read surface and was the reason this ticket read as implementable. It now states what `DecodedArtifact` exposes, that the entry/binding/launch rows are held but unreachable, that this is a promotion boundary rather than an encoding gap, and what that implies for how a runtime is written. No public item, dependency, or contract changed.

## Outcome

Both blockers above are resolved and the reachable deliverables are landed in `crates/tiler-runtime`. Blocker 1 closed when Tom approved the crate (`admit-the-device-free-runtime-validation-crate`, ADR 0081). Blocker 2 closed as a *decision* rather than as an implementation: Tom decided a decoded envelope is a dispatch record bound by identity and never a reconstruction, so this ticket's fourth deliverable is answered by binding rather than by evaluation, and the encoding work that would widen it is the other ticket's.

**The dependency on `carry-reconstructable-kernel-programs-in-the-neutral-envelope` was removed and re-recorded as `related`.** That ticket is `review`, which never satisfies a dependent, and its remaining work is amending `docs/artifact-abi.md` under `contracts/artifacts` — a scope this ticket does not hold and does not need. The question it existed to gate here (does the loader evaluate expressions, or bind by identity?) is decided; keeping the edge would have deadlocked a p0 behind documentation it does not depend on.

### What landed

`crates/tiler-runtime` is three stages expressed as three types, so ADR 0051's one-way routing commit is structural rather than a rule to remember:

- `DecodedProgram::decode` — bytes to a fully validated view, or `LoadRejection::Artifact` carrying `ArtifactCodecFailure` whole. The validation is `tiler-artifact`'s; this crate re-implements none of it and cannot weaken it. Holding a `DecodedProgram` is the evidence the bytes passed framing, digests, schema, canonical order, arena closure, feature negotiation, and identity re-derivation.
- `DecodedProgram::preflight(&ExecutionEnvironment, &CanonicalArtifactProgramIdentity)` — every remaining decidable obligation, in the order whose first refusal is most useful: program identity binding, then routing policy and variant cardinality, then payload selection, then declared-target-profile classification, then execution policy, then object resolution.
- `Preflight::commit` — consumes the preflight and is **infallible**. There is no `Result`, because every decidable obligation was discharged in the stage before, and consuming `self` is what makes the commit one-way: a caller cannot afterwards hold the value a fallback would need. A caller takes a fallback by not calling it, which is exactly "fallback only before program work".

`ExecutionEnvironment::classify` is a typed classification, not a boolean: `Compatible`, `ProfileKeyMismatch`, `DescriptorMismatch`. ADR 0043 makes the exact descriptor a feasibility input rather than a hint, and the two failures mean different things — a wrong target family means look for another artifact, the same family under another descriptor means rebuild. Collapsing them to `false` erases that at the moment a caller needs it.

`RoutingPolicy` and `ArtifactExecutionPolicy` are matched exhaustively rather than through a wildcard. Neither is `#[non_exhaustive]` (ADR 0074 convention 5b), so a policy added to the artifact layer is a build failure here instead of silently reusing stable-priority selection or being handed to a device that cannot translate it.

### The dependency direction the ticket asked for is mechanically enforced

"The runtime path must not import semantic IR, optimizer state, backend internals, or proof-sidecar semantics." The crate's entire dependency closure is `[tiler-artifact]`, pinned in `scripts/check_workspace.py`'s `EXPECTED_DEPENDENCIES`. `tiler-ir` is not even a direct edge — every type the loader names is an artifact-layer type — so the prohibition is a checked contract rather than a discipline.

### Three refusals, and why each is the honest shape rather than a gap to route around

- **More than one packaged variant** is refused. Choosing among variants means evaluating applicability guards, and a guard is reachable only through a `VerifiedArtifactProgram` no decode produces. Taking the first variant would treat declaration order as a decided guard.
- **More than one payload descriptor** is refused at object resolution. `BackendPayloadDescriptor::digest` is the digest of the payload's *compilation subject*, not of its object; the section table is content-addressed and deduplicates equal objects; and the descriptor-to-section map (`PayloadSections`) is `pub(crate)` with no accessor on `DecodedArtifact`. One descriptor plus one object section is the only cardinality in which the association is forced rather than guessed.
- **Any execution policy but `NativeImage`** is refused, because device translation is by definition not device-free.

### What is deliberately not claimed

- **No cache.** `AGENTS.md`'s cache obligations — complete identity, validation on every hit, immutable entries, atomic publication, defined crash/race behaviour — are **not implemented here and are not claimed**, because this crate holds no cache. Validation-on-every-hit is available to a cache built over it (`DecodedProgram::decode` revalidates unconditionally and cannot be skipped), but the entry store, its publication, and its crash/race behaviour belong to the `tiler-cache` crate Tom approved separately. Nothing here was tested for a crash or a race, and no such property is asserted.
- **No checked expression evaluation.** Superseded by Tom's binding-by-identity decision rather than deferred silently: a decoded envelope carries no evaluable guard, byte range, or launch formula.
- **No positive-path test in this crate.** Six unit cases cover the decode classification (foreign bytes are malformed rather than damaged, empty input is refused, the classified rejection keeps the codec failure reachable as its `source`) and the whole of the compatibility classifier. `preflight` and `commit` have **no test in this crate**, because exercising them needs a valid artifact and this crate cannot build one — `ArtifactProgramBuilder` needs a `VerifiedKernelProgram`, which needs `tiler-ir` and roughly 250 lines of fixture that already exist twice in the workspace. Their evidence belongs with a real compilation, in `route-the-runtime-proof-through-the-artifact-envelope`, and that ticket is blocked; see its own note. Until it lands, `preflight` and `commit` are **implemented and untested**, which is a weaker claim than the rest of this outcome and is recorded as one.

### Measurement

`cargo nextest run -p tiler-runtime`: 6 tests, 6 passed. `cargo clippy -p tiler-runtime --all-targets`: clean under the workspace's `pedantic` set. `cargo fmt -p tiler-runtime -- --check`: clean. The complete `scripts/check_repository.py` result is recorded on the branch's final commit.
