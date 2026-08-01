//! Where an inline expansion looks for its expansion cache, stated once.
//!
//! `tiler_cache::expansion::ExpansionCache::open` takes a root from its caller
//! and never consults the environment: a host-relative decision inside a storage
//! protocol is what would stop that protocol being testable without a host. The
//! chooser therefore belongs to the frontend, and this module is it.
//!
//! # The policy
//!
//! Two inputs, in this precedence:
//!
//! 1. `TILER_EXPANSION_CACHE_DIR`, when set. An absolute directory path is the
//!    root verbatim; the exact value `off` expands with no cache at all; every
//!    other value is refused.
//! 2. Otherwise `$HOME/Library/Caches/ai.moderately.tiler/expansion`.
//!
//! Nothing else is consulted, and there is no third step. A root that cannot be
//! derived is a typed refusal a consumer reads, never a silent miss and never a
//! silent second location — a cache that quietly relocates is a cache that
//! quietly recompiles, and the developer sees only that builds became slow.
//!
//! # Why the user cache rather than the consumer's tree
//!
//! [`docs/integration/frontends.md`][frontends] and
//! [`docs/backends/metal.md`][metal] both state the accepted shape: "a default
//! macOS user cache … rather than consumer `OUT_DIR`", with "a CI/sandbox
//! override". What this module adds is the exact derivation, the exact
//! precedence, and what happens when neither input is usable.
//!
//! `$HOME/Library/Caches` is the per-user cache directory macOS creates with
//! mode `0700`, which is what makes the default satisfy `ExpansionCache::open`'s
//! stated requirement that the root be "private to the user running Tiler".
//! That requirement is not decoration: the cache's integrity validation catches
//! corruption and partial writes, and explicitly "does not make a shared
//! writable cache an adversarial boundary, because an attacker able to replace
//! files can construct new internally consistent bytes".
//!
//! Deriving the root from `CARGO_MANIFEST_DIR` was the obvious alternative and
//! is rejected for reasons stated in
//! [`docs/research/cache/root-policy.md`][policy]: a checkout directory is not
//! private to one user, and a per-package root confines sharing to one package
//! when the whole value of a content-addressed cache is that identical
//! invocations share compiler work across packages, checkouts, and build tools.
//! Reachability was never the binding constraint.
//!
//! # Why this cannot read a driver apart, and must not
//!
//! [`RootEnvironment`] carries exactly two names. `rust-analyzer` populates a
//! proc macro's environment from the crate graph it loaded rather than only from
//! the editor's process environment, so `CARGO_PKG_NAME` is present under both
//! drivers and does not distinguish them; `std::env::current_exe()` does. A
//! policy that could tell them apart could give them different roots, and two
//! roots for one project is precisely the split that makes the editor recompile
//! what the terminal already built. The snapshot's shape is the mechanism:
//! `observation_reads_exactly_the_two_policy_variables` in this module's tests
//! fails the moment a third name is read.
//!
//! # Accepted surface
//!
//! Every consumer-visible name here — the variable spelling, the `off` value,
//! the derived path, and the exact refusal text — was accepted by Tom on
//! 2026-07-31 under ADR 0075, recorded in ADR 0089
//! (`docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md`).
//! Changing one is a superseding decision rather than an edit. The module is
//! crate-private and nothing but its own tests calls it yet.
//!
//! [frontends]: https://github.com/moderately-ai/tiler/blob/main/docs/integration/frontends.md
//! [metal]: https://github.com/moderately-ai/tiler/blob/main/docs/backends/metal.md
//! [policy]: https://github.com/moderately-ai/tiler/blob/main/docs/research/cache/root-policy.md

#![allow(
    dead_code,
    reason = "the cache-root policy is accepted (ADR 0089) and not yet reached: the expansion \
              cache exists to share *external* compilation, and every region states \
              `FallbackOnly`, which ADR 0053 defines as invoking no backend compiler. There is \
              therefore nothing to cache, and resolving a root anyway would let an unset `HOME` \
              refuse an expansion that opens no cache. \
              `generate-cfg-gated-artifact-family-delivery` landed the delivery half — the \
              versioned consumer-`cfg` map and the gated tokens — without changing that: it emits \
              what a compilation produced and does not itself compile. \
              `prototype-inline-aot-integration-proof` is the slice that first invokes the \
              backend compiler, and it is what consumes this resolver. The surface reserved is \
              the whole of the stated policy — the override variable, the `off` value, the \
              derived user-cache root, and every refusal a consumer can read."
)]

