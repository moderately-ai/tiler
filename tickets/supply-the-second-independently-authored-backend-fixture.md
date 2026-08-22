---
id: supply-the-second-independently-authored-backend-fixture
title: Supply the second independently authored backend fixture
status: in-progress
priority: p2
dependencies: []
related: [publish-the-backend-provider-conformance-suite]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [conformance, backend-providers, evidence]
claimed_from: todo
assignee: worker-fixture
lease_expires_at: 1787424892
---
## User-visible outcome

A second, independently authored backend fixture exists that shares the portfolio's neutral, non-self-certifying structural and execution subjects — so `publish-the-backend-provider-conformance-suite`'s release condition names a deliverable that something in the graph actually owns.

## Why this exists

Filed 2026-08-22 by the coordinator from a **graph gap** the deferred/blocked sweep found: a p1's release trigger names a deliverable with **no node in the graph**.

**Fact (reported by the sweep, not re-derived by me — verify it first).** `publish-the-backend-provider-conformance-suite` (p1) states its release trigger as "one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects". No ticket owns producing that fixture, so the p1 cannot become ready by any path currently in the graph.

**Fact (reported, unverified by me).** `.ticketsplease/decision-queue.md` item 14 (accepted 2026-08-18) specifies a bounded extraction that would supply exactly this. Read that item as the specification rather than inventing a shape.

**Why "independently authored" is the whole point, and the trap to avoid.** A conformance suite validated only by the fixture that motivated it proves the suite agrees with itself. The value is a *second* author's reading of the same neutral subjects. So a fixture derived by copying `crates/tiler-build/tests/custom_backend` and renaming it would satisfy the words and defeat the purpose. **If you conclude the only tractable route is a derivative of the existing fixture, stop and report** — that is a decision about what the conformance claim is worth, not an implementation detail.

**Prior art that is genuinely independent, and worth reading before designing.** `restore-multi-family-metal-delivery-evidence-under-per-family-profiles` established that the scalar-host backend can hold two artifact families *honestly* — it runs no target compiler, its payload is an image its own in-process translator writes, and one profile key covers both triples with every declared axis holding for both. That is the shape of a neutral, non-self-certifying subject.

## Source-first Fact audit — 2026-08-22 at base `e20ed09e0c40ec777e22a2cd43ec70cb0c5ccdea`

Both Facts were reported by the coordinator's read-only graph sweep and marked un-re-derived. Both were re-read here, in full, at this base. Every anchor below was run with `grep -c` against the file the citation names before being written down.

| Claim | Verdict | Evidence |
| --- | --- | --- |
| The p1 states its release trigger as "one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects" | **Verified, verbatim** | `tickets/publish-the-backend-provider-conformance-suite.md`, `## Trigger check log`, anchor `one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects` returns 1. The source sentence is `The trigger is one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects.` |
| No node in the graph owns producing that fixture | **Verified** | `grep -l "independently authored" tickets/*.md` returns seven files; the four that are not this ticket, the p1, or its owning decision are `own-or-close-the-adr-internal-open-questions` (`done`, about target profiles), `reconsider-registered-quantitative-capability-axis-schemas` (`deferred`, target-profile trigger), `refuse-empty-live-domains-before-routing-commit` (`done`, launch preconditions), and `revise-adr-0108-with-a-complete-data-dependent-index-vertical` (`done`, index descriptions). None owns a backend fixture. |
| The p1's one-line log entry is the whole trigger | **False — it is a compressed restatement, and the authority is stronger** | The p1 says of itself `This carrier defers on that decision's named reopening trigger rather than dispatching on the decision ticket's completion`. That decision is `decide-the-backend-provider-conformance-harness-public-surface`, whose Trigger 1 reads `A bounded extraction demonstrates two independently authored backend fixtures using one device-free structural subject and one execution subject` (anchor returns 1) and continues `It also proves typed host unavailability, caller-owned execution policy` (anchor returns 1), after which the same sentence names tiler-reference as the sole mathematical oracle and adapter-owned terminal resource lifetime. A fixture whose existence is asserted but which proves none of those four does **not** fire the trigger. This ticket's Required work did not name them; they are added below. |
| `.ticketsplease/decision-queue.md` item 14 "specifies a bounded extraction" that should be read "as the specification rather than inventing a shape" | **False as stated** | Item 14 is four bullets and carries no module, type, container, entry, or subject spelling at all. Its only sentence about the extraction is the Recommendation's `One bounded two-fixture extraction with typed host unavailability, caller-owned policy` (anchor returns 1), which continues by naming tiler-reference as sole mathematical oracle and adapter-owned terminal lifetime as sufficient to reopen a partial facade independently. That names four properties the extraction must **prove**; it does not specify a shape, so the instruction to read it as the specification is not followable. Item 14's own `Presentation order and release trigger` bullet is about when to present the packet to Tom, not about a fixture. The only usable specification is the decision ticket's Trigger 1 plus the source seams themselves. |
| The p1's reproduce command can observe the trigger firing | **False — the check cannot say yes** | The p1's log line ends `Reproduce: enumerate independently authored fixtures under `crates/tiler-conformance/` and compare their structural/execution subjects` (anchor `enumerate independently authored fixtures under` returns 1). At this base `crates/tiler-conformance/` holds `Cargo.toml` and `src` and no `tests` directory, and the owning decision itself locates every existing out-of-crate fixture elsewhere — `crates/tiler-compiler/tests/external_physical_provider.rs`, `crates/tiler-build/tests/custom_backend/main.rs`, the runtime adapter fixtures, and `spikes/runtime/backend-provider-portfolio/`. An enumeration rooted at `crates/tiler-conformance/` therefore returned the empty set for a reason unrelated to the trigger. This work makes the command's root non-empty rather than leaving it a check that could only ever report absence. |

