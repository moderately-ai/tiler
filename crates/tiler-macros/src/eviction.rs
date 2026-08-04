//! When a delivering expansion trims its cache, and what states the bound.
//!
//! Tom decided on 2026-08-04, recorded in
//! `decide-the-expansion-cache-collection-schedule`, that the expansion cache
//! evicts old entries automatically under a policy configured through
//! environment variables, and that no maintenance command ships. This module is
//! the frontend half of that decision, and it exists here for
//! [`crate::cache_root`]'s reason: `tiler_cache` reads no environment and
//! schedules nothing, so the variable name, the parsing, the default, the
//! opt-out, and the trigger all belong beside the ADR 0089 root resolution the
//! same expansion already performs.
//!
//! # The policy
//!
//! One variable, [`MAX_ENTRY_AGE_VARIABLE`]:
//!
//! 1. Unset states [`MaxEntryAge::DEFAULT`] — the constant `tiler_cache` names
//!    and never applies. A consumer who configures nothing gets it, which is
//!    what makes the eviction automatic rather than opt-in.
//! 2. The exact value `off` is the opt-out: nothing is ever removed.
//! 3. Anything else is an age, spelled as a whole number and one of the unit
//!    suffixes in [`UNITS`] — `45s`, `90m`, `12h`, `30d`.
//! 4. Every other value is a typed refusal *of the eviction*, and the
//!    expansion continues: the artifact is compiled, published, and embedded,
//!    and the cache simply is not trimmed. A build must not fail over a hygiene
//!    setting, and a value nobody can parse must never become a guessed bound —
//!    the one direction that deletes a developer's compiled artifacts on the
//!    strength of a misreading.
//!
//! Only the *age* ceiling is configurable. [`CollectionBound`] also carries a
//! byte and an entry ceiling, and both are deliberately left absent: they select
//! by publication recency, which the collector's own documentation records as
//! able to evict a hot working set, and the decision Tom made was age-based.
//! `configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction` holds
//! that question with its activation trigger.
//!
//! The spelling `off` is [`crate::cache_root::DISABLE_VALUE`] itself rather than
//! a second constant of the same text. One environment surface should have one
//! word for "do not", and a superseding decision that changed ADR 0089's
//! spelling would otherwise leave two.
//!
//! # The trigger
//!
//! [`crate::aot::deliver`] runs a pass **after a publication and nowhere else**.
//! Not inside `get_or_publish`, which the bounded-collection design record
//! refused on performance grounds — a walk of every shard on the path the cache
//! exists to make fast, run hardest exactly when the cache is coldest. Not on a
//! hit: a resolution that read an existing entry does no filesystem work worth
//! amortizing a scan against, and a developer whose every expansion hits should
//! pay nothing. Not on the `fallback-only` route, which opens no cache at all.
//!
//! A publication is the one moment the cost is already paid: reaching it means
//! this expansion just ran `metal` and `metallib` as external processes, so a
//! directory scan rides on work orders of magnitude larger than itself.
//!
//! # The amortization rule
//!
//! **At most one pass per process.** [`EvictionGate`] holds one flag, the first
//! publishing expansion in a process claims it, and every later publication in
//! that process runs no pass at all. The rule is stated as a rule rather than as
//! a probability or an interval, because a bound has to have a trigger a person
//! can name: "the first expansion that publishes, in each build process" is a
//! sentence; "one build in eight" and "whenever a clock passed a threshold" are
//! the two triggers the design record eliminated as unexplainable.
//!
//! What that means in each driver is measured rather than assumed
//! (`docs/research/cache/build-tool-exercise.md`): under Cargo the expanding
//! process is `rustc`, one per crate compilation, so a build sweeps at most once
//! per crate that published anything; under rust-analyzer it is
//! `rust-analyzer-proc-macro-srv`, one process for the editor session, so a
//! session of thousands of expansions sweeps once. That is the case this rule
//! exists for — the analyzer is where "collect after every publish" would walk
//! every shard hundreds of times an afternoon.
//!
//! No clock decides whether it is time, no state is persisted to say when the
//! last pass ran, and nothing is spawned. A persisted timestamp was eliminated
//! twice over: it is durable state in the cache root that would have to be
//! reconciled after a crash, which the design record refused, and its answer to
//! "why did my entry go away" is a threshold rather than an act.
//!
//! # What becomes of the report
//!
//! [`CollectionReport`] is **dropped**, deliberately, and the derivation is
//! worth stating because a discarded report is the shape this crate is usually
//! built against.
//!
//! The schedule elimination used two discriminators — a report must terminate in
//! a reader, and a bound must arrive with its trigger — and Tom's decision
//! re-weighted exactly those: automation over per-act attribution, the
//! `cargo`/`sccache` shape, where routine hygiene is silent. So the report has
//! no consumer to reach. The channels that exist were each refused for their own
//! reason: a `compile_error!` fails a build over housekeeping; a build-log line
//! per eviction is noise attached to whichever invocation happened to publish
//! first; a marker file is durable state the design refused.
//!
//! What replaces the report is the policy being *readable back*. An entry leaves
//! only for the age the consumer's own environment stated, the zero-config value
//! is a documented constant, and anyone who wants the per-entry list has the
//! same public `ExpansionCache::collect` this module calls, returning the same
//! report with every removal named. A scan failure is dropped for the same
//! reason: the artifact this expansion published is already correct, and a cache
//! that cannot be walked must not fail the build that filled it.
//!
//! An **unusable statement** is not covered by that silence, and the asymmetry
//! is the point: a removal is Tiler doing its job, while an unparseable value is
//! the consumer's own input doing nothing. Silence there would leave a setting
//! that looks configured and is not, so one line naming the variable, the value,
//! and the remedy is written to the expanding process's standard error, at most
//! once per process.
//!
//! **Measurement — macOS 27.0 build 26A5388g, `cargo 1.97.0` and
//! `cargo +nightly-2026-07-19`, 2026-08-04.** A proc macro's `eprintln!` reaches
//! the terminal under `cargo build`, once, between the `Compiling` and
//! `Finished` lines, on both toolchains. The boundary: that is Cargo forwarding
//! `rustc`'s standard error, so it holds for a build that *expands*; a fully
//! warm build runs no macro and prints nothing. Where the rust-analyzer server's
//! standard error is surfaced was not measured, so the message is best effort
//! there rather than a promise.

