#![cfg_attr(
    not(target_os = "macos"),
    allow(
        dead_code,
        reason = "publishing a member emits MSL and invokes `metal` and `metallib`, so this module and its `proof` child are called only from `crate::envelope::apple` and are dead on every other host: the publication context and its two entry points, the per-member stage, the failure vocabulary, and the whole proof-case corpus, its operand tables, and its encoder. What a non-Apple host does hold this module to is `tests` — the published row count that keeps an artifact-derived shape distinguishable from this crate's own, and the removal that keeps a routed run from leaving its envelopes on disk — which is why the module is compiled everywhere rather than gated. Stated under `not(target_os = \"macos\")`, so an item nothing uses is still a red build on the host that publishes."
    )
)]

//! Publishing, in this run, the envelopes [`crate::envelope`] then routes.
//!
//! # Why this exists, and what changed when it did
//!
//! The routed half used to open envelopes a separate executable had written,
//! named by the ambient input `TILER_CONFORMANCE_ARTIFACT_BASE`. Nothing in
//! `make full` sets it, so the whole artifact-delivered route reported its
//! boundary unavailable on every gate run and only the device-free half ran: the
//! route was *reachable* rather than reached. This module is the missing call
//! rather than any new reach — every crate it touches was already declared for
//! exactly this vertical.
//!
//! # What producing in process does and does not weaken
//!
//! **It does not weaken the delivery claim.** What
//! [`crate::envelope`] establishes is that a route can be carried out from a
//! *published artifact alone*: the entry symbol, the argument-table index of
//! every buffer, the byte window each must reach, the launch geometry, and the
//! object bytes the device loads all come from the decoded envelope, and the only
//! thing this process compiles on the routing side is used to *name* the packaged
//! program by identity. That property is about where the routing facts come from,
//! not about which operating-system process wrote the file, and it is unchanged:
//! the bytes are encoded, written to disk, read back, decoded, associated with
//! their sidecar, and validated before anything is bound.
//!
//! **It does weaken one thing, and it is stated rather than absorbed.** The
//! producer and the consumer are now one process compiled from one tree, so a
//! disagreement between two *independently maintained* halves is no longer
//! observable here. It never was observable in the gate — the ambient input was
//! unset — and it remains observable where it already lives: the
//! `prototypes/serial-sum-compile` and `prototypes/serial-sum-run` pair still
//! publishes and routes across a file interface no code crosses, and both halves
//! still pin their idea of the names and shapes against the other's.
//!
//! # The published rows are deliberately not this crate's own
//!
//! [`PUBLISHED_ROWS`] is one and `crate::serial_sum::ROWS` is four, and the
//! inequality is load-bearing rather than incidental. The routed half must derive
//! the program it compares identities against from *the artifact's* declared
//! shape; a run that substituted its own row count would be invisible if the two
//! agreed. `tests::the_published_rows_are_not_the_direct_paths_own` is what stops
//! them from converging.
//!
//! # Cost, stated because it is charged to every gate run
//!
//! Publishing reaches the real offline Apple toolchain once per member, and each
//! pass emits MSL, invokes `metal` and `metallib`, assembles the neutral
//! artifact, and resolves it through the expansion cache. The cache is opened
//! inside a private temporary directory that [`Published`] removes, so every run
//! misses and compiles: a cache shared across runs would make the gate's cost
//! depend on hidden state and its determinism on a mutable path. The measured
//! wall-clock this adds is recorded on
//! `produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate`.
//!
//! **The gate charges eight of the twelve members.** Six serial-sum members and
//! two contraction members — the adversarial `2x2x3` and the `w_decode_kv` cell —
//! are published on every run. The other four contraction members are the L3
//! prefill cells, published only by the `#[ignore]`d run
//! `crate::envelope::tests::the_prefill_cells_carry_their_retained_digests`,
//! because their *oracles* are what costs: the reference folds 1,094,713,344
//! multiply-accumulate steps to state their expected bytes. That run is measured
//! at 30.8 s and a 323 MB peak resident set on an Apple M4 Max, against the whole
//! crate's 0.8 s otherwise.

