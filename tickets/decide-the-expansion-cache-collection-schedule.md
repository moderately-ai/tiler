---
id: decide-the-expansion-cache-collection-schedule
title: Decide what schedules an expansion cache collection
status: done
priority: p3
dependencies: [accept-the-tiler-cache-public-boundary]
related: [design-bounded-expansion-cache-garbage-collection, exercise-the-expansion-cache-under-cargo-and-rust-analyzer]
scopes: [research/cache, implementation/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, durability, concurrency]
---
`design-bounded-expansion-cache-garbage-collection` decided that a collection is **never automatic and never on the expansion path**: it is an explicit call returning a report, because a bound has to have a trigger a person can name. It eliminated collecting inside `get_or_publish` on a miss (a walk of every shard on the path the cache exists to make fast, run hardest when the cache is coldest), a background thread the cache spawns (threads inside a compiler process nobody asked to be concurrent, no lifetime in a process that may exit immediately, and a report returned to nobody), and collecting on a fraction of publications (an unexplainable trigger).

What it deliberately did not decide is **what calls it in production**, because no caller exists: there is no proc-macro frontend and no maintenance command, and choosing a schedule against an imagined consumer is designing for a caller nobody can see. So today nothing schedules a collection and a cache grows until someone measures and collects it deliberately — which is recorded rather than hidden.

## Trigger for reconsideration

The arrival of a real caller: a proc-macro frontend, or a `tiler` maintenance command. Either makes the question answerable instead of hypothetical.

## What this ticket would then owe

- Decide whether the schedule is a user-invoked maintenance command, a periodic hook, a build-session boundary, or some combination, and state what each enables and prevents.
- Decide where the bound's *value* comes from — configuration, an environment variable, a default derived from a measured working-set size — noting that `design-bounded-expansion-cache-garbage-collection` refused to pick a default precisely because the note says exact defaults require workload measurement. A schedule that supplies a default bound re-opens that decision and must argue it rather than inherit it.
- Decide what surfaces the report. A collection that removes entries and reports to a log nobody reads is the silence this crate is built not to produce.
- Take the process-pattern evidence from `exercise-the-expansion-cache-under-cargo-and-rust-analyzer`, which is what establishes how many concurrent expansions a real Cargo and rust-analyzer session produces.

Depends on `accept-the-tiler-cache-public-boundary`: every collection type is `pub(crate)` under ADR 0074 convention 7, so nothing outside `tiler-cache` can call a collection at all until that facade is accepted.

## Activated 2026-08-02 — the trigger fired, and the paragraph above it is now false

**Do not read "no caller exists" forward.** The reconsideration trigger was "the arrival of a real caller: a proc-macro frontend, or a `tiler` maintenance command". A proc-macro frontend arrived, and it holds a cache.

**Fact.** `crates/tiler-macros/Cargo.toml:11` declares `proc-macro = true` and `:46` declares `tiler-cache.workspace = true`. `crates/tiler-macros/src/aot.rs:649` is `fn open_cache`, which returns `ExpansionCache::open(root)` or `ExpansionCache::disabled()` depending on the resolved cache-root decision. So the expansion path in production opens the cache this ticket schedules collection for.

**Fact — the closing caveat is also spent.** It said nothing outside `tiler-cache` can call a collection until the facade is accepted. `accept-the-tiler-cache-public-boundary` is `done`, and the API is public: `pub fn collect(&self, bound: &CollectionBound) -> Result<CollectionReport, CacheUnavailable>` at `crates/tiler-cache/src/expansion/collect.rs:628`, with `pub struct CollectionReport` at `:328`. Reproduce:

```sh
rg -n 'pub fn collect|pub struct CollectionReport' crates/tiler-cache/src/expansion/collect.rs
```

**Fact — the process-pattern evidence this ticket was told to take is available.** `exercise-the-expansion-cache-under-cargo-and-rust-analyzer` is `done`, so the concurrent-expansion counts *What this ticket would then owe* draws on exist rather than being awaited.

