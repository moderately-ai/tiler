---
id: decide-whether-the-citation-checker-should-reach-spike-records
title: Decide whether the citation checker should reach spike records
status: done
priority: p2
dependencies: []
related: [reach-every-spike-record-from-the-experiment-catalog, state-the-spike-currency-convention-where-readers-look]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [gates, documentation, spikes, blind-spot]
---
## User-visible outcome

Either a rotted link inside a retained spike record fails `make citations`, or the repository states plainly that it does not — so nobody reads a green gate as covering a population it never scans.

## Why this exists

Found 2026-08-22 by `worker-tileprotocol`, **by perturbation rather than by reading**: it introduced a deliberately broken link under `spikes/` and `make citations` returned **exit 0, green**. It then resolved all sixteen links in its own documents by hand, because the gate could not.

**Fact — `spikes/**` is outside the checker's population, and the script says so.** `check-citations.sh` records its scanned set as `tickets/**`, `docs/**`, and the tracked markdown at the repository root. Verified by the coordinator at `97e7fef1`. This is a stated scope, not an accident — which is exactly why the decision below is a decision and not a bug fix.

**Why it matters more than it looks.** AGENTS.md's own description of this gate said it "resolves every local markdown link in an open ticket or a live document, so a catalog row or cross-reference that points at nothing fails the gate" — quoted here as the wording that motivated this ticket, and **retired by this ticket's own change**, so a grep hit on it is this sentence rather than a live claim. A reader can take that as covering the evidence a document cites. It does not cover the spike records themselves — and the spike currency convention that landed recently tells readers to go and read a spike's own dated claim. If the links *inside* those records rot silently, the route the convention sends readers down is the one nothing guards.

**This is the class AGENTS.md names.** *"Verify that a check reaches its subject at all."* Several checks here could not: `cargo doc` cannot fail for a `#[cfg(test)]` module or an integration test under `tests/`; a `^(pub )?`-anchored grep cannot match `pub(crate)`. This is one more, found the way that section prescribes — by asking what it would take for the check to say *no*, and discovering the case was unreachable.

## Fact audit at `77cd0104`

- **Fact — `spikes/**` is outside the checker's population, and the script says so. VERIFIED, and stronger than stated.** Reproduced the perturbation: appending `[planted broken link](./no-such-file-at-all.md)` to `spikes/README.md` left `make citations` at **exit 0**, and the output was **byte-identical** to the unperturbed run. Byte-identity is the signature of a file that was never opened rather than one that passed — the same signature the script's header records for the repository-root documents on 2026-08-08.
- **Claim — "some [spike citations] deliberately point at paths that do not exist yet", so a green gate would depend on spike hygiene. FALSE for links, and false in kind for citations.** Measured by running the checker over the corpus with the exclusion switched off: **590 local markdown links across 68 records, every one resolving; zero dangling and zero forward references.** Of the 50 pinned citations that qualify, 41 resolve, 2 are skipped as version-pinned dependency sources, and 7 fail — and not one of the 7 is a forward reference. Six are staleness in one dated audit record (`spikes/numerics/delivered-realization-record/README.md`, whose own body says `No production edit`) and the seventh is an SDK header cited without provenance.
- **Framing — option 2 is a real narrowing of option 1. FALSE.** Only **3 of the 68** tracked markdown files under `spikes/` are not a `README.md`, and two of those three are the `PROTOCOL-*.md` files option 2 names. Option 2 therefore selects 67 of 68 files, differing by a single `RUN.md`, and **both** citation-failing files are `README.md` — the exact population option 2 keeps. Harness sources are `.rs`, `.metal`, and `.toml`, which no markdown checker reads under either option.

## The decision: option 4, which the ticket did not enumerate

**Check spike records' markdown links; deliberately decline their pinned citations.** The ticket's own User-visible outcome and Closes-when are both stated over a rotted **link**, and this satisfies them exactly while costing zero repairs.

