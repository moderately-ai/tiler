#!/bin/sh
# Resolve the pinned source citations and the local markdown links in open
# tickets and live documents against this working tree.
#
# WHAT A GREEN RUN MEANS, AND WHAT IT DOES NOT
#
# Green means every pinned citation in an open ticket or a live document points
# *somewhere*: the file exists, the line is inside it, the quoted anchor occurs
# in it. That is all. It is emphatically **not** evidence that a ticket's Facts
# or an ADR's prose are true. A citation resolves perfectly and still supports a
# claim the code no longer makes -- that is what happened on 2026-08-07, when a
# claim about an obligation class named the right file and the right symbol and
# described behaviour the code does not have. This checker would have passed it,
# and it passes the deliberately wrong sentence carried in the built-in fixture
# below.
#
# AGENTS.md carries the control that actually governs this: a ticket's stated
# Facts are stale until re-read at your own base, and a worker's first
# deliverable is a per-Fact verdict. This script is the cheap mechanical floor
# underneath that obligation, never a substitute for it.
#
# THE BUILT-IN FIXTURE, AND WHY IT IS NOT A TICKET
#
# That demonstration used to live in a ticket. On 2026-08-07 the ticket was
# recorded `done`; terminal tickets are skipped by design; and the fixture went
# inert in the same motion -- taking every anchor-form citation on the tree with
# it, because they all lived in that one file. Nothing failed, because a code
# path that runs zero times reports no failures.
#
# So the fixture belongs to this script now. It is written to a temporary file
# that is always the first member of the checked population. It has no
# frontmatter, no `status`, and no id in the work graph, so no ticket transition
# can reach it and `tkt` does not know it exists. Switching it off means editing
# this file -- the same edit that would have to delete the floors below, in one
# diff, in front of the one reviewer who is already looking at the checker.
#
# PER-FORM POPULATION FLOORS
#
# Every form this script supports is counted, and a count of zero fails the run
# naming the starved form: line-only, anchor-only, line+anchor, an anchor that
# matches only once whitespace is collapsed, and a code span assembled across a
# line break in prose. The fixture supplies one citation of each, so a zero
# means the *matcher* stopped recognising that form -- corpus drift cannot
# produce one. Drift is not a defect; a matcher that has stopped reaching its
# subject is, and without a floor it is indistinguishable from a clean run.
#
# WHAT COUNTS AS A CITATION
#
# A citation is an inline code span holding a path *plus a pin* -- a line
# number, a quoted anchor, or both:
#
#   `crates/tiler-compiler/src/explain.rs:3883`
#   `crates/tiler-compiler/src/explain.rs:3870-3890`
#   `crates/tiler-conformance/src/device_buffer.rs "#[allow(unsafe_code,"`
#   `crates/tiler-compiler/src/policy.rs:1092-1133 "UNPLANNED_OPERATIONS"`
#
# A bare path with no pin is deliberately NOT checked. Tickets name paths that
# do not resolve for two legitimate reasons: a file the ticket is asking
# someone to create, and a file whose deletion the ticket is recording.
# `scripts/check_workspace.py` is the standing example -- deleted at `e197176f`
# when the Python gate became the Makefile, still named by the terminal tickets
# that record that history: 32 of the 33 files `grep -rl
# 'scripts/check_workspace.py' tickets/` returns are terminal as of 2026-08-07,
# and the population only grows. Demanding those resolve would be an
# unsatisfiable condition. A line number or an anchor is different in kind: it
# is a claim about what a file contains *now*, so it is checkable now. The
# census prints how many bare paths were skipped, so the exclusion is counted
# rather than silent.
#
# An extensionless, directory-less token is the other deliberate exclusion, and
# it is narrower than it looks. `ADR:87` and `0:100` are pinned code spans by
# shape, but `ADR` and `0` name a decision record and a tensor axis rather than
# files, and demanding they resolve would be the same unsatisfiable condition.
# So a token with no directory component and no extension is a citation only when
# this tree carries a file of that name -- at the root, or as the tail of a
# tracked path -- and is otherwise dropped. The census prints that drop in spans
# and in distinct names, so this exclusion is a number beside the bare paths
# rather than a silence.
#
# The tail half of that test is what closed a fail-open on 2026-08-19. Until then
# the drop asked only whether the name sat at the repository root, so a pinned
# `LICENSE:5` -- a plausible spelling, since this repository vendors nine files
# by that name and the section below cites one of them by its full path -- was
# discarded before any counter and before the ambiguity check, on a run that
# stayed green. Nine tracked paths end with `LICENSE`, so the branch below would
# have failed it as an ambiguity absent from the ledger. qualifies() carries the
# measurement showing the two populations separate cleanly.
#
# The same rule is what keeps a dated correction writable. A correction that
# retires a citation quotes the retired line number in prose, or as the bare
# `:789-810` suffix the house style already uses, rather than pinning it to a
# path -- so it is not a citation here and cannot be demanded to resolve.
#
# HOW A PARTIAL PATH RESOLVES, AND THE ONE THING NO WORKING TREE RECORDS
#
# A citation whose path is not in the tree is matched against every
# "/"-boundary suffix of every tracked path, because tickets and documents
# routinely cite a partial one -- `codec/encode.rs:443`, `semantic/identity.rs`
# -- which is unambiguous to a reader. Exactly one match resolves; more than one
# is ambiguous and is skipped; none is a failure.
#
# A suffix matching exactly one file is stable under deletion. Delete the file it
# names and the count falls to zero, which is the loud "no file in the tree is or
# ends with" failure, and deleting anything else cannot change what it names. The
# *ambiguous* case is the unstable one, and it is unstable in the wrong
# direction: an ambiguous suffix is skipped, so the day its family shrinks to one
# member the citation stops being skipped and starts resolving -- against
# whichever twin survived, which is not the file the citation was written about.
#
# That is not hypothetical. `a2e98b27` deleted
# `crates/tiler-ir/src/index/refinement.rs`, leaving
# `crates/tiler-ir/src/semantic/accuracy/refinement.rs` alone under the suffix
# `refinement.rs`. One live snapshot citation began resolving green against the
# surviving file, which it was never about; four siblings failed only because
# their line numbers ran past that file, which is 659 lines. A longer survivor
# would have passed all five in silence.
#
# Nothing in a working tree tells "this suffix has always named one file" apart
# from "this suffix named two until last week" -- the deleted twin leaves no
# trace in the index. So the memory is written down, in the retired-ambiguity
# ledger below, and the rule it buys is a pair. Both halves are failures rather
# than skips, because a skip is the silence this section exists to end.
#
#   - A ledgered suffix now matching exactly one file FAILS. The family
#     collapsed, so resolving would point the citation at the survivor.
#   - A cited suffix matching more than one file and absent from the ledger
#     FAILS. It became ambiguous after the ledger was written -- a file was
#     added, not deleted -- so the citation has silently stopped being checked,
#     and it becomes the first case the moment that new file goes away.
#
# The repair for both is one sentence: re-read which file the claim is about and
# pin a path long enough to be unique on its own. Recording the suffix in the
# ledger is the alternative for the second case, and only for the second.
#
# The ledger holds suffixes and nothing else -- no candidate paths, no counts --
# so a file moving inside a family does not touch it, and it changes only when a
# cited suffix first becomes ambiguous. An entry whose family has gone quiet is
# inert rather than wrong, so nothing has to be pruned; `refinement.rs` is seeded
# for exactly that reason, having been observed ambiguous before its collapse and
# carrying no live citation now. What the ledger already covers is reproduced by:
#
#   ./check-citations.sh --verbose |
#     sed -n 's/^SKIP  [^:]*: `\(.*\)` (ambiguous.*/\1/p' |
#     sed -E 's/[:[:space:]].*$//' | LC_ALL=C sort -u
#
# That is a coverage listing and not a discovery tool: an unledgered ambiguity
# fails rather than printing a SKIP line, so it never appears there. The failure
# names the suffix to add, which is the only way an entry is ever added.
#
# WHAT THE LEDGER IS NOT, AND THE TWO RULES MEASURED AND REJECTED
#
# Measured 2026-08-19 at `bda38064` with the pre-change script, which is what
# these two were weighed against: 470 citations resolve by unique suffix over 99
# distinct suffixes, and 329 are skipped as ambiguous over 40. Of those 40, 22
# match exactly two tracked files and are one deletion from the failure above,
# carrying 117 citations; replaying both possible deletions against each, 77 of
# the 117 would resolve silently green against the survivor.
#
#   - "Require at least one directory component" is not the rule. Six of those 22
#     already carry one -- `program/verify.rs`, `program/model.rs`,
#     `program/handles.rs`, `program/mod.rs`, `builder/proof.rs`, `bf16/tests.rs`
#     -- so it would leave the exposed population only partly covered, while
#     failing the 420 of those 470 that are bare basenames and resolve correctly
#     today. It fails the wrong ones and misses six of the right ones.
#
#   - Deriving the memory from `git log --diff-filter=D` is not the rule either,
#     though it needs no maintenance. It cannot tell a deletion from a rename and
#     cannot tell whether two paths ever coexisted, so it poisons a suffix
#     forever on a move. Measured over the 65 paths ever deleted here, its whole
#     effect was to break one live citation, the partial `:289` pin on
#     `check_provenance` in
#     `docs/research/extensions/backend-provider-composition.md`, whose only
#     historical twin is a deleted `prototypes/serial-sum-compile/src/payload.rs`
#     and which reading confirms correctly names
#     `crates/tiler-artifact/src/program/codec/payload.rs`.
#
#     That rejection stands, and the review it was weighed against did not
#     weaken it. `git log` is a review list here and never the rule: nothing
#     enters the ledger from a deletion line, and the reading is what decides.
#     What the reading changed, on 2026-08-19 at `ea321967`, is the verdict on
#     that one suffix. Re-cut at that base the overlap is still exactly one --
#     over 66 ever-deleted paths, and over the 82 that `--no-renames` reports,
#     which is the wider list because a rename collapses a family too. Reading
#     its history rather than its deletion line shows the ambiguity was real and
#     live: `7e01f3b7` added the prototype twin on 2026-07-25 beside the codec
#     file, `2d2a7bd7` removed it on 2026-07-28 with 152 lines deleted and no
#     matching addition anywhere in that commit, and in between a `todo` ticket
#     cited the bare suffix -- `tickets/stop-recomputing-pure-derivations-in-the-codec.md`
#     pinned `:403` on it at `3dacabce`. This script did not exist until
#     `7e3a7367` on 2026-08-07, so no run could have recorded what was there.
#
#     So an entry is admitted two ways, and only two. From an ambiguity a run
#     observed while a live citation rested on it, which is how every other entry
#     below arrived. Or from one a reading reconstructs across a collapse that
#     predates this script, where the deletion is named and shown not to be a
#     rename, the two paths are shown to have coexisted, and a live citation is
#     shown to have rested on the bare suffix inside that window. What is still
#     refused is the thing the rule above would have done: an entry inferred from
#     a deletion line alone, with none of that read.
#
# WHAT ROOTS A CITATION IN ANOTHER PROJECT, AND WHY THAT IS WRITTEN DOWN TOO
#
# A path that is nowhere in this tree, not even as a suffix, is usually drift --
# but not always. `docs/**` cites upstream sources the way their own repositories
# spell them, with the revision named in the prose beside the span:
# `candle-core/src/metal_backend/device.rs:101` at `huggingface/candle`
# `31f35b14`, and `MTLDevice.h:233-242` under `MacOSX26.5.sdk` in the macOS SDK
# build `25F70`. Demanding those resolve against Tiler is the unsatisfiable kind
# of condition the bare-path rule above already names.
#
# Until 2026-08-19 that judgement was inferred from the tree: a multi-segment
# path was called upstream when its leading component matched no component of
# any tracked path. That reads the wrong thing. It asks which directories exist
# today, and a directory can stop existing. A live `adapter_route/adapter.rs:422`
# fails loudly while `adapter_route` is still a tracked component; delete
# `crates/tiler-runtime/tests/adapter_route/` -- the one deletion that breaks
# every citation under it -- and the same citation is silently reclassified as
# belonging to another project. Measured 2026-08-19 at `23eb1bf4` by deleting
# exactly that directory: of the six pinned citations it broke, five named a path
# beginning `crates/` or `tests/` and failed, while the sixth, the partial one,
# printed `(rooted outside this tree: no tracked path has a adapter_route
# component)` and reported nothing. Same deletion, same defect, and the citation
# that went quiet was the one whose leading component the deletion removed.
#
# So the memory is written down, the way the ambiguity ledger above is: an
# upstream root is listed by name, and nothing else is upstream. A multi-segment
# path that resolves nowhere and is not rooted in a listed project FAILS, and the
# failure names the root to add -- which is the only way an entry is ever added.
#
# Two properties make the list safe to hold. First, it can only ever skip fewer
# citations than the component test did, because a listed root is required to be
# absent from every tracked path: anything this skips, the old rule skipped too.
# Measured 2026-08-19 at `23eb1bf4`, it converts none of the skipped population
# into failures -- all 16 citations reaching the branch, which are 16 pinned
# occurrences spelling 10 distinct spans, are rooted in `candle-core`,
# `candle-metal-kernels`, or `MacOSX26.5.sdk`. Second, a listed root that IS a
# tracked component would silence this tree rather than another one, so it aborts
# the run at exit 2 naming the collision instead of being quietly honoured.
#
# The list carries no floor, and the asymmetry with the ledger is the reason
# rather than an omission. A truncated ledger silently weakens the check: every
# entry lost is a failure that stops firing. A truncated upstream list does the
# opposite -- the citations resting on it start failing on the next run -- so
# truncation is already loud and a floor would add nothing.
#
# The version-pinned spelling is the same category reached by a different road,
# `metal-0.33.0/src/device.rs:74-82` and
# `objc2-metal-0.3.2/src/generated/MTLDevice.rs:238`, and it carries the same
# collision guard rather than trusting the shape alone. That guard is not
# theoretical: five tracked directories already carry a version-pinned name
# (`docs/research/numerics/sources/arrow-25.0.0` and four siblings), so on the
# shape test alone a partial citation rooted in one of them would be skipped as
# somebody else's dependency while the file it names sits in the index.
#
# The two skips are counted separately, and that separation is the point. They
# shared one counter until 2026-08-19, under a census line that named only the
# second; the 32 it printed at `bda38064` was 16 of each, and it was read off as
# the size of one branch into a ticket Fact. A census line that sums two
# exclusions under one name is a silence of its own.
#
# A single-segment name is never upstream. A bare filename is the shorthand this
# repository uses for its own files, and one that resolves nowhere is drift or a
# citation that should have carried its provenance -- both of which must fail
# rather than skip.
#
# WHAT COUNTS AS A LINK, AND WHY IT IS A SECOND POPULATION
#
# A markdown link is a different claim from a citation and is checked
# separately. A citation says "this file contains this text"; a link says
# "follow me and you arrive somewhere". Every catalog, index, and cross-
# reference in `docs/` is navigated by link, including the two entry points
# AGENTS.md sends every reader to, and until 2026-08-08 not one of them was
# resolved: a link target is not an inline code span, so it never reached
# classify() at all. Replacing a live row in `docs/decisions/README.md` with
# `](9999-no-such-adr-...)` left this script at exit 0 with byte-identical
# output -- including an unchanged `bare path mention(s)` count, which is the
# proof that the dangling target was not landing in that bucket but was not
# being seen at all.
#
# The forms read are the inline link `[text](target)` -- matched on `](target)`
# so a code span inside the link text cannot hide it -- the image `![alt](src)`,
# which is the same syntax, and the reference definition `[label]: target`.
# `[^label]:` is a footnote, not a definition, and is excluded.
#
# Link targets are read only outside inline code spans and outside fenced
# blocks, and both exclusions carry their weight. A target quoted as text --
# which is how the two tickets describing the planted failure above spell
# `](9999-no-such-adr-...)` -- is prose about a link rather than a link. A
# fenced block is content proposed for somewhere else: the four dangling-looking
# targets in `tickets/catalog-the-kani-verification-research-and-spike.md` are
# catalog rows the ticket is asking someone to paste into `docs/research/`, so
# they are relative to that directory and not to the ticket that quotes them.
# Both are the rule classify() already applies to citations, reaching a second
# population.
#
# WHAT IS DELIBERATELY NOT RESOLVED, AND WHY. Each of these is counted in the
# census, so every exclusion is a number a reader can see rather than a silence.
#
#   - External targets: `http://`, `https://`, `mailto:`, `tel:`, and any other
#     `scheme://`. Resolving them is a claim about the network, not about this
#     working tree. It would need network access on a gate that has none, would
#     be nondeterministic, and link-rot in a cited upstream specification is not
#     a defect in this repository. 661 of them reach this branch on 2026-08-08:
#     637 inline `](https://...)` targets over the whole tree less those in
#     terminal tickets, plus the reference definitions, which are almost all
#     upstream URLs.
#
#   - Heading anchors, both same-document (`#section`) and into another document
#     (`other.md#section`). For a link that carries a path the *path* is
#     resolved and the fragment is not, and a pure `#section` is skipped whole.
#     The anchor slug is produced by whatever renders the markdown, and this
#     repository pins no renderer: GitHub, editors, and rustdoc disagree on
#     punctuation stripping, on unicode, and on how duplicate headings are
#     suffixed. A checker that picked one would fail links that work where the
#     documents are actually read, and a check that invents failures gets
#     weakened rather than repaired. The two counts are reported so the size of
#     the unresolved property stays visible.
#
#   - Vendored upstream sources under `docs/research/*/sources/`. Their links
#     are relative to the upstream site or repository they were copied from, and
#     only the one file of each upstream tree was copied here, so essentially
#     none of them can resolve: `Broadcasting.md` from the ONNX operator
#     reference names a sibling that was never vendored, and `/onnx/defs` names
#     a web root. Measured 2026-08-08 across those 15 files: 507 `](...)`
#     targets, 212 of them reaching this branch once the external ones are
#     counted first, and 92 that a resolver run over them reports as dangling.
#     Demanding those resolve is the unsatisfiable condition the bare-path rule
#     above already names, not 92 caught defects, and repairing them would mean
#     editing evidence that is supposed to be a verbatim copy. This is the one
#     place where the fail-closed choice the pinned checker makes for these same
#     files is NOT free -- there it was measured at zero cost, here at 92 -- so
#     the two populations decide it differently and both say why.
#
#   - A target with whitespace in it. Markdown requires `<...>` around such a
#     target, and nothing here uses that; what the pattern actually catches is
#     pseudo-code in a vendored specification (`](%max_trip_count, %keepgoing)`).
#     An empty `]()` is likewise malformed rather than broken. Six reach this
#     branch, measured 2026-08-19 at `04823326`, all six in
#     `docs/research/numerics/sources/onnx-v1.22.0/Operators.md`.
#
#     This test runs first in link(), ahead of the external and vendored ones,
#     and that order is deliberate: whether a string is a link target at all is
#     prior to where the file carrying it lives. It does mean the vendored count
#     is measured after this one, and the reconciliation is those same six. All
#     six sit in a vendored file, so moving this test below the vendored one
#     would raise that census line from 212 to 218 -- confirmed by running it
#     both ways at `04823326` -- and would leave this counter reading zero on a
#     corpus where the condition occurs six times, because the only files that
#     produce it are vendored. A counter that the sole population producing its
#     condition can never reach reports clean whatever the tree holds, which is
#     the shape of silence this whole section exists to remove. So the 212 is the
#     population reaching that branch and 218 is the vendored total; both are
#     printed, on their own numbers, on the census line below.
#
# WHAT A LINK IS RESOLVED AGAINST. The target is joined to the directory of the
# file that carries it, `.` and `..` segments are collapsed, and the result must
# be a tracked file or a tracked directory. The git index is the authority
# rather than the filesystem for three reasons: a link is a promise to a reader
# who has a clone, so a target that exists only in someone's working tree is
# already broken for them; awk cannot tell a directory from an empty file, and
# probing one emits an i/o error and reports success, so directory targets
# (`../../spikes/runtime/inline-dispatch`) need the index anyway; and this host
# is case-insensitive, so a filesystem test would accept `docs/README.MD`.
#
# A path-shaped target absent from the index but present on disk is accepted and
# counted on its own census line, so a document created and not yet staged does
# not fail the run while the fact that it is unstaged stays visible. A `..` that
# walks above the repository root fails; so does a site-absolute `/path` outside
# the vendored subtree, because nothing here is served from a web root.
#
# The link population is floored the same way the citations are: once per corpus
# in report(), plus a form floor under the fixture link, so a matcher that stops
# finding `](` cannot report a clean run.
#
# SPIKE RECORDS: LINKS CHECKED, PINNED CITATIONS DELIBERATELY NOT
#
# `spikes/**` was outside every population until 2026-08-22, and the gap was
# found the way this file keeps finding them -- by perturbation, not by reading.
# `worker-tileprotocol` planted a broken link under `spikes/` and this script
# returned exit 0. Re-run at `77cd0104` the output was not merely green but
# byte-identical to the unperturbed run, which is the signature of a file that
# was never opened rather than one that passed.
#
# The two properties are split here, and the split is the whole decision. A
# spike record is checked for its links and deliberately not for its pinned
# citations, because the two make different claims and only one of them is a
# claim about this tree today.
#
# Why the citations are declined. `spikes/README.md` states the governing
# convention under "Whether a spike still runs": a spike is "evidence about the
# base its own record names, not about `main`", spikes are "repaired on demand"
# rather than kept green, and that is a recorded decision with two mechanical
# alternatives costed and rejected against real breakage. A spike record pinning
# `honourability.rs:1492-1498` is describing the tree at its own dated base, so
# demanding it resolve at the tip is the unsatisfiable condition the bare-path
# rule above already names -- and it is the same rule the terminal-ticket and
# `superseded`-document skips state over different metadata: skip a record whose
# citations describe a tree it is no longer authoritative over, because
# rewriting one to match today's line numbers destroys the account of what was
# actually done.
#
# It would also be the failure AGENTS.md forbids by name. Nothing here builds or
# runs a spike, so a landing in `crates/` that moves a line would redden
# `make citations` through a spike record -- turning exploratory material into a
# repository gate, which is what AGENTS.md refuses with "manually from
# documented commands so exploratory dependencies do not silently become
# repository gates".
# Measured at `77cd0104` by running this script over the corpus with the
# declination below switched off: 50 pinned spans qualify as citations here, of
# which 41 resolve, 7 do not, and 2 are skipped as version-pinned dependency
# sources. All 7 are that case rather than drift this gate should police, and
# the 7 decompose as 4 reaching the resolver and 3 failing as an ambiguity absent
# from the ledger -- which is why the checked count reads 45 rather than 48, a
# sum a reader should not have to reconstruct from a census line. Six sit
# in `spikes/numerics/delivered-realization-record/README.md`, a dated audit
# record whose own body says "No production edit" -- two line pins now past the
# end of a `honourability.rs` that has since shrunk to 1044 lines, one naming a
# `metal_profile.rs` that no longer exists, and three on a `realization.rs`
# suffix that has since turned ambiguous across three tracked files. The seventh
# is `MTLComputeCommandEncoder.h:31-34`, an Apple SDK header cited without its
# provenance. Not one is a defect in the record; every one is the record being
# older than the tree, which is what a spike record is for.
#
# Why the links are checked, and why they are free. A link is a promise to a
# reader who follows it now, not a claim about a base -- the distinction the
# link section above already draws. The currency convention itself is navigated
# by link: `spikes/README.md` sends a reader to the metadata contract for
# `last_verified` and routes the entire "repaired on demand" rationale through a
# link into `target-profiles/scalar-cpu-vertical/README.md`. If those rot, the
# route the convention sends readers down is the one nothing guards. Measured at
# `77cd0104` across the 68 tracked markdown files in this corpus: 598 link
# targets reach the checker, 590 of them local and resolving, 1 external and 7
# same-document anchors. Zero dangle, and zero are forward references to paths
# that do not exist yet -- so this population closes a gap at no repair cost, the
# way the repository-root documents did on 2026-08-08.
#
# What this does still expose, stated rather than left for someone to find. A
# spike link into `crates/` is resolved like any other link, and 12 of them
# exist -- 11 inline targets plus the one reference definition in this corpus.
# So deleting or renaming a file a spike record links to does fail this
# gate. That is the intended asymmetry rather than a leak in the argument above:
# a line moving is the ordinary consequence of any landing and can never redden
# this gate through a spike, while a path disappearing breaks a promise to a
# reader who follows it today -- and `docs/**` and `tickets/**` already carry
# exactly that exposure, by the thousand.
#
# The floors mirror that split. There is a file-count floor sized from the index
# and a link floor, and deliberately no citation floor: this population resolves
# no citation at all by construction, so a citation floor here would fail on a
# correct tree -- the same reason the repository-root population carries none.
# The declined citations are counted on the census line rather than dropped, so
# the exclusion is a number a reader can see rather than a silence.
#
# What was rejected. Checking spike citations too, and checking only the
# `README.md` and `PROTOCOL-*.md` records rather than everything under
# `spikes/`. The second is not the narrowing it looks like: only 3 of the 68
# markdown files here are not a `README.md`, two of them `PROTOCOL-*.md`, so it
# selects 67 of 68 files and both citation-failing files are READMEs. Harness
# sources are `.rs`, `.metal`, and `.toml`, which no markdown checker reads
# either way.
#
# THE FOUR POPULATIONS, AND WHAT TERMINAL MEANS IN EACH
#
# `tickets/**`, `docs/**`, `spikes/**`, and the tracked markdown at the
# repository root are read and counted separately, and each carries its own
# floor, so none of them can collapse into another and read as a clean run on
# the strength of the ones that still work.
#
# The skip rule is the same rule in each, stated over different metadata: skip a
# record whose citations describe a tree it is no longer authoritative over,
# because rewriting one to match today's line numbers destroys the account of
# what was actually done. Check the records a reader will follow into the code,
# which is exactly where a stale citation does its damage.
#
# Terminal tickets (`done`, `closed` -- read from `ticketsplease.toml`, not
# hardcoded here) are skipped. A comment file inherits the status of its parent
# ticket, because it is part of the ticket a worker is told to read in full.
#
# For a document the equivalent is `superseded`, and only that. It is the one
# status value in `docs/document-metadata.md` that means *replaced*: it is a
# `decision_status` on an ADR and a `disposition` on a research record, and this
# script reads either. Three files carry it as of 2026-08-07. Neighbouring
# values are deliberately not terminal and the distinction is the whole point --
# an accepted ADR, a `complete` research record, and a `rejected` or
# `informational` disposition are all still the standing account of their own
# conclusion, and AGENTS.md ranks the first of those as the highest evidence
# tier there is. `implementation_status` is never consulted: the metadata
# contract calls it a retained high-water mark rather than a live mirror of the
# tree, so it says nothing about whether a citation should resolve.
#
# The other half of a document's history stays writable because of the
# bare-path rule above, not because of anything here. A dated correction quotes
# the retired extent in prose or as a bare `:497-548`, never pinned to a path,
# so it is not a citation and cannot be demanded to resolve -- which is what
# keeps a convention that requires the retired text to stay from colliding with
# a check that requires it to be gone.
#
# A document with no status facet at all is checked rather than skipped, and the
# census counts how many there are. Twenty-four files under `docs/` reach that
# branch on 2026-08-07 and they are two unrelated populations, both correctly
# checked. The repository-root documents reach the same branch by the same rule
# and are counted on their own census line, for the reasons the next section
# gives; the twenty-four is a `docs/` count and stays one.
#
# That reading was correct when it was taken and is left standing; what has moved
# under it is the corpus, not the branch. A third population landed afterwards,
# and re-counted 2026-08-19 at `04823326` the census prints 1092: the same nine
# Tiler documents, the same fifteen vendored specifications, and 1068 files under
# `docs/research/documentation/ticket-audit-2026-08-10/`. The three partition the
# 1092 exactly. So the twenty-four has not gone wrong -- it is two of the three
# bullets below, at the date it names -- and a reader reconciling it against
# today's census is looking at a corpus that grew rather than at a branch that
# started admitting files it should not. All three are correctly checked, for the
# reason each bullet gives.
#
#   - Nine Tiler documents whose `kind` has no status facet at all. The kind
#     table in `docs/document-metadata.md` requires one of contract, decision,
#     research, experiment, roadmap, and questions, and requires none of portal
#     or prior-art. Seven portals and two prior-art records is exactly that set.
#     A portal is live by construction -- it is the entry point a reader is sent
#     to -- so there is nothing here to skip on.
#
#   - Fifteen vendored upstream specifications under `docs/research/*/sources/`,
#     which carry no `tiler-doc/v1` frontmatter because no status could be added
#     to them without editing evidence that is supposed to be a verbatim copy.
#     Checking them is the fail-closed direction and it is free today: measured
#     2026-08-07, the only pinned spans in that subtree are `0:100` and `0:10`,
#     which name no file and are dropped by qualifies() before anything is
#     demanded of them. If one ever does resolve to a real claim about this
#     tree, the failure names the file and a reader decides; a carve-out here
#     would instead be a hole nobody sees.
#
#   - The ticket-audit records under
#     `docs/research/documentation/ticket-audit-2026-08-10/`, 1068 of them at
#     `04823326`, which is the population the twenty-four predates. They are
#     per-ticket audit reports carrying no `tiler-doc/v1` frontmatter, so they
#     reach this branch the way AGENTS.md and CLAUDE.md reach it -- nothing was
#     seen, so nothing was seen that retires them. Checking them is right rather
#     than merely harmless: they are the standing account of what an audit found,
#     they cite the tree by line and anchor throughout, and a stale citation in
#     one misroutes exactly the reader who went looking for the audit. Their
#     citations resolve today, so the fail-closed direction costs nothing here
#     either.
#
# THE REPOSITORY-ROOT DOCUMENTS, AND WHY THEIR FLOOR IS A FILE COUNT
#
# `README.md`, `AGENTS.md`, and `CLAUDE.md` are the first files a reader opens,
# and until 2026-08-08 they were the only tracked markdown that no population
# reached. Appending `[planted](docs/decisions/9999-no-such-adr.md)` to
# AGENTS.md left this script at exit 0 -- not one defect fewer than expected,
# but no defect there catchable at all, because the file was never opened.
# AGENTS.md links the ADR that governs every unsafe site in the workspace, and
# README.md is six links of route into `docs/`, `spikes/`, and AGENTS.md itself,
# so the reader a dangling link here misroutes is the one being onboarded.
#
# Measured 2026-08-08 before they were added: 3 files, 7 local links (README.md
# 6, AGENTS.md 1, CLAUDE.md 0), 0 pinned citations, and all 7 resolve. This
# population closes a gap; it did not repair a break.
#
# Their status is decided by the `doc` rule, which they share rather than
# duplicate, because that rule already answers the question they ask: a document
# with no status facet at all is checked rather than skipped. README.md is a
# `tiler-doc/v1` portal, the same kind as seven of the nine Tiler documents that
# reach that branch under `docs/`, and the kind table requires no status facet of
# a portal. AGENTS.md and CLAUDE.md carry no frontmatter whatever and reach the
# same branch by a different road: nothing was seen, so nothing was seen that
# retires them. `superseded` retires a root document exactly as it retires any
# other, because the branch is shared whole rather than sliced. What made them a
# third population was never the status rule -- it is that the fall-through in
# role_of() called them tickets, and the ticket rule demands a `status` key that
# a portal and a plain markdown file both legitimately lack.
#
# What is deliberately NOT shared is the counting. Every counter is keyed by
# role, so the census reports the two on separate lines and each carries its own
# floor. That separation is the point: seven links inside the 5205 that `docs/`
# contributes would be invisible, and a root population that read zero files
# would sail through the `docs/` link floor untouched.
#
# The root floor is on the file count, which is where this population differs
# from the other two. It carries no pinned citation at all -- every path in these
# three files is a bare mention with no line and no anchor, which the bare-path
# rule above deliberately declines to check -- so a citation floor here would
# fail on a correct tree. The file count is what says the `*.md` glob still
# matches and role_of() still routes them; the link floor beside it is what says
# their prose was actually walked. A population that silently reads zero files
# prints the same green as one that read all three, and only a floor tells them
# apart.
#
# Usage: ./check-citations.sh [--verbose]