**Boundary — one part of this is Tom's and must not be self-answered.** Which schedule the collection gets — user-invoked maintenance command, periodic hook, build-session boundary, or a combination — is a product decision about the consumer's experience, and `design-bounded-expansion-cache-garbage-collection` deliberately refused to pick a default bound because exact defaults require workload measurement. Run the elimination against correctness, maintainability, and measured working-set size first and discard what fails; escalate only what genuinely survives, as one atomic question. Status moves `deferred` → `todo` because the elimination is now runnable, not because the answer is settled.

## The elimination, run 2026-08-04

Read the derivation rather than the conclusion: every candidate below is discarded against a named test, so refuting a discard means refuting that test rather than disagreeing with the outcome. The two subsidiary decisions this ticket names — where the bound's value comes from, and what surfaces the report — are answered here because their eliminations leave one survivor each. The schedule itself leaves one survivor *as a schedule* and two survivors as a **form**, and the form question is the one atomic question at the end.

### What does not discriminate, and why saying so first matters

**Measurement — a collection concurrent with live expansions is already safe, at every candidate's concurrency.** [The bounded-collection note](../docs/research/cache/bounded-collection.md) records `expansion::harness::collection_races_active_processes_at_one_eight_and_thirty_two`: at 1, 8, and 32 real writer processes, a real collecting process holding `max_entries = 2` runs twelve rounds against publishers — half sharing one key so a lock is held almost continuously — while the parent reads throughout, asserting at every scale that every child completes, that every collection accounted for its whole selection, that something was actually removed, and that everything surviving still validates. A descriptor opened before the race and read after it yielded exactly the published bytes across a real process boundary.

**Fact — the mechanism that makes that true is in the source, not in the schedule.** `crates/tiler-cache/src/expansion/collect.rs` takes each key lock with `KeyLock::try_acquire` and never waits (`:717`), removes only after a re-`stat` agrees with the scan on length and modification time (`:730`), and the only mutation it performs on a content path is `remove_file`. A held lock is counted as `contended` rather than waited on, so no schedule can deadlock a collector against a writer.

**Fact — dying mid-collection needs no recovery under any schedule.** A collection is a sequence of independent single-file unlinks, each under its own lock, with no journal, no in-progress marker, and no durable accounting to reconcile; the namespace a killed collector leaves is indistinguishable from one given a looser bound.

**Inference — therefore two of this ticket's three correctness tests eliminate nothing.** Concurrency safety and crash safety are properties of the collector, and they hold whoever calls it. That is worth stating explicitly, because the intuitive elimination — "an automatic collection would race a build" — is *false* here and would have discarded the wrong candidates for the wrong reason. What actually discriminates is the third test.

### The discriminator, stated exactly

Two properties do all the work, and both are already contract rather than preference.

**A report must terminate in a reader.** **Fact:** `crates/tiler-cache/Cargo.toml` declares exactly one dependency, `tiler-artifact` — no logging facade — and `CollectionReport` has no `Display` implementation (`rg -n 'impl fmt::Display' crates/tiler-cache/src/expansion/collect.rs` reports no match). The crate therefore has no channel to emit anything: a report is a return value and nothing else. A schedule is admissible only if the process it fires in has somewhere to put a return value a person reads.

**A bound must arrive with its trigger.** [The bounded-collection note](../docs/research/cache/bounded-collection.md) eliminated a default ceiling on the ground that "exact defaults require workload measurement" and that a guessed default "would delete a user's compiled artifacts on the strength of that guess, invisibly, presenting as 'the build got slower' rather than 'your cache was collected'". `CollectionBound::UNBOUNDED` is the crate's `Default` and removes nothing. So a schedule firing where no person is present must carry a number nobody chose, which re-opens a refusal that has no new evidence behind it.

### Candidate: a periodic hook — eliminated, in all three of its forms

