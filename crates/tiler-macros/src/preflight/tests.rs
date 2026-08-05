//! The automatic root probe, its amortization, and the line it writes.
//!
//! Every test here supplies its own gate and its own root, and writes to a
//! stream it owns rather than to the process's standard error, so no test's
//! outcome depends on whether another ran first and what a consumer reads is a
//! value that can be asserted on.
//!
//! Two of them touch a real filesystem, because that is what is under test: an
//! unwritable root's verdicts cannot be constructed — `PreflightReport`'s fields
//! are private to `tiler_cache` and it publishes no constructor — so the only
//! way to watch this line fire is to provoke the state that produces it.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use tiler_cache::expansion::{ExpansionCache, PreflightVerdict};

use super::{
    DISABLE_VALUE, OVERRIDE_VARIABLE, PROPERTIES, PreflightGate, UnsuitableRoot, reported_to,
};

/// A cache root private to one test.
fn scratch(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "tiler-macros-preflight-{label}-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o700));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("the scratch directory is creatable");
    path
}

/// Restores a directory's permissions however the test that made it read-only
/// ended.
///
/// A guard rather than a call at the end of the test, so a failed assertion
/// still leaves the scratch directory removable by the next run.
struct RestorePermissions<'a>(&'a Path);

impl Drop for RestorePermissions<'_> {
    fn drop(&mut self) {
        let _ = fs::set_permissions(self.0, fs::Permissions::from_mode(0o700));
    }
}

/// The amortization rule: one process, one probe, however many expansions.
///
/// Asserted on the gate rather than through a cache, because what is being
/// stated is that the *permission* is issued once — a test that watched messages
/// would also pass on a gate that admitted every caller against a root that
/// answers for everything.
#[test]
fn a_gate_admits_one_probe_per_process() {
    let gate = PreflightGate::new();
    assert!(gate.claim(), "the first delivering expansion probes");
    for attempt in 0..8 {
        assert!(
            !gate.claim(),
            "expansion {attempt} after the first must probe nothing",
        );
    }

    // A fresh gate is a fresh process, which is what keeps one build's
    // amortization from silencing the next one's.
    assert!(PreflightGate::new().claim());
}

/// A root nothing can be written under is reported, once, with everything a
/// consumer needs to act.
///
/// **This is the watched firing.** The root is made mode `0o500`, so the probe's
/// own `create_dir_all` under the cache namespace fails and every property
/// answers `NotRun` — the exact state
/// `PreflightVerdict::NotRun`'s documentation calls out as "most often that the
/// root is not writable, which is itself worth reporting rather than reading as
/// a filesystem verdict".
///
/// The second call is the amortization, and it is asserted over the same buffer:
/// a rust-analyzer session expanding all afternoon must not repeat one
/// misconfiguration into its log forever.
#[test]
fn an_unwritable_root_reports_one_attributable_line() {
    let scratch = scratch("unwritable");
    let root = scratch.join("cache");
    fs::create_dir_all(&root).expect("the root is creatable");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o500))
        .expect("the root permissions are settable");
    let restore = RestorePermissions(&root);

    let gate = PreflightGate::new();
    let cache = ExpansionCache::open(root.clone());
    let mut written = Vec::new();
    let reported = reported_to(&gate, &cache, &mut written).expect("an unwritable root reports");

    assert_eq!(
        reported.missing.len(),
        PROPERTIES.len(),
        "a root that could not be prepared answers for nothing: {reported:?}",
    );
    for (capability, verdict) in &reported.missing {
        assert_eq!(
            *verdict,
            PreflightVerdict::NotRun,
            "a permission is an unrunnable probe rather than a refutation: {capability}",
        );
    }

    let line = String::from_utf8(written.clone()).expect("the message is text");
    assert_eq!(line.lines().count(), 1, "one line per process: {line}");
    for required in [
        "`tiler::tensor!`:",
        &root.display().to_string(),
        "not probed",
        "The expansion continues",
        OVERRIDE_VARIABLE,
        DISABLE_VALUE,
    ] {
        assert!(
            line.contains(required),
            "the line must name `{required}`: {line}",
        );
    }

    assert_eq!(
        reported_to(&gate, &cache, &mut written),
        None,
        "a second expansion in one process must probe nothing",
    );
    assert_eq!(
        String::from_utf8(written).expect("the message is text"),
        line,
        "a second expansion in one process must add nothing",
    );

    drop(restore);
    let _ = fs::remove_dir_all(scratch);
}

