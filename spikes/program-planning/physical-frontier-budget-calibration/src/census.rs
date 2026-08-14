//! Load-bearing population checks and the compiler-owned provider census.

use std::fs;
use std::path::{Path, PathBuf};

use tiler_compiler::physical_provider::{
    InstalledPhysicalProviders, PhysicalImplementationProvider,
};
use tiler_compiler::target::TargetProfile;
use tiler_ir::semantic::SemanticProgram;

use crate::program::{
    GOVERNED_FIVE_OP_DISTINCT_SUBJECTS, compile_governed_only, compile_installed, five_op_program,
    tiny_pointwise_program,
};
use crate::providers::{Answer, CountingProvider, as_dyn, flock, shared_tally};

/// One named population check and what it observed.
#[derive(Clone, Debug)]
pub struct Check {
    /// Stable check name.
    pub name: &'static str,
    /// What the check expected.
    pub expected: String,
    /// What it observed.
    pub observed: String,
    /// Whether observed equalled expected.
    pub passed: bool,
}

impl Check {
    #[allow(clippy::needless_pass_by_value)]
    fn eq(name: &'static str, expected: impl ToString, observed: impl ToString) -> Self {
        let expected = expected.to_string();
        let observed = observed.to_string();
        Self {
            name,
            passed: expected == observed,
            expected,
            observed,
        }
    }
}

/// How a check's subject is perturbed. The assertion is unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Perturb {
    /// Do not perturb.
    None,
    /// Count a fixture source that contains a second production impl.
    ExtraProductionProvider,
    /// Compile the tiny pointwise program instead of the five-op program.
    TinyProgram,
    /// Install no extra provider, so invocations are zero.
    MissingObserver,
    /// A "decline" provider that is actually silent.
    SilentDecline,
    /// A "specialist" that declines instead of proposing.
    DeclineInsteadOfPropose,
    /// Only one additive specialist when the check requires two.
    OneAdditiveSpecialist,
    /// An "infeasible" specialist that actually fits the profile.
    FeasibleInsteadOfInfeasible,
    /// Recalculate from three installed specialists instead of two.
    LimitRecommendationPopulation,
    /// Recalculate the full provider-slot population from 29 specialists.
    FullLimitPopulation,
}

impl Perturb {
    /// Parses a perturbation name.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "none" => Some(Self::None),
            "extra-production-provider" => Some(Self::ExtraProductionProvider),
            "tiny-program" => Some(Self::TinyProgram),
            "missing-observer" => Some(Self::MissingObserver),
            "silent-decline" => Some(Self::SilentDecline),
            "decline-instead-of-propose" => Some(Self::DeclineInsteadOfPropose),
            "one-additive-specialist" => Some(Self::OneAdditiveSpecialist),
            "feasible-instead-of-infeasible" => Some(Self::FeasibleInsteadOfInfeasible),
            "limit-recommendation-population" => Some(Self::LimitRecommendationPopulation),
            "full-limit-population" => Some(Self::FullLimitPopulation),
            _ => None,
        }
    }

    /// Every perturbation this harness can demonstrate.
    pub const ALL: &[Self] = &[
        Self::ExtraProductionProvider,
        Self::TinyProgram,
        Self::MissingObserver,
        Self::SilentDecline,
        Self::DeclineInsteadOfPropose,
        Self::OneAdditiveSpecialist,
        Self::FeasibleInsteadOfInfeasible,
        Self::LimitRecommendationPopulation,
        Self::FullLimitPopulation,
    ];

    /// Stable name used on the command line and in the record.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ExtraProductionProvider => "extra-production-provider",
            Self::TinyProgram => "tiny-program",
            Self::MissingObserver => "missing-observer",
            Self::SilentDecline => "silent-decline",
            Self::DeclineInsteadOfPropose => "decline-instead-of-propose",
            Self::OneAdditiveSpecialist => "one-additive-specialist",
            Self::FeasibleInsteadOfInfeasible => "feasible-instead-of-infeasible",
            Self::LimitRecommendationPopulation => "limit-recommendation-population",
            Self::FullLimitPopulation => "full-limit-population",
        }
    }
}