**An in-process timer thread.** Eliminated, and not newly: this is the "background thread the cache spawns" [the bounded-collection note](../docs/research/cache/bounded-collection.md) already discarded, and nothing since gives it a lifetime or a reader. The measured process patterns make it worse rather than better in both drivers. **Fact, from [the build-tool exercise](../docs/research/cache/build-tool-exercise.md):** under Cargo the expanding process is `rustc`, one short-lived process per crate compilation; under rust-analyzer it is `rust-analyzer-proc-macro-srv`, one process for the editor session. **Inference:** in the Cargo case the thread has no lifetime — the process may exit the moment expansion finishes — and in the analyzer case it has one, which means a thread inside the developer's editor removing compiled artifacts during interactive editing with no channel to say so.

**A persisted last-collection timestamp consulted on the expansion path.** Eliminated on three independent grounds. It requires durable state in the cache root, which [the bounded-collection note](../docs/research/cache/bounded-collection.md) refused for a stated reason — "a durable index … would, after a crash, disagree with the filesystem; reconciling them needs a repair rule; and a repair rule that is wrong either deletes live entries or trusts a stale size" — and a timestamp is smaller than a size index but carries the identical crash question and the identical second-authority problem. It puts the scan on the expansion path, which is the first alternative that note eliminated. And its answer to "why did my entry go away" is "a clock passed a threshold during an unrelated build", which is not distinguishable in explainability from the fraction-of-publications trigger already discarded as unnameable.

**An OS scheduler — a `launchd` agent running something daily.** Eliminated as a *schedule*, and the derivation matters because this is the form that looks safest. It does not answer the question; it presupposes the surviving candidate, because something invocable must exist for `launchd` to run. It then fails the two discriminators exactly: its report goes to a log file, which is the ticket's own named failure ("a collection that removes entries and reports to a log nobody reads is the silence this crate is built not to produce"), and its bound is a number written into a plist months before the removal it causes. Note the honest boundary — the architectural contract's inline-developer-experience invariant refuses a required consumer prepare step for *expansion*, and an agent a user installs deliberately is not literally that, so the invariant is not what eliminates this. The two discriminators are. This costs nothing: a user who wants a periodic collection can schedule the surviving invocation themselves, stating a bound as they do it, which keeps the trigger nameable.

### Candidate: a build-session boundary — eliminated on the measured process patterns

**Fact — there is no build session observable from inside an expansion.** A proc macro is invoked per invocation and is never told which invocation is the last. The process boundaries that do exist are measured: `rustc`, "every invocation in one crate", lifetime "one crate compilation"; the analyzer's server, "every invocation it is asked to expand", lifetime "the editor session" ([the build-tool exercise](../docs/research/cache/build-tool-exercise.md), Section 1).

**Inference — so "collect at session end" means one of two things, and both are already discarded shapes.** In `rustc` it means collect once per *crate*, in every concurrently compiling process that loaded the macro — which is a walk of every shard multiplied by every building process, the exact cost profile that eliminated collecting inside `get_or_publish`, relocated to the process's exit instead of its miss. In the analyzer it means collect when the developer closes the editor, which is not a build boundary at all. Neither has a reader: `rustc` has no diagnostic channel after expansion, and a report emitted at process teardown reaches nobody by construction.

**Inference — the only genuine build boundary is outside both processes**, in a wrapper the consumer runs instead of `cargo build`. That is a Cargo subcommand or a prepare step, which the accepted inline-developer-experience invariant in `AGENTS.md` refuses by name; and it still has no reader unless the wrapper prints the report, at which point it is the surviving candidate with a build glued to its front.

**Inference — and the bound problem is decisive independently.** At a session boundary nobody is present to state a ceiling, so the collection must carry a default. There is no workload measurement to derive one from (see the next section), so any value is the guess the note refused.

### Candidate: combinations — eliminated by their worst member