use core::fmt;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The one environment variable that states or disables the cache root.
///
/// Named for the *expansion* cache rather than for Tiler as a whole because
/// ADR 0082's residue foresees a runtime pipeline-state cache and a compiler
/// plan cache. A generic `TILER_CACHE_DIR` would silently acquire those the day
/// they exist, changing what a consumer's existing setting means; a precise name
/// makes each later cache ask for its own variable.
pub(crate) const OVERRIDE_VARIABLE: &str = "TILER_EXPANSION_CACHE_DIR";

/// The variable the default root derives from.
pub(crate) const HOME_VARIABLE: &str = "HOME";

/// The one override value that means "expand with no cache".
///
/// A sandbox with no writable per-user location has to be able to say so. The
/// alternative — refusing, and leaving the consumer to point the override at a
/// scratch directory — spends a directory and a cleanup obligation to express
/// something the consumer meant literally. Matched exactly, with no case folding
/// and no synonyms, so that a value which is *nearly* the sentinel is a refusal
/// rather than a guess.
pub(crate) const DISABLE_VALUE: &str = "off";

/// The path components appended to `$HOME` to reach the default root.
///
/// `Library/Caches` is the per-user cache directory macOS creates with mode
/// `0700`. The reverse-DNS component follows Apple's convention for a cache
/// namespace, and the final component leaves room for a sibling to hold a
/// different Tiler cache without either colliding with the other's layout.
const USER_CACHE_COMPONENTS: [&str; 4] = ["Library", "Caches", "ai.moderately.tiler", "expansion"];

/// Directory trees macOS makes writable by every user on the machine.
///
/// **Measurement — macOS 27.0, 2026-07-31.** `ls -ld` reports mode `1777` for
/// `/private/tmp`, `/private/var/tmp`, `/var/tmp`, and `/Users/Shared`; `/tmp`
/// is a symbolic link to `private/tmp` and `/var` to `private/var`, so both
/// spellings of each pair are listed rather than resolved, because resolving
/// one would mean touching the filesystem from a decision that must stay pure.
///
/// A cache root at or under any of them is writable by another user of the same
/// machine, which the root's privacy requirement forbids. macOS's own per-user
/// temporary directory is *not* affected: `$TMPDIR` is `/var/folders/…/T/` with
/// mode `0700`, so the ordinary CI remedy remains available.
const WORLD_WRITABLE_TREES: [&str; 5] = [
    "/tmp",
    "/private/tmp",
    "/var/tmp",
    "/private/var/tmp",
    "/Users/Shared",
];

/// The environment the root decision is a function of.
///
/// Observation is separated from decision so that the decision is pure: a
/// snapshot goes in, a decision or a refusal comes out, and every case below is
/// reachable from a test without a filesystem, a process, or a build tool.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RootEnvironment {
    stated: Option<OsString>,
    home: Option<OsString>,
}

impl RootEnvironment {
    /// Snapshots the two variables the policy reads, through `lookup`.
    ///
    /// The indirection exists so a test can record *which* names were read.
    /// Nothing else here could detect a third variable creeping in, and a third
    /// variable is how two build tools would come to disagree about one root.
    pub(crate) fn observe(mut lookup: impl FnMut(&str) -> Option<OsString>) -> Self {
        Self {
            stated: lookup(OVERRIDE_VARIABLE),
            home: lookup(HOME_VARIABLE),
        }
    }

    /// Snapshots this process's environment.
    ///
    /// The one impure function in this module, and it decides nothing.
    #[must_use]
    pub(crate) fn from_process() -> Self {
        Self::observe(|name| std::env::var_os(name))
    }

    /// Builds a snapshot directly, for tests and for a caller that already holds
    /// the values.
    #[must_use]
    pub(crate) fn new(stated: Option<OsString>, home: Option<OsString>) -> Self {
        Self { stated, home }
    }
}