/// A root that answers for everything writes nothing.
///
/// The paired negative of the test above, and it is not a restatement: without
/// it, "an unsuitable root is reported" is also what an implementation that
/// narrated every delivering expansion would produce, and a build log that
/// announces a healthy cache on every crate is the noise the eviction's report
/// disposition was already decided against.
///
/// The second assertion is what keeps it from passing vacuously: the report has
/// to say every property *held*, not merely that nothing was refuted, since a
/// probe that ran nothing would also write no line here.
#[test]
fn a_root_that_answers_for_everything_reports_nothing() {
    let root = scratch("healthy");
    let gate = PreflightGate::new();
    let cache = ExpansionCache::open(root.clone());
    let mut written = Vec::new();

    assert_eq!(reported_to(&gate, &cache, &mut written), None);
    assert!(
        written.is_empty(),
        "a usable root must write nothing to a consumer's build output: {}",
        String::from_utf8_lossy(&written),
    );
    assert!(
        cache.preflight().all_probed_properties_hold(),
        "the scratch root must answer for every property, or the silence above says nothing",
    );

    let _ = fs::remove_dir_all(root);
}

/// `TILER_EXPANSION_CACHE_DIR=off` probes nothing and leaves the process's one
/// probe unspent.
///
/// The second half is the load-bearing one. rust-analyzer supplies a proc
/// macro's environment per expansion request, so one process can resolve `off`
/// for one crate and a real root for the next; a gate claimed by the disabled
/// cache would silence the root that actually has a filesystem to answer for.
#[test]
fn a_disabled_cache_probes_nothing_and_spends_no_gate() {
    let scratch = scratch("disabled");
    let gate = PreflightGate::new();
    let mut written = Vec::new();

    assert_eq!(
        reported_to(&gate, &ExpansionCache::disabled(), &mut written),
        None,
        "a cache with no root has nothing to probe",
    );
    assert!(written.is_empty(), "`off` is a decision, not a fault");

    let root = scratch.join("cache");
    fs::create_dir_all(&root).expect("the root is creatable");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o500))
        .expect("the root permissions are settable");
    let restore = RestorePermissions(&root);
    assert!(
        reported_to(&gate, &ExpansionCache::open(root.clone()), &mut written).is_some(),
        "the disabled expansion must not have spent this process's probe",
    );

    drop(restore);
    let _ = fs::remove_dir_all(scratch);
}

/// Every probed property is rendered, and a refuted one reads as refuted.
///
/// The count is the claim: [`PROPERTIES`] mirrors a table in `tiler_cache` that
/// nothing links it to, so a row added there and missed here would be a property
/// whose refutation reached no consumer while
/// `PreflightReport::all_probed_properties_hold` still counted it.
///
/// The rendering is exercised over a value constructed here because a refuted
/// verdict cannot be provoked from this crate: refuting `same_device` needs a
/// root straddling two filesystems, and refuting the others needs a filesystem
/// whose primitives report success without excluding anything. The measurement
/// boundary is therefore exact — the *reachability* of a refutation is
/// `tiler_cache`'s to establish, and what is checked here is that one reads as
/// itself rather than as an unrunnable probe.
#[test]
fn every_probed_property_is_rendered_and_a_refutation_reads_as_one() {
    assert_eq!(
        PROPERTIES.len(),
        5,
        "the properties this frontend renders are counted, not sampled",
    );

    let rendered = UnsuitableRoot {
        root: PathBuf::from("/Users/example/Library/Caches/ai.moderately.tiler/expansion"),
        missing: PROPERTIES
            .into_iter()
            .map(|(capability, _)| (capability, PreflightVerdict::Refuted))
            .collect(),
    }
    .to_string();

    for (capability, _) in PROPERTIES {
        assert!(
            rendered.contains(capability),
            "every property must be nameable in the line: {capability}",
        );
    }
    assert_eq!(
        rendered.matches("(refuted)").count(),
        PROPERTIES.len(),
        "a refuted property must read as refuted rather than as an unrunnable probe: {rendered}",
    );
    assert_eq!(
        rendered.matches("(not probed)").count(),
        0,
        "no row was unrunnable, so none may read that way: {rendered}",
    );
    assert_eq!(rendered.lines().count(), 1, "one line: {rendered}");
}