use std::path::{Path, PathBuf};

use tiler_artifact::program::ArtifactCodecFailure;
use tiler_build::{
    BoundMetalCompileDeclaration, MetalPlanBuildError, accept_or_publish_metal_plan,
};
use tiler_cache::expansion::{ExpansionCache, Resolution};
use tiler_compiler::session::{NumericalContract, PlanAlternative};
use tiler_ir::semantic::SemanticProgram;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::OptimizationLevel;

use crate::envelope::{
    ContractionMember, PLAN_ROLES, REDUCTION_CLASSES, contraction_program, proof_member,
    sidecar_path,
};
use crate::serial_sum::{CompileRefusal, compile_under, serial_sum_program};

mod proof;

pub(crate) use proof::{ProofFamily, SidecarFailure};

/// Rows of every published serial-sum member's input.
///
/// **One, and the number is chosen against `crate::serial_sum::ROWS` rather than
/// for any property of its own.** The routed half compiles the artifact's
/// *declared* shape to name the program it packages, and the historic defect this
/// crate inherited was a consumer compiling its own row count instead —
/// undetectable while the two agreed. Four and one disagree, so a substitution
/// produces a foreign program identity and the route is refused.
///
/// The published sidecars still supply five independent operand classes per row,
/// including exceptional and contraction-sensitive values; row count is not being
/// used as a proxy for numerical coverage.
pub(crate) const PUBLISHED_ROWS: u64 = 1;

/// The optimization level every published member is compiled at.
///
/// Separate from the declaration because no ledger row is scoped to it: the
/// measured numerical rows are isolated by the fast-math attributes the front end
/// emitted, which `-O` does not change.
const OPTIMIZATION: OptimizationLevel = OptimizationLevel::Default;

/// The numerical contract every published member is compiled under.
///
/// Stated rather than defaulted. The strict contract is unhonourable on this
/// measured target, so the publication says which contract its programs mean
/// instead of discovering that by reading a rejection.
const CONTRACT: NumericalContract = NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32;

/// A set of published members, and the temporary directory holding them.
///
/// **The directory is removed when this value drops**, including while a panic
/// unwinds, which is what keeps a gate run from leaving megabytes of envelopes
/// behind on every host that routes. The routed run holds it for exactly as long
/// as it is reading the files.
pub(crate) struct Published {
    /// The private directory the members and the miss-only cache live in.
    directory: PathBuf,
    /// The base path every member's name is derived from.
    base: PathBuf,
}

impl Published {
    /// Creates a private directory for one routed run's members.
    ///
    /// The name carries the process and thread identity because the gate runs
    /// each test in its own process and may run several at once; two runs sharing
    /// a directory would let one remove the other's members mid-route.
    ///
    /// # Errors
    ///
    /// Returns [`PublicationFailure::Directory`] when the directory cannot be
    /// created.
    fn open(label: &str) -> Result<Self, PublicationFailure> {
        let directory = std::env::temp_dir().join(format!(
            "tiler-conformance-published-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id(),
        ));
        // Removed first so a directory left by an earlier process that happened
        // to reuse this identity cannot contribute a stale member.
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).map_err(|cause| {
            PublicationFailure::Directory(directory.display().to_string(), cause)
        })?;
        let base = directory.join("conformance.tiler");
        Ok(Self { directory, base })
    }

    /// The base path the published members' names are derived from.
    pub(crate) fn base(&self) -> &Path {
        &self.base
    }
}

