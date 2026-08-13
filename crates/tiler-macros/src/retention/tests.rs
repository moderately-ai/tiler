//! What a populated retention makes an expansion write, and what it does not.
//!
//! Every test writes to a stream it owns rather than to the process's standard
//! error, so what a consumer reads is a value that can be asserted on and no
//! test's outcome depends on whether another ran first.
//!
//! The retentions are built through `DebugRetention`'s own public constructors
//! rather than stated as bytes, so a bound or a label rule the cache changes
//! reaches these tests as a build or construction failure rather than as a stale
//! fixture that keeps passing.

use tiler_cache::expansion::{DebugRetention, MAX_RETAINED_RUN_BYTES};

use super::{SpokenRetention, reported_to, spoken};

/// The two runs a silent Metal compilation retains, exactly as
/// `crates/tiler-build/src/metal_cache.rs` states them.
///
/// The labels are that producer's `{BACKEND}.{delivery}.{stage.tool()}` shape.
/// They are restated here rather than imported because `tiler-build` publishes
/// no constructor for one, and the point of the fixture is the *shape* a quiet
/// compilation has — two named stages, no bytes — rather than the exact text.
fn quiet() -> DebugRetention {
    DebugRetention::none()
        .retaining("tiler.metal.0.metal", b"")
        .expect("a governed label and an empty run are retainable")
        .retaining("tiler.metal.0.metallib", b"")
        .expect("a governed label and an empty run are retainable")
}

/// A quiet compilation writes nothing, though its retention is not empty.
///
/// **This is the load-bearing negative and the reason `is_empty` is not the
/// predicate.** The Metal producer names every stage of every delivery position
/// and records a silent stage as an empty run, so `DebugRetention::is_empty`
/// answers `false` for a compilation that said absolutely nothing. An
/// implementation gated on it would print a header with nothing under it on
/// every delivering expansion in every build — which is the noise the eviction's
/// report disposition was already decided against.
///
/// The second assertion is what keeps this from passing vacuously: without it,
/// silence here would also be what an implementation that never printed anything
/// at all produces.
#[test]
fn a_quiet_compilation_writes_nothing_though_its_retention_is_not_empty() {
    let retention = quiet();
    assert!(
        !retention.is_empty(),
        "the fixture must reproduce the producer's shape, or this test states nothing",
    );
    assert_eq!(retention.runs().len(), 2, "one compilation is two stages");

    let mut written = Vec::new();
    assert_eq!(reported_to(&retention, &mut written), None);
    assert!(
        written.is_empty(),
        "a silent toolchain must write nothing to a consumer's build output: {}",
        String::from_utf8_lossy(&written),
    );
}

/// An entry that retained nothing at all writes nothing.
///
/// The absent-section case: an entry published by a build predating retention is
/// a hit with nothing to show, never a fault.
#[test]
fn a_retention_with_no_runs_writes_nothing() {
    let mut written = Vec::new();
    assert_eq!(reported_to(&DebugRetention::none(), &mut written), None);
    assert!(written.is_empty());
}

/// A stage that spoke is reported, with its label and its text, and the quiet
/// stage beside it is not named.
///
/// **This is the watched firing.** The second half is the half that fails
/// against an implementation which prints the whole retention: a reader sent to
/// look at `tiler.metal.0.metallib` because it was listed beside a real diagnostic
/// would be looking for a compiler that never spoke.
#[test]
fn a_speaking_stage_is_reported_and_a_silent_one_beside_it_is_not_named() {
    let retention = quiet()
        .retaining(
            "tiler.metal.1.metal",
            b"program_source:5:10: warning: unused variable 'x'",
        )
        .expect("a governed label and a real diagnostic are retainable");

    let mut written = Vec::new();
    let reported = reported_to(&retention, &mut written).expect("a speaking stage reports");
    assert_eq!(
        reported.runs.len(),
        1,
        "only the run that spoke: {reported:?}"
    );

    let message = String::from_utf8(written).expect("the message is text");
    for required in [
        "`tiler::tensor!`:",
        "tiler.metal.1.metal",
        "unused variable 'x'",
        "later frontend emission can still refuse",
        "rather than anything this invocation can change",
    ] {
        assert!(
            message.contains(required),
            "the message must name `{required}`: {message}",
        );
    }
    assert!(
        !message.contains("tiler.metal.0.metal"),
        "a stage that said nothing must not be named beside one that did: {message}",
    );
}