A combination inherits the failures of every trigger in it: adding a periodic or session trigger to an explicit invocation reintroduces the removal nobody chose and the report nobody reads, and the explicit half does not cure either. The one composition that survives is not a Tiler schedule — a user scheduling the surviving invocation on their own machine, stating a bound in the command they schedule — and it follows from shipping something invocable rather than being a separate design.

### Candidate: an explicit human-initiated invocation — survives

It is the only candidate that passes all three tests. Concurrency: the measured ladder is exactly this shape, a collecting process racing real publishers. Crash: nothing to recover. Report: a person initiated the call, so the return value has a reader by construction. Bound: the number arrives in the same act as the trigger, so "why did my entry go away" answers "because I ran this, under this bound, and here is the line naming the entry" — which is precisely the property [the bounded-collection note](../docs/research/cache/bounded-collection.md) required and the only candidate that supplies it.

### Subsidiary decision — where the bound's value comes from

**Answered: from the invocation, and Tiler supplies no default.** An invocation stating no bound collects under `CollectionBound::UNBOUNDED`, which is a pure measurement, so the natural sequence is measure, choose, collect. The alternatives:

- **A compiled-in default.** Eliminated. It re-opens the note's refusal with no new evidence, and it would make the crate's own stated default false at the only place a user observes it: `CollectionBound::UNBOUNDED` is `Default` and removes nothing, so a caller shipping a different default would be overriding a contract written specifically to prevent that.
- **A default derived from a measured working-set size.** Eliminated — the measurement does not exist. What exists is *per-entry* size, not working-set size: [the self-contained embedding note](../docs/research/embedding/self-contained-embedding.md) records envelopes of 32,136–47,803 bytes carrying genuine compiled `metallib` objects, and [the frontend contract](../docs/integration/frontends.md) measures one out-of-tree consumer holding one `deliver macos;` region. Deriving a ceiling from one region's entry size would be the guess, wearing a measurement's clothes.
- **An environment variable.** Eliminated for an explicit invocation: it buys nothing an argument does not, and it separates the number from the act, so a removal becomes attributable to a variable exported months earlier. It is the right shape only for a schedule firing with nobody present, and no such schedule survived.
- **A configuration file.** Eliminated: the same separation, plus a search path, a parser, and a second authority for a number used once.

### Subsidiary decision — what surfaces the report

**Answered: the process that asked, rendered where the person who asked is looking.** The alternatives:

- **A log line.** Eliminated by this ticket's own sentence and by the dependency graph: `tiler-cache` depends on `tiler-artifact` alone, and adding a logging facade to a storage-protocol crate to announce removals to a file nobody opens is the silence, not its cure.
- **A proc-macro diagnostic.** Eliminated twice: it requires the collection to happen on the expansion path, which the accepted design forbids, and it would attach a removal to a consumer invocation that did not cause it.
- **A marker file in the cache root.** Eliminated: durable state the design refused, and unread besides.

**Fact — and this costs `tiler-cache` no new public boundary.** `CollectionReport` exposes `accounting()`, `bound()`, `order()`, `removed()` as a per-entry list with bytes, `reclaimed_bytes()`, `contended()`, `superseded()`, `already_absent()`, `failed()`, `selected()`, `outcome()`, and `accounts_for_every_entry()`. Rendering is presentation and belongs to whoever surfaces it, so the accessors are the right split and no `Display` implementation needs adding. Adding one later would be a public-boundary addition for Tom, and neither surviving form requires it.

### The architectural consequence a shipped command carries, surfaced before the decision rather than after

**Fact — a command cannot reuse the root policy as it stands.** `crates/tiler-macros/src/lib.rs:79` declares `mod cache_root;` with no `pub`, and `crates/tiler-macros/Cargo.toml:11` declares `proc-macro = true`. **Measurement — pinned toolchain `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, 2026-08-04:** compiling a one-line crate containing `pub fn shared() -> u32 { 7 }` with `rustc --crate-type proc-macro` fails with "``proc-macro`` crate types currently cannot export any items other than functions tagged with `#[proc_macro]`, `#[proc_macro_derive]`, or `#[proc_macro_attribute]`". So making the module public would not help either: no crate can call `resolve` from `tiler-macros` at all.