impl Drop for Published {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

/// Why one publication did not produce the members a routed run needs.
///
/// The stages are kept apart rather than collapsed into one message: a program
/// this build does not compile, a target that cannot honour the declared
/// numerics, a plan the artifact layer refuses, and a record that does not
/// describe its envelope are four different things to do next.
#[derive(Debug)]
pub(crate) enum PublicationFailure {
    /// The private publication directory could not be created.
    Directory(String, std::io::Error),
    /// A published file could not be written.
    Write(String, std::io::Error),
    /// A program did not compile against the declared profile.
    Compile(CompileRefusal),
    /// The portfolio retained no selected plan.
    NoSelection,
    /// The portfolio retained no materialized alternative.
    NoMaterializedAlternative,
    /// The checked Metal plan did not reach an accepted artifact.
    Plan(Box<MetalPlanBuildError>),
    /// The verified artifact did not encode.
    Encode(ArtifactCodecFailure),
    /// Re-encoding the accepted artifact did not reproduce the accepted bytes.
    UnstableEncoding,
    /// The proof-case sidecar was refused.
    Sidecar(SidecarFailure),
}

impl std::fmt::Display for PublicationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Directory(path, cause) => write!(
                formatter,
                "the publication directory {path} could not be created: {cause}",
            ),
            Self::Write(path, cause) => write!(formatter, "{path} could not be written: {cause}"),
            Self::Compile(cause) => write!(formatter, "a published program did not compile: {cause}"),
            Self::NoSelection => formatter.write_str("the portfolio retained no selected plan"),
            Self::NoMaterializedAlternative => formatter.write_str(
                "the portfolio retained no materialized alternative, so the matrix cannot compare a \
                 fused program against the multi-dispatch program computing the same function",
            ),
            Self::Plan(cause) => write!(formatter, "the checked Metal plan failed: {cause}"),
            Self::Encode(cause) => write!(formatter, "the envelope did not encode: {cause}"),
            Self::UnstableEncoding => formatter
                .write_str("re-encoding the accepted artifact did not reproduce its own bytes"),
            Self::Sidecar(cause) => cause.fmt(formatter),
        }
    }
}

impl std::error::Error for PublicationFailure {}

/// The four things every member of one publication shares.
///
/// Grouped rather than passed member by member so [`publish_member`]'s signature
/// shows what varies between two members and what cannot: one cache, one
/// toolchain, one declaration, and one base path are the whole of a publication's
/// context, and a member that took its own would be publishable against a
/// different target than its siblings.
struct Publication<'a> {
    /// The miss-only cache every member is accepted or published through.
    cache: &'a ExpansionCache,
    /// The resolved offline Apple toolchain every member compiles with.
    toolchain: &'a Toolchain,
    /// The authoritative declaration every member is compiled and emitted under.
    declaration: &'a BoundMetalCompileDeclaration,
    /// The base path every member's name is derived from.
    base: &'a Path,
}