/// Which input produced a root, or failed to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RootSource {
    /// The consumer stated the root through [`OVERRIDE_VARIABLE`].
    Override,
    /// The root was derived from the per-user macOS cache directory.
    UserCache,
}

impl RootSource {
    /// The variable a diagnostic should name for this source.
    const fn variable(self) -> &'static str {
        match self {
            Self::Override => OVERRIDE_VARIABLE,
            Self::UserCache => HOME_VARIABLE,
        }
    }
}

/// What the policy decided for one expansion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CacheRootDecision {
    /// Open the expansion cache at this root.
    Directory {
        /// The root to hand to `ExpansionCache::open`.
        root: PathBuf,
        /// Which input it came from, so a diagnostic can say.
        source: RootSource,
    },
    /// The consumer stated [`DISABLE_VALUE`]: expand, compile, embed, and cache
    /// nothing.
    ///
    /// Distinct from every refusal because it is not a failure. It is also
    /// distinct from an *absent* decision, which this module never returns —
    /// caching nothing happens because a consumer asked for it, never because a
    /// lookup quietly came back empty.
    Disabled,
}

impl CacheRootDecision {
    /// The root, when one was decided.
    #[must_use]
    pub(crate) fn root(&self) -> Option<&Path> {
        match self {
            Self::Directory { root, .. } => Some(root),
            Self::Disabled => None,
        }
    }
}

/// Why no cache root could be decided.
///
/// Typed and non-erasing under ADR 0074 convention 1: which input was wrong and
/// what was wrong with it both survive to the diagnostic, because "could not
/// determine a cache root" tells a consumer nothing about which of two variables
/// to change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RootRefusal {
    /// [`OVERRIDE_VARIABLE`] is set to an empty value.
    ///
    /// Deliberately not treated as unset. An exported-but-empty variable is the
    /// residue of a script that failed to compute a path, and falling through to
    /// the default would hide exactly that failure — the consumer would get a
    /// working build against a root it did not choose.
    OverrideEmpty,
    /// A stated or derived root is not an absolute path.
    NotAbsolute {
        /// Which input carried it.
        source: RootSource,
        /// The offending value.
        value: PathBuf,
    },
    /// A stated or derived root lies in a tree every user of the machine can
    /// write.
    NotPrivate {
        /// Which input produced it.
        source: RootSource,
        /// The root that was refused.
        value: PathBuf,
        /// The world-writable tree containing it.
        shared_tree: &'static str,
    },
    /// [`HOME_VARIABLE`] is unset or empty, so no default root exists.
    HomeUnavailable,
}

impl fmt::Display for RootRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OverrideEmpty => write!(
                formatter,
                "`{OVERRIDE_VARIABLE}` is set to an empty value, which states no cache root; \
                 Tiler will not read an empty override as though the variable were unset. \
                 Set it to an absolute directory path only you can write, or to `{DISABLE_VALUE}` \
                 to expand without a cache, or unset it to use \
                 `$HOME/{}`",
                USER_CACHE_COMPONENTS.join("/"),
            ),
            Self::NotAbsolute { source, value } => write!(
                formatter,
                "`{}` is set to `{}`, which is not an absolute path, so `tiler::tensor!` cannot \
                 resolve its expansion cache root. A proc macro runs in the build tool's working \
                 directory rather than yours, and `cargo` and `rust-analyzer` need not agree on \
                 it, so a relative root would name different directories in one project. Set \
                 `{OVERRIDE_VARIABLE}` to an absolute directory path only you can write, or to \
                 `{DISABLE_VALUE}` to expand without a cache",
                source.variable(),
                value.display(),
            ),
            Self::NotPrivate {
                source,
                value,
                shared_tree,
            } => write!(
                formatter,
                "`{}` resolves the expansion cache root to `{}`, which lies under `{shared_tree}` \
                 — a directory macOS makes writable by every user of this machine. The expansion \
                 cache requires a root private to the user running Tiler: it validates every \
                 entry against corruption, and that is not a defence against another user writing \
                 internally consistent bytes of their own. Set `{OVERRIDE_VARIABLE}` to an \
                 absolute directory path only you can write — `$TMPDIR` is per-user on macOS — or \
                 to `{DISABLE_VALUE}` to expand without a cache",
                source.variable(),
                value.display(),
            ),
            Self::HomeUnavailable => write!(
                formatter,
                "`{HOME_VARIABLE}` is unset or empty, so `tiler::tensor!` cannot derive its \
                 default expansion cache root `$HOME/{}`, and it will neither pick another \
                 location nor quietly expand without a cache. Set `{OVERRIDE_VARIABLE}` to an \
                 absolute directory path only you can write, or to `{DISABLE_VALUE}` to expand \
                 without a cache",
                USER_CACHE_COMPONENTS.join("/"),
            ),
        }
    }
}