set -eu

cd "$(dirname "$0")"

verbose=0
for arg in "$@"; do
	case "$arg" in
	--verbose) verbose=1 ;;
	*)
		printf 'check-citations: unknown argument: %s\n' "$arg" >&2
		exit 2
		;;
	esac
done

if [ ! -f ticketsplease.toml ]; then
	printf 'check-citations: ticketsplease.toml not found; run from the repository.\n' >&2
	exit 2
fi

if [ ! -d docs ]; then
	printf 'check-citations: docs/ not found; run from the repository.\n' >&2
	exit 2
fi

# Terminal states come from the workflow table so the skip list tracks the
# config instead of a memorised pair of names.
terminal=$(awk '
	/^\[workflow\.states\./ {
		state = $0
		sub(/^\[workflow\.states\./, "", state)
		sub(/\]$/, "", state)
		next
	}
	/^\[/ { state = "" }
	state != "" && /^category[ \t]*=/ && /"terminal"/ { printf "%s ", state }
' ticketsplease.toml)

if [ -z "$terminal" ]; then
	printf 'check-citations: no terminal states found in ticketsplease.toml.\n' >&2
	exit 2
fi

# `git ls-files` supplies partial-path disambiguation only; existence is always
# tested against the filesystem. The index lives outside the repository so a
# killed run cannot leave an untracked file for someone else to stage.
indexfile=$(mktemp)
fixture=$(mktemp)
ledgerfile=$(mktemp)
upstreamfile=$(mktemp)
trap 'rm -f "$indexfile" "$fixture" "$ledgerfile" "$upstreamfile"' EXIT INT TERM
git ls-files >|"$indexfile"

# The upstream-root list, quoted heredoc so nothing here is expanded. One leading
# path component per line; blank lines and `#` comments are ignored by the
# reader. A citation whose path resolves nowhere in this tree is called somebody
# else's only when its leading component is named here -- never because the
# directory it names happens to be absent today, which is the fail-open rule the
# section above records and measures. Entries are added when a real upstream
# citation lands and the run names the root it wants; a root that is also a
# component of a tracked path aborts the run rather than being honoured.
cat >|"$upstreamfile" <<'UPSTREAM'
# huggingface/candle, cited at the revision named in the prose beside each span.
candle-core
candle-metal-kernels
# The macOS SDK headers, cited with the SDK build named beside them.
MacOSX26.5.sdk
UPSTREAM

# The retired-ambiguity ledger, quoted heredoc so nothing here is expanded. One
# "/"-boundary suffix per line; blank lines and `#` comments are ignored by the
# reader. Every entry records a suffix that really was ambiguous while a live
# citation rested on it -- seen by a run, or reconstructed by reading a collapse
# that predates this script, which the section above bounds to a named deletion
# shown not to be a rename with coexistence and a live citation both read out of
# the window. Entries are added and never pruned: the floor in report() is what
# makes a truncation loud rather than a quietly weaker check.
cat >|"$ledgerfile" <<'LEDGER'
accuracy.rs
adapter.rs
applicability.rs
bf16.rs
bf16/tests.rs
boundary.rs
builder.rs
builder/proof.rs
conformance.rs
contraction.rs
expansion.rs
host.rs
identity.rs
key.rs
lib.rs
main.rs
model.rs
numerics.rs
# Retired: `2d2a7bd7` deleted `prototypes/serial-sum-compile/src/payload.rs` on
# 2026-07-28 -- 152 lines removed, no matching addition in the commit -- leaving
# `crates/tiler-artifact/src/program/codec/payload.rs` alone under this suffix.
# `git ls-tree 2d2a7bd7^` carries both paths and `git ls-tree 2d2a7bd7` carries
# one, so they coexisted rather than one being the other renamed. A live `todo`
# ticket pinned the bare suffix inside that window, at `3dacabce`. This script
# postdates the collapse, so no run saw it and the entry is the reconstruction
# the section above admits. No live citation carries the bare suffix now.
payload.rs
perturb.rs
pointwise.rs
program.rs
program/handles.rs
program/mod.rs
program/model.rs
program/verify.rs
proof.rs
region.rs
registry.rs
# Retired: `a2e98b27` deleted `crates/tiler-ir/src/index/refinement.rs` and left
# `crates/tiler-ir/src/semantic/accuracy/refinement.rs` alone under this suffix.
# No live citation carries it now, so the entry costs nothing and stops the next
# one from resolving against the survivor.
refinement.rs
request.rs
rms_norm.rs
route.rs
scheduled-region-model.md
silu.rs
softmax.rs
sourced.rs
src/adapter.rs
target.rs
tests.rs
validate.rs
verify.rs
LEDGER

# The fixture, quoted heredoc so backticks and `$` reach the file intact. Every
# citation below resolves against this tree and each one is the only guaranteed
# instance of its form; the closing paragraph resolves and is false, which is
# the boundary demonstration. If a subject moves, repair the subject -- never
# the assertion, and never the false sentence.
cat >|"$fixture" <<'FIXTURE'
# Built-in fixture

Written by check-citations.sh on every run, checked before anything under
tickets/. It carries no frontmatter and no `status`, so nothing in the work
graph can move it out of the population.

Line-only form: `AGENTS.md:1`.

Line-and-anchor form: `ticketsplease.toml:1 "schema_version = 1"`.

Anchor that matches only after whitespace is collapsed, because the construct
wraps in the source it names: `AGENTS.md "origin/main...main # 0 0"`.

A citation whose code span straddles a line break in prose, which a
line-oriented matcher loses in silence: `AGENTS.md
"Priorities: **correctness, long-term maintainability, then performance**."`

Ambiguity recorded rather than resolved, and the guaranteed instance the ledger
floor names: `lib.rs:1`. Forty-four tracked files end with that suffix, so it
names no file; the ledger records the ambiguity, and a tree that deleted all but
one of them would fail here instead of resolving against the one left standing.

Link form, and the guaranteed instance the link form floor names: [the agent
guide](AGENTS.md). The fixture has no directory of its own -- it is written to a
temporary path -- so its links resolve from the repository root, which is where
a reader of this file would read them from anyway.

The excluded link shapes, one each, so the census lines that count them are fed
by something on every run rather than only when the corpus happens to carry one:
[an external target](https://example.invalid/no-such-page) is not resolved,
[a heading anchor into another document](AGENTS.md#research) has its path
resolved and its `#research` fragment left alone, and [a same-document
anchor](#built-in-fixture) has nothing to resolve at all.

**Deliberately false, retained as the boundary demonstration -- do not "fix"
it.** `make check` runs the citation check **last**, after the test target
(`Makefile "check: citations fmt build lint test"`). The anchor is verbatim, so
the citation resolves and this file passes; the sentence is wrong all the same,
because `citations` is prerequisite #1 and runs before `fmt`. Green means the
citations point somewhere. It has never meant the prose around them is true.
FIXTURE

# Ticket files first, then comment files: a comment inherits the status of its
# parent ticket, so the parent must have been read by the time it is reached.
# Each glob is tested rather than assumed -- an unmatched glob is passed through
# literally by the shell, and awk would abort on the non-existent name.
set -- tickets/*.md
if [ ! -e "$1" ]; then
	printf 'check-citations: no ticket files matched tickets/*.md.\n' >&2
	exit 2
fi
# Append the comment glob whole, once, rather than one file at a time: growing
# the argument list element by element is quadratic and cost seconds here.
for f in tickets/*.comments/*.md; do
	if [ -e "$f" ]; then
		set -- "$@" tickets/*.comments/*.md
		break
	fi
done

# The document population, appended after the tickets. `find` rather than a
# glob because POSIX sh has no recursive one, and the fixed-depth chain that
# substitutes for it (`docs/*.md docs/*/*.md docs/*/*/*.md`) drops a whole
# subtree in silence the day someone nests one level deeper -- which is the
# shape of failure the floors below exist to make loud. Sorted so the report is
# reproducible rather than in directory order.
docs_before=$#
IFS='
'
# One append, not one per file. Growing the argument list element by element is
# quadratic: measured on this tree, a `while read` loop appending the 256
# documents one at a time costs 1.18s against 0.03s here, on a run that is
# otherwise 0.6s end to end.
# shellcheck disable=SC2046 # Deliberate: IFS above splits the list on newlines
# and nothing else, which is exactly the split wanted. Quoting it would pass all
# 256 paths to awk as one filename.
set -- "$@" $(find docs -type f -name '*.md' | LC_ALL=C sort)
if [ "$#" -eq "$docs_before" ]; then
	unset IFS
	printf 'check-citations: no document files matched docs/**/*.md.\n' >&2
	exit 2
fi

# The spike records, appended after the documents and reached by the same
# recursive `find` for the same reason: they nest to six components
# (`spikes/program-planning/reduction-dispatch-crossover/results/<run>/RUN.md`),
# so a fixed-depth glob chain would drop a subtree in silence. IFS is still the
# newline-only split set above.
spikes_before=$#
# shellcheck disable=SC2046 # Deliberate, as above: IFS splits on newlines only.
set -- "$@" $(find spikes -type f -name '*.md' | LC_ALL=C sort)
unset IFS
if [ "$#" -eq "$spikes_before" ]; then
	printf 'check-citations: no spike records matched spikes/**/*.md.\n' >&2
	exit 2
fi

# The repository-root documents, appended last. The glob is over the root rather
# than over the three names it matches today, so a document added beside them is
# covered the day it lands; the floor in report() is what says the glob still
# matches. Appended one file at a time, which the docs comment above rejects as
# quadratic: that cost was measured over 256 files, and testing each name is
# what an unmatched glob needs here -- the shell passes `*.md` through literally
# when nothing matches, and awk would abort on that as a filename.
root_before=$#
for f in *.md; do
	[ -e "$f" ] || continue
	set -- "$@" "$f"
done
if [ "$#" -eq "$root_before" ]; then
	printf 'check-citations: no document files matched *.md at the repository root.\n' >&2
	exit 2
fi

# The fixture leads, so its forms are counted even when a later population is
# empty. Populations are appended above and classified by path inside awk rather
# than assumed to be tickets, which is what keeps `tickets/**` from being the
# only thing this script can read. A further population is an append there, a
# branch in role_of(), an answer in decide() for how its status is determined --
# reused from an existing branch where one already fits, as the repository-root
# documents reuse the `doc` rule -- and its own line in the census, and, if it is
# a corpus rather than a fixture, its own floor beside the others in report().
set -- "$fixture" "$@"

# The program below is one single-quoted shell word, so it must contain no
# apostrophe anywhere -- prose included. An apostrophe closes the quote and awk
# then reports a missing function from somewhere unrelated, which reads as a
# logic bug rather than a quoting one.
awk -v terminal="$terminal" -v verbose="$verbose" -v indexfile="$indexfile" -v fixture="$fixture" -v ledgerfile="$ledgerfile" -v upstreamfile="$upstreamfile" '
function slurp(path,   line, s) {
	if (path in content) return content[path]
	s = ""
	while ((getline line < path) > 0) s = s line "\n"
	close(path)
	content[path] = s
	return s
}

function linecount(path,   line, n) {
	if (path in lines) return lines[path]
	n = 0
	while ((getline line < path) > 0) n++
	close(path)
	lines[path] = n
	return n
}

function exists(path,   r) {
	if (path in seen_exists) return seen_exists[path]
	# getline returns -1 when the file cannot be opened, 0 at EOF (a present
	# but empty file), 1 on a read.
	r = (getline _probe < path)
	close(path)
	seen_exists[path] = (r >= 0) ? 1 : 0
	return seen_exists[path]
}

function squeeze(s) {
	gsub(/[ \t\r\n]+/, "", s)
	return s
}

function fail(span, msg) {
	failures++
	printf "FAIL  %s\n        citation: `%s`\n        %s\n", ticket, span, msg
}

# An extensionless, directory-less token is a citation only when this tree really
# carries a file of that name -- at the root, or as the tail of a tracked path.
# Both tests are load-bearing and in opposite directions. Without the root test
# `ADR:87` would be read as a path and fail forever, because it names a decision
# record rather than a file. Without the suffix test the drop failed open: until
# 2026-08-19 a pinned `LICENSE:5` was discarded here in silence, before any
# counter and before the ambiguity check, though nine tracked files end with that
# name and the branch below would have failed it as an unledgered ambiguity.
#
# The suffix test is what separates the two populations, and it separates them
# cleanly rather than approximately. Measured 2026-08-19 at `04823326`: the nine
# spans reaching this branch across `tickets/**` and `docs/**` spell five
# distinct tokens -- `ADR`, `path`, `0`, `named`, `carries` -- and no tracked
# path ends with any of them, so all nine still drop. `LICENSE` ends nine tracked
# paths and `NOTICE` two, so both now reach the unledgered-ambiguity failure;
# `LICENSE-APACHE`, `LICENSE-MIT`, and `Makefile` end exactly one and resolve by
# unique suffix. Letting the whole extensionless case fall through instead was
# measured and rejected: it fails all nine, which is the "fail forever" the root
# test above exists to prevent.
#
# The drop is counted in the census, in spans and in distinct names, so what is
# excluded here is a number a reader can see rather than a hole.
function qualifies(p) {
	if (p ~ /\//) return 1
	if (p ~ /\.[A-Za-z0-9]+$/) return 1
	# Membership rather than a subscript read: referencing suffix_count[p] would
	# create the element, and an existing entry is always at least one anyway.
	if (p in suffix_count) return 1
	if (exists(p)) return 1
	unqualified++
	if (!(p in unqualified_seen)) { unqualified_seen[p] = 1; unqualified_distinct++ }
	if (verbose) printf "SKIP  %s: `%s` (not a path: no tracked file is or ends with it, and nothing of that name is at the root)\n", ticket, p
	return 0
}

BEGIN {
	# The fixture lives at a temporary path; report it by what it is.
	FIXTURE_LABEL = "<built-in fixture>"

	# The repository-root population is three tracked documents -- README.md,
	# AGENTS.md, CLAUDE.md -- and awk offers no type to size the enumeration
	# from, so the floor is asserted by hand and the census prints the count
	# beside it for a reader to compare. It is a floor and not an equality: a
	# document added at the root should raise the census without editing this.
	ROOT_FLOOR = 3

	# The retired-ambiguity ledger. Sized by hand for the same reason ROOT_FLOOR
	# is -- awk offers no type to count the enumeration from -- and the census
	# prints the count beside it. A floor and not an equality: an entry added
	# when a cited suffix first turns ambiguous should raise the census without
	# editing this. Lowering it is the deliberate edit that removes a guard, in
	# the same diff as the entry it removes, in front of the same reviewer.
	LEDGER_FLOOR = 42
	while ((getline led < ledgerfile) > 0) {
		if (led == "" || led ~ /^#/) continue
		if (led in was_ambiguous) continue
		was_ambiguous[led] = 1
		ledger_entries++
	}
	close(ledgerfile)

	# The recorded upstream roots. Read the same way as the ledger and
	# deliberately floored differently: the header section says why a truncation
	# here is already loud on the next run, so a floor would guard nothing. What
	# this list carries instead is the collision check after the index below.
	while ((getline up < upstreamfile) > 0) {
		if (up == "" || up ~ /^#/) continue
		if (up in upstream_root) continue
		upstream_root[up] = 1
		upstream_entries++
	}
	close(upstreamfile)

	n_terminal = split(terminal, tstates, " ")
	for (i = 1; i <= n_terminal; i++)
		if (tstates[i] != "") is_terminal[tstates[i]] = 1

	# Index every "/"-boundary suffix of every tracked path. Tickets routinely
	# cite a partial path -- `codec/encode.rs:443`, `semantic/identity.rs:384`
	# -- which is unambiguous to a reader and resolves here whenever exactly
	# one tracked file ends with it.
	while ((getline p < indexfile) > 0) {
		# The size of the ticket corpus, taken from the index so the floor in
		# report() is derived rather than written down and left to go stale.
		if (p ~ /^tickets\/.*\.md$/) tickets_tracked++
		# The size of the spike corpus, derived from the index for the same
		# reason: a hand-written number is satisfied by a find that has quietly
		# stopped covering its domain.
		if (p ~ /^spikes\/.*\.md$/) spikes_tracked++
		suffix = p
		while (1) {
			suffix_count[suffix]++
			suffix_path[suffix] = p
			if (!sub(/^[^\/]*\//, "", suffix)) break
		}
		# Every "/"-separated component of every tracked path, at any depth.
		# This no longer decides that a path is upstream -- the recorded list
		# does, and the header says why inferring it from the tree fails open on
		# a directory deletion. What it decides now is the opposite direction: a
		# name this tree uses at any depth cannot be claimed by another project,
		# which is what the collision check below and the version-pinned skip in
		# classify() both rest on.
		nseg = split(p, segs, "/")
		for (si = 1; si <= nseg; si++) component[segs[si]] = 1

		# Link targets resolve against the index rather than the filesystem;
		# the header states the three reasons. Every directory prefix is
		# recorded too, because a link may legitimately name a directory and
		# awk cannot tell one from an empty file.
		tracked[p] = 1
		dir = p
		while (sub(/\/[^\/]*$/, "", dir)) treedir[dir] = 1
	}
	close(indexfile)

	# A recorded upstream root that is also a component of a tracked path would
	# silence this tree rather than another one: every unresolvable citation
	# under that name would skip instead of failing, which is exactly the defect
	# the list was written to remove. There is no safe way to honour such an
	# entry, so the run stops and names it. This is also what lets the skip in
	# classify() test the list alone, with no component test beside it.
	for (up in upstream_root)
		if (up in component)
			collisions = collisions "\n        " up
	if (collisions != "") {
		# stderr, with every other fatal in this script. A gate that redirects
		# stdout to a log and reads the terminal would otherwise see exit 2 with
		# no message at all, which is the failure shape AGENTS.md tells a reader
		# of a redirected gate to go looking for.
		printf "check-citations: upstream root(s) recorded in check-citations.sh are also components of tracked paths, so citations under them would be skipped instead of checked:%s\n", collisions > "/dev/stderr"
		aborted = 1
		exit 2
	}

	# PATHRE is path-shaped on its face: it carries an extension. PATHRE_LOOSE
	# also admits an extensionless name so `Makefile:34` and
	# `docs/research/numerics/sources/jax-v0.11.0/LICENSE:5` stay reachable;
	# qualifies() is what stops that from swallowing prose.
	PATHRE = "[A-Za-z0-9_][A-Za-z0-9_./-]*\\.[A-Za-z0-9]+"
	PATHRE_LOOSE = "[A-Za-z0-9_][A-Za-z0-9_./-]*"
}

# The population is a list of paths; what a path *is* decides how its status is
# read and which counter it feeds. Everything downstream reads `role` rather
# than re-testing FILENAME, so a new population is named once, here.
function role_of(path) {
	if (path == fixture) return "fixture"
	if (path ~ /^docs\//) return "doc"
	# Spike records are read for their links and never for their pinned
	# citations; classify() carries the declination and the header says why.
	if (path ~ /^spikes\//) return "spike"
	if (path ~ /\.comments\//) return "comment"
	# A markdown file with no directory component is a repository-root document.
	# The test is over the shape rather than over the three names, so a document
	# added beside them is classified the day it lands. The fixture lives at an
	# absolute temporary path and is matched by exact path above, so it never
	# reaches here. Without this branch the fall-through calls them tickets and
	# the ticket rule in decide() fails them for having no `status` key.
	if (path !~ /\//) return "root"
	return "ticket"
}

FNR == 1 {
	end_file()
	role = role_of(FILENAME)
	ticket = (role == "fixture") ? FIXTURE_LABEL : FILENAME
	files_read[role]++
	status = ""
	doc_superseded = 0
	doc_status_seen = 0
	in_fence = 0
	in_frontmatter = 0
	decided = 0
	skip_file = 0
	in_span = 0
	span = ""
	span_wrapped = 0

	# A link is relative to the directory of the file carrying it. The fixture
	# has no directory of its own -- it is written to a temporary path -- so its
	# links resolve from the repository root, which is where a reader would read
	# them from anyway.
	linkdir = FILENAME
	if (role == "fixture" || !sub(/\/[^\/]*$/, "/", linkdir)) linkdir = ""
	# Vendored upstream copies link into the site or repository they came from,
	# never into this tree, and only one file of each was copied. The header
	# records the measurement behind treating that as unsatisfiable rather than
	# as 507 defects. Pinned citations in these same files stay checked.
	vendored = (FILENAME ~ /^docs\/research\/[^\/]+\/sources\//)
}

{
	if (FNR == 1 && $0 == "---") { in_frontmatter = 1; next }

	if (in_frontmatter) {
		# `nextfile` abandons a terminal ticket at its frontmatter instead of
		# reading its body: 1023 of the 1259 files here are terminal, and they
		# are most of the ~10 MB. The report prints both numbers, so a reader
		# never has to trust this one.
		if ($0 == "---") { in_frontmatter = 0; decide(); if (skip_file) nextfile; next }
		if ($0 ~ /^status:[ \t]*/) {
			status = $0
			sub(/^status:[ \t]*/, "", status)
			gsub(/[ \t\r]/, "", status)
		}
		# A document carries its state in a kind-specific facet rather than in
		# one field named `status`, so the key is matched by shape and only the
		# retiring value is acted on. Seeing any facet at all is recorded
		# separately, because a document that has none is a different population
		# from one that has a live status, and the census says which.
		if ($0 ~ /^(disposition|[a-z_]*status):[ \t]*"?[a-z-]+"?[ \t\r]*$/) {
			doc_status_seen = 1
			if ($0 ~ /^(disposition|[a-z_]*status):[ \t]*"?superseded"?[ \t\r]*$/)
				doc_superseded = 1
		}
		next
	}

	if (!decided) decide()
	if (skip_file) nextfile

	# A fence toggles regardless of span state; fenced blocks are transcripts
	# and command output, not citations.
	if ($0 ~ /^[ \t]*```/) { in_fence = !in_fence; in_span = 0; span = ""; next }
	if (in_fence) next

	# A reference definition carries its target on its own line rather than
	# inside `](...)`, and its label may itself hold a code span
	# (``[`Literal::byte_string`]: ...``), so it is read from the raw line before
	# the span walk. `[^label]:` is a footnote, not a definition.
	if ($0 ~ /^\[[^^][^]]*\]:[ \t]*[^ \t]/) {
		refdest = $0
		sub(/^\[[^]]*\]:[ \t]*/, "", refdest)
		sub(/[ \t].*$/, "", refdest)
		link(refdest)
	}

	scan_line($0)
}

# A BEGIN abort still runs END, so the report would print a census over an
# empty run and exit 1, hiding both the message and the exit status that says
# the run could not be trusted rather than that it found defects.
END { if (aborted) exit 2; end_file(); report() }

function decide(   parent) {
	decided = 1
	# The fixture holds no status, which is the point: a ticket transition has
	# nothing to act on, so it cannot remove the only guaranteed instance of
	# each citation form. It is never terminal and never skipped.
	if (role == "fixture") return

	# A document is retired by `superseded` and by nothing else -- the header
	# states which neighbouring values were considered and why each is still
	# live. A document with no status facet at all is checked rather than
	# skipped, and counted separately so that population stays visible. An
	# absent status is a property of the document kind or of its being a
	# verbatim upstream copy, never a statement that the record is retired, so
	# it must not read as a licence to skip; the header enumerates both sets.
	#
	# The repository-root documents share this branch rather than carrying a
	# parallel one, because it already answers the question they ask. README.md
	# is a `tiler-doc/v1` portal whose kind requires no status facet; AGENTS.md
	# and CLAUDE.md carry no frontmatter at all and reach the same conclusion by
	# a different road. Only the counters differ, and they are keyed by role so
	# the census and the floors keep the two populations apart.
	# Spike records share this branch too. `spikes/README.md` is a `tiler-doc/v1`
	# portal and the records carry research frontmatter, so `superseded` retires
	# one exactly as it retires any other document, and a record with no status
	# facet is checked rather than skipped. Only the counters differ, and they are
	# keyed by role so the census and the floors keep the populations apart.
	if (role == "doc" || role == "root" || role == "spike") {
		if (doc_superseded) {
			skip_file = 1
			files_terminal[role]++
			return
		}
		if (!doc_status_seen) no_status[role]++
		files_live[role]++
		return
	}

	if (role == "comment") {
		parent = ticket
		sub(/\.comments\/.*$/, ".md", parent)
		if (!(parent in status_of)) {
			printf "FAIL  %s\n        parent ticket %s was not read; cannot decide whether to check this comment\n", ticket, parent
			failures++
			skip_file = 1
			return
		}
		status = status_of[parent]
	} else {
		if (status == "") {
			printf "FAIL  %s\n        no `status` in frontmatter; cannot decide whether to check it\n", ticket
			failures++
			skip_file = 1
			return
		}
		status_of[ticket] = status
	}

	if (status in is_terminal) {
		skip_file = 1
		files_terminal[role]++
		return
	}
	files_live[role]++
}

# Walk one line, toggling in/out of inline code spans. Spans are handed to
# classify() as they close, so a span that wraps across lines is assembled
# rather than lost -- non-fence lines in tickets/ routinely end mid-span, and a
# line-oriented matcher silently misses every citation inside them. The report
# line named `spanned` counts how many citations this recovered, and the floor
# under it makes a recovery of zero fail instead of reading as a clean run.
function scan_line(line,   n, parts, i) {
	n = split(line, parts, "`")
	for (i = 1; i <= n; i++) {
		if (in_span) span = span parts[i]
		else scan_links(parts[i])
		if (i < n) {
			if (in_span) { classify(span, span_wrapped); span = ""; in_span = 0 }
			else { in_span = 1; span = ""; span_wrapped = 0 }
		}
	}
	# The newline inside a wrapped span reads as a space, as it does in
	# Markdown. The flag rides along so the assembly path can be counted and
	# floored: it is the one branch here whose failure is pure silence.
	if (in_span) { span = span " "; span_wrapped = 1 }
}

# Pull every link target out of one stretch of prose. The match is on
# `](target)` rather than on the whole `[text](target)`, so a code span inside
# the link text -- which the caller has already split this string on -- cannot
# hide the link. Measured 2026-08-08, no line in either corpus ends inside a
# `](`, so a target is always complete on the line that opens it.
function scan_links(s,   p, dest) {
	while ((p = index(s, "](")) > 0) {
		s = substr(s, p + 2)
		p = index(s, ")")
		if (p == 0) return
		dest = substr(s, 1, p - 1)
		s = substr(s, p + 1)
		link(dest)
	}
}

function link_fail(dest, msg) {
	link_failures++
	printf "FAIL  %s\n        link: [...](%s)\n        %s\n", ticket, dest, msg
}

# Resolve one link target against this tree. Every branch that declines to
# resolve feeds a counter that report() prints, so an exclusion is a number
# rather than a silence; the header argues each one.
function link(dest,   target, resolved, n, segs, out, i, k) {
	# An empty `]()` and a target carrying whitespace are malformed markdown
	# rather than broken links; the header names what the pattern otherwise
	# catches in a vendored specification. Counted since 2026-08-19, and counted
	# here rather than after the vendored test below on purpose: this asks whether
	# the thing is a link target at all, which is prior to asking where the file
	# carrying it lives. The header reconciles the vendored count with the six
	# this takes from it.
	if (dest == "" || dest ~ /[ \t]/) { link_malformed++; return }

	if (dest ~ /^[A-Za-z][A-Za-z0-9+.-]*:\/\// || dest ~ /^mailto:/ || dest ~ /^tel:/) {
		link_external++
		return
	}
	if (dest ~ /^#/) { link_selfanchor++; return }
	if (vendored) { link_vendored++; return }

	target = dest
	if (sub(/#.*$/, "", target)) link_fragment++
	if (target == "") { link_selfanchor++; return }

	link_checked++; link_ck[role]++

	if (target ~ /^\//) {
		link_fail(dest, "site-absolute target, but nothing in this tree is served from a web root")
		return
	}

	# Join to the linking directory and collapse `.` and `..`. An empty segment
	# absorbs a doubled or trailing slash, so `research/numerics/` names the
	# directory rather than an empty child of it.
	n = split(linkdir target, segs, "/")
	k = 0
	for (i = 1; i <= n; i++) {
		if (segs[i] == "" || segs[i] == ".") continue
		if (segs[i] == "..") {
			if (k == 0) {
				link_fail(dest, "walks above the repository root")
				return
			}
			k--
			continue
		}
		out[++k] = segs[i]
	}
	# Everything cancelled: the target is the repository root, which is there.
	if (k == 0) return
	resolved = out[1]
	for (i = 2; i <= k; i++) resolved = resolved "/" out[i]

	if (tracked[resolved] || treedir[resolved]) return
	# Path-shaped and untracked: accept it if it is on disk, so a document
	# created and not yet staged does not fail the run, and count it so the fact
	# that a reader with a clone cannot follow it stays visible. The shape test
	# keeps exists() away from directories, which it cannot read.
	if (resolved ~ /\.[A-Za-z0-9]+$/ && exists(resolved)) { link_untracked++; return }
	link_fail(dest, "no tracked file or directory at " resolved)
}

function end_file() {
	if (ticket == "") return
	if (in_span) {
		unterminated++
		unterminated_files = unterminated_files "\n        " ticket
	}
	ticket = ""
}

function classify(t, wrapped,   path, pin, anchor, form, ln, lo, hi, resolved, hay, lead, has_dir) {
	sub(/^[ \t]+/, "", t)
	sub(/[ \t]+$/, "", t)
	if (t == "") return

	path = ""; pin = ""; anchor = ""; form = ""

	if (match(t, "^" PATHRE_LOOSE ":[0-9]+(-[0-9]+)?[ \t]+\"[^\"]+\"$")) {
		path = t; sub(/[ \t]+".*$/, "", path)
		anchor = t; sub(/^[^"]*"/, "", anchor); sub(/"$/, "", anchor)
		pin = path; sub(/^.*:/, "", pin)
		sub(/:[0-9]+(-[0-9]+)?$/, "", path)
		form = "both"
	} else if (match(t, "^" PATHRE_LOOSE "[ \t]+\"[^\"]+\"$")) {
		path = t; sub(/[ \t]+".*$/, "", path)
		anchor = t; sub(/^[^"]*"/, "", anchor); sub(/"$/, "", anchor)
		form = "anchor"
	} else if (match(t, "^" PATHRE_LOOSE ":[0-9]+(-[0-9]+)?$")) {
		path = t; sub(/:[0-9]+(-[0-9]+)?$/, "", path)
		pin = t; sub(/^.*:/, "", pin)
		form = "line"
	} else if (match(t, "^" PATHRE "$")) {
		bare_paths++
		return
	} else {
		return
	}

	if (!qualifies(path)) return

	# A spike record pins lines into the tree at the dated base its own record
	# names, not at the tip, so demanding one resolve here is the unsatisfiable
	# condition the bare-path rule names -- and a landing in crates/ would redden
	# this gate through exploratory material. Counted rather than dropped, on its
	# own census line, so the exclusion is a number rather than a silence.
	#
	# Declined ahead of the form counters deliberately. Those counters feed the
	# form floors, whose question is whether the matcher still recognises a form
	# in the corpus it actually checks; a citation this run will never resolve
	# must not be what keeps one of them off zero. The fixture guarantees an
	# instance of every form regardless, so nothing is lost by not counting these.
	if (role == "spike") {
		spike_declined++
		if (verbose) printf "SKIP  %s: `%s` (spike record, pinned to its own base)\n", ticket, t
		return
	}

	if (form == "both") cit_both++
	else if (form == "anchor") cit_anchor++
	else cit_line++
	if (wrapped) spanned++

	resolved = path
	if (!exists(path)) {
		# Both skips below say the citation belongs to another project, and both
		# decide that from the leading component. A path with no directory
		# component is never either one: a bare filename is the shorthand this
		# repository uses for its own files, so one that resolves nowhere is
		# drift and has to fail.
		lead = path
		has_dir = sub(/\/.*$/, "", lead)

		if (has_dir && !(lead in component) && lead ~ /^[A-Za-z0-9_-]+-[0-9]+\.[0-9]+\.[0-9]+$/) {
			# A dependency source pinned by version, e.g.
			# `objc2-metal-0.3.2/src/generated/MTLDevice.rs:238`. It names a
			# real revision of a real crate that simply is not in this tree,
			# so it is outside what a working-tree check can decide.
			#
			# The component test beside the shape test is what keeps this from
			# being the fail-open hole the header describes. Vendored sources
			# here carry version-pinned directory names of their own, five of
			# them under docs/research/numerics/sources/, so on the shape alone
			# a partial citation rooted in one of those would be skipped while
			# the file it names sits in the index.
			external_pinned++
			if (verbose) printf "SKIP  %s: `%s` (external crate source)\n", ticket, t
			return
		}
		if (suffix_count[path] == 1) {
			# One tracked file ends with this suffix, which is stable under
			# deletion -- unless the suffix is on the ledger, which records that it
			# once named several and that a twin has since gone. Resolving here
			# would silently point the citation at whatever survived; the section
			# in the header carries the case this was filed from.
			if (path in was_ambiguous) {
				checked++; cit_checked[role]++
				ledger_collapsed++
				fail(t, sprintf("%s is on the retired-ambiguity ledger in check-citations.sh, and exactly one tracked file ends with it now: %s. A twin was deleted, so resolving would point this citation at the survivor rather than at the file it was written about. Re-read which file the claim is about and pin a path long enough to be unique on its own.", path, suffix_path[path]))
				return
			}
			resolved = suffix_path[path]
			partial_resolved++
		} else if (suffix_count[path] > 1) {
			# Several files end with this suffix. The citation is genuinely
			# ambiguous rather than absent; guessing a path would invent a
			# failure or hide one. Skipping is safe only while the ambiguity is on
			# record: an unledgered one has just formed, so this citation stopped
			# being checked without saying so, and it is one deletion away from the
			# branch above with nothing recorded to catch it.
			if (!(path in was_ambiguous)) {
				checked++; cit_checked[role]++
				ledger_stale++
				fail(t, sprintf("%s matches %d tracked files but is absent from the retired-ambiguity ledger in check-citations.sh, so it turned ambiguous after that ledger was written and this citation has silently stopped being checked. Pin a path long enough to be unique on its own, or add the suffix to the ledger so a later deletion cannot repoint it.", path, suffix_count[path]))
				return
			}
			ambiguous++
			ledger_matched++
			if (verbose) printf "SKIP  %s: `%s` (ambiguous, %d candidates, on the ledger)\n", ticket, t, suffix_count[path]
			return
		} else {
			# Nothing here is or ends with this path. Before calling that a
			# broken citation, ask whether the path was ever about this tree.
			# `docs/**` cites upstream sources the way their own repositories
			# spell them -- `candle-core/src/metal_backend/device.rs:101`, with
			# the revision named in the prose beside it -- and demanding those
			# resolve against Tiler is the unsatisfiable kind of condition, not
			# a caught defect. It is the same category the version-pinned skip
			# above already carries, reached by a different spelling.
			#
			# The answer comes from the recorded list of upstream roots and from
			# nothing else. Reading it off the tree instead -- calling a path
			# upstream when no tracked path carries its leading component --
			# fails open on the one event that breaks every citation under a
			# directory at once, which is deleting the directory; the header
			# carries the measurement. A listed root that is also a tracked
			# component aborts the run in BEGIN, so the list alone is enough
			# here and this skip can never be wider than the old test was.
			if (has_dir && (lead in upstream_root)) {
				external_rooted++
				if (verbose) printf "SKIP  %s: `%s` (rooted in %s, a recorded upstream project)\n", ticket, t, lead
				return
			}
			checked++; cit_checked[role]++
			if (has_dir)
				fail(t, sprintf("no file in the tree is or ends with %s, and %s is not a recorded upstream root in check-citations.sh. If this names a file in this repository the citation has drifted, so re-read the source at your own base and pin a path long enough to be unique on its own. If it names another project, add %s to that list so the skip is recorded rather than inferred from whichever directories happen to exist today.", path, lead, lead))
			else
				fail(t, "no file in the tree is or ends with " path)
			return
		}
	}

	checked++; cit_checked[role]++

	if (pin != "") {
		lo = pin; hi = pin
		if (pin ~ /-/) { sub(/-.*$/, "", lo); sub(/^.*-/, "", hi) }
		lo += 0; hi += 0
		ln = linecount(resolved)
		if (lo < 1 || hi < lo)
			fail(t, "line range is not ascending or starts below 1")
		else if (hi > ln)
			fail(t, sprintf("line %d is past end of file: %s has %d lines", hi, resolved, ln))
	}

	if (anchor != "") {
		hay = slurp(resolved)
		if (index(hay, anchor) == 0) {
			# Fall back to a whitespace-insensitive comparison so an anchor
			# still matches a construct that wraps in the source. The standing
			# case is `#[allow(unsafe_code,`: all four real attributes in this
			# workspace wrap after the `(`, which is why a plain grep for it
			# finds only a doc comment and none of them.
			if (index(squeeze(hay), squeeze(anchor)) == 0)
				fail(t, "anchor occurs nowhere in " resolved)
			else
				anchor_wrapped++
		}
	}
}

# One floor per corpus population, per checked property. The fixture needs none:
# it is floored six times over by the per-form floors below, each of which names
# a fixture instance that should have fed it. A corpus is different -- every
# citation and every link in it is contingent, so the only thing that says the
# population was reached at all is that something in it was checked.
function population_floor(n, name, unit, hint) {
	if (n + 0 > 0) return 0
	printf "\nEMPTY  the %s population contributed 0 checked %s, so nothing in it was verified.\n", name, unit
	printf "       %s\n", hint
	return 1
}

# One population floor. A form parsed zero times is a branch this run never
# executed, and a branch that never executes cannot say no -- so it reports
# clean whatever the tree looks like. `example` names the fixture citation that
# should have fed this counter, because the useful question on a zero is "which
# guaranteed instance stopped being recognised", not "is the corpus thin".
function form_floor(n, form, example) {
	if (n + 0 > 0) return 0
	printf "\nUNEXERCISED  %s: parsed 0 times, so nothing exercised that path.\n", form
	printf "             The built-in fixture carries %s, which should have fed it.\n", example
	return 1
}

# A floor on how many files a population reached, rather than on what they
# contained. The repository-root corpus carries no pinned citation at all and
# only seven links, so what it contributes cannot be the thing that proves it was
# read; the file count can. A glob that has stopped matching reads zero files and
# is otherwise indistinguishable from a clean run.
function count_floor(n, floor, name, unit, hint) {
	if (n + 0 >= floor) return 0
	printf "\nSHORT  the %s population reached %d %s, below its floor of %d.\n", name, n + 0, unit, floor
	printf "       %s\n", hint
	return 1
}

function report(   starved, empty, live) {
	live = files_live["ticket"] + files_live["comment"] + files_live["doc"] + files_live["root"] + files_live["spike"]
	printf "\ncitations: %d pinned citation(s) resolved across %d live ticket/comment/document file(s), the repository-root documents and the spike records among them, and the built-in fixture\n", checked + 0, live + 0
	# Each corpus is reported on its own line, and floored on its own counts
	# below, so a population that stopped being reached cannot ride another one
	# to a green run.
	printf "  tickets      %d citation(s) from %d open file(s) of %d read (%d ticket, %d comment), %d skipped as terminal (%s)\n", \
		cit_checked["ticket"] + cit_checked["comment"], files_live["ticket"] + files_live["comment"], \
		files_read["ticket"] + files_read["comment"], files_read["ticket"] + 0, files_read["comment"] + 0, \
		files_terminal["ticket"] + files_terminal["comment"], terminal
	printf "  docs         %d citation(s) from %d live file(s) of %d read, %d skipped as superseded, %d carrying no status facet\n", \
		cit_checked["doc"] + 0, files_live["doc"] + 0, files_read["doc"] + 0, files_terminal["doc"] + 0, no_status["doc"] + 0
	printf "  root         %d citation(s) from %d live file(s) of %d read against a floor of %d, %d skipped as superseded, %d carrying no status facet\n", \
		cit_checked["root"] + 0, files_live["root"] + 0, files_read["root"] + 0, ROOT_FLOOR, files_terminal["root"] + 0, no_status["root"] + 0
	# The spike corpus resolves no citation by construction, so its number here
	# is the count declined rather than the count checked, and it is printed on
	# its own line so nobody reads the zero-shaped population as an unread one.
	printf "  spikes       %d pinned citation(s) DECLINED (pinned to the base each record names, never to the tip) from %d live file(s) of %d read against a floor of %d, %d skipped as superseded\n", \
		spike_declined + 0, files_live["spike"] + 0, files_read["spike"] + 0, spikes_tracked + 0, files_terminal["spike"] + 0
	printf "  comments     %d checked, inheriting the status of their parent ticket\n", files_live["comment"] + 0
	printf "  fixture      %d citation(s) from %s, which holds no status for a ticket transition to change\n", \
		cit_checked["fixture"] + 0, FIXTURE_LABEL
	printf "  forms        %d line-only, %d anchor-only, %d line+anchor\n", \
		cit_line + 0, cit_anchor + 0, cit_both + 0
	# The two "belongs to another project" skips are counted on their own
	# numbers. They shared one counter under a line that named only the second
	# until 2026-08-19, and the sum was read off as the size of that branch into
	# a ticket Fact -- a census line that adds two exclusions under one name is
	# a silence in the same way an uncounted exclusion is.
	printf "  partial path %d resolved by unique suffix, %d skipped as ambiguous, %d skipped as a version-pinned dependency source, %d skipped as rooted in a recorded upstream project\n", \
		partial_resolved + 0, ambiguous + 0, external_pinned + 0, external_rooted + 0
	printf "  upstream     %d recorded upstream root(s), none of which may be a component of any tracked path\n", upstream_entries + 0
	printf "  ambiguity    %d ledger entry(s) against a floor of %d, %d citation(s) matched one, %d collapsed to a survivor, %d ambiguous off the ledger\n", \
		ledger_entries + 0, LEDGER_FLOOR, ledger_matched + 0, ledger_collapsed + 0, ledger_stale + 0
	# Both citation-side exclusions, on one line and on separate numbers. The
	# second was uncounted until 2026-08-19: a span dropped by qualifies() landed
	# in no census line at all, which is the silence every other number here
	# exists to prevent.
	printf "  not checked  %d bare path mention(s) carrying no line or anchor, %d pinned span(s) over %d distinct extensionless name(s) no tracked path ends with\n", \
		bare_paths + 0, unqualified + 0, unqualified_distinct + 0
	# Printed unconditionally, both of them: these two counters are floored
	# below, so a zero is a failure and must be visible rather than omitted.
	printf "  wrapped      %d anchor(s) matched only after collapsing whitespace\n", anchor_wrapped + 0
	printf "  spanned      %d citation(s) assembled across a line break in prose\n", spanned + 0
	if (unterminated)
		printf "  parse warn   %d file(s) ended inside an unclosed code span, so citations in them may have been missed:%s\n", \
			unterminated, unterminated_files

	# The links are a second property over the same population and are reported
	# on their own block, floored on their own counts, and failed on their own
	# message. A citation says a file contains some text; a link says a reader
	# who follows it arrives somewhere. Neither verdict stands in for the other.
	printf "\nlinks: %d local markdown link(s) resolved across the same population\n", link_checked + 0
	printf "  tickets      %d link(s) from the open ticket and comment files above\n", \
		link_ck["ticket"] + link_ck["comment"] + 0
	printf "  docs         %d link(s) from the live document files above\n", link_ck["doc"] + 0
	printf "  root         %d link(s) from the repository-root document files above\n", link_ck["root"] + 0
	printf "  spikes       %d link(s) from the live spike record files above, which is the one property checked in that corpus\n", link_ck["spike"] + 0
	printf "  fixture      %d link(s) from %s\n", link_ck["fixture"] + 0, FIXTURE_LABEL
	printf "  not resolved %d external (scheme://, mailto:, tel:), %d same-document heading anchor(s), %d in vendored upstream sources under docs/research/*/sources/, %d malformed (empty target, or whitespace inside one)\n", \
		link_external + 0, link_selfanchor + 0, link_vendored + 0, link_malformed + 0
	printf "  fragments    %d resolved link(s) carried a #heading into another document; the path was resolved and the anchor deliberately was not\n", \
		link_fragment + 0
	printf "  untracked    %d resolved on the filesystem only and are absent from the index, so a reader with a clone cannot follow them\n", \
		link_untracked + 0

	if (checked + 0 == 0) {
		printf "\ncheck-citations: parsed ZERO citations. A run that examines nothing cannot report a clean result -- the matcher has stopped reaching its subject.\n"
		exit 1
	}

	empty = population_floor(cit_checked["ticket"] + cit_checked["comment"], "tickets/**", "citation(s)", \
		"An open ticket citing a line or an anchor is the ordinary case here; zero means the glob, the frontmatter reader, or the terminal-state skip stopped reaching them.")
	empty += population_floor(cit_checked["doc"], "docs/**", "citation(s)", \
		"ADRs and contracts pin lines into the tree by the hundred; zero means the find, the superseded skip, or role_of stopped reaching them.")
	empty += population_floor(link_ck["ticket"] + link_ck["comment"], "tickets/** markdown link", "link(s)", \
		"An open ticket links to its siblings and to the documents it cites; zero means scan_links stopped reaching the prose, or every target was classified away as external, anchored, or vendored.")
	empty += population_floor(link_ck["doc"], "docs/** markdown link", "link(s)", \
		"Every catalog, index, and cross-reference in docs/ is navigated by link, in the hundreds; zero means the population the entry points live in went unresolved -- which is exactly the state this check was added to end.")
	# Two floors on the root population, guarding two different failures. The
	# file count says the *.md glob still matches and role_of() still routes
	# them; the link count says their prose was actually walked. Neither implies
	# the other: three files read with zero links resolved is a matcher that
	# stopped reaching them, and it would otherwise print exactly as green as a
	# clean run. There is deliberately no citation floor here -- this population
	# carries no pinned citation at all, so one would fail on a correct tree.
	empty += count_floor(files_read["root"], ROOT_FLOOR, "repository-root", "file(s)", \
		"README.md, AGENTS.md, and CLAUDE.md are tracked at the root and none of them is going away; fewer than that means the *.md glob or role_of stopped reaching them, and a population that reads zero files prints the same green as one that read all three.")
	# The ledger is the retired-ambiguity memory, and a truncated one is a
	# weaker check that prints exactly the same green as a whole one: every
	# entry lost turns a recorded ambiguity back into a suffix that will
	# resolve against a survivor the day its family collapses. Floored on the
	# entry count for the same reason the repository-root population is -- what
	# it contributes on a clean tree is zero failures either way.
	empty += count_floor(ledger_entries, LEDGER_FLOOR, "retired-ambiguity ledger", "entry(s)", \
		"Every entry records a suffix observed ambiguous while a live citation rested on it, and entries are added rather than pruned; fewer than the floor means the heredoc was truncated or the reader stopped parsing it, and a suffix off the ledger resolves silently against whichever twin outlives the others.")
	# The ticket corpus is reached by two fixed-depth globs -- `tickets/*.md` and
	# `tickets/*.comments/*.md` -- which is exactly the shape the docs population
	# uses `find` to avoid, because it drops a whole subtree in silence the day
	# someone nests one level deeper. This is the floor that makes that loud, and
	# it is sized from the index rather than by hand: a hand-written number is
	# satisfied by a glob that has quietly stopped covering its domain, which is
	# the one failure it would exist to catch. Reading *more* than the index holds
	# is deliberately not floored -- an unstaged ticket is matched by the glob and
	# is not a defect.
	empty += count_floor(files_read["ticket"] + files_read["comment"], tickets_tracked, "tickets/**", "file(s)", \
		"Every tracked markdown file under tickets/ must be reached by tickets/*.md or tickets/*.comments/*.md; a shortfall means one is nested deeper than either glob reaches and is being passed over in silence. Reach it with the same find the docs population uses, or return the file to a depth the globs cover.")
	empty += population_floor(link_ck["root"], "repository-root markdown link", "link(s)", \
		"README.md is six links of route into docs/ and spikes/, and AGENTS.md links the ADR governing every unsafe site in the workspace; zero means scan_links stopped reaching them, or the entry points stopped pointing anywhere at all.")
	# Two floors on the spike corpus and deliberately not a third. The file count
	# is sized from the index, like the ticket one and for the same reason, and it
	# is what says the find still reaches a corpus that nests six components deep.
	# The link count is what says the prose was walked: 68 files read with zero
	# links resolved is a matcher that stopped reaching them and prints exactly as
	# green as a clean run. There is no citation floor here, because this
	# population declines every citation it finds -- one would fail on a correct
	# tree, which is the reason the repository-root population carries none either.
	empty += count_floor(files_read["spike"], spikes_tracked, "spikes/**", "file(s)", \
		"Every tracked markdown file under spikes/ must be reached by the recursive find; a shortfall means one is nested where the find no longer looks and is being passed over in silence.")
	# Floored at one link per live record rather than at "more than zero", and the
	# difference was measured rather than assumed. Disabling scan_links for this
	# role entirely left the counter reading 1, not 0, because a reference
	# definition reaches link() on its own path -- and exactly one exists in this
	# corpus, in spikes/cache/README.md. A greater-than-zero floor is therefore
	# satisfied by a single stray line while all 68 records go unwalked, which is
	# the silence it would exist to end. The bar is derived from the corpus rather
	# than written down, so it cannot go stale as records are added.
	empty += count_floor(link_ck["spike"], files_live["spike"], "spikes/** markdown link", "link(s)", \
		"A spike record links to the document it supports, to its own results, and to the catalog row that reaches it -- 590 links across 68 records at 77cd0104, so fewer than one per live record means scan_links stopped walking the prose of the corpus whose currency convention is itself navigated by link.")
	if (empty > 0)
		printf "\ncheck-citations: %d population floor(s) went unmet -- a population contributed ZERO checked citations or links, read fewer files than it must, or holds fewer recorded entries than it must. Another population passing says nothing about this one; separate counts exist so that none can stand in for the others.\n", empty

	starved = form_floor(cit_line, "the line-only form (`path:LINE`)", "`AGENTS.md:1`")
	starved += form_floor(cit_anchor, "the anchor-only form (`path \"anchor\"`)", "`Makefile \"check: citations fmt build lint test\"`")
	starved += form_floor(cit_both, "the line+anchor form (`path:LINE \"anchor\"`)", "`ticketsplease.toml:1 \"schema_version = 1\"`")
	starved += form_floor(anchor_wrapped, "the whitespace-collapsing anchor fallback", "`AGENTS.md \"origin/main...main # 0 0\"`, whose subject wraps in the source")
	starved += form_floor(spanned, "code-span assembly across a line break", "a citation whose backticks straddle two lines of prose")
	starved += form_floor(link_ck["fixture"], "local markdown link resolution (`[text](target)`)", "`[the agent guide](AGENTS.md)`")
	starved += form_floor(ledger_matched, "the retired-ambiguity ledger lookup", "`lib.rs:1`, a suffix 44 tracked files end with")

	if (starved > 0)
		printf "\ncheck-citations: %d citation or link form(s) were parsed ZERO times. The fixture supplies one of each, so this is the matcher losing a form, not a corpus drifting -- and an unexercised branch reports no failures no matter what the tree contains.\n", starved
	if (failures > 0) {
		printf "\ncheck-citations: %d citation(s) do not resolve against this tree.\n", failures
		printf "Repair the citation by re-reading the source at your own base. Prefer a quoted anchor over a bare line number: `path.rs \"distinctive phrase\"`.\n"
	}
	if (link_failures > 0) {
		printf "\ncheck-citations: %d markdown link(s) do not resolve against this tree.\n", link_failures
		printf "Repair the link by naming a target that exists, relative to the directory of the file carrying it. A link is a promise that a reader who follows it arrives somewhere.\n"
	}
	if (empty > 0 || starved > 0 || failures > 0 || link_failures > 0) exit 1

	printf "\ncheck-citations: every pinned citation and every local markdown link resolves. This says they point somewhere; it does NOT say the tickets and documents around them are true.\n"
}
' "$@"