**Consequence for this ticket.** The premise survives: a p1's release condition names a deliverable no node owns, and this ticket owns it. What changes is the bar. The deliverable is not "a second fixture exists" but a fixture that expresses **one device-free structural subject and one execution subject** and proves **typed host unavailability, caller-owned execution policy, `tiler-reference` as the sole mathematical oracle, and adapter-owned terminal resource lifetime**, none of which the ticket named before this audit.

## Scope added — 2026-08-22

`implementation/conformance` was added with `tkt set --add-scope`, and the reason is a dependency fact rather than a preference. `crates/tiler-build`'s manifest lists `tiler-artifact`, `tiler-cache`, `tiler-compiler`, `tiler-ir`, `tiler-metal`, and `tiler-metal-aot` and **neither `tiler-runtime` nor `tiler-reference`**, so no test target under `implementation/build` can route a plan or compare against the oracle: the execution subject and the sole-oracle proof are unreachable from the originally declared scope. `crates/tiler-conformance`'s manifest already lists all ten crates the vertical crosses, including both, so an integration test there reaches every seam with **no manifest and no `Cargo.lock` change**. It is also the crate whose header names its subject as cross-layer executed evidence, and the crate the p1's own reproduce command roots at. No live lane holds `implementation/conformance`: the two other live claims declare `implementation/{ir,reference,compiler}` plus `contracts/*` and `research/target-profiles`.

`implementation/build` was then **removed** rather than left declared. Nothing under `crates/tiler-build/**` was touched — `tkt guard` at the true base reports `affected scopes: implementation/conformance, project/tickets` — and an exclusive scope declared but unused blocks other lanes from a crate this work never enters.

## Required work

- Re-audit both Facts above at your actual base and report a per-Fact verdict before designing. Quote the release trigger verbatim from `publish-the-backend-provider-conformance-suite` and decision-queue item 14.
- Derive what "neutral and non-self-certifying" requires of the fixture, from the portfolio's own subjects rather than from this ticket's summary of them. State the subjects it must share and the ones it must **not** inherit.
- Build the fixture, and state plainly what makes it independently authored rather than a rename.
- **Perturb the subject:** show the conformance suite failing against a fixture that certifies itself, and quote the failure. A suite that passes both a sound and an unsound fixture has not been demonstrated.
- **Added by the Fact audit above, because the trigger's real authority names them and this ticket did not.** The fixture must additionally prove all four properties the owning decision's Trigger 1 requires of the extraction: a typed host-unavailable outcome that cannot compare equal to a pass; an execution policy owned by the caller rather than read from an ambient environment variable; `tiler-reference` as the sole mathematical oracle, with no expected value restated in the fixture; and adapter-owned terminal resource lifetime, meaning the adapter retains its resources through terminal device use and the route mints no second completion authority.