use core::fmt;
use std::ffi::{OsStr, OsString};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tiler_cache::expansion::{
    CollectionBound, CollectionReport, ExpansionCache, MaxEntryAge, MaxEntryAgeRefusal,
};

use crate::cache_root::DISABLE_VALUE;

/// The one environment variable the automatic eviction is configured by.
///
/// Named for the *entry age* rather than for the cache as a whole, following
/// [`crate::cache_root::OVERRIDE_VARIABLE`]'s own argument one level down: the
/// typed policy carries three ceilings, so a variable spelled
/// `TILER_EXPANSION_CACHE_MAX_AGE` would be the obvious name for a byte or entry
/// ceiling too, and the day one of those is configurable a consumer's existing
/// setting would have to mean something else. The cost is length, which ADR 0089
/// already accepted for the same reason.
pub(crate) const MAX_ENTRY_AGE_VARIABLE: &str = "TILER_EXPANSION_CACHE_MAX_ENTRY_AGE";

/// The unit suffixes a stated age may carry, and the seconds each names.
///
/// One table, read by the parser, by the renderer that spells a duration back,
/// and by every diagnostic that lists what is accepted — so a unit cannot be
/// added to one of the three and missed by the others.
///
/// Ascending, because [`spelled`] walks it in reverse to find the largest unit
/// that divides a duration exactly.
///
/// Four units and no compound form: `1d12h` is refused rather than summed,
/// because a grammar that accepts sequences has to decide what `1h1h` and `1d1d`
/// mean, and nothing here needs an age a single unit cannot state. Lowercase
/// only, matching the exact-match rule ADR 0089 applies to `off`: a value that
/// is *nearly* a legal spelling is a refusal rather than a guess.
const UNITS: [(&str, u64); 4] = [("s", 1), ("m", 60), ("h", 3_600), ("d", 86_400)];

/// The environment the eviction policy is a function of.
///
/// Observation is separated from decision for [`crate::cache_root`]'s reason: a
/// policy that reaches for the process environment cannot be exercised without
/// one, and this crate forbids the `unsafe` a test would need to mutate it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EvictionEnvironment {
    stated: Option<OsString>,
}

impl EvictionEnvironment {
    /// Snapshots the one variable the policy reads, through `lookup`.
    ///
    /// The indirection exists so a test can record *which* names were read.
    /// A second variable creeping in is how two build tools would come to trim
    /// one cache under two policies.
    pub(crate) fn observe(mut lookup: impl FnMut(&str) -> Option<OsString>) -> Self {
        Self {
            stated: lookup(MAX_ENTRY_AGE_VARIABLE),
        }
    }