**Inference — a command therefore has three routes to the root, and two are bad.** Duplicating the ADR 0089 derivation creates a second authority that can disagree with the macro's, which is the failure mode the corpus names repeatedly. Requiring the root on the command line makes the ordinary case restate a path the macro derived and the user never saw. Moving the resolver into a crate both can depend on is the sound route, and it needs a *new* crate: `tiler-cache` deliberately never consults the environment (a property the root-policy note protects), `tiler-build` reaches `tiler-metal-aot` and would link a process-spawning Apple toolchain driver into a maintenance binary, and `tiler` the facade cannot be depended on by `tiler-macros`. A new crate is scaffolding, which is Tom's under `AGENTS.md`'s implementation boundary — which is exactly why it belongs in front of the decision rather than behind it.

### Why the growth this bounds is real but small

**Fact — the cache subject folds the artifact program identity and the backend compilations** (`crates/tiler-build/src/payload_cache.rs:480`), and the compiler fingerprint is an input to compilation identity ([the frontend contract](../docs/integration/frontends.md)). **Fact — "in a live rust-analyzer session every settled edit *inside* a region costs exactly one expansion"** (same contract, measured on macOS 27.0 arm64 / Apple M4 Max, 2026-08-01).

**Inference — entries accumulate per distinct region text ever expanded, not per region currently in source, and every Apple toolchain update re-keys the whole set at once, orphaning everything published before it.** Growth is monotone and never self-limiting. **Inference — and the rate is modest:** at the measured 32,136–47,803 bytes per envelope, an afternoon of a few hundred settled in-region edits publishes on the order of ten to twenty megabytes. That is a real operational problem on a machine that never collects and a slow-moving one, which is the shape that makes the cheapest correct answer likely to be the right one and makes deferral genuinely survivable.

### What is left for Tom — one question

The schedule has one survivor, so the *schedule* is not the question. What survives the elimination in two forms, on correctness and maintainability grounds alone, is the **form** of the invocation, and the two differ in product experience rather than in correctness:

**Option A — Tiler ships an invocable maintenance command.** A binary — not in `tiler-macros`, which cannot hold one, and not in the `tiler` facade, which would put a binary target in every consumer's dependency — reading `account`, `collect`, and `purge` and printing the report. *Enables:* a developer whose cache has grown can name the trigger, state the bound, and read which entries left; every mechanism this project built (named removals, five disjoint dispositions, `BoundNotReached`, quarantine accounting, the out-of-service rename that a plain `rm -r` cannot match because it can unlink a live lock inode) becomes reachable by the person who owns the cache. *Prevents:* it commits a new consumer-visible product surface — a name, an argument spelling, an output format, and a distribution answer — and it requires the root resolver to move into a new shared crate, which is scaffolding under the implementation boundary.

**Option B — the public library API is the whole answer.** No command; the frontend contract states that bounding the cache means writing a caller over the accepted public `ExpansionCache::open` / `account` / `collect` / `purge`, or removing the root by hand. *Enables:* no new product surface, no new crate, no scaffolding decision, and the project stays research- and architecture-first. *Prevents:* the only remedies an ordinary consumer actually has are `rm -r` on the root — which [the bounded-collection note](../docs/research/cache/bounded-collection.md) records as correctness-preserving but strictly worse than `purge` — and adding a dependency on an internal Tiler crate in order to write a maintenance binary for a cache the frontend created on their machine without asking.

**Point.** A bound has to have a trigger a person can name, and a trigger a person can name has to be a thing a person can type. Under Option B nothing is typeable, so the mechanism is unreachable by exactly the audience it was designed to answer to.

