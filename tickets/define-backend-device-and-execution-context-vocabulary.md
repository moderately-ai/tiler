---
id: define-backend-device-and-execution-context-vocabulary
title: Define backend, device, and execution-context vocabulary
status: in-progress
priority: p1
dependencies: [correct-stale-public-compiler-boundary-authorities]
related: [draft-public-extension-seam-ownership-adr, multi-device-and-sharding-scope-gate]
scopes: [contracts/foundation, contracts/artifacts, contracts/integrations]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, documentation, architecture]
---
## User-visible outcome

A reader and future API author can distinguish a backend, backend family, provider, target profile, artifact family, representation, live device, execution context, device-free execution environment, and runtime adapter without inferring meaning from the current Metal crate split.

## Why this slice exists

Only `Target profile` is defined as shared glossary vocabulary. Backend and device responsibilities are coherent but scattered, `BackendKey` is represented without a conceptual ownership contract, and the public device-free `ExecutionEnvironment` can be mistaken for the live device/context it deliberately does not contain. The word `family` currently names backend, target-profile, artifact, and GPU/device subjects.

## Implementation keys

- Derive definitions from accepted ADRs 0043, 0047, 0072, 0078, 0081, and 0085 and from the exact construction sites of `BackendKey`, `RepresentationKey`, `TargetProfileRef`, and `ExecutionEnvironment`.
- State that a target profile is typed compile-time data, a device is a live execution resource, and a runtime execution context scopes device objects, queues, caches, and asynchronous lifetimes.
- Define a backend by responsibility rather than by copying `tiler-metal` packaging: it consumes verified physical work and produces a target representation, while AOT invocation, artifact assembly, loading, and live execution may have different owners.
- Distinguish provider identity from backend-family and representation identity.
- Add crate-role naming guidance such as `tiler-<backend>` only where the role is actually backend-owned; do not prescribe one package topology for every target.
- Preserve the accepted one-device initial profile and explicitly defer multi-device semantics to its existing scope gate.
- Prove every absence/conflict search used by the update can fail, and read every edited contract in full.

## Closes when

The glossary and governed contracts use one subject for each term, every ambiguous use of `family` or `environment` in the affected passages is qualified, no proposed provider-bundle API is presented as accepted, local links and hand-maintained catalogs agree, and `tkt lint` plus `git diff --check` pass.

## Graph maintenance

- Follow the stale compiler-boundary correction rather than duplicating its installation-status edits.
- Feed these terms into the provider-composition research and every later public-boundary ticket.
- Keep `multi-device-and-sharding-scope-gate` deferred; this ticket defines vocabulary and does not activate that product decision.

## Outcome

The eleven terms are defined in [the glossary](../docs/glossary.md), and a new section separates them from one another and indexes the senses of `family` and `environment`. Nothing here creates a public API, and no proposed provider-bundle surface is presented as accepted.

### What the terms were derived from

**Fact — only one of the eleven had a glossary row before this change.** Extract the base file and count fixed-string row prefixes:

```
git show b623670:docs/glossary.md > /tmp/g.md
for t in Backend "Backend family" Provider "Target profile" "Artifact family" \
         Representation "Live device" "Execution context" "Runtime adapter" "NOT A REAL TERM"; do
  printf '%-22s %s\n' "$t" "$(rg -cF "| $t | " /tmp/g.md || echo 0)"
done
```

`Target profile` reports 1 and every other term reports 0, which is the check's own proof that it distinguishes a present row from an absent one; the trailing `NOT A REAL TERM` reports 0 in both directions, so a spurious match would be visible. Run against the current file, all eleven report 1 or 2 — 2 where a term appears both as a term row and as a row of a disambiguation table.

**Fact — construction and consumption sites, read in full rather than grepped.**