    /// Snapshots this process's environment.
    ///
    /// The one impure function here, and it decides nothing. Taken per
    /// expansion rather than once per process because rust-analyzer supplies a
    /// proc macro's environment per expansion request rather than only at spawn,
    /// so a snapshot cached for a process lifetime could apply one crate's
    /// setting to another's.
    #[must_use]
    pub(crate) fn from_process() -> Self {
        Self::observe(|name| std::env::var_os(name))
    }

    /// Builds a snapshot directly, for tests and for a caller that already holds
    /// the value.
    #[must_use]
    #[allow(
        dead_code,
        reason = "constructed by tests supplying an exact environment; production uses \
                  `from_process`. Kept because a stated-environment constructor is the seam \
                  every non-process caller of the policy needs."
    )]
    pub(crate) fn new(stated: Option<OsString>) -> Self {
        Self { stated }
    }
}

/// What the environment states about trimming the cache.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvictionPolicy {
    /// Collect under this bound after a publication.
    Bounded(CollectionBound),
    /// The consumer stated [`DISABLE_VALUE`]: never remove anything.
    ///
    /// Distinct from every refusal because it is not a failure, and distinct
    /// from an absent statement, which yields [`MaxEntryAge::DEFAULT`] instead —
    /// nothing is retained forever because a lookup came back empty.
    Disabled,
}

/// Why a stated eviction policy is not one.
///
/// Typed and non-erasing under ADR 0074 convention 1: which value was refused
/// and what was wrong with it both survive to the message, because "the eviction
/// policy is unusable" tells a consumer nothing about what to type instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EvictionRefusal {
    /// The variable is set to an empty value.
    ///
    /// Deliberately not read as unset, for [`crate::cache_root::RootRefusal`]'s
    /// reason: an exported-but-empty variable is the residue of a script that
    /// computed nothing, and falling through to the default would hide exactly
    /// that while quietly enabling removals under a bound nobody stated.
    Empty,
    /// The value is not text this policy can read.
    NotText {
        /// The offending value, as the host reported it.
        value: OsString,
    },
    /// The value is not a whole number followed by one accepted unit.
    Malformed {
        /// The offending value.
        value: String,
    },
    /// The value is a legal spelling of an age the cache refuses as a bound.
    NotABound {
        /// The offending value.
        value: String,
        /// The cache's own refusal, carried rather than restated.
        source: MaxEntryAgeRefusal,
    },
    /// The value states more seconds than a duration can hold.
    ///
    /// Refused rather than saturated. A saturated age would mean "never evict",
    /// which is what `off` says plainly, and inventing that meaning from an
    /// arithmetic accident is the guess this module exists to refuse.
    TooLarge {
        /// The offending value.
        value: String,
    },
}

impl fmt::Display for EvictionRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let remedy = format!(
            "state a whole number and one of {}, as in `{}`, or `{DISABLE_VALUE}` to keep every \
             entry, or unset it for the default `{}`. Nothing was removed",
            rendered_units(),
            spelled(Duration::from_mins(90)),
            spelled(MaxEntryAge::DEFAULT.as_duration()),
        );
        match self {
            Self::Empty => write!(
                formatter,
                "`{MAX_ENTRY_AGE_VARIABLE}` is set to an empty value, which states no maximum \
                 entry age; Tiler will not read an empty statement as though the variable were \
                 unset, and will not evict under a bound nobody chose. To fix it, {remedy}"
            ),
            Self::NotText { value } => write!(
                formatter,
                "`{MAX_ENTRY_AGE_VARIABLE}` is set to `{}`, which is not text `tiler::tensor!` can \
                 read as a maximum entry age. To fix it, {remedy}",
                value.to_string_lossy(),
            ),
            Self::Malformed { value } => write!(
                formatter,
                "`{MAX_ENTRY_AGE_VARIABLE}` is set to `{value}`, which is not a maximum entry age: \
                 an age is a whole number and exactly one unit suffix, with no sign, no decimal \
                 point, no space, and no second unit. To fix it, {remedy}"
            ),
            Self::NotABound { value, source } => write!(
                formatter,
                "`{MAX_ENTRY_AGE_VARIABLE}` is set to `{value}`, which the expansion cache refuses \
                 as a bound: {source}. To fix it, {remedy}"
            ),
            Self::TooLarge { value } => write!(
                formatter,
                "`{MAX_ENTRY_AGE_VARIABLE}` is set to `{value}`, which is more time than Tiler can \
                 represent. To fix it, {remedy}"
            ),
        }
    }
}