**Counterpoint.** Option B is not a correctness failure and the measured growth rate does not make it urgent; every entry is a pure function of its subject, so a cache destroyed by any means costs recompilation and never correctness. `AGENTS.md` says not to scaffold crates or stabilize APIs before Tom moves the project into that phase, and Option A is both.

**Recommendation — Option A, with the narrowest surface, and not urgently.** The evidence favours it on substance: the collector's entire design case is that an operator sees which entry left under which bound, and Option B leaves that reachable only by writing Rust. The counterpoint is real and does not reverse it — it argues for *when* and *how small*, not for *whether* — so the concrete recommendation is one binary, one root-resolving crate shared with `tiler-macros`, three subcommands mapping one-to-one onto the accepted API, a bound stated as an argument with no default, and no schedule of any kind shipped with it. If Tom prefers to defer, Option B with the frontend contract stating plainly that the cache grows until a consumer collects it is a coherent recorded position rather than an omission — but it should be recorded as a decision, since today it is only an absence.

### Found while running this, not fixed here

- **`crates/tiler-cache/src/expansion/collect.rs:97-99` asserts a fact the tree refutes.** Its module documentation says scheduling "belongs to a consumer that does not exist yet — there is no proc-macro frontend and no maintenance command to hang it on". `crates/tiler-macros/src/aot.rs:649` opens a cache from the expansion path (`:501`). The elimination above did not depend on that sentence, but a doc comment is a claim the next worker reads as fact. `implementation/cache` is declared on this ticket and this dispatch authorized no crate edits, so it is reported rather than corrected.
- **`docs/research/embedding/self-contained-embedding.md:31` carries the same stale premise** — "`tiler::tensor!` states `ArtifactDeliveryPolicy::FallbackOnly` … it embeds no bytes and opens no cache, and `cache_root.rs` is a crate-private resolver nothing but its own tests calls". Reproduce the refutation with `grep -n 'open_cache(environment)' crates/tiler-macros/src/aot.rs`. That file is `research/embedding`, which this ticket does not hold.
- **This ticket declared `shared_scopes: []`** while every other in-progress ticket declares `project/tickets`, so a worker editing this ticket's own file had no declared scope for it. Added here, for the reason `AGENTS.md` states: the declaration is scheduling metadata, not product scope.

## Decision — 2026-08-04, Tom, direct session message to the orchestrator

Tom answered the form question, and the answer is neither option as posed: **no dedicated CLI ("overkill"); instead the cache evicts old entries automatically, with the policy definable/customizable through environment variables; and the cache's efficiency is to be verified.**

This is the product owner deliberately re-weighting the two discriminators the elimination used. The report-terminates-in-a-reader and bound-arrives-with-its-trigger properties were product weightings, and Tom weighted background hygiene (the cargo/sccache shape) above per-act attribution. The elimination's *correctness* findings stand unchanged and constrain the implementation: the collector is concurrency- and crash-safe whoever calls it (measured); an eviction pass must stay off the hot path (the `get_or_publish`-miss walk was refused on performance grounds and that refusal has no new evidence against it); and `tiler-cache` deliberately never reads the environment, so environment variables are read at the frontend and arrive as explicit typed bounds. What Tom's decision supersedes is the "never automatic" schedule conclusion of `design-bounded-expansion-cache-garbage-collection` — that supersession is recorded explicitly by the implementation ticket, preserving the original rationale.

Successor tickets: `admit-an-age-bounded-automatic-eviction-into-the-expansion-cache` (cache-side mechanism), `wire-the-env-configured-eviction-policy-through-the-deliver-path` (frontend trigger and env-var reading), `measure-the-expansion-cache-hot-path-efficiency` (the efficiency verification, measurement-first). This ticket's outcome — decide what schedules a collection — is therefore supported: an age-bounded automatic eviction, configured by environment variables, triggered from the frontend off the hot path.
