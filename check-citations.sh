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
#     forever on a move. Measured over the 65 paths ever deleted here: one live
#     citation would newly fail, `payload.rs:289` in
#     `docs/research/extensions/backend-provider-composition.md`, whose only
#     historical twin is a deleted `prototypes/serial-sum-compile/src/payload.rs`
#     and which reading confirms correctly names `check_provenance` in
#     `crates/tiler-artifact/src/program/codec/payload.rs`. One invented failure,
#     no caught defect. The ledger records ambiguity this checker observed while
#     a live citation rested on it, never ambiguity inferred backwards from
#     history.
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
# into failures -- all 16 citations reaching the branch are rooted in
# `candle-core`, `candle-metal-kernels`, or `MacOSX26.5.sdk`. Second, a listed
# root that IS a tracked component would silence this tree rather than another
# one, so it aborts the run at exit 2 naming the collision instead of being
# quietly honoured.
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
#     An empty `]()` is likewise malformed rather than broken.
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
# THE THREE POPULATIONS, AND WHAT TERMINAL MEANS IN EACH
#
# `tickets/**`, `docs/**`, and the tracked markdown at the repository root are
# read and counted separately, and each carries its own floor, so none of them
# can collapse into another and read as a clean run on the strength of the ones
# that still work.
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
# reader. Every entry was observed ambiguous while a live citation rested on it,
# and the section above states what that buys and what it deliberately is not.
# Entries are added, never retro-derived and never pruned: the floor in report()
# is what makes a truncation loud rather than a quietly weaker check.
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
unset IFS
if [ "$#" -eq "$docs_before" ]; then
	printf 'check-citations: no document files matched docs/**/*.md.\n' >&2
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

# An extensionless, directory-less token is a citation only when a file of that
# exact name is really there. Without this, `ADR:87` -- which appears twenty
# times in tickets/ and names a decision record, not a file -- would be read as
# a path and fail forever.
function qualifies(p) {
	if (p ~ /\//) return 1
	if (p ~ /\.[A-Za-z0-9]+$/) return 1
	return exists(p)
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
	LEDGER_FLOOR = 41
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
		printf "check-citations: upstream root(s) recorded in check-citations.sh are also components of tracked paths, so citations under them would be skipped instead of checked:%s\n", collisions
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
	if (role == "doc" || role == "root") {
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
	# catches in a vendored specification.
	if (dest == "" || dest ~ /[ \t]/) return

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
	live = files_live["ticket"] + files_live["comment"] + files_live["doc"] + files_live["root"]
	printf "\ncitations: %d pinned citation(s) resolved across %d live ticket/comment/document file(s), the repository-root documents among them, and the built-in fixture\n", checked + 0, live + 0
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
	printf "  not checked  %d bare path mention(s) carrying no line or anchor\n", bare_paths + 0
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
	printf "  fixture      %d link(s) from %s\n", link_ck["fixture"] + 0, FIXTURE_LABEL
	printf "  not resolved %d external (scheme://, mailto:, tel:), %d same-document heading anchor(s), %d in vendored upstream sources under docs/research/*/sources/\n", \
		link_external + 0, link_selfanchor + 0, link_vendored + 0
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
	empty += population_floor(link_ck["root"], "repository-root markdown link", "link(s)", \
		"README.md is six links of route into docs/ and spikes/, and AGENTS.md links the ADR governing every unsafe site in the workspace; zero means scan_links stopped reaching them, or the entry points stopped pointing anywhere at all.")
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