- `BackendKey` is macro-generated at `crates/tiler-artifact/src/program/keys.rs:182-186`, documented "a governed backend family key". Production construction: `crates/tiler-build/src/metal_assembly.rs:153` and `:216` from `BACKEND = "tiler.metal"` (`metal_assembly.rs:27`); `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs:231` from `profile::BACKEND_KEY = "tiler.cpu.scalar"` (`profile.rs:47`); wire decode at `crates/tiler-artifact/src/program/codec/decode.rs:521` and `:857`. Consumption: field of `BackendPayloadDescriptor` (`program/model.rs:495`), folded into the payload canonical key and therefore artifact identity (`model.rs:529-563`), owner of a `BackendFeatureRequirement` (`program/requirement.rs:197`), and two routing refusals — `UnexecutablePayload` at `crates/tiler-runtime/src/load.rs:440-449` and `ForeignRouteRequirementOwner` at `load.rs:585-595`.
- `RepresentationKey` is generated at `keys.rs:187-191`, "a governed executable-representation key of one backend payload". Values reached in production: `metallib` (`metal_assembly.rs:28`), `metal-source` as a *source* representation (`metal_payload.rs:10`), and `tiler.cpu.scalar-image-v1` with `tiler.cpu.kernel-identity-list-v1` in the CPU vertical (`profile.rs:50`, `:59`). It is compared with the backend key as a pair in one condition at `load.rs:440-441`.
- `TargetProfileRef` is at `keys.rs:258-269`: governed key plus exact descriptor identity, with the ADR 0043 reason stated on the type. Construction at `crates/tiler-build/src/metal_declaration.rs:372-379` and `crates/tiler-build/src/metal_plan.rs:331-338`. It has two distinct consumption roles — `VariantSpec::target_profile`, the plan's declared requirement, and `BackendPayloadDescriptor::compatibility`, the object's own contract, whose non-collapsibility `model.rs:504-513` states — and portfolio-wide agreement is enforced at `program/builder.rs:1208-1210`.
- `ExecutionEnvironment` is at `crates/tiler-runtime/src/load/host.rs:29-38`: `{ target_profile, backend, representation }` and nothing else. Its module header (`host.rs:4-12`) states why the host declares rather than the loader discovers — "Discovery needs a device" — and `classify` (`host.rs:49-62`) separates a wrong-artifact key mismatch from a rebuild-me descriptor mismatch instead of returning a boolean. Production construction only at `prototypes/serial-sum-run/src/proof.rs:645-661` and `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs:221-235`, each documenting that reading the profile back out of the artifact would make `classify` a tautology.

**Fact — `ExecutionEnvironment` appeared nowhere in `docs/` before this change.** `git grep -c ExecutionEnvironment b623670 -- docs/` returns no lines; the same command for `TargetProfileRef` returns one, which is the control showing the search reaches `docs/` at that commit. The concept travelled only as the prose "a host's stated execution environment".

### The term table installed

| Term | One-line definition | Where it now lives |
| --- | --- | --- |
| Backend | The responsibility of translating verified physical work into one target representation and declaring the target facts it depends on — a role, not a package | `docs/glossary.md` term row; `docs/architecture.md` component-ownership paragraph; `docs/backends/cpu.md` |
| Backend family | The governed identity a `BackendKey` names, saying whose vocabulary owns a representation, a route requirement, and entry symbols | `docs/glossary.md` term row and `family` table |
| Provider | A registered authority whose versioned capability admits, lowers, or proposes work; provenance rather than meaning, and never believed without re-derivation | `docs/glossary.md` term row |
| Target profile | Typed compile-time data referenced as key plus exact descriptor; never a live device or a live-device observation | `docs/glossary.md` term row, extended |
| Artifact family | One governed compilation target a payload is built for and delivered under; selects an SDK, triple, and deployment minimum and decides no GPU | `docs/glossary.md` term row and `family` table |
| Representation | The governed executable form of a payload's bytes within a backend family | `docs/glossary.md` term row |
| Live device | An execution resource bound at run time, and what `LiveDevicePreflight` observes; exactly one under the accepted initial profile | `docs/glossary.md` term row |
| Execution context | The runtime scope owning device objects, queues, pipeline states, and asynchronous lifetimes | `docs/glossary.md` term row |
| Execution environment (host declaration) | `tiler_runtime::load::ExecutionEnvironment` — what a loading host states, device-free, and deliberately not the live device or context | `docs/glossary.md` term row and `environment` table; qualified in `docs/architecture.md` |
| Execution environment (measurement) | `tiler_compiler::target::TargetExecutionEnvironment` — where a behaviour was observed, retained as provenance | `docs/glossary.md` term row and `environment` table; qualified in `docs/backends/metal.md` |
| Execution environment (host process) | The OS and process a kernel happens to run in; not fixed at expansion time, so no artifact identity names it | `docs/glossary.md` term row and `environment` table; qualified in `docs/artifact-abi.md`, `docs/backends/metal.md`, `docs/integration/candle.md` |
| Runtime adapter | The consumer- and device-specific component binding an artifact to a live device and context, and answering its own backend's route requirements | `docs/glossary.md` term row; `docs/integration/candle.md` ownership boundary |

### Disambiguation, and the boundary drawn around it