/// Every stage that spoke is named, not just the first.
///
/// A retention is a per-compilation fact covering several stages and several
/// delivery positions, so reporting one and dropping the rest would lose a
/// diagnostic that has no other reader.
#[test]
fn every_speaking_stage_is_named() {
    let retention = DebugRetention::none()
        .retaining("tiler.metal.0.metal", b"front end had something to say")
        .expect("retainable")
        .retaining("tiler.metal.0.metallib", b"linker had something to say")
        .expect("retainable");

    let mut written = Vec::new();
    let reported = reported_to(&retention, &mut written).expect("two speaking stages report");
    assert_eq!(reported.runs.len(), 2);

    let message = String::from_utf8(written).expect("the message is text");
    for required in [
        "tiler.metal.0.metal",
        "front end had something to say",
        "tiler.metal.0.metallib",
        "linker had something to say",
    ] {
        assert!(
            message.contains(required),
            "the message must name `{required}`: {message}",
        );
    }
}

/// A real Apple diagnostic spans several lines, and they survive verbatim.
///
/// The retention is captured byte-preserving so a reader can match it against a
/// direct `metal` invocation, and a renderer that collapsed or re-indented the
/// caret line would break exactly that. The check is on the tool's own text
/// rather than on the message as a whole, because the preamble is this crate's
/// prose and may be rewrapped freely.
#[test]
fn a_multi_line_tool_diagnostic_survives_verbatim() {
    let diagnostic =
        "program_source:5:10: warning: unused variable 'x'\n    float x = 1.0;\n          ^";
    let retention = DebugRetention::none()
        .retaining("tiler.metal.0.metal", diagnostic.as_bytes())
        .expect("retainable");

    let mut written = Vec::new();
    assert!(reported_to(&retention, &mut written).is_some());
    let message = String::from_utf8(written).expect("the message is text");

    assert!(
        message.contains(diagnostic),
        "the tool's own bytes must reach a reader unaltered: {message}",
    );
}

/// A run the bounds cut is reported as a prefix rather than as the whole.
///
/// `RetainedText`'s own renderer carries the marker, and this states that the
/// message goes through it: a reader who cannot tell a bounded prefix from a
/// complete diagnostic would stop looking after the last line shown.
#[test]
fn a_truncated_run_says_that_it_is_one() {
    let retention = DebugRetention::none()
        .retaining_with_stated_total("tiler.metal.0.metal", b"the first of very many bytes", 4096)
        .expect("a stated total above the supplied length is retainable");

    let mut written = Vec::new();
    assert!(reported_to(&retention, &mut written).is_some());
    let message = String::from_utf8(written).expect("the message is text");

    assert!(
        message.contains("truncated"),
        "a prefix must not be shown as the whole diagnostic: {message}",
    );
    assert!(
        message.contains("4096"),
        "the total the producer had must reach the reader: {message}",
    );
}

/// Bytes that are not UTF-8 are written exactly, and said to be what they are.
///
/// A tool's output is whatever it wrote. The retention keeps the bytes and
/// `RetainedText` answers which case a reader is in, so a replacement character
/// must not stand in for those bytes, and the typed marker must sit outside the
/// run.
#[test]
fn a_run_that_is_not_utf8_is_written_exactly_and_labelled() {
    let diagnostic = [0xffu8, 0xfe, 0xfd];
    let retention = DebugRetention::none()
        .retaining("tiler.metal.0.metal", &diagnostic)
        .expect("retainable");

    let mut written = Vec::new();
    assert!(reported_to(&retention, &mut written).is_some());
    assert_exact_run(&written, "tiler.metal.0.metal", &diagnostic);
    assert!(
        !contains_bytes(&written, "\u{FFFD}".as_bytes()),
        "U+FFFD must not stand in for the tool's own bytes: {}",
        String::from_utf8_lossy(&written),
    );
    assert!(
        contains_bytes(&written, b"not valid UTF-8"),
        "a non-UTF-8 run must be labelled rather than left to look like text: {}",
        String::from_utf8_lossy(&written),
    );
}

/// A leading whitespace byte is part of the tool run, not decoration.
#[test]
fn a_leading_whitespace_byte_is_written_exactly() {
    let diagnostic = b"  program_source:5:10: warning: unused variable 'x'";
    let retention = DebugRetention::none()
        .retaining("tiler.metal.0.metal", diagnostic)
        .expect("retainable");

    let mut written = Vec::new();
    assert!(reported_to(&retention, &mut written).is_some());
    assert_exact_run(&written, "tiler.metal.0.metal", diagnostic);
}

/// A trailing whitespace byte is part of the tool run, not decoration.
#[test]
fn a_trailing_whitespace_byte_is_written_exactly() {
    let diagnostic = b"program_source:5:10: warning: unused variable 'x'  ";
    let retention = DebugRetention::none()
        .retaining("tiler.metal.0.metal", diagnostic)
        .expect("retainable");

    let mut written = Vec::new();
    assert!(reported_to(&retention, &mut written).is_some());
    assert_exact_run(&written, "tiler.metal.0.metal", diagnostic);
}