/// Production `PhysicalImplementationProvider` impls in `tiler-compiler`.
///
/// Test modules and the integration-test crate are stripped before the count,
/// so a fixture in `frontier.rs`'s `#[cfg(test)]` module cannot satisfy this
/// check. The expected value is one: `GovernedPhysicalProvider`.
#[must_use]
pub fn production_provider_impls(crate_src: &Path, perturb: Perturb) -> (usize, Vec<String>) {
    let mut names = Vec::new();
    let mut stack = vec![crate_src.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).expect("the compiler source tree is readable");
        for entry in entries {
            let entry = entry.expect("a directory entry is readable");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("tests.rs") {
                continue;
            }
            let source = fs::read_to_string(&path).expect("a rust file is readable");
            let stripped = production_source(&source);
            for name in impl_names(stripped) {
                names.push(format!("{}::{name}", path.display()));
            }
        }
    }
    names.sort();
    if matches!(perturb, Perturb::ExtraProductionProvider) {
        names.push("fixture::SyntheticSecondProvider".to_owned());
    }
    (names.len(), names)
}

/// Runs every load-bearing population check against `perturb`.
#[must_use]
pub fn run_checks(repo: &Path, perturb: Perturb) -> Vec<Check> {
    let crate_src = repo.join("crates/tiler-compiler/src");
    let (production_count, production_names) = production_provider_impls(&crate_src, perturb);
    let program = match perturb {
        Perturb::TinyProgram => tiny_pointwise_program(),
        _ => five_op_program(4, 3),
    };
    let profile = TargetProfile::governed();
    let declared = crate::profile::declared_workgroup_profile("test.calibrate-census.v1", 64);

    let mut checks = vec![Check::eq(
        "compiler-owned-production-providers",
        1_usize,
        production_count,
    )];
    let _ = production_names;

    checks.extend(observer_checks(&program, &profile, perturb));
    checks.extend(decline_checks(&program, &profile, perturb));
    checks.extend(specialist_checks(&program, &profile, perturb));
    checks.extend(additive_checks(&program, &profile, perturb));
    checks.extend(infeasible_checks(&program, &declared, perturb));
    checks.extend(empty_checks(&program, &profile));
    checks.extend(governed_only_checks(&program, &profile));
    checks.extend(installation_checks());
    checks
}

fn installation_checks() -> Vec<Check> {
    // This is a finite witness for the source reading: `installed` collects the
    // entire iterator and validates identity, but carries no count branch. It is
    // deliberately one above the old 128-outcome failure population so an
    // unrelated explain bound cannot be misread as an installation bound.
    let tally = shared_tally();
    let providers = flock("installation", 129, Answer::Empty, &tally);
    let environment = InstalledPhysicalProviders::installed(as_dyn(&providers))
        .expect("the public installation type has no count refusal");
    vec![Check::eq(
        "type-admits-129-installed-providers",
        129,
        environment.identities().len(),
    )]
}

fn observer_checks(
    program: &SemanticProgram,
    profile: &TargetProfile,
    perturb: Perturb,
) -> Vec<Check> {
    let tally = shared_tally();
    let compiled = if perturb == Perturb::MissingObserver {
        compile_governed_only(program, profile)
    } else {
        let observer = CountingProvider::new("observer", Answer::Empty, shared_tally_clone(&tally));
        compile_installed(
            program,
            profile,
            [&observer as &dyn PhysicalImplementationProvider],
        )
    };
    let invocations = tally.borrow().invocations;
    vec![
        Check::eq(
            "distinct-region-subjects",
            GOVERNED_FIVE_OP_DISTINCT_SUBJECTS,
            invocations,
        ),
        Check::eq(
            "observer-emits-nothing",
            "0/0",
            format!("{}/{}", tally.borrow().proposals, tally.borrow().declines),
        ),
        Check::eq(
            "observer-compile-succeeds",
            "ok",
            if compiled.failure.is_none() {
                "ok"
            } else {
                "fail"
            },
        ),
    ]
}