The new glossary section indexes eleven subjects for `family` and eleven for `environment`, and closes with the rule that neither word may appear unqualified in normative text or diagnostics.

**The scope of "every ambiguous use" was bounded deliberately, and the bound is stated rather than left implicit.** `rg -c -iw 'family|families' docs/` reports 983 matching lines across 88 files and `environment|environments` reports 368 across 65. Qualifying all of them is a corpus-wide rewrite touching six scopes this ticket does not hold. What was qualified is every ambiguous use in the passages this vocabulary actually touches, inside this ticket's scopes: `docs/architecture.md` (the host declaration at the loader row, the offered-provider environment at the build row), `docs/artifact-abi.md` (the route-requirement *row* family, twice; the host-process environment), `docs/backends/metal.md` (the measurement environment twice, the GPU-family compile guarantee, the host-process environment), and `docs/integration/candle.md` (the host-process environment). The index is what makes the remainder resolvable by a reader without editing it.

**Duplicated authority was avoided rather than created.** The [compile-profile authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md) already separates the offline compilation environment from the execution environment and names their union `MeasuredEnvironment`, and it already carries the sharpest family separation in the corpus, the row scope "the Apple family, not the artifact family". The glossary cites both and restates neither. That file is in `research/target-profiles` and was not edited.

### Verification

**Link resolution.** No gate checks local links, so a resolver was written and run over every edited file: 158 links seen, 152 local, 0 broken. **Its ability to fail was proven three ways**, each perturbation reverted and re-checked green afterwards: a nonexistent file (`decisions/0047-NO-SUCH-FILE.md`) reported `missing file`; a real file with a nonexistent anchor (`architecture.md#no-such-anchor`) reported `missing anchor`; and for the cross-directory links added under `docs/backends/`, both a bad anchor on `../glossary.md#…` and a wrong relative depth (`glossary.md#…` from `docs/backends/`) were reported.

**The resolver's own slug logic was wrong at first, and that is worth recording.** A corpus-wide run reported 21 broken anchors; every one was a false positive from collapsing whitespace runs, where the corpus follows GitHub in substituting one hyphen per whitespace character, so a heading containing an em-dash yields a double hyphen. After the fix, the corpus-wide run over 181 files and 2,495 local links reports exactly one broken link — a real, pre-existing one, filed below. The pre-existing `glossary.md#operation-names-shared-across-expression-layers` links in `docs/ir.md` resolve, which is the independent control that the slug logic matches the convention already in use.

**Absence checks** are the two above, each carrying its own positive control inside the same command rather than in a separate assertion.

### Out-of-scope defects found and filed, not absorbed

- [`repair-the-broken-proof-budget-anchor`](repair-the-broken-proof-budget-anchor.md) — `docs/compiler/fusion-and-scheduling.md:35` links to `optimizer.md#refinement-is-exhaustive-finite-evidence-with-an-explicit-gap`, which no heading matches. `contracts/optimizer`, not held here.
- [`refresh-the-stale-identity-ledger-in-status`](refresh-the-stale-identity-ledger-in-status.md) — `docs/status.md` reports artifact program v11 and manifest schema 9.0 where `crates/tiler-artifact/src/program/model.rs:168` declares `tiler.artifact-program.v12`, `codec/encode.rs:65` declares `(10, 0)`, and `docs/artifact-abi.md:166` agrees with the source. Verified against the source constants rather than inferred from the contract alone.

### Deliberately not done

- No ADR was written or amended. The vocabulary is derived from accepted ADRs 0043, 0047, 0072, 0078, 0081, and 0085, all read in full; `contracts/decisions` is not held by this ticket and no accepted record needed correcting for this work.
- No `BackendProvider`, provider bundle, emitter registration, or runtime-adapter registration is defined. The `Provider` row says so explicitly and points at [`specify-the-consumer-neutral-backend-provider-composition-contract`](specify-the-consumer-neutral-backend-provider-composition-contract.md), whose stated gap this vocabulary is a dependency of.
- No code identifier was renamed. The three unrelated `*ExecutionEnvironment*` spellings are separated in prose only; whether `tiler_compiler::target::TargetExecutionEnvironment` and `tiler_runtime::load::ExecutionEnvironment` should be spelled differently is a public-boundary question for Tom, not a docs change.
- `multi-device-and-sharding-scope-gate` is untouched and still `deferred`. The one-device profile is restated in `docs/architecture.md` and the glossary as ADR 0047's accepted initial execution profile, pointing at that gate rather than narrowing it.
