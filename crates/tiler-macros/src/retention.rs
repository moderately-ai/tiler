//! What an expansion emits from the debug retention its entry carries.
//!
//! `tiler_cache::expansion::DebugRetention` is text a producer kept beside a
//! published entry, and `crates/tiler-build/src/metal_cache.rs` states one on
//! every publication — "**Always stated, never discovered.**" That decision is
//! the producer's and this module does not revisit it: nothing here selects
//! retention, reads an environment variable, or consults a build profile. What
//! this module owns is the other half, which nothing owned before: a retention
//! now arrives populated on every delivering expansion, and this is what the
//! frontend does with it.
//!
//! # Why a note, and never a `compile_error!`
//!
//! The producer already settled the principle in
//! [`DebugRetention`]'s own vocabulary, and `stage_retention` states it: "Failing
//! a successful compilation over a diagnostic would make a warning a compilation
//! input in the only way that actually matters." A retention exists *because* the
//! compilation succeeded — a failing stage takes `crate::aot::retained`'s
//! family-scoped `compile_error!` path instead and never reaches here. So the
//! only text this module can ever hold is text from a compilation that produced a
//! validated artifact the cache accepted. Reporting sits after that acceptance
//! and before payload-cardinality checks, delivery-plan construction, token
//! emission, and `guarded_emission`, so a later frontend refusal can still
//! prevent embedding. A fatal diagnostic over the retained text would fail a
//! build whose artifact is already correct.
//!
//! That is the same disposition [`crate::preflight`] and [`crate::eviction`]
//! reached for their own non-fatal facts, and the reasons compose rather than
//! merely rhyme: each of the three describes something that costs a consumer
//! nothing about the artifact they receive.
//!
//! # Why the note does not land on the invocation's span
//!
//! A span-attributed warning says "your region caused this", and
//! [`crate::aot`] establishes at length that it did not: "**No region text can
//! make `metal` reject the emitted source.** Everything a consumer writes is a
//! shape, an operation, a contract name, or a family name; none of it reaches the
//! MSL as a token." The same argument covers a *warning* exactly as it covers a
//! rejection — `tiler_metal` names every entry point, helper, allocation, and
//! buffer parameter from an identity digest, so a diagnostic about the emitted
//! translation unit names constructs no invocation wrote.
//!
//! A proc macro *can* emit a non-fatal spanned diagnostic on this toolchain:
//! `#![feature(proc_macro_diagnostic)]` with `Diagnostic::spanned(…,
//! Level::Warning, …).emit()` compiles under `nightly-2026-07-19` and renders a
//! warning on the call site without failing the build. It is declined for two
//! reasons rather than for availability. The first is the attribution above:
//! pointing at `tiler::tensor! { … }` would send a consumer to edit a region that
//! is not at fault. The second is that `Diagnostic::emit` can only run inside an
//! expanding macro and writes somewhere no test can read, which is the precise
//! defect [`crate`]'s own module documentation gives as the reason the `tokens`
//! module exists — "anything written directly against those types has diagnostics
//! no test can observe."
//!
//! So the note takes the shape `docs/integration/frontends.md` already specifies
//! and this crate already writes twice: a line on the expanding process's
//! standard error, attributed to the macro that wrote it, through an
//! [`io::Write`] seam that makes the message a value a test can assert on.
//!
//! # Why there is no once-per-process gate, unlike its two siblings
//!
//! [`crate::preflight`] and [`crate::eviction`] each claim a process-wide flag,
//! and copying that here would be the wrong half of the pattern. Their messages
//! are *process-scoped facts* — one resolved root, one environment variable — so
//! a second printing is the same sentence again and pure noise. A retention is a
//! *per-compilation* fact: two regions are two translation units with two
//! different tool outputs, and a gate would suppress the second region's
//! diagnostic on the strength of the first's having existed.
//!
//! Silence in the healthy case is what bounds the volume instead, and it is
//! structural rather than hoped for: [`spoken`] writes nothing unless a run
//! actually carries bytes, and a quiet Apple toolchain retains two empty runs. A
//! build whose compilations say nothing prints nothing, however many regions it
//! expands.
//!
//! # An empty retention and a quiet one are different things
//!
//! [`DebugRetention::is_empty`] is the wrong predicate here and the distinction
//! is load-bearing. The Metal producer names *every* stage of *every* delivery
//! position, and its documentation is explicit that "**A silent stage is retained
//! as an empty run**" — so a completely quiet compilation yields a retention of
//! two runs for which `is_empty` answers `false`. Gating on it would print a
//! header with nothing under it on every single delivering expansion.
//!
//! [`spoken`] therefore selects on [`RetainedText::is_empty`], run by run, and
//! keeps only the runs that have something to show.
//!
//! # Accepted caller-visible boundary
//!
//! Tom accepted this note on 2026-08-11 as an ungated, nonfatal, byte-faithful
//! read-back. It is `pub(crate)` and adds no public item. The tool's own bytes
//! are written exactly through [`SpokenRetention::write_to`]; provenance,
//! invalid-UTF-8 status, and truncation totals sit outside that run.
//! [`RetainedText`]'s `Display` remains the cache's public lossy view and is not
//! this path. The note names only the completed AOT/cache phase.