fn decline_checks(
    program: &SemanticProgram,
    profile: &TargetProfile,
    perturb: Perturb,
) -> Vec<Check> {
    let tally = shared_tally();
    let answer = match perturb {
        Perturb::SilentDecline => Answer::Empty,
        _ => Answer::Decline,
    };
    let provider = CountingProvider::new("decline", answer, shared_tally_clone(&tally));
    let compiled = compile_installed(
        program,
        profile,
        [&provider as &dyn PhysicalImplementationProvider],
    );
    let tally = tally.borrow();
    vec![
        Check::eq("many-declines-proposals", 0_u64, tally.proposals),
        Check::eq(
            "many-declines-count",
            GOVERNED_FIVE_OP_DISTINCT_SUBJECTS,
            tally.declines,
        ),
        Check::eq(
            "many-declines-raw-outcomes",
            GOVERNED_FIVE_OP_DISTINCT_SUBJECTS,
            tally.raw_outcomes(),
        ),
        Check::eq(
            "many-declines-not-selected",
            "offered-unselected",
            if compiled.selected_providers == 1 && compiled.offered == 2 {
                "offered-unselected"
            } else {
                "other"
            },
        ),
    ]
}

fn specialist_checks(
    program: &SemanticProgram,
    profile: &TargetProfile,
    perturb: Perturb,
) -> Vec<Check> {
    let tally = shared_tally();
    let answer = match perturb {
        Perturb::DeclineInsteadOfPropose => Answer::Decline,
        _ => Answer::Specialize { threads: 32 },
    };
    let provider = CountingProvider::new("specialist", answer, shared_tally_clone(&tally));
    let compiled = compile_installed(
        program,
        profile,
        [&provider as &dyn PhysicalImplementationProvider],
    );
    let tally = tally.borrow();
    let expected_proposals = tally.baseline_subjects;
    vec![
        Check::eq(
            "external-vertical-proposals",
            expected_proposals,
            tally.proposals,
        ),
        Check::eq(
            "external-vertical-declines",
            tally.coverless_or_unspellable,
            tally.declines,
        ),
        Check::eq(
            "external-vertical-raw-outcomes",
            GOVERNED_FIVE_OP_DISTINCT_SUBJECTS,
            tally.raw_outcomes(),
        ),
        Check::eq(
            "external-vertical-selected",
            "selected",
            if compiled.selected_providers >= 2 {
                "selected"
            } else {
                "not-selected"
            },
        ),
        Check::eq(
            "external-vertical-invocations",
            GOVERNED_FIVE_OP_DISTINCT_SUBJECTS,
            tally.invocations,
        ),
    ]
}

fn additive_checks(
    program: &SemanticProgram,
    profile: &TargetProfile,
    perturb: Perturb,
) -> Vec<Check> {
    let tally = shared_tally();
    let count = match perturb {
        Perturb::OneAdditiveSpecialist => 1,
        _ => 2,
    };
    let providers = flock(
        "additive",
        count,
        Answer::Specialize { threads: 32 },
        &tally,
    );
    let refs = as_dyn(&providers);
    let governed = compile_governed_only(program, profile);
    let compiled = compile_installed(program, profile, refs);
    let tally = tally.borrow();
    vec![
        Check::eq("two-additive-provider-count", 2_usize, count),
        Check::eq(
            "two-additive-invocations",
            GOVERNED_FIVE_OP_DISTINCT_SUBJECTS.saturating_mul(2),
            tally.invocations,
        ),
        Check::eq(
            "equal-cost-grows-alternatives",
            "grew",
            if compiled.alternatives > governed.alternatives {
                "grew"
            } else {
                "did-not-grow"
            },
        ),
        Check::eq("equal-cost-offered", 3_usize, compiled.offered),
    ]
}