/// Resolves one expansion's cache root from an environment snapshot.
///
/// Pure and total: the same snapshot always yields the same decision, no
/// filesystem is touched, and nothing outside `environment` is read. The root is
/// deliberately *not* part of cache identity — a resolution is a function of the
/// composed subject, so moving the root changes where entries live and never
/// what they mean, and no generated token ever names a cache path.
///
/// # Errors
///
/// Returns [`RootRefusal::OverrideEmpty`] when the override is set to an empty
/// value, [`RootRefusal::NotAbsolute`] when a stated or derived root is
/// relative, [`RootRefusal::NotPrivate`] when it lies in a world-writable tree,
/// and [`RootRefusal::HomeUnavailable`] when no override is stated and `HOME` is
/// unset or empty.
pub(crate) fn resolve(environment: &RootEnvironment) -> Result<CacheRootDecision, RootRefusal> {
    match environment.stated.as_deref() {
        Some(stated) => resolve_stated(stated),
        None => resolve_user_cache(environment.home.as_deref()),
    }
}

/// Reads the override the consumer stated.
fn resolve_stated(stated: &OsStr) -> Result<CacheRootDecision, RootRefusal> {
    if stated.is_empty() {
        return Err(RootRefusal::OverrideEmpty);
    }
    if stated == OsStr::new(DISABLE_VALUE) {
        return Ok(CacheRootDecision::Disabled);
    }
    checked(PathBuf::from(stated), RootSource::Override)
}

/// Derives the default root under the per-user macOS cache directory.
fn resolve_user_cache(home: Option<&OsStr>) -> Result<CacheRootDecision, RootRefusal> {
    let home = home
        .filter(|value| !value.is_empty())
        .ok_or(RootRefusal::HomeUnavailable)?;
    let home = Path::new(home);
    if !home.is_absolute() {
        // Reported against `HOME` rather than against the joined root, because
        // the joined root is Tiler's construction and the thing the consumer can
        // correct is the variable.
        return Err(RootRefusal::NotAbsolute {
            source: RootSource::UserCache,
            value: home.to_path_buf(),
        });
    }
    let root = USER_CACHE_COMPONENTS
        .iter()
        .fold(home.to_path_buf(), |path, component| path.join(component));
    checked(root, RootSource::UserCache)
}

/// Applies the checks a path alone can decide.
///
/// Absoluteness and shared-tree membership are the *only* privacy properties
/// decidable without touching the filesystem, and this function claims nothing
/// more: a root that passes is asserted private by whoever named it, not proven
/// private by Tiler. `ExpansionCache::preflight` reports the filesystem
/// properties the publication protocol needs, and neither it nor this decides
/// ownership or mode.
fn checked(root: PathBuf, source: RootSource) -> Result<CacheRootDecision, RootRefusal> {
    if !root.is_absolute() {
        return Err(RootRefusal::NotAbsolute {
            source,
            value: root,
        });
    }
    if let Some(shared_tree) = world_writable_tree_containing(&root) {
        return Err(RootRefusal::NotPrivate {
            source,
            value: root,
            shared_tree,
        });
    }
    Ok(CacheRootDecision::Directory { root, source })
}

