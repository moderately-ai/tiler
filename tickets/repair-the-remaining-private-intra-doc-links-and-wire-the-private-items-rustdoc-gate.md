---
id: repair-the-remaining-private-intra-doc-links-and-wire-the-private-items-rustdoc-gate
title: Repair the remaining private intra-doc links and wire the private-items rustdoc gate
status: in-progress
priority: p3
dependencies: []
related: [repair-the-private-intra-doc-links-the-public-rustdoc-gate-cannot-see]
scopes: [implementation/ir, implementation/reference, implementation/frontend, implementation/artifact, implementation/build, implementation/cache, implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-doc-gate
lease_expires_at: 1787088563
---
## User-visible outcome

`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items` exits 0, and the `doc` target in the `Makefile` runs it, so a broken or redundant intra-doc link in a private item fails the gate instead of being invisible to it.

## Why this exists — filed 2026-08-18 from `repair-the-private-intra-doc-links-the-public-rustdoc-gate-cannot-see`

That ticket repaired `tiler-compiler`'s sixteen broken private intra-doc links and asked whether `--document-private-items` should join a checked gate. The recommendation there is yes, and the reason is the AGENTS.md rule about a check reaching its subject: the gate's rustdoc step is `cargo doc --workspace --no-deps`, which never renders a private or `pub(crate)` item, so no diagnostic about one can reach it. `tiler-compiler` declares five `pub` modules out of forty-odd, so the unchecked share of that crate alone is most of it.

Wiring it could not land there because the rest of the workspace is not clean, and the compiler ticket's scope was `implementation/compiler`. This ticket owns the remainder and the wiring.

**Measurement (2026-08-18, on base `01fc9682` with that ticket's `tiler-compiler` repairs applied, macOS M3 Pro).** With `tiler-compiler` clean, the workspace private-items run reports **36 diagnostics across seven crates**, and every other member — `tiler-compiler`, `tiler-conformance`, `tiler-digest`, `tiler-metal`, `tiler-metal-aot`, `tiler-runtime`, and the three prototypes — is clean:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked --keep-going
```

| crate | diagnostics |
| --- | --- |
| `tiler-ir` | 18 |
| `tiler-reference` | 6 |
| `tiler-macros` | 4 |
| `tiler-artifact` | 3 |
| `tiler` | 2 |
| `tiler-build` | 2 |
| `tiler-cache` | 1 |

Two lint classes, and they need different judgements. Twenty-four are `rustdoc::redundant_explicit_links` — a doc link whose explicit target repeats the path its link text already resolves to, only reported once private items are rendered. Twelve are `rustdoc::broken_intra_doc_links`: `ArithmeticType::canonical_type_key`, `BuildError::ExtentSource`, `ShapeEnv`, `s`, `super::super::kernel::builder` (`tiler-ir`); `crate::aot::retained`, `crate::aot::open_cache`, `RetainedText::Display`, `buildable_target` (`tiler-macros`); `decode_platform` (`tiler-artifact`); `Compilation`, `DecodedEntry::payload` (`tiler-build`).

**These counts are a measurement at that commit, not a Fact about your base.** Re-run the command and enumerate the population yourself before repairing, per AGENTS.md.

## Required content

- Repair each diagnostic by reading the doc's intent — link to the right item where a path exists, drop the explicit target where the link text already resolves to it, or convert to a plain code span where the referent is `#[cfg(test)]` and therefore unreachable by rustdoc. Never delete a doc's semantic content to satisfy the resolver. Doc-comment changes only.
- Wire the gate. The proposed `Makefile` change is a second command on the existing `doc` target:

  ```make
  doc:
  	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
  	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked
  ```

  Both commands, not one. Measured on `tiler-compiler` (the largest crate) with a warm target directory, the private-items pass costs 1.52s against the public pass's 1.28s, so the added time is not an argument either way. Keeping the public pass is what states that the *shipped* documentation is clean: the private pass renders a different page set, and a future rustdoc that treats a lint differently between the two modes would silently retire the public claim if it were the only run. Note that `rustdoc::private_intra_doc_links` fires in **both** modes today — under `--document-private-items` it says "this link resolves only because you passed `--document-private-items`, but will break without" — so the private run does not lose that check.
- Make the new command fail deliberately before trusting it, per AGENTS.md: break one link in a crate the public run cannot see, quote the failure text, and revert. The compiler ticket recorded that demonstration for its own crate; repeat it here for the wired workspace command.

## Closes when

The workspace private-items run exits 0, the `Makefile` runs it, the new step has been observed failing on a perturbed subject with its message quoted, and `make full` is green.

## Fact audit at base `1ae823c9` (2026-08-18)

The ticket's measurement is **stale by one diagnostic**, which is exactly the shape it warned about ("These counts are a measurement at that commit, not a Fact about your base"). Re-enumerated at `1ae823c9` with the ticket's own command, the workspace private-items run exits 101 on **37** diagnostics across the same seven crates, not 36:

| crate | ticket said (`01fc9682`) | measured here (`1ae823c9`) |
| --- | --- | --- |
| `tiler-ir` | 18 | 18 |
| `tiler-reference` | 6 | 6 |
| `tiler-macros` | 4 | 4 |
| `tiler-artifact` | 3 | 3 |
| `tiler` | 2 | 2 |
| `tiler-build` | 2 | **3** |
| `tiler-cache` | 1 | 1 |

The whole delta is one new broken link in `tiler-build`: `BoundMetalCompileDeclaration`, in `crates/tiler-build/src/metal_subgroup_declaration.rs`. That file does not exist at `01fc9682` — `git cat-file -e 01fc9682:crates/tiler-build/src/metal_subgroup_declaration.rs` fails — and was added by `586c508a`, one of the 31 commits in `01fc9682..1ae823c9`. Nothing in the ticket's list moved or was already repaired; the lint split becomes **24 `redundant_explicit_links` and 13 `broken_intra_doc_links`**, the thirteenth being that new one.

The rest of the ticket's enumeration is **verified**: every named broken link is present at this base under the name given, and the other members — `tiler-compiler`, `tiler-conformance`, `tiler-digest`, `tiler-metal`, `tiler-metal-aot`, `tiler-runtime`, and the three prototypes — are clean, each reached and documented in the same `--keep-going` run.

**One correction to the ticket's framing of the broken-link causes.** The ticket lists `crate::aot::retained`, `crate::aot::open_cache`, and `super::super::kernel::builder` beside the genuinely absent names, but these three are a different class: the referent *exists* and is spelled correctly. It is private inside a module the citing module is not a descendant of, so no path to it exists from that scope, and `--document-private-items` does not change name resolution. `open_cache` and `retained` are bare `fn` in `crate::aot`; `builder` is `mod builder;` in `crate::kernel`. Within `aot.rs` itself the same `[`open_cache`]` spelling resolves and is not reported. These are the AGENTS.md `#[cfg(test)]` case by a different route, and take the same disposition.

## Repair census — 37 links, 37 dispositions

### Broken intra-doc links (13)

**Referent existed and was reachable — linked to it (7).**

| site | anchor | was | now | why |
| --- | --- | --- | --- | --- |
| `tiler-ir/src/index/law.rs` | `which is the single durable spelling` | `ArithmeticType::canonical_type_key` | reference definition to `crate::schedule::ArithmeticType::canonical_type_key` | `pub enum ArithmeticType` is in `src/schedule/numerics.rs` with `pub const fn canonical_type_key`; `law.rs` imports from `crate::schedule` but not this type. The body three lines below calls `.canonical_type_key()`, so the referent is not in doubt. |
| `tiler-ir/src/index/sourced.rs` | `lets a verifier prove` | `ShapeEnv` | reference definition to `crate::shape::ShapeEnv` | `shape.rs` re-exports it flat: `pub use env::{… ShapeEnv …}`. This file already carries a reference-definition block for six shape-layer names, so the repair joins that block rather than inlining a seventh spelling. |
| `tiler-ir/src/semantic/operation.rs` | `re-derive every` | `BuildError::ExtentSource` | reference definition to `super::BuildError::ExtentSource` | `semantic.rs` has `pub use error::{BuildError, …}`; `operation.rs` does not import it. Prose left byte-identical. |
| `tiler-artifact/src/program/codec/payload.rs` | `either promotes them into the versioned shape` | `decode_platform` | `Self::platform` | **No `decode_platform` has ever existed.** `git show 4fc3c0ed` — the commit that introduced the line — adds the doc and the method `fn platform(&self, tagged: bool)` in the same diff. That method is the referent verbatim: it returns `PayloadPlatform::VersionedSdk` (promotes) or refuses each stated field and returns `Unversioned` (proves they were unstated). |
| `tiler-macros/src/aot.rs` | `is where that is checked` | `buildable_target` | `require_buildable` | **Never existed either**; `git show 1e8b21da` adds the link with no such item. `require_buildable` is the function that compares a stated selection against `declaration.aot_target()` family by family and returns `AotRefusal::UnbuildableFamilies`, which is what the sentence describes. The one other occurrence of the string is a local binding in `aot/tests.rs`. |
| `tiler-build/src/metal_plan.rs` | `so target, feasibility, and provider facts` | `Compilation` | reference definition to `tiler_compiler::session::Compilation` | `pub struct Compilation` is in `tiler-compiler/src/session.rs`, and `session` is one of that crate's five `pub mod`s. This file imports `tiler_compiler::session::PlanAlternative` only. |
| `tiler-build/src/metal_subgroup_declaration.rs` | `Why this is not a row on` | `BoundMetalCompileDeclaration` | reference definition to `crate::BoundMetalCompileDeclaration` | Declared in the sibling `metal_declaration.rs` and re-exported by `lib.rs`; this file imports five other names from that module but not this one. |
| `tiler-build/src/payload_cache.rs` | `which is the only authority that says` | `DecodedEntry::payload` | reference definition to `tiler_artifact::program::DecodedEntry::payload` | `program/mod.rs` has `pub use codec::{… DecodedEntry …}`; `pub fn payload(self, delivery: usize)` maps a delivery position to a payload-descriptor position, which is what the sentence claims for it. |

That table has eight rows for seven links because `tiler-build` contributes three; the count of *linked-to-a-reachable-referent* repairs is 8. Two of them (`decode_platform`, `buildable_target`) named a function that never existed, so the repair replaces a false claim rather than a rotted path.

**Referent unreachable from the citing scope — converted to a plain code span (4).** Each names a private item in a module the citing module is not a descendant of. Nothing was deleted: the full path stays in the text, so a reader can still find it in the source.

| site | anchor | referent | evidence it is unreachable |
| --- | --- | --- | --- |
| `tiler-ir/src/program/builder.rs` | `the builder is recoverable` | `crate::kernel::builder` | `kernel/mod.rs` declares `mod builder;` with no `pub`. The parallel it draws is that module's `take_data`, whose own doc says "**The builder is left recoverable, not consistent.**" Spelling changed from the relative `super::super::kernel::builder` to the absolute `crate::kernel::builder`, which is the same module and legible as plain text. |
| `tiler-macros/src/preflight.rs` | `probes the root it has just opened` | `crate::aot::open_cache` | `aot.rs` declares `fn open_cache(` with no `pub`, so it is visible only in `aot` and its descendants. |
| `tiler-macros/src/retention.rs` | `a failing stage takes` | `crate::aot::retained` | `aot.rs` declares `fn retained(` with no `pub`. Same rule. |
| `tiler-ir/src/shape/sourced.rs` | `which is a semantic-layer quantity` | none — not a link | `"initial output shape[s]"` is a **quotation from the corpus**, not a reference; rustdoc read the plural bracket as a link and offered its own fix ("to escape `[` and `]` characters, add '\' before them"). Now `shape\[s\]`, which renders exactly as before. |

**Referent is a trait impl, which has no intra-doc path (1).**

| site | anchor | was | now |
| --- | --- | --- | --- |
| `tiler-macros/src/retention.rs` | `remains the cache's public lossy view` | `[`RetainedText::Display`]` | ``[`RetainedText`]'s `Display` `` |

`RetainedText` is imported and public (`tiler_cache::expansion`), and `impl fmt::Display for RetainedText` is at `tiler-cache/src/expansion/retention.rs`. `Display` is not an associated item of the struct, which is what rustdoc says: "the struct `RetainedText` has no field or associated item named `Display`". The type keeps its link and the impl is named beside it.

### Redundant explicit links (24)

All 24 are the same defect — an explicit target repeating a path the label already resolves to — and all 24 take rustdoc's own suggested fix: drop the target, keep the link. No label text changed, so every rendered sentence and every destination is what it was. Four sites were re-joined onto one line where dropping a long target left a two-word continuation line; that is whitespace inside a doc comment and changes no rendered output.

| crate | sites |
| --- | --- |
| `tiler-ir` (13) | `index/builder.rs` `IndexDomainUnknownReason::InsufficientFacts`; `index/refinement.rs` `IndexRealizationLaw`; `kernel/lower.rs` `KernelDiagnostic::UnorderedStagedRewrite`; `kernel/model.rs` `SynchronizationSubject` ×2; `program/model.rs` `KernelProgramDiagnostic::UncoveringStage` ×2; `schedule/cooperative.rs` `SyncPointId` and `SynchronizationPlacement::RoundBoundary`; `schedule/model.rs` `PointwiseBf16Expression` and `SynchronizationSubject`; `schedule/synchronization.rs` `AntiDependencyEdge` and `SynchronizationRule::UnadmittedKind` |
| `tiler-reference` (6) | `rms_norm.rs` `rsqrt_enclosure`; `silu.rs` `exp_enclosure` ×2; `standard.rs` `ArithmeticType::F32`, `ArithmeticType::Bf16`, `GatherF32Reference` |
| `tiler-artifact` (2) | `program/codec/payload.rs` `BackendEntryKey`; `program/codec/view.rs` `RoutingPolicy::StablePriority` |
| `tiler` (2) | `route.rs` `route_with_adapter` and `DispatchAdapter::dispatcher` |
| `tiler-cache` (1) | `expansion/bundle.rs` `DebugRetention` |

No code item was renamed, moved, or otherwise touched: the diff is `///` and `//!` lines plus the `Makefile`.

## The wired gate, and the demonstration that it can say no

The `Makefile`'s `doc` target now runs both commands, as proposed:

```make
doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked
```

The reasoning the ticket records — a check whose subject is out of frame, the shipped-page-set claim the public run states, `private_intra_doc_links` firing in both modes, and the 1.28s/1.52s cost — is written into the target's comment so it is next to the command rather than only here.

### Perturbation — the new step fails, and the old step cannot

The subject perturbed is the repair above at `crates/tiler-ir/src/index/law.rs`: its reference definition was deleted, leaving `[`ArithmeticType::canonical_type_key`]` unresolved again. `governs_result_arithmetic` is a private `fn` in `mod law;`, which `index/mod.rs` declares without `pub` — so the public run renders no page for it. `make doc` on that tree:

```text
$ make doc
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
    Finished `dev` profile [optimized + debuginfo] target(s) in 8.03s
   Generated .../target/doc/tiler/index.html and 15 other files
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked
error: unresolved link to `ArithmeticType::canonical_type_key`
   --> crates/tiler-ir/src/index/law.rs:849:7
    |
849 | /// [`ArithmeticType::canonical_type_key`], which is the single durable spelling
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no item named `ArithmeticType` in scope
    |
    = note: `-D rustdoc::broken-intra-doc-links` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(rustdoc::broken_intra_doc_links)]`

error: could not document `tiler-ir`
make: *** [doc] Error 101
```

Both halves are in that one transcript, which is why the perturbation was run through `make` rather than through the two commands separately: the **first** command finished and generated its page set on the same broken tree, and the **second** refused. That is the AGENTS.md requirement to confirm a check can reach its subject — before this change, `make doc` on this tree exited 0.

Reverted. `shasum -a 256 crates/tiler-ir/src/index/law.rs` returns `7f5d445869a5725661b63c28daac34d00bbbeffb9171c1caa99bf60097fac6e5` both before the perturbation and after the revert.

## Commands and results (base `1ae823c9`)

| command | result |
| --- | --- |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked --keep-going` (before) | exit 101, 37 diagnostics |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked --keep-going` (after) | **exit 0** |
| `make doc` (both commands, wired) | exit 0 |
| `make doc` on the perturbed tree | exit 101 at the second command, quoted above |
| `make full` | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| `make citations` | exit 0 |
| `ticketsplease lint` | exit 0 |
| `git diff --check` | exit 0 |

`cargo doc` emits one non-fatal `warning: the following packages contain code that will be rejected by a future version of Rust: block v0.1.6` on every run, before and after. It is a third-party dependency's future-incompatibility notice, not a rustdoc lint, and `RUSTDOCFLAGS="-D warnings"` does not promote it.

**Gate provenance.** `make full` was run on the code tree of `e667611b` and reported exit 0. That value was read from the command's own captured status rather than from the redirected log, because the log was written by `make full > … 2>&1; echo …` and the trailing `echo` would have reported 0 whatever `make` did — the compound-status case AGENTS.md names. All four terminal stages are visible in the log: both `doc` commands with their `Generated …/target/doc/tiler/index.html` lines, `1282 tests run: 1282 passed, 3 skipped` from the release nextest, `ticketsplease lint` → `ok: no problems found`, and `shellcheck --severity style deps.sh check-citations.sh` completing silently.

The commit after it is a `tickets/`-only delta — `git diff --stat e667611b <head>` shows one file — so it carries that gate under the AGENTS.md delta rule, which `tickets/` is deliberately outside the gated set for. `ticketsplease lint` and `make citations` were rerun by hand on the newer tree, as that rule requires, and both exit 0.

**Unchecked population, restated.** Every private and `pub(crate)` item's documentation in every workspace member is now inside the gate. What remains outside it is what rustdoc does not compile at all: `#[cfg(test)]` modules, in either mode. The plain-code-span conversions in [`repair-the-private-intra-doc-links-the-public-rustdoc-gate-cannot-see`](repair-the-private-intra-doc-links-the-public-rustdoc-gate-cannot-see.md) and the four here are that population made visible rather than covered.

**One defect noticed in that sibling ticket, not repaired here because its scope is `implementation/compiler`.** Its census prose says "Nine had a resolvable target" and "Seven name an item rustdoc never compiles", but its own two tables carry **ten** rows and **six** rows. The totals agree at sixteen and the tables are the authority — every row names its site and its disposition — so nothing in that work is wrong; only the prose split is. A reader taking the 9/7 figures at face value would look for a seventh conversion that does not exist. Its neighbouring claim that "the four sites where the referent is test-only now say so" is correct as written: four of the six converted sites gained the words "test-only", and the two `request.rs` ones were left as bare code spans to match the plain-span test names already beside them in the same paragraph.