/// Publishes the six serial-sum members the routed matrix opens.
///
/// Three reduction classes — an empty domain, a singleton, and a nontrivial
/// reduction — times two plan roles, the portfolio's selected (fused) plan and
/// the retained materialized alternative that dispatches two stages through one
/// intermediate. The two roles are genuinely different programs on the device,
/// which is what makes their bit-for-bit agreement evidence about the optimizer
/// rather than about one program agreeing with itself.
///
/// # Errors
///
/// Returns the exact stage that refused; nothing is skipped or worked around.
pub(crate) fn publish_serial_sum_matrix(
    toolchain: &Toolchain,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<Published, PublicationFailure> {
    let published = Published::open("matrix")?;
    let cache = ExpansionCache::open(published.directory.join("cache"));
    let publication = Publication {
        cache: &cache,
        toolchain,
        declaration,
        base: published.base(),
    };
    for (class, columns) in REDUCTION_CLASSES {
        let program = serial_sum_program(PUBLISHED_ROWS, columns);
        let compilation =
            compile_under(declaration, &program, CONTRACT).map_err(PublicationFailure::Compile)?;
        for role in PLAN_ROLES {
            let plan = if role == "selected" {
                compilation
                    .selected()
                    .ok_or(PublicationFailure::NoSelection)?
            } else {
                // The retained alternative the portfolio did not rank first,
                // asked for by shape rather than by position: "not fused" is the
                // property the matrix needs, and an index would silently follow a
                // reordering of the portfolio.
                compilation
                    .alternatives()
                    .find(|alternative| !alternative.is_fused())
                    .ok_or(PublicationFailure::NoMaterializedAlternative)?
            };
            publish_member(
                &publication,
                class,
                role,
                ProofFamily::SerialSum {
                    rows: PUBLISHED_ROWS,
                    columns,
                },
                &program,
                plan,
            )?;
        }
    }
    Ok(published)
}

/// Publishes one contraction member the routed run opens.
///
/// One member per publication rather than the whole set at once, because the
/// routed runs open them one at a time and publishing the rest would compile
/// members nothing in that run reads — which on the L3 cells means operand
/// streams up to `3072x1024` and records up to fifteen megabytes, each of which
/// costs a reference fold to state.
///
/// # Errors
///
/// Returns the exact stage that refused.
pub(crate) fn publish_contraction(
    toolchain: &Toolchain,
    declaration: &BoundMetalCompileDeclaration,
    member: &ContractionMember,
) -> Result<Published, PublicationFailure> {
    let published = Published::open(member.class)?;
    let cache = ExpansionCache::open(published.directory.join("cache"));
    let (m, n, k) = member
        .family
        .contraction_extents()
        .expect("every contraction member declares contraction extents");
    let program = contraction_program(m, n, k);
    let compilation =
        compile_under(declaration, &program, CONTRACT).map_err(PublicationFailure::Compile)?;
    // One role rather than a pair: at this shape the portfolio retains exactly one
    // alternative for the contraction, so there is no materialized twin to publish
    // beside it and claiming a role pair would name a file that cannot be written.
    let plan = compilation
        .selected()
        .ok_or(PublicationFailure::NoSelection)?;
    publish_member(
        &Publication {
            cache: &cache,
            toolchain,
            declaration,
            base: published.base(),
        },
        member.class,
        "selected",
        member.family,
        &program,
        plan,
    )?;
    Ok(published)
}

/// Emits, compiles, assembles, validates, and writes one member.
///
/// The order is deliberate: everything is validated before either file is
/// written, so a member the artifact layer refuses stops the publication instead
/// of leaving an envelope on disk that no route would accept.
fn publish_member(
    publication: &Publication<'_>,
    class: &str,
    role: &str,
    family: ProofFamily,
    program: &SemanticProgram,
    plan: PlanAlternative<'_>,
) -> Result<(), PublicationFailure> {
    let entries = plan.kernels().len();
    let accepted = accept_or_publish_metal_plan(
        publication.cache,
        publication.toolchain,
        program,
        plan,
        std::slice::from_ref(publication.declaration),
        OPTIMIZATION,
    )
    .map_err(|cause| PublicationFailure::Plan(Box::new(cause)))?;
    let artifact = accepted.artifact();
    let bytes = match accepted.resolution() {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => {
            entry.envelope_bytes().to_vec()
        }
        Resolution::Uncached { envelope, .. } => envelope.clone(),
    };

    // The cache route has already decoded and validated these bytes. Re-encoding
    // the producer-side verified view proves that no field was lost while the
    // hit/publication path crossed the decoded representation.
    if artifact.encode().map_err(PublicationFailure::Encode)? != bytes {
        return Err(PublicationFailure::UnstableEncoding);
    }

    // Built before either file is written, so a record the artifact layer refuses
    // stops the publication instead of leaving an envelope on disk with nothing
    // describing it.
    let record = proof::encoded(artifact, program, family).map_err(PublicationFailure::Sidecar)?;

    let envelope_path = proof_member(publication.base, class, role);
    let record_path = sidecar_path(&envelope_path);
    std::fs::write(&envelope_path, &bytes)
        .map_err(|cause| PublicationFailure::Write(envelope_path.display().to_string(), cause))?;
    std::fs::write(&record_path, &record)
        .map_err(|cause| PublicationFailure::Write(record_path.display().to_string(), cause))?;

    eprintln!(
        "  published {class}.{role}: {entries} entr(y/ies), {} envelope byte(s), {} record byte(s)",
        bytes.len(),
        record.len(),
    );
    Ok(())
}

#[cfg(test)]
mod tests;