/// The world-writable tree containing `root`, if any.
///
/// [`Path::starts_with`] compares whole components, so `/tmpfiles` is not under
/// `/tmp` — a textual prefix test would have refused it.
fn world_writable_tree_containing(root: &Path) -> Option<&'static str> {
    WORLD_WRITABLE_TREES
        .into_iter()
        .find(|tree| root.starts_with(tree))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::{
        CacheRootDecision, DISABLE_VALUE, HOME_VARIABLE, OVERRIDE_VARIABLE, RootEnvironment,
        RootRefusal, RootSource, USER_CACHE_COMPONENTS, WORLD_WRITABLE_TREES, resolve,
    };

    fn snapshot(stated: Option<&str>, home: Option<&str>) -> RootEnvironment {
        RootEnvironment::new(stated.map(OsString::from), home.map(OsString::from))
    }

    fn expected_default(home: &str) -> PathBuf {
        USER_CACHE_COMPONENTS
            .iter()
            .fold(PathBuf::from(home), |path, component| path.join(component))
    }

    /// The default root is the per-user macOS cache directory, spelled exactly.
    #[test]
    fn the_default_root_derives_from_the_macos_user_cache_directory() {
        assert_eq!(
            resolve(&snapshot(None, Some("/Users/example"))).expect("a private absolute home"),
            CacheRootDecision::Directory {
                root: PathBuf::from("/Users/example/Library/Caches/ai.moderately.tiler/expansion"),
                source: RootSource::UserCache,
            },
        );
    }

    /// A stated override wins over the default, and is used verbatim.
    ///
    /// Verbatim matters: appending Tiler's own components to a stated path would
    /// make the variable name a directory the consumer did not name, and a
    /// consumer inspecting or clearing the cache would be looking in the wrong
    /// place.
    #[test]
    fn a_stated_override_takes_precedence_over_the_default() {
        assert_eq!(
            resolve(&snapshot(Some("/ci/tiler-cache"), Some("/Users/example")))
                .expect("an absolute private override"),
            CacheRootDecision::Directory {
                root: PathBuf::from("/ci/tiler-cache"),
                source: RootSource::Override,
            },
        );
    }

    /// `off` disables the cache even where a perfectly good default exists.
    #[test]
    fn the_disable_value_wins_over_a_usable_default() {
        let decision = resolve(&snapshot(Some(DISABLE_VALUE), Some("/Users/example")))
            .expect("`off` is a valid statement, not a failure");
        assert_eq!(decision, CacheRootDecision::Disabled);
        assert_eq!(decision.root(), None);
    }

    /// A value that merely resembles the sentinel is a root, not a disable.
    ///
    /// The paired negative of the test above: without it, "matched exactly"
    /// would also be what a case-folding or prefix comparison reported.
    #[test]
    fn a_value_resembling_the_disable_sentinel_is_not_a_disable() {
        assert_eq!(
            resolve(&snapshot(Some("/OFF"), Some("/Users/example"))).expect("an absolute path"),
            CacheRootDecision::Directory {
                root: PathBuf::from("/OFF"),
                source: RootSource::Override,
            },
        );
        assert_eq!(
            resolve(&snapshot(Some("Off"), Some("/Users/example")))
                .expect_err("`Off` is neither the sentinel nor an absolute path"),
            RootRefusal::NotAbsolute {
                source: RootSource::Override,
                value: PathBuf::from("Off"),
            },
        );
    }

    /// An exported-but-empty override is refused rather than read as unset.
    #[test]
    fn an_empty_override_is_refused_rather_than_falling_through_to_the_default() {
        assert_eq!(
            resolve(&snapshot(Some(""), Some("/Users/example")))
                .expect_err("an empty override states nothing"),
            RootRefusal::OverrideEmpty,
        );
    }

    /// A relative override is refused, because the drivers' working directory is
    /// not the consumer's to rely on.
    #[test]
    fn a_relative_override_is_refused() {
        assert_eq!(
            resolve(&snapshot(Some("relative/cache"), Some("/Users/example")))
                .expect_err("a relative root is ambiguous"),
            RootRefusal::NotAbsolute {
                source: RootSource::Override,
                value: PathBuf::from("relative/cache"),
            },
        );
    }

    /// Every world-writable tree is refused, and the refusal names which one.
    ///
    /// Parametrized over the whole list rather than over one member, because the
    /// list is the claim: a member added without a matching refusal would leave
    /// a non-private root reachable while this test still passed.
    #[test]
    fn every_world_writable_tree_is_refused_as_a_root() {
        assert!(
            !WORLD_WRITABLE_TREES.is_empty(),
            "the population this test covers must not be empty",
        );
        for tree in WORLD_WRITABLE_TREES {
            let root = format!("{tree}/tiler-cache");
            assert_eq!(
                resolve(&snapshot(Some(&root), Some("/Users/example")))
                    .expect_err("a world-writable tree is not private"),
                RootRefusal::NotPrivate {
                    source: RootSource::Override,
                    value: PathBuf::from(&root),
                    shared_tree: tree,
                },
                "root `{root}` must be refused as non-private",
            );
            assert_eq!(
                resolve(&snapshot(Some(tree), Some("/Users/example")))
                    .expect_err("the tree itself is not private either"),
                RootRefusal::NotPrivate {
                    source: RootSource::Override,
                    value: PathBuf::from(tree),
                    shared_tree: tree,
                },
            );
        }
    }

    /// A path whose *name* begins with a shared tree's name is not under it.
    ///
    /// The paired negative of the test above, and the reason the check compares
    /// components rather than text.
    #[test]
    fn a_sibling_of_a_world_writable_tree_is_not_refused() {
        assert_eq!(
            resolve(&snapshot(Some("/tmpfiles/tiler"), Some("/Users/example")))
                .expect("`/tmpfiles` is not under `/tmp`"),
            CacheRootDecision::Directory {
                root: PathBuf::from("/tmpfiles/tiler"),
                source: RootSource::Override,
            },
        );
    }

    /// macOS's per-user temporary directory stays usable, which is what makes
    /// the shared-tree refusal affordable in CI.
    #[test]
    fn the_per_user_macos_temporary_directory_is_accepted() {
        let root = "/var/folders/7k/00gbj8p92d938w7bqf3k78040000gn/T/tiler-cache";
        assert_eq!(
            resolve(&snapshot(Some(root), None)).expect("$TMPDIR is per-user on macOS"),
            CacheRootDecision::Directory {
                root: PathBuf::from(root),
                source: RootSource::Override,
            },
        );
    }

    /// A missing or empty `HOME` refuses with the override as the remedy.
    #[test]
    fn an_unusable_home_refuses_rather_than_guessing() {
        for home in [None, Some("")] {
            assert_eq!(
                resolve(&snapshot(None, home)).expect_err("no default root exists"),
                RootRefusal::HomeUnavailable,
                "home {home:?} must refuse",
            );
        }
    }

    /// A relative `HOME` is refused against `HOME`, not against the joined root.
    #[test]
    fn a_relative_home_is_refused_and_names_home() {
        let refusal = resolve(&snapshot(None, Some("example")))
            .expect_err("a relative home derives a relative root");
        assert_eq!(
            refusal,
            RootRefusal::NotAbsolute {
                source: RootSource::UserCache,
                value: PathBuf::from("example"),
            },
        );
        let rendered = refusal.to_string();
        assert!(rendered.contains(HOME_VARIABLE), "{rendered}");
    }

    /// A `HOME` inside a world-writable tree makes the *derived* root
    /// non-private, and the refusal shows the derived root.
    #[test]
    fn a_home_inside_a_world_writable_tree_refuses_the_derived_root() {
        assert_eq!(
            resolve(&snapshot(None, Some("/tmp/impostor")))
                .expect_err("a home under /tmp derives a non-private root"),
            RootRefusal::NotPrivate {
                source: RootSource::UserCache,
                value: expected_default("/tmp/impostor"),
                shared_tree: "/tmp",
            },
        );
    }

    /// The decision is a function of the snapshot and nothing else.
    #[test]
    fn resolution_is_deterministic_over_its_snapshot() {
        let environment = snapshot(None, Some("/Users/example"));
        let first = resolve(&environment);
        let second = resolve(&environment);
        assert_eq!(first, second);
        assert_eq!(
            resolve(&snapshot(None, Some("/Users/example"))),
            first,
            "an equal snapshot must resolve equally, or one project would hold two roots",
        );
    }

    /// Observation reads exactly the two names the policy is defined over, on
    /// every combination of them being present.
    ///
    /// This is the check that keeps the two build tools on one root. A third
    /// name — `CARGO_MANIFEST_DIR`, `CARGO_PKG_NAME`, `PWD`, anything a driver
    /// sets differently — would let `cargo` and `rust-analyzer` derive different
    /// roots for one project, and nothing else in this module could notice.
    ///
    /// The whole presence cross-product is covered rather than one case,
    /// because a third read is most naturally written as a *fallback* — read
    /// `HOME`, and if it is missing read something else. Asserting only the
    /// case where both variables are present would leave exactly that shape
    /// undetected; perturbing the source with such a fallback is what showed it.
    #[test]
    fn observation_reads_exactly_the_two_policy_variables() {
        let cases = [
            (None, None),
            (Some("/ci/cache"), None),
            (None, Some("/Users/example")),
            (Some("/ci/cache"), Some("/Users/example")),
        ];
        assert_eq!(
            cases.len(),
            4,
            "the population is every presence combination of two variables, counted",
        );
        for (stated, home) in cases {
            let seen = RefCell::new(Vec::new());
            let environment = RootEnvironment::observe(|name| {
                seen.borrow_mut().push(name.to_owned());
                match name {
                    _ if name == OVERRIDE_VARIABLE => stated.map(OsString::from),
                    _ if name == HOME_VARIABLE => home.map(OsString::from),
                    other => panic!("the policy read an unexpected variable `{other}`"),
                }
            });
            assert_eq!(
                seen.into_inner(),
                vec![OVERRIDE_VARIABLE.to_owned(), HOME_VARIABLE.to_owned()],
                "with override {stated:?} and home {home:?}",
            );
            assert_eq!(
                environment,
                snapshot(stated, home),
                "the snapshot must carry what the lookup returned",
            );
        }
    }

    /// Both drivers resolve one root, because neither can be told from the
    /// other by anything the policy reads.
    ///
    /// `rust-analyzer` populates a proc macro's environment from the crate graph
    /// it loaded, so `CARGO_PKG_NAME` is present under both drivers and does not
    /// distinguish them. The snapshot each driver produces for a given user is
    /// therefore the same snapshot, and the test asserts the consequence rather
    /// than restating the premise.
    #[test]
    fn both_drivers_resolve_one_root() {
        let under_cargo = snapshot(None, Some("/Users/example"));
        let under_analyzer = snapshot(None, Some("/Users/example"));
        assert_eq!(resolve(&under_cargo), resolve(&under_analyzer));
        assert_eq!(
            resolve(&under_cargo)
                .expect("a private absolute home")
                .root()
                .map(Path::to_path_buf),
            Some(expected_default("/Users/example")),
        );
    }

    /// Every refusal names the override variable and the disable value, so a
    /// consumer reading one compile error has both remedies without a document.
    #[test]
    fn every_refusal_states_both_remedies() {
        let refusals = [
            RootRefusal::OverrideEmpty,
            RootRefusal::NotAbsolute {
                source: RootSource::Override,
                value: PathBuf::from("relative"),
            },
            RootRefusal::NotAbsolute {
                source: RootSource::UserCache,
                value: PathBuf::from("relative"),
            },
            RootRefusal::NotPrivate {
                source: RootSource::Override,
                value: PathBuf::from("/tmp/cache"),
                shared_tree: "/tmp",
            },
            RootRefusal::NotPrivate {
                source: RootSource::UserCache,
                value: PathBuf::from("/tmp/impostor/Library/Caches"),
                shared_tree: "/tmp",
            },
            RootRefusal::HomeUnavailable,
        ];
        assert_eq!(
            refusals.len(),
            6,
            "the population this test covers is every refusal shape, counted",
        );
        for refusal in refusals {
            let rendered = refusal.to_string();
            assert!(
                rendered.contains(OVERRIDE_VARIABLE),
                "refusal must name the override variable: {rendered}",
            );
            assert!(
                rendered.contains(DISABLE_VALUE),
                "refusal must name the disable value: {rendered}",
            );
        }
    }

    /// A non-private refusal names the tree it refused and the per-user remedy.
    #[test]
    fn the_non_private_refusal_names_the_tree_and_the_remedy() {
        let rendered = RootRefusal::NotPrivate {
            source: RootSource::Override,
            value: PathBuf::from("/Users/Shared/tiler"),
            shared_tree: "/Users/Shared",
        }
        .to_string();
        assert!(rendered.contains("/Users/Shared/tiler"), "{rendered}");
        assert!(rendered.contains("$TMPDIR"), "{rendered}");
        assert!(
            rendered.contains("private to the user running Tiler"),
            "{rendered}"
        );
    }
}
