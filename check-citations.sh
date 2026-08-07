#!/bin/sh
# Resolve the pinned source citations in open tickets against this working tree.
#
# WHAT A GREEN RUN MEANS, AND WHAT IT DOES NOT
#
# Green means every pinned citation in an open ticket points *somewhere*: the
# file exists, the line is inside it, the quoted anchor occurs in it. That is
# all. It is emphatically **not** evidence that a ticket's Facts are true. A
# citation resolves perfectly and still supports a claim the code no longer
# makes -- that is what happened on 2026-08-07, when a claim about an
# obligation class named the right file and the right symbol and described
# behaviour the code does not have. This checker would have passed it, and it
# passes the deliberately wrong citation kept as a live fixture in
# `tickets/pin-ticket-source-citations-against-the-tree-they-name.md`.
#
# AGENTS.md carries the control that actually governs this: a ticket's stated
# Facts are stale until re-read at your own base, and a worker's first
# deliverable is a per-Fact verdict. This script is the cheap mechanical floor
# underneath that obligation, never a substitute for it.
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
# when the Python gate became the Makefile, still named in ten tickets that are
# accurately describing history. Demanding those resolve would be an
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
# Terminal tickets (`done`, `closed` -- read from `ticketsplease.toml`, not
# hardcoded here) are skipped. Their citations describe a tree at merge time
# and rot by design; rewriting a closed record to match today's line numbers
# would destroy the account of what was actually done. Open tickets are the
# ones a worker will follow into the code, which is exactly where a stale
# citation does its damage. A comment file inherits the status of its parent
# ticket, because it is part of the ticket a worker is told to read in full.
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
trap 'rm -f "$indexfile"' EXIT INT TERM
git ls-files >|"$indexfile"

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

awk -v terminal="$terminal" -v verbose="$verbose" -v indexfile="$indexfile" '
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
	}
	close(indexfile)

	# PATHRE is path-shaped on its face: it carries an extension. PATHRE_LOOSE
	# also admits an extensionless name so `Makefile:34` and
	# `docs/research/numerics/sources/jax-v0.11.0/LICENSE:5` stay reachable;
	# qualifies() is what stops that from swallowing prose.
	PATHRE = "[A-Za-z0-9_][A-Za-z0-9_./-]*\\.[A-Za-z0-9]+"
	PATHRE_LOOSE = "[A-Za-z0-9_][A-Za-z0-9_./-]*"
}