The split is forced by `spikes/README.md`, which already records the governing convention under "Whether a spike still runs": a spike is "evidence about the base its own record names, not about `main`", and spikes are "repaired on demand" rather than kept green — a decision with two mechanical alternatives costed and rejected against real breakage. A pinned citation is a claim about a tree; a spike's is a claim about *its own dated base*, so demanding it resolve at the tip is the unsatisfiable condition the checker's bare-path rule already names, and is the same rule the terminal-ticket and `superseded`-document skips state over different metadata. A link is the other kind of claim — a promise to a reader who follows it now — and the currency convention is itself navigated by link, so the route it sends readers down was the one nothing guarded.

**Rejected, with reasons:**

- **Option 1 (all of `spikes/**`, links and citations).** Fails the gate red today on 7 citations in 2 files, both outside this ticket's declared scope. Structurally worse than the count: it lets a landing in `crates/` redden `make full` through exploratory material, which is what AGENTS.md refuses with "manually from documented commands so exploratory dependencies do not silently become repository gates", and what the `spikes/README.md` decision already rejected on evidence.
- **Option 2 (records only).** Not the narrowing it is framed as — 67 of 68 files, and it retains both failing files. It carries every defect of option 1 for a one-file reduction.
- **Option 3 (record the exclusion, change nothing).** Dominated. It leaves 590 navigation links unguarded — including the two links `spikes/README.md` routes the currency convention through — at zero cost to guard. Option 4 delivers option 3's documentation *and* the link coverage, so nothing is given up by preferring it.

## Evidence

Every property perturbed separately, at `77cd0104` plus this change:

1. **A broken spike link fails.** Planted in the depth-6 record `spikes/program-planning/reduction-dispatch-crossover/results/2026-08-18-apple-m4-max-macos27.0-26A5406e/RUN.md`, which also proves the recursive `find` reaches the deepest member: script **exit 1**, `FAIL ... link: [...](./no-such-sibling.md)` / `no tracked file or directory at ...`.
2. **A sound spike link still passes, and a stale spike pin still does not fail.** One line adding both a resolvable link and a `honourability.rs` pin at `:99999` — 98955 lines past the end of that 1044-line file, written here as a bare suffix because pinning it to its path would make this sentence a citation the gate must then demand resolve — left the run at **exit 0**, with the census moving 590 → 591 links and 50 → 51 declined citations. The declination is reachable and non-vacuous rather than green by accident.
3. **The file-count floor fires.** Narrowing the find to `-maxdepth 3`: `SHORT the spikes/** population reached 66 file(s), below its floor of 68.` on a run that had no other failure — the silence it exists to end.
4. **The link floor fires, after being strengthened because the first version did not.** Disabling `scan_links` for the role left the counter at **1, not 0**, because a reference definition reaches `link()` on its own path and exactly one exists, the `ExpansionCache` reference definition in `spikes/cache/README.md`. A greater-than-zero floor was therefore satisfiable by one stray line while all 68 records went unwalked. Re-floored at one link per live record, derived from the corpus rather than written down: `SHORT the spikes/** markdown link population reached 1 link(s), below its floor of 68.`

The existing corpus is unperturbed by the change: `checked` stays 1324, and the tickets/docs/root citation counts stay 118/1201/0.

**Residual exposure, stated rather than left to be found.** Spike *links* into `crates/` are resolved like any other — 12 of them, 11 inline targets plus the single reference definition this corpus carries — so deleting or renaming a file a spike links to does fail the gate. That asymmetry is intended: a line moving can never redden the gate through a spike, while a path disappearing breaks a promise to a reader today, and `docs/**` and `tickets/**` already carry that exposure by the thousand.

## Non-goals

Adding `spikes/` to `Cargo.toml` members, to `make full`, or to any build gate — AGENTS.md forbids it and that is settled. Repairing individual spike links, which belongs to the catalog ticket. Changing the currency convention.

## Closes when

A broken link inside a retained spike record either fails the gate with its output quoted, or the exclusion is stated in `check-citations.sh` and in AGENTS.md's description of what `make citations` covers — and in either case the population was sized before the choice was made.