fn infeasible_checks(
    program: &SemanticProgram,
    profile: &TargetProfile,
    perturb: Perturb,
) -> Vec<Check> {
    let tally = shared_tally();
    let threads = match perturb {
        Perturb::FeasibleInsteadOfInfeasible => 32,
        _ => 512,
    };
    let provider = CountingProvider::new(
        "infeasible",
        Answer::Infeasible { threads },
        shared_tally_clone(&tally),
    );
    let compiled = compile_installed(
        program,
        profile,
        [&provider as &dyn PhysicalImplementationProvider],
    );
    let observed = if compiled.selected_providers >= 2 {
        "selected"
    } else if compiled.failure.is_none() && compiled.offered == 2 {
        "rejected"
    } else {
        "other"
    };
    vec![
        Check::eq("infeasible-proposals-emitted", "emitted", {
            if tally.borrow().proposals > 0 {
                "emitted"
            } else {
                "none"
            }
        }),
        Check::eq("infeasible-not-selected", "rejected", observed),
        Check::eq(
            "infeasible-compile-still-succeeds",
            "ok",
            if compiled.failure.is_none() {
                "ok"
            } else {
                "fail"
            },
        ),
    ]
}

fn empty_checks(program: &SemanticProgram, profile: &TargetProfile) -> Vec<Check> {
    let tally = shared_tally();
    let providers = flock("empty", 4, Answer::Empty, &tally);
    let compiled = compile_installed(program, profile, as_dyn(&providers));
    let tally = tally.borrow();
    vec![
        Check::eq("empty-providers-proposals", 0_u64, tally.proposals),
        Check::eq("empty-providers-declines", 0_u64, tally.declines),
        Check::eq(
            "empty-providers-invocations",
            GOVERNED_FIVE_OP_DISTINCT_SUBJECTS.saturating_mul(4),
            tally.invocations,
        ),
        Check::eq("empty-providers-offered", 5_usize, compiled.offered),
        Check::eq(
            "empty-providers-selected-only-governed",
            1_usize,
            compiled.selected_providers,
        ),
    ]
}

fn governed_only_checks(program: &SemanticProgram, profile: &TargetProfile) -> Vec<Check> {
    let compiled = compile_governed_only(program, profile);
    vec![
        Check::eq("governed-only-offered", 1_usize, compiled.offered),
        Check::eq(
            "governed-only-compiles",
            "ok",
            if compiled.failure.is_none() {
                "ok"
            } else {
                "fail"
            },
        ),
        Check::eq(
            "governed-only-has-alternatives",
            "nonzero",
            if compiled.alternatives > 0 {
                "nonzero"
            } else {
                "zero"
            },
        ),
    ]
}

fn shared_tally_clone(tally: &crate::providers::SharedTally) -> crate::providers::SharedTally {
    std::rc::Rc::clone(tally)
}

fn production_source(source: &str) -> &str {
    let mut search = source;
    while let Some(rel) = search.find("#[cfg(test)]") {
        let abs = source.len() - search.len() + rel;
        let after = skip_leading_docs_and_attrs(&source[abs + "#[cfg(test)]".len()..]);
        if module_starts(after) {
            return &source[..abs];
        }
        search = &source[abs + 1..];
    }
    source
}

fn skip_leading_docs_and_attrs(source: &str) -> &str {
    let mut cursor = source;
    loop {
        cursor = cursor.trim_start();
        if cursor.starts_with("///") || cursor.starts_with("//!") || cursor.starts_with("//") {
            cursor = cursor.split_once('\n').map_or("", |(_, rest)| rest);
            continue;
        }
        if cursor.starts_with("#[")
            && let Some(end) = cursor.find(']')
        {
            cursor = &cursor[end + 1..];
            continue;
        }
        break;
    }
    cursor
}

fn module_starts(source: &str) -> bool {
    source.starts_with("mod ")
        || source.starts_with("pub mod ")
        || source.starts_with("pub(crate) mod ")
}

fn impl_names(source: &str) -> Vec<String> {
    let needle = "impl PhysicalImplementationProvider for ";
    source
        .match_indices(needle)
        .filter_map(|(idx, _)| {
            let rest = &source[idx + needle.len()..];
            let name = rest
                .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                .next()?;
            if name.is_empty() {
                None
            } else {
                Some(name.to_owned())
            }
        })
        .collect()
}

/// Repository root discovered from this spike's location.
#[must_use]
pub fn repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .nth(3)
        .expect("this spike lives at spikes/program-planning/<name>")
        .to_path_buf()
}