## Non-goals

Publishing the conformance suite — that is `publish-the-backend-provider-conformance-suite`'s own work, and this ticket exists to make it reachable. Any Metal second-family route: the iOS families are hardware- and decision-gated and `second_artifact_family_fixture` was deleted in `1f6ec214`, so that path is closed. Changing the portfolio's neutral subjects.

## Outcome — 2026-08-22

**The fixture is `crates/tiler-conformance/tests/independent_backend/`**, a Cargo integration-test target of 3,348 lines (`wc -l` after `cargo fmt`) in five files: `main.rs` (749, the cases and the subject census), `nodefold.rs` (745, the declared target profile, the translator, and the assembly), `nodefold_graph.rs` (582, the executable representation and its decoder), `nodefold_adapter.rs` (1,144, the runtime adapter, its worker host, and the route), and `workload.rs` (128, the program and the reference evaluation). It adds no public item, no manifest entry, and no `Cargo.lock` edge; `tiler-conformance` already depends on every crate the vertical crosses. Twelve tests, all device-free.

**One self-inflicted regression, caught by the gate and repaired rather than worked around.** The four non-`main` files were first named `backend.rs`, `graph.rs`, `adapter.rs`, and `oracle.rs`. `make citations` resolves a pinned citation by unique path suffix, so the new `oracle.rs` turned `crates/tiler-reference/src/oracle.rs` into a two-way match and **ten** live citations across seven research documents stopped being checkable: `check-citations: 10 citation(s) do not resolve against this tree.` The repair is the rename above — every file a directory-based integration test may name freely now carries a basename `git ls-files` shows no other tracked file ends with, and `main.rs` is Cargo's and was already a suffix twenty-seven tracked files carry. Nothing under `docs/` and nothing in `check-citations.sh` was edited; the ledger was not widened to absorb an ambiguity this work created. Worth recording separately: the shell reported this failure as success. `make full > log 2>&1; echo "EXIT=$?"; tail log` exits with `tail`'s status, so the compound command returned 0 while the log said `make: *** [citations] Error 1`. The log line is the verdict, exactly as AGENTS.md warns.

### The two subjects it shares with the portfolio

**Device-free structural subject — a producer cannot state a fact the plan already decided.** Not "the artifact validates", which any careful backend satisfies, but that `assemble_plan_artifact` offers no parameter through which such a fact could be supplied. `the_assembled_artifact_carries_facts_this_backend_never_supplied` asserts the target-profile key and exact descriptor digest, the feasibility rule-set key and revision, the zero deferred prepared-entry predicates, and each packaged entry's `BackendEntryKey`, against the compilation rather than against the producer. The same subject is already asserted independently by `crates/tiler-build/tests/custom_backend`'s `the_derived_subjects_follow_the_compilation_and_not_the_producer`.

**Execution subject — a routed result's verdict comes from `tiler-reference` and never from the adapter.** `the_routed_result_agrees_with_the_reference_oracle` routes through `route_with_adapter` with a caller-selected adapter and compares twelve `f32` bit patterns against the oracle. The same subject is asserted independently by the retained three-family portfolio spike's CPU leg.

### What makes it independently authored rather than a rename

Every row below is a place the seam left a choice, and each is checkable in the source rather than asserted here.

| Choice the seam left open | `tests/custom_backend` | this backend |
| --- | --- | --- |
| payload model | symbol, transport list, and work-item count per entry | a single-assignment node table with a declarative store plan |
| control flow | none carried; the image describes entries, not bodies | predication is one optional guard ordinal on the store plan |
| evaluation | none; the image is never executed | demand-driven, one forward pass, dead nodes never evaluated |
| framing | big-endian | little-endian |
| entry symbol | a governed digest over the canonical identity and the family triple | positional, carrying no identity, reaching no digest crate |
| execution host | none | a worker thread the adapter owns, acquired before the commit |
| delivery positions | two families under one profile | one, and the profile declares one triple |

Two further rows are **convergence rather than difference**, and reporting them that way is the point. Both backends declare `SubnormalMode::Preserve` exact and refuse both flushing modes, because that is what an honest host-arithmetic interpreter under `STRICT_F32` can say; and both declare a **non-identity** transport mapping. A second author reaching the same declaration independently is evidence about the seam, not about copying — and the transport row is now shown to be a live hazard rather than a stylistic claim (see the fifth perturbation below).