use std::io;

use tiler_cache::expansion::{DebugRetention, RetainedText};

/// The phase the note may claim, and nothing later.
///
/// Reporting sits in [`crate::aot::deliver`] after cache/artifact acceptance and
/// before payload-cardinality checks, delivery-plan construction, token
/// emission, and `guarded_emission`. `fmt::Display` cannot carry invalid UTF-8,
/// so this prose is written through [`SpokenRetention::write_to`] rather than
/// formatted as the tool run itself.
const PREAMBLE: &str = "the offline Metal toolchain wrote output while compiling this region's \
     artifact, and it is retained beside the cache entry this expansion resolved to. Offline \
     compilation plus cache/artifact acceptance succeeded — later frontend emission can still \
     refuse. This is what the tools said rather than a refusal. No text a region declares reaches \
     the emitted MSL, so this describes the source Tiler's own backend emitted rather than \
     anything this invocation can change";

/// The runs of one retention that actually have something to show.
///
/// Typed and non-erasing under ADR 0074 convention 1: the runs survive to the
/// message with their own labels, because "the toolchain said something" tells a
/// reader neither which stage spoke nor what it said — and because a retention
/// mixing a quiet stage with a speaking one must name only the second.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpokenRetention {
    /// Every run carrying bytes, in the order the producer named them.
    runs: Vec<RetainedText>,
}

impl SpokenRetention {
    /// Writes the note through the process-facing `io::Write` seam.
    ///
    /// Order is provenance, then the tool's own bytes, then typed metadata.
    /// `RetainedText::Display` is the cache's public lossy view and is not this
    /// path: it trims and substitutes, and `fmt::Write` cannot carry an invalid
    /// sequence even if it did not. Truncation and invalid-UTF-8 markers come
    /// from [`RetainedText::is_truncated`] and [`RetainedText::is_valid_utf8`],
    /// written after [`RetainedText::as_bytes`] so they are never inserted into
    /// the run they describe.
    fn write_to(&self, out: &mut impl io::Write) -> io::Result<()> {
        write!(out, "`tiler::tensor!`: {PREAMBLE}")?;
        for run in &self.runs {
            write!(out, "\n{}: ", run.label())?;
            out.write_all(run.as_bytes())?;
            if !run.is_valid_utf8() {
                write!(out, " [output was not valid UTF-8]")?;
            }
            if run.is_truncated() {
                write!(
                    out,
                    " [truncated: {} of {} bytes retained]",
                    run.as_bytes().len(),
                    run.total_bytes(),
                )?;
            }
        }
        writeln!(out)
    }
}

/// Reports whatever a resolved entry's retention has to show, on this process's
/// standard error.
///
/// Returns nothing, because there is nothing a caller may decide from it: the
/// expansion proceeds identically either way, exactly as [`crate::preflight`]'s
/// report does.
pub(crate) fn report_retained_output(retention: &DebugRetention) {
    let _ = reported_to(retention, &mut io::stderr());
}

/// [`report_retained_output`], writing to `out` rather than to the process.
///
/// The seam exists for [`crate::preflight`]'s and [`crate::eviction`]'s reason: a
/// check that could only observe the real standard error would be asserting on
/// the harness rather than on what a consumer reads. The value is returned for
/// the same reason, and production ignores it.
fn reported_to(retention: &DebugRetention, out: &mut impl io::Write) -> Option<SpokenRetention> {
    let spoken = spoken(retention)?;
    // Best effort. A closed or failing standard error is not a reason to fail an
    // expansion whose artifact is correct either way.
    let _ = spoken.write_to(out);
    Some(spoken)
}

/// Reads a retention as the runs that have something to show, or `None` when
/// none does.
///
/// `None` covers both a retention with no runs at all — an entry published
/// before any of this existed, which [`DebugRetention::is_empty`] answers for —
/// and the far commoner case of a healthy compilation whose every named stage
/// was silent. Neither is a fault and neither has anything to print.
fn spoken(retention: &DebugRetention) -> Option<SpokenRetention> {
    let runs: Vec<RetainedText> = retention
        .runs()
        .iter()
        .filter(|run| !run.is_empty())
        .cloned()
        .collect();
    (!runs.is_empty()).then_some(SpokenRetention { runs })
}

#[cfg(test)]
mod tests;