/// The producer's own "nothing was retained" run reaches a reader.
///
/// `metal_cache.rs`'s `elided_retention` states one run saying why no stage
/// output is present, deliberately as a positive statement rather than as an
/// absent section — "a reader that could not tell the two apart would go looking
/// for a compiler that never spoke". It carries bytes, so it must report.
#[test]
fn the_producers_elision_run_reaches_a_reader() {
    let retention = DebugRetention::none()
        .retaining(
            "tiler.metal.retention-elided",
            b"no Metal stage output was retained: a retention carries at most 16 runs",
        )
        .expect("retainable");

    let mut written = Vec::new();
    assert!(reported_to(&retention, &mut written).is_some());
    assert!(
        String::from_utf8(written)
            .expect("the message is text")
            .contains("retention-elided"),
    );
}

/// A run at exactly the retained bound is not, by itself, a truncated one.
///
/// The pairing keeps the truncation marker from reading as decoration: it
/// appears when the producer states a larger total and stays away when the
/// supplied bytes are everything there was.
#[test]
fn a_full_but_complete_run_is_not_reported_as_truncated() {
    let whole = vec![b'w'; MAX_RETAINED_RUN_BYTES];
    let retention = DebugRetention::none()
        .retaining("tiler.metal.0.metal", &whole)
        .expect("a run at exactly the bound is retainable");

    let mut written = Vec::new();
    assert!(reported_to(&retention, &mut written).is_some());
    assert!(
        !String::from_utf8(written)
            .expect("the message is text")
            .contains("truncated"),
        "everything the producer had was retained, so nothing was cut",
    );
}

/// The rendering is exercised through the write seam, so the preamble is
/// stated once and checked rather than only reached as formatted text.
#[test]
fn the_message_names_what_it_costs_and_who_it_is_about() {
    let mut written = Vec::new();
    assert!(
        reported_to(
            &DebugRetention::none()
                .retaining("tiler.metal.0.metal", b"something")
                .expect("retainable"),
            &mut written,
        )
        .is_some()
    );
    let message = String::from_utf8(written).expect("the preamble is text");

    for required in [
        "cache/artifact acceptance succeeded",
        "later frontend emission can still refuse",
        "rather than a refusal",
        "No text a region declares reaches the emitted MSL",
    ] {
        assert!(
            message.contains(required),
            "the message must state `{required}`: {message}",
        );
    }
    for retired in [
        "The expansion succeeded",
        "compiled, validated, and embedded",
    ] {
        assert!(
            !message.contains(retired),
            "the message must not claim a later phase (`{retired}`): {message}",
        );
    }
}

/// Whether `haystack` contains `needle` as a contiguous byte run.
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// Asserts the tool's own bytes follow `{label}: ` unaltered.
///
/// A search for the run anywhere in the note is the wrong check: the provenance
/// separator is itself a space, so dropping a leading space of the tool run
/// still leaves a two-space window in the written bytes. The bytes immediately
/// after the label are the run, or the renderer trimmed it.
fn assert_exact_run(written: &[u8], label: &str, run: &[u8]) {
    assert!(
        !run.is_empty(),
        "the fixture must carry bytes, or this assertion states nothing",
    );
    let mut prefix = Vec::from(b"\n".as_slice());
    prefix.extend_from_slice(label.as_bytes());
    prefix.extend_from_slice(b": ");
    let start = written
        .windows(prefix.len())
        .position(|window| window == prefix.as_slice())
        .unwrap_or_else(|| {
            panic!(
                "the run's provenance must be written: {}",
                String::from_utf8_lossy(written),
            )
        });
    let body = &written[start + prefix.len()..];
    assert!(
        body.len() >= run.len(),
        "the writer ended before the tool run was complete: {}",
        String::from_utf8_lossy(written),
    );
    assert_eq!(
        &body[..run.len()],
        run,
        "the tool's own bytes must follow the run provenance unaltered: {}",
        String::from_utf8_lossy(written),
    );
}

/// A quiet retention renders nothing at all, at the selection step.
///
/// Asserted on [`spoken`] rather than through the writer, so the claim is that
/// the *selection* answers `None` rather than that a writer happened to be
/// handed nothing.
#[test]
fn selection_answers_none_for_every_silent_run() {
    assert_eq!(spoken(&quiet()), None);
    assert_eq!(spoken(&DebugRetention::none()), None);
}

/// A `SpokenRetention` is only ever built from runs that carry bytes.
///
/// The type is private and this is the one place it is constructed by hand, so
/// the invariant is stated where a future edit to [`spoken`] would break it.
#[test]
fn a_spoken_retention_holds_only_speaking_runs() {
    let retention = quiet()
        .retaining("tiler.metal.1.metal", b"a diagnostic")
        .expect("retainable");
    let SpokenRetention { runs } = spoken(&retention).expect("a speaking run selects");

    assert!(
        runs.iter().all(|run| !run.is_empty()),
        "an empty run must never reach the message: {runs:?}",
    );
}