Three structural differences are worth naming because they produce validation obligations the other design does not have. A node's ordinal *is* its result, so the decoder must prove every operand is an **earlier** ordinal (`ForwardReference`) and must rebuild the kind table itself rather than transport it (`OperandTypeDisagreement`); and because the store plan is declarative, evaluation is demand-driven, so a load under a false guard is never evaluated at all rather than being skipped by an interpreter that had to remember to.

### The perturbations, each with its quoted failure

Every one perturbs the **subject** — the backend, the adapter, or the declared mapping — and leaves the assertions untouched. The first four are standing cases that assert the refusal; each was additionally applied to the *sound* path and watched turning the suite red, which is the demonstration that the suite can say no. The three red transcripts below were re-captured against the final tree after every case, every rename, every prose correction, and `cargo fmt` had landed, so their line numbers resolve at this commit; the durable anchors are the test names, and a later reader should re-locate by those rather than by the number.

1. **A self-certifying adapter** (`Evaluation::Certify`): reaches the routing commit, receives the storage, reports its terminal use, and folds nothing. Everything the adapter itself could be asked about is green. Applied to the sound path, `the_routed_result_agrees_with_the_reference_oracle` fails:

   ```text
   panicked at crates/tiler-conformance/tests/independent_backend/main.rs:555:40:
   the routed result disagrees: element 0 is 0x00000000 and tiler-reference requires 0x3fc00000
   ```

2. **A producer-minted entry key** (`EntryPerturbation::ForgedEntryKey`): the backend states an entry identity instead of transporting the one the stage kernel decided. Applied to the sound path, `the_assembled_artifact_carries_facts_this_backend_never_supplied` fails:

   ```text
   panicked at crates/tiler-conformance/tests/independent_backend/main.rs:260:52:
   the assembled envelope decodes: Invalid { detail: "UnmappedBackendEntry { payload: 0 }" }
   ```

3. **An adapter that returns before terminal use** (`Lifetime::ReturnBeforeTerminalUse`): submits the entry, does not await the worker's completion, and returns. Applied to the sound path:

   ```text
   panicked at crates/tiler-conformance/tests/independent_backend/main.rs:544:6:
   this caller required the execution host and this host supplied it: "the committed route failed: nodefold.dispatch: 1 entr(y/ies) were submitted and 0 terminal use(s) were witnessed; the routed storage is still outstanding"
   ```

4. **An adapter that reports what it prefers** (`Binding::Preferred`): reports a representation it cannot decode. The loader compares and refuses:

   ```text
   runtime.no-eligible-variant: this host can execute none of the 1 packaged variant(s), and no guard was evaluated: variant 0: entry 0 is realized by a tiler.test.nodefold/tiler.test.nodefold-graph-v1 payload and this host states tiler.test.nodefold/tiler.test.nodefold-graph-v2
   ```

5. **A transport map that disagrees with the payload** (`EntryPerturbation::IdentityTransports`): every identity in the artifact is still the plan's, so the envelope assembles, decodes, routes, and reports terminal success — and the arithmetic is wrong. Caught by the oracle and by nothing else:

   ```text
   disagreeing transport map refused by the oracle: element 0 is 0x00000000 and tiler-reference requires 0x3fc00000
   ```

   This is the sharpest form of the whole argument: nothing above the backend has anything to compare, because the mapping is a statement no plan makes and the bytes it disagrees with are opaque to every layer that could read them.

### The four properties the owning decision's Trigger 1 additionally requires