/// Resolves one expansion's eviction policy from an environment snapshot.
///
/// Pure and total: the same snapshot always yields the same answer, no
/// filesystem is touched, no clock is read, and nothing outside `environment` is
/// consulted.
///
/// # Errors
///
/// Returns the exact [`EvictionRefusal`] the stated value earned. Every one of
/// them means the same thing to the expansion — do not evict — and they stay
/// distinct anyway, because what a consumer must change differs.
pub(crate) fn resolve(
    environment: &EvictionEnvironment,
) -> Result<EvictionPolicy, EvictionRefusal> {
    let Some(stated) = environment.stated.as_deref() else {
        return Ok(EvictionPolicy::Bounded(age_bound(MaxEntryAge::DEFAULT)));
    };
    if stated.is_empty() {
        return Err(EvictionRefusal::Empty);
    }
    if stated == OsStr::new(DISABLE_VALUE) {
        return Ok(EvictionPolicy::Disabled);
    }
    let text = stated.to_str().ok_or_else(|| EvictionRefusal::NotText {
        value: stated.to_owned(),
    })?;
    Ok(EvictionPolicy::Bounded(age_bound(stated_age(text)?)))
}

/// The bound one stated age produces.
///
/// The aggregate ceilings are absent rather than derived, which is what keeps
/// `CollectionBound::UNBOUNDED`'s promise true from here: the only entries this
/// policy can select are ones that reached the stated age.
const fn age_bound(max_entry_age: MaxEntryAge) -> CollectionBound {
    CollectionBound {
        max_total_bytes: None,
        max_entries: None,
        max_entry_age: Some(max_entry_age),
    }
}

/// Reads `<whole number><unit>` as a maximum entry age.
fn stated_age(stated: &str) -> Result<MaxEntryAge, EvictionRefusal> {
    let malformed = || EvictionRefusal::Malformed {
        value: stated.to_owned(),
    };
    // Split before the last character rather than scanning for the first
    // non-digit, so a value carrying several unit letters (`1h30m`) is malformed
    // instead of parsing as its first component and silently discarding the
    // rest. `split_at_checked` declines a split inside a multi-byte character,
    // which is why the boundary is not asserted.
    let (digits, unit) = stated
        .len()
        .checked_sub(1)
        .and_then(|last| stated.split_at_checked(last))
        .ok_or_else(malformed)?;
    let seconds_per_unit = UNITS
        .into_iter()
        .find_map(|(suffix, seconds)| (suffix == unit).then_some(seconds))
        .ok_or_else(malformed)?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        // Every rejection this covers is deliberate: a sign, a decimal point, a
        // digit separator, a space, and an empty count are each a value whose
        // meaning would have to be guessed.
        return Err(malformed());
    }
    let seconds = digits
        .parse::<u64>()
        .ok()
        .and_then(|count| count.checked_mul(seconds_per_unit))
        .ok_or_else(|| EvictionRefusal::TooLarge {
            value: stated.to_owned(),
        })?;
    MaxEntryAge::new(Duration::from_secs(seconds)).map_err(|source| EvictionRefusal::NotABound {
        value: stated.to_owned(),
        source,
    })
}

/// Spells a duration the way [`stated_age`] reads one.
///
/// Used for the default and for the example in every diagnostic, so that what a
/// consumer is shown is a value they can paste back — and so that a change to
/// [`MaxEntryAge::DEFAULT`] cannot leave a stale number in a message here.
///
/// The largest unit that divides exactly, so thirty days renders as `30d` rather
/// than as `2592000s`. A duration no unit divides falls back to seconds, and a
/// sub-second remainder is dropped, which cannot reach a diagnostic: every
/// duration spelled here came from this module's own units or from a constant
/// stated in whole hours.
fn spelled(duration: Duration) -> String {
    let seconds = duration.as_secs();
    UNITS
        .into_iter()
        .rev()
        .find(|(_, per_unit)| seconds >= *per_unit && seconds.is_multiple_of(*per_unit))
        .map_or_else(
            || format!("{seconds}s"),
            |(suffix, per_unit)| format!("{}{suffix}", seconds / per_unit),
        )
}