FNR == 1 {
	end_file()
	ticket = FILENAME
	files_read++
	if (ticket ~ /\.comments\//) comments_read++
	status = ""
	in_fence = 0
	in_frontmatter = 0
	decided = 0
	skip_file = 0
	in_span = 0
	span = ""
}

{
	if (FNR == 1 && $0 == "---") { in_frontmatter = 1; next }

	if (in_frontmatter) {
		# `nextfile` abandons a terminal ticket at its frontmatter instead of
		# reading its body: 1012 of the 1248 files here are terminal, and they
		# are most of the ~10 MB.
		if ($0 == "---") { in_frontmatter = 0; decide(); if (skip_file) nextfile; next }
		if ($0 ~ /^status:[ \t]*/) {
			status = $0
			sub(/^status:[ \t]*/, "", status)
			gsub(/[ \t\r]/, "", status)
		}
		next
	}

	if (!decided) decide()
	if (skip_file) nextfile

	# A fence toggles regardless of span state; fenced blocks are transcripts
	# and command output, not citations.
	if ($0 ~ /^[ \t]*```/) { in_fence = !in_fence; in_span = 0; span = ""; next }
	if (in_fence) next

	scan_line($0)
}

END { end_file(); report() }

function decide(   parent) {
	decided = 1
	if (ticket ~ /\.comments\//) {
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
		files_terminal++
		return
	}
	files_checked++
	if (ticket ~ /\.comments\//) comments_checked++
}

# Walk one line, toggling in/out of inline code spans. Spans are handed to
# classify() as they close, so a span that wraps across lines is assembled
# rather than lost -- 67 non-fence lines in tickets/ currently end mid-span,
# and a line-oriented matcher silently misses every citation inside them.
function scan_line(line,   n, parts, i) {
	n = split(line, parts, "`")
	for (i = 1; i <= n; i++) {
		if (in_span) span = span parts[i]
		if (i < n) {
			if (in_span) { classify(span); span = ""; in_span = 0 }
			else { in_span = 1; span = "" }
		}
	}
	# The newline inside a wrapped span reads as a space, as it does in
	# Markdown.
	if (in_span) span = span " "
}

function end_file() {
	if (ticket == "") return
	if (in_span) {
		unterminated++
		unterminated_files = unterminated_files "\n        " ticket
	}
	ticket = ""
}

function classify(t,   path, pin, anchor, form, ln, lo, hi, resolved, hay) {
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

	resolved = path
	if (!exists(path)) {
		if (path ~ /^[A-Za-z0-9_-]+-[0-9]+\.[0-9]+\.[0-9]+\//) {
			# A dependency source pinned by version, e.g.
			# `objc2-metal-0.3.2/src/generated/MTLDevice.rs:238`. It names a
			# real revision of a real crate that simply is not in this tree,
			# so it is outside what a working-tree check can decide.
			external++
			if (verbose) printf "SKIP  %s: `%s` (external crate source)\n", ticket, t
			return
		}
		if (suffix_count[path] == 1) {
			resolved = suffix_path[path]
			partial_resolved++
		} else if (suffix_count[path] > 1) {
			# Several files end with this suffix. The citation is genuinely
			# ambiguous rather than absent; guessing a path would invent a
			# failure or hide one.
			ambiguous++
			if (verbose) printf "SKIP  %s: `%s` (ambiguous, %d candidates)\n", ticket, t, suffix_count[path]
			return
		} else {
			checked++
			fail(t, "no file in the tree is or ends with " path)
			return
		}
	}

	checked++

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

function report() {
	printf "\ncitations: %d pinned citation(s) resolved across %d open ticket/comment file(s)\n", checked + 0, files_checked + 0
	printf "  population   %d file(s) read (%d ticket, %d comment), %d skipped as terminal (%s)\n", \
		files_read + 0, files_read - comments_read, comments_read + 0, files_terminal + 0, terminal
	printf "  comments     %d checked, inheriting the status of their parent ticket\n", comments_checked + 0
	printf "  forms        %d line-only, %d anchor-only, %d line+anchor\n", \
		cit_line + 0, cit_anchor + 0, cit_both + 0
	printf "  partial path %d resolved by unique suffix, %d skipped as ambiguous, %d skipped as external crate source\n", \
		partial_resolved + 0, ambiguous + 0, external + 0
	printf "  not checked  %d bare path mention(s) carrying no line or anchor\n", bare_paths + 0
	if (anchor_wrapped)
		printf "  wrapped      %d anchor(s) matched only after collapsing whitespace\n", anchor_wrapped
	if (unterminated)
		printf "  parse warn   %d file(s) ended inside an unclosed code span, so citations in them may have been missed:%s\n", \
			unterminated, unterminated_files

	if (checked + 0 == 0) {
		printf "\ncheck-citations: parsed ZERO citations. A run that examines nothing cannot report a clean result -- the matcher has stopped reaching its subject.\n"
		exit 1
	}
	if (failures > 0) {
		printf "\ncheck-citations: %d citation(s) do not resolve against this tree.\n", failures
		printf "Repair the citation by re-reading the source at your own base. Prefer a quoted anchor over a bare line number: `path.rs \"distinctive phrase\"`.\n"
		exit 1
	}
	printf "\ncheck-citations: every pinned citation resolves. This says the citations point somewhere; it does NOT say the tickets are true.\n"
}
' "$@"