- **Typed host unavailability.** The adapter acquires a worker thread in `plan_dispatch`, before the commit, so an unavailable host is a refusal a caller may still take a fallback from. `an_unavailable_execution_host_is_typed_and_cannot_pass` reaches it with an unsatisfiable stack request: `nodefold.host-unavailable: this host cannot supply an execution thread with a 18446744073709551615-byte stack: invalid stack size`. `ExecutionOutcome` implements no equality, holds no default, and answers `None` from `completed()` for an unavailable host, and the oracle comparison takes bits rather than an outcome — so there is no expression that reaches a comparison from an unavailable host.
- **Caller-owned execution policy.** `HostPolicy::{Require, Report}` is applied by `apply_policy` at the call site. The adapter's report is byte-identical in both runs; only the caller's classification differs. No file in the fixture reads an environment variable.
- **`tiler-reference` as the sole mathematical oracle.** No expected value is written down anywhere in the fixture; `oracle::reference_bits` derives it. The oracle-agreement case additionally asserts the reference output differs from the operands, so a program that had degenerated to the identity — which would let a backend that copied its input through compare equal — fails there rather than passing quietly.
- **Adapter-owned terminal resource lifetime.** The routed storage is **moved** into the worker and comes back only with a `TerminalUse` value the worker loop alone can construct. `dispatch` refuses unless it has witnessed one per submission. It is not a second completion authority: the token is minted by the same worker whose write it attests, the loader is never consulted, and nothing outside the adapter can see it.

### Populations sized from the type, not by hand

- `Subject::ALL` is sized by `std::mem::variant_count`, so a third subject added without a coverage row is an array-length error at the declaration. The coverage rows name their cases as **strings**, which checks nothing on its own, so the census reads this file's own source back through `include_str!` and requires each row to name a case the file declares. Perturbed by renaming one row's string, that check says:

  ```text
  panicked at crates/tiler-conformance/tests/independent_backend/main.rs:740:13:
  Structural's coverage row names `a_backend_minted_entry_key_is_rejected`, and this file declares no such case; a census that names a case which has been renamed away reports coverage it does not have
  ```
- `every_named_graph_refusal_is_reachable_from_bytes` reaches **all twelve** members of `GraphRefusal` from constructed byte runs and asserts the count of distinct discriminants equals `std::mem::variant_count::<GraphRefusal>()`. A refusal added to the vocabulary and left unreached fails there. Census printed by the run: `ForeignDomain, UnsupportedSchema { major: 10, minor: 0 }, Truncated, TrailingBytes, UnknownNodeTag(255), MalformedSymbol, ForwardReference { node: 1, operand: 9 }, OperandTypeDisagreement { node: 3, required: F32, found: Index }, UndeclaredStoreBuffer(9), UndeclaredLoadBuffer(9), EmptySignature, StoreThroughReadBuffer(0)`.

### What the trigger now has, and the gap that remains

The evidence the owning decision names as **sufficient on its own** to reopen candidate D1 — *"one second independently authored fixture that shares exact structural and execution subjects with the portfolio without those defects"* — now exists in the gate. *Those defects* are the three the preceding sentence in that record names — an extraction that would *"group independently selected responsibilities, publish Metal-specific machinery, or trust caller-supplied success"* — and this backend avoids each: it claims four responsibility rows separately and groups none, it links no Metal type and carries no Metal-shaped fixture, and its verdict comes from the oracle rather than from any success the adapter reports. Two gaps are named rather than implied.

- **The execution subject's in-gate counterpart is still one fixture.** `crates/tiler-build/tests/custom_backend` shares only the *structural* subject, and it cannot share the execution one: `crates/tiler-build`'s manifest has no `tiler-runtime` and no `tiler-reference` edge, so no test target there can route a plan or reach the oracle. The other fixture asserting the execution subject is the retained portfolio spike, which `spikes/` places outside the workspace `members` and therefore outside `make full`. Two independently authored fixtures do assert the execution subject; only one of them gates.
- **No extraction was performed, and none was authorized.** Trigger 1 speaks of a *bounded extraction*; what exists here is the same subject expressed twice, independently, with no shared code. Building the shared expression is the reopened decision's own work, and this ticket's Non-goals exclude it. Whether the trigger has fired is the coordinator's and Tom's call: `publish-the-backend-provider-conformance-suite`'s `## Trigger check log` and `decide-the-backend-provider-conformance-harness-public-surface` were **not** edited here.

### Not done, and why

No `docs/` or ADR sweep. This ticket holds neither `contracts/decisions` nor `contracts/foundation`, and `contracts/decisions` is held by a live identity-migration lane. If the coordinator records the trigger as fired, the catalog and contract language that follow from reopening D1 belong in that decision's own carrier.

## Closes when

The fixture exists, its independence is argued rather than asserted, the self-certifying perturbation is quoted failing, `publish-the-backend-provider-conformance-suite`'s release trigger is satisfied or its remaining gap is stated exactly, and the touched package's gates are green.