/// Renders the accepted unit suffixes the way a diagnostic lists them.
fn rendered_units() -> String {
    UNITS
        .into_iter()
        .map(|(suffix, _)| format!("`{suffix}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The once-per-process amortization an automatic eviction runs under.
///
/// One flag for the pass and one for the refusal message, because they fire
/// independently: a host with an unusable statement never sweeps, and a host
/// with a perfectly good one never reports.
///
/// `Relaxed` on both, deliberately. The flags order nothing and guard no shared
/// data — losing a race means one extra directory scan or one extra line on
/// standard error, and the collector is measured safe against concurrent
/// collectors and publishers at 1, 8, and 32 processes. Paying for a stronger
/// ordering would buy an exactness that has no consumer.
#[derive(Debug)]
pub(crate) struct EvictionGate {
    swept: AtomicBool,
    reported: AtomicBool,
}

impl EvictionGate {
    /// A gate that has not yet run anything.
    pub(crate) const fn new() -> Self {
        Self {
            swept: AtomicBool::new(false),
            reported: AtomicBool::new(false),
        }
    }

    /// The one gate this process amortizes against.
    ///
    /// A `static` rather than a value threaded from [`crate::expand`], because
    /// the rule is *per process* and an expansion has no other object with that
    /// lifetime. A test constructs its own with [`Self::new`] instead, so no
    /// test's eviction depends on whether another test ran first.
    pub(crate) fn process() -> &'static Self {
        static PROCESS: EvictionGate = EvictionGate::new();
        &PROCESS
    }

    /// True for the first caller in this process, and false for every later one.
    fn claim_sweep(&self) -> bool {
        !self.swept.swap(true, Ordering::Relaxed)
    }

    /// True for the first refusal reported in this process.
    fn claim_report(&self) -> bool {
        !self.reported.swap(true, Ordering::Relaxed)
    }
}

/// One expansion's eviction policy, together with what its process has already
/// done.
///
/// Two halves with two lifetimes: the environment is snapshotted per expansion,
/// because the analyzer supplies it per request, while the gate is the process's
/// and is borrowed rather than copied — a gate cloned per expansion would
/// amortize nothing.
#[derive(Debug)]
pub(crate) struct EvictionSchedule<'a> {
    environment: EvictionEnvironment,
    gate: &'a EvictionGate,
}

impl<'a> EvictionSchedule<'a> {
    /// Reads this process's environment, against this process's gate.
    #[must_use]
    pub(crate) fn from_process(gate: &'a EvictionGate) -> Self {
        Self {
            environment: EvictionEnvironment::from_process(),
            gate,
        }
    }

    /// States the environment directly, for tests and for a caller that already
    /// holds it.
    #[must_use]
    #[allow(
        dead_code,
        reason = "constructed by tests stating an exact policy; production uses `from_process`. \
                  Kept for `EvictionEnvironment::new`'s reason: the stated-environment seam is \
                  what makes every branch of the policy reachable without mutating the process."
    )]
    pub(crate) fn stated(environment: EvictionEnvironment, gate: &'a EvictionGate) -> Self {
        Self { environment, gate }
    }

    /// The bound this expansion may collect under, or `None` not to collect at
    /// all.
    ///
    /// An unusable statement is reported once per process on standard error and
    /// then behaves exactly like the opt-out: no eviction, no guessed bound, and
    /// an expansion that carries on to publish and embed its artifact.
    pub(crate) fn bound(&self) -> Option<CollectionBound> {
        self.bound_reported_to(&mut io::stderr())
    }

    /// [`Self::bound`], writing any refusal to `out` rather than to the process.
    ///
    /// The seam exists so the message is a testable value: a check that could
    /// only observe the real standard error would be asserting on the harness
    /// rather than on what a consumer reads.
    fn bound_reported_to(&self, out: &mut impl io::Write) -> Option<CollectionBound> {
        match resolve(&self.environment) {
            Ok(EvictionPolicy::Bounded(bound)) => Some(bound),
            Ok(EvictionPolicy::Disabled) => None,
            Err(refusal) => {
                if self.gate.claim_report() {
                    // The write is best effort. A closed or failing standard
                    // error is not a reason to fail an expansion that has
                    // already produced a correct artifact.
                    let _ = writeln!(out, "`tiler::tensor!`: {refusal}");
                }
                None
            }
        }
    }

    /// Runs at most one eviction pass per process, and returns what it found.
    ///
    /// The caller decides *when* this may be called — [`crate::aot::deliver`]
    /// calls it after a publication and nowhere else — and this decides whether
    /// the process has one left. Returns `None` when the gate was already
    /// claimed or when the scan itself failed, which the module documentation
    /// explains is dropped rather than surfaced.
    ///
    /// The report is returned rather than discarded here so that a test can
    /// state what a pass did; production ignores it.
    pub(crate) fn sweep(
        &self,
        cache: &ExpansionCache,
        bound: CollectionBound,
    ) -> Option<CollectionReport> {
        if !self.gate.claim_sweep() {
            return None;
        }
        cache.collect(&bound).ok()
    }
}

#[cfg(test)]
mod tests;
