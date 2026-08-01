//! The complete content identity of one offline Metal compilation.
//!
//! ADR 0050 stores "one immutable self-validating bundle per **complete
//! compilation key**", and every other property that record decides rests on
//! that key being complete: an omitted input does not make a cache slower, it
//! makes a validated hit return an artifact built from different inputs. This
//! module is the driver's half of that key — the canonical subject naming every
//! input that determines the `metallib` bytes a compilation produces.
//!
//! # "Complete" of the compilation, not of the artifact
//!
//! Read the first line exactly: this subject is complete with respect to the
//! *compilation* and not with respect to the *artifact*. A cache bundle carries a
//! whole artifact envelope — the plan portfolio, ABI bindings, routing, declared
//! target requirements, and selected providers wrapped around the compiled object
//! — and these bytes say nothing about any of that. Two artifacts agreeing on
//! source, flags, and toolchain and differing in their plan portfolio produce one
//! subject here, which is why this is a *facet* of a cache key and never a cache
//! key on its own.
//!
//! `tiler_cache::expansion::ComposedSubject` is where the facets are joined. It
//! **wraps** these bytes rather than restating them: they appear unaltered as one
//! run of its backend-compilations facet, so the evidence tag encoded below
//! travels with them and the `SameHost` reuse bound survives composition
//! untouched. Nothing here needs to know that, and this crate acquires no
//! dependency to say it; the note exists so a reader of *these* bytes does not
//! mistake them for the whole key.
//! # It is bytes, not a digest, for the reason `family.rs` gives
//!
//! This crate's dependency closure is empty by decision (ADR 0077 item 2), so
//! it owns no hash function, and the governed content digest is
//! `tiler.digest.sha-256.v1` in `tiler-artifact`. A digest computed here would
//! be a second identity authority over the same subject. The subject is
//! therefore emitted as canonical bytes and the caller that already owns the
//! governed algorithm digests them, exactly as
//! [`ArtifactFamilySelection::canonical_bytes`](crate::family::ArtifactFamilySelection::canonical_bytes)
//! does (ADR 0074 convention 2).
//!
//! # What this module is *not*: the cache protocol
//!
//! It is the key subject and nothing else. It holds no cache root, no
//! namespace, no lock, no bundle framing, and no publication step. Those belong
//! to `tiler-cache`, which ADR 0082 admitted as the expansion cache owner on
//! Tom's decision of 2026-07-25, keeping this crate's dependency closure empty
//! and ADR 0077 item 1 standing. Emitting the key subject here is independent of
//! that decision, because the subject is a fact about *this crate's* inputs
//! whichever component consumes it.
//!
//! # The evidence class is the load-bearing field
//!
//! [`CompilationIdentity`] does not merely name the compilation's inputs. It
//! also names how well the *toolchain* that will run it is identified, and it
//! bounds where the resulting entry may be reused accordingly. The reasoning is
//! under [`ToolchainEvidence`]; the short form is that reported version strings
//! cannot distinguish two component builds, so the identity this crate can
//! establish today is sound for reuse on the host that observed the toolchain
//! and is not sound across hosts. That refusal is typed
//! ([`IdentityError::CrossHostReuseUnsupported`]) rather than approximated.
//!
//! # Public boundary and construction authority
//!
//! What is public is the derived subject and the reuse bound it licenses; the
//! encoding is not. [`CompilationIdentity`] has no public constructor. A caller
//! obtains it only from
//! [`PreparedCompilation::identity`](crate::driver::PreparedCompilation::identity)
//! after [`Toolchain::prepare`](crate::driver::Toolchain::prepare) resolves the
//! absolute paths that the same prepared token will execute. This is ADR 0074
//! convention 2 made structural: no caller can mint identity from an invented
//! [`ResolvedToolchain`], and cache lookup cannot be keyed by one resolution
//! before a miss silently compiles through another.
//!
//! `CompilationIdentity::encode`, the domain tag, and the framing helpers
//! remain private or crate-private, so no out-of-crate caller can frame a run
//! under this crate's convention or present bytes this module did not derive.
//! The identity type remains public because bytes alone do not carry the reuse
//! scope: two hosts whose tools report equal versions derive identical bytes,
//! while [`ToolchainEvidence::ReportedVersions`] licenses same-host reuse only.
//!
//! ```no_run
//! use tiler_metal_aot::driver::Toolchain;
//! use tiler_metal_aot::identity::{IdentityReuseScope, ToolchainEvidence};
//! use tiler_metal_aot::input::{
//!     ApplePlatform, CompileRequest, DeploymentMinimum, MetalTarget, MslVersion,
//!     NumericalRealization, OptimizationLevel,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let request = CompileRequest::new(
//!     "kernel void main0() {}",
//!     MetalTarget::new(
//!         ApplePlatform::MacOs,
//!         DeploymentMinimum::new(14, 0),
//!         MslVersion::Metal3_1,
//!     )?,
//!     OptimizationLevel::Default,
//!     NumericalRealization::strict_baseline(),
//! );
//! let prepared = Toolchain::system().prepare(&request)?;
//! assert!(!prepared.identity().as_bytes().is_empty());
//! assert_eq!(
//!     prepared.identity().evidence(),
//!     ToolchainEvidence::ReportedVersions,
//! );
//! assert_eq!(
//!     prepared.identity().reuse_scope(),
//!     IdentityReuseScope::SameHost,
//! );
//! let _artifact = prepared.compile()?;
//! # Ok(())
//! # }
//! ```

use core::fmt;

use crate::input::CompileRequest;
use crate::record::{ResolvedTool, ResolvedToolchain, SdkIdentity};

/// Versioned domain tag opening the canonical compilation-key bytes.
///
/// ADR 0074 convention 3: bytes produced for one subject can never be mistaken
/// for another subject's. In a cache this is not a style rule — a bundle whose
/// embedded key was derived over a different subject must be rejected rather
/// than compared field by field against a key that happens to be the same
/// width.
const COMPILATION_DOMAIN: &[u8] = b"tiler.metal-aot.compilation-identity.v1\0";

/// How well the toolchain that runs a compilation is identified.
///
/// # Why this is a field of identity rather than of provenance
///
/// [`ArtifactProvenance`](crate::record::ArtifactProvenance) already records the
/// toolchain, and [`CompilerFingerprint`](crate::record::CompilerFingerprint) already documents the gap: "two
/// component builds can report the same front-end version string". A cache key
/// that folded those version strings in and said nothing else would be
/// *indistinguishable* from a complete key while being wrong across hosts — the
/// two would agree on every observable byte and disagree only about what the
/// bytes license. Naming the evidence class in the subject is what makes the
/// difference observable: an entry published under one class can never be
/// served to a consumer requiring another, because the classes produce
/// different bytes.
///
/// # The cross-host question `prototype-apple-aot-driver` deferred here
///
/// **Decision.** Cross-host identity does require a content digest of the
/// `metal` and `metallib` tool binaries and of the SDK. Reported versions plus
/// the SDK's canonical name, version, and build identifier are *not* a
/// sufficient discriminator, for the reason [`CompilerFingerprint`](crate::record::CompilerFingerprint) states
/// about the compiler component; and the resolved tool paths cannot close the
/// gap, because a path is a fact about where a host keeps a file rather than
/// about what the file contains — two hosts with identical toolchains at
/// different paths would fail to share, and two hosts with different toolchains
/// at the same path would collide. Paths are therefore excluded from the
/// subject (ADR 0074 convention 3's exclusion of transient identifiers) rather
/// than added to it.
///
/// **What a reported version now describes, which changed.** The version folded
/// here is read by running the binary that was located, and that same binary is
/// the one the compilation executes. It was previously a second, independent
/// `xcrun <tool> --version` selection, so the folded version could describe a
/// binary other than the one producing the bytes — on one host, at one instant,
/// with nothing comparing them. The evidence class is unchanged and still does
/// not license cross-host reuse; what changed is that on the host that compiled,
/// the version is a fact about the compiler that ran.
///
/// **What is implemented.** Only [`Self::ReportedVersions`], which is what
/// [`ResolvedToolchain`] can supply. A content-digest class is a *reserved*
/// extension point, not implemented support and not a tested guarantee: adding
/// it is a compile error at `CompilationIdentity::encode` because the
/// encoder matches this enum exhaustively, and the tag it writes moves the
/// bytes, so no entry published under the weaker class can be mistaken for one
/// published under the stronger.
///
/// Deliberately **not** `#[non_exhaustive]`. This is an ADR 0074 convention 5b
/// type: the encoder maps it totally onto an identity tag, and a wildcard arm
/// there could only invent a tag the variant alone determines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolchainEvidence {
    /// The `metal` and `metallib` reported version strings, plus the SDK's
    /// canonical name, version, and build identifier.
    ///
    /// This is everything the driver observes without hashing a file, and it is
    /// the whole of what a [`ResolvedToolchain`] carries that is portable.
    ReportedVersions,
}

impl ToolchainEvidence {
    /// Returns where an entry keyed under this evidence class may be reused.
    ///
    /// A total map: each class determines its own scope, so a wildcard arm
    /// could only widen or narrow a claim the class alone decides.
    #[must_use]
    pub const fn reuse_scope(self) -> IdentityReuseScope {
        match self {
            Self::ReportedVersions => IdentityReuseScope::SameHost,
        }
    }

    /// Returns this class's stable identity tag.
    ///
    /// An arm that states its constant, never a discriminant read from
    /// declaration order (ADR 0074 convention 3, as amended 2026-07-24).
    const fn tag(self) -> u8 {
        match self {
            Self::ReportedVersions => 0x01,
        }
    }

    /// Returns this class's stable lowercase identifier, for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReportedVersions => "reported-versions",
        }
    }
}

impl fmt::Display for ToolchainEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Where an entry keyed by a [`CompilationIdentity`] may soundly be reused.
///
/// This is a property of the key, not a cache configuration option. A cache may
/// choose a narrower policy than the key permits; it must never choose a wider
/// one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityReuseScope {
    /// Sound only on the host whose toolchain observation produced the key.
    ///
    /// A cache root under this scope must be host-local. Placing one on a shared
    /// or network volume would let a host serve an entry compiled by a component
    /// build it cannot distinguish from its own.
    SameHost,
    /// Sound on any host.
    ///
    /// **Reserved, not reachable.** No [`ToolchainEvidence`] class yields this
    /// today; see that type for what a class would have to establish first.
    /// It exists so [`CompilationIdentity::require_cross_host_reuse`] is a total
    /// function over a stated target state rather than a permanent refusal
    /// dressed as one, and so the day a digest class lands the widening is a
    /// visible change to a match arm.
    CrossHost,
}

impl IdentityReuseScope {
    /// Returns this scope's stable lowercase identifier, for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SameHost => "same-host",
            Self::CrossHost => "cross-host",
        }
    }
}

impl fmt::Display for IdentityReuseScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The complete content identity of one offline Metal compilation.
///
/// A *derived* identity in the sense of ADR 0074 convention 2: it has no
/// wrapping constructor, so no caller can assemble one naming a compilation
/// nobody encoded. Its storage is private and [`Self::as_bytes`] is its only
/// reader; equality, ordering, and cache keying use those bytes.
///
/// ```compile_fail,E0624
/// use tiler_metal_aot::identity::CompilationIdentity;
/// let _forbidden_constructor = CompilationIdentity::new;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompilationIdentity {
    bytes: Vec<u8>,
    evidence: ToolchainEvidence,
}

impl CompilationIdentity {
    /// Derives the complete key subject for one request against one resolved
    /// toolchain.
    ///
    /// The toolchain is required rather than optional. A key derived from the
    /// request alone would be complete with respect to what the caller asked
    /// for and silently incomplete with respect to what would compile it, which
    /// is the failure mode the whole module exists to prevent.
    pub(crate) fn new(request: &CompileRequest, toolchain: &ResolvedToolchain) -> Self {
        let evidence = ToolchainEvidence::ReportedVersions;
        Self {
            bytes: Self::encode(request, toolchain, evidence),
            evidence,
        }
    }

    /// Returns the canonical bytes identifying this compilation.
    ///
    /// The caller that owns `tiler.digest.sha-256.v1` digests these to obtain
    /// the fixed-width cache key; this crate deliberately does not.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the evidence class the toolchain was identified by.
    #[must_use]
    pub const fn evidence(&self) -> ToolchainEvidence {
        self.evidence
    }

    /// Returns where an entry under this identity may soundly be reused.
    #[must_use]
    pub const fn reuse_scope(&self) -> IdentityReuseScope {
        self.evidence.reuse_scope()
    }

    /// Checks that this identity licenses reuse on a host other than the one
    /// that derived it.
    ///
    /// # Errors
    ///
    /// Returns [`IdentityError::CrossHostReuseUnsupported`] when the evidence
    /// class bounds reuse to one host. The refusal is explicit because the
    /// alternative — returning a key that *looks* complete and is not — is the
    /// silent wrong-artifact failure ADR 0050's complete-identity requirement
    /// exists to exclude.
    pub const fn require_cross_host_reuse(&self) -> Result<(), IdentityError> {
        match self.reuse_scope() {
            IdentityReuseScope::CrossHost => Ok(()),
            IdentityReuseScope::SameHost => Err(IdentityError::CrossHostReuseUnsupported {
                evidence: self.evidence,
            }),
        }
    }

    /// Encodes the canonical subject.
    ///
    /// Domain-separated, length-prefixed before every variable-length run, free
    /// of declaration ordinals and of local filesystem paths, and exhaustive
    /// over every enum it writes (ADR 0074 convention 3).
    ///
    /// The request is destructured irrefutably so that a field added to
    /// [`CompileRequest`] fails to compile here rather than silently leaving
    /// identity, and the resolved toolchain records likewise. That is the
    /// mechanism, not the field list below: the list is what conforms today,
    /// the destructuring is what keeps it conforming.
    fn encode(
        request: &CompileRequest,
        toolchain: &ResolvedToolchain,
        evidence: ToolchainEvidence,
    ) -> Vec<u8> {
        let CompileRequest {
            source,
            target,
            optimization: _,
            numerical: _,
        } = request;
        let ResolvedToolchain {
            sdk,
            metal,
            metallib,
        } = toolchain;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(COMPILATION_DOMAIN);
        bytes.push(evidence.tag());

        // The SDK selector is an invocation input that never appears in the
        // compiler flags: it is passed to `xcrun --sdk`, and it is what selects
        // which `metal` binary runs at all.
        push_str(&mut bytes, target.sdk().selector());
        push_str(&mut bytes, target.platform().as_str());
        push_str(&mut bytes, &target.triple());

        // The exact ordered invocations rather than the structured choices they
        // are derived from. `optimization` and `numerical` are destructured
        // above and deliberately not encoded a second time: encoding both would
        // create two records of one fact that a future change could move apart,
        // and the flags are the ground truth of what the compiler is asked to
        // do. An output-affecting flag added to `CompileRequest::compile_flags`
        // reaches identity with no edit here.
        push_strs(&mut bytes, &request.compile_flags());
        push_strs(&mut bytes, &request.link_flags());

        push_sdk(&mut bytes, sdk);
        push_tool_version(&mut bytes, metal);
        push_tool_version(&mut bytes, metallib);

        push_str(&mut bytes, source);
        bytes
    }
}

/// Encodes the SDK's portable identity.
///
/// Every field is destructured irrefutably rather than read through `.`, so a
/// field added to [`SdkIdentity`] is a compile error here and its presence in
/// or absence from identity becomes a decision instead of a default.
fn push_sdk(bytes: &mut Vec<u8>, sdk: &SdkIdentity) {
    let SdkIdentity {
        canonical_name,
        version,
        build,
    } = sdk;
    push_str(bytes, canonical_name);
    push_str(bytes, version);
    push_str(bytes, build);
}

/// Encodes one tool's reported version, excluding its local path.
///
/// The path is destructured and dropped rather than ignored with `..`, so a
/// field added to [`ResolvedTool`] is a compile error here and its exclusion
/// from identity becomes a decision instead of a default.
fn push_tool_version(bytes: &mut Vec<u8>, tool: &ResolvedTool) {
    let ResolvedTool { path: _, version } = tool;
    push_str(bytes, version);
}

/// Writes a fixed-width big-endian count before a repeated run.
///
/// This crate's sole copy of the workspace's canonical length framing, and the
/// sole one it is permitted. `tiler_ir::identity` owns that framing everywhere
/// else, but ADR 0077 item 2 pins this crate's dependency closure empty — it
/// declares no workspace dependency at all — so the framing cannot be imported
/// here and has to be restated. A gate once admitted exactly this definition
/// and [`push_str`] beside it, so a second copy in this crate failed rather
/// than growing quietly; `e197176` deleted that gate along with the rest of the
/// Python tooling and gave it no successor. **A third copy appearing here is
/// now caught only by review of the diff that adds it.**
///
/// `u64` matches the workspace's canonical form and is wide enough for every
/// run a 64-bit host can address, so there is no bound here that could reject
/// or truncate a real subject. What makes the conversion total is the
/// supported-platform policy — `AGENTS.md` states Tiler develops on macOS only,
/// and every admitted target is 64-bit — rather than a check: the gate that
/// once asserted it is gone.
pub(crate) fn push_len(bytes: &mut Vec<u8>, len: usize) {
    let len = u64::try_from(len).expect("the admitted profiles have a 64-bit address space");
    bytes.extend_from_slice(&len.to_be_bytes());
}

/// Writes a fixed-width big-endian length before a variable-length run.
///
/// Admitted alongside [`push_len`] for the reason stated there, and a `&str`
/// rather than a `&[u8]` run because every variable-width field this crate
/// encodes is textual.
pub(crate) fn push_str(bytes: &mut Vec<u8>, value: &str) {
    push_len(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

/// Writes a counted sequence of length-prefixed runs.
fn push_strs(bytes: &mut Vec<u8>, values: &[String]) {
    push_len(bytes, values.len());
    for value in values {
        push_str(bytes, value);
    }
}

/// Why a compilation identity does not license a requested use.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a rejection vocabulary a
/// caller forwards or partially classifies, which no crate maps totally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdentityError {
    /// The toolchain evidence class bounds reuse to the host that derived it.
    CrossHostReuseUnsupported {
        /// The class that was established.
        evidence: ToolchainEvidence,
    },
}

impl fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CrossHostReuseUnsupported { evidence } => write!(
                formatter,
                "toolchain evidence `{evidence}` identifies the compiler by reported version \
                 only, which cannot distinguish two component builds; reuse is bounded to {}",
                evidence.reuse_scope(),
            ),
        }
    }
}

impl std::error::Error for IdentityError {}

#[cfg(test)]
mod tests {
    use super::{
        COMPILATION_DOMAIN, CompilationIdentity, IdentityError, IdentityReuseScope,
        ToolchainEvidence,
    };
    use crate::input::{
        ApplePlatform, CompileRequest, DeploymentMinimum, Fp32Functions, FpContract, MathMode,
        MetalTarget, MslVersion, NumericalRealization, OptimizationLevel,
    };
    use crate::record::{ResolvedTool, ResolvedToolchain, SdkIdentity};
    use std::path::PathBuf;

    fn toolchain() -> ResolvedToolchain {
        ResolvedToolchain {
            sdk: SdkIdentity {
                canonical_name: "macosx".to_owned(),
                version: "26.5".to_owned(),
                build: "25F70".to_owned(),
            },
            metal: ResolvedTool {
                path: PathBuf::from("/usr/bin/metal"),
                version: "Metal 32023.883".to_owned(),
            },
            metallib: ResolvedTool {
                path: PathBuf::from("/usr/bin/metallib"),
                version: "metallib 32023.883".to_owned(),
            },
        }
    }

    fn request() -> CompileRequest {
        CompileRequest::new(
            "kernel void main0() {}",
            MetalTarget::new(
                ApplePlatform::MacOs,
                DeploymentMinimum::new(14, 0),
                MslVersion::Metal3_1,
            )
            .expect("the fixture target is valid"),
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
    }

    fn bytes_of(request: &CompileRequest, toolchain: &ResolvedToolchain) -> Vec<u8> {
        CompilationIdentity::new(request, toolchain)
            .as_bytes()
            .to_vec()
    }

    fn baseline() -> Vec<u8> {
        bytes_of(&request(), &toolchain())
    }

    /// The subject opens with its versioned domain tag and carries content after
    /// it.
    #[test]
    fn the_subject_is_domain_separated() {
        let bytes = baseline();
        assert!(bytes.starts_with(COMPILATION_DOMAIN));
        assert!(
            bytes.len() > COMPILATION_DOMAIN.len(),
            "the domain tag must precede content rather than be the whole subject",
        );
    }

    /// Deriving the same compilation twice yields the same bytes.
    ///
    /// The negative tests below only mean something paired with this one: they
    /// show facets *move* the bytes, and this shows nothing else does.
    #[test]
    fn the_subject_is_a_function_of_its_inputs_alone() {
        assert_eq!(baseline(), baseline());
        assert_eq!(
            CompilationIdentity::new(&request(), &toolchain()),
            CompilationIdentity::new(&request(), &toolchain()),
        );
    }

    /// Every request facet that changes the produced bytes changes the key.
    ///
    /// An omitted facet is the failure ADR 0050's complete-key requirement
    /// exists to exclude: a validated hit would return an artifact compiled from
    /// different inputs, and nothing downstream could detect it.
    #[test]
    fn every_request_facet_reaches_the_subject() {
        let baseline = baseline();
        let toolchain = toolchain();

        let mut other_source = request();
        other_source.source = "kernel void main1() {}".to_owned();
        assert_ne!(
            baseline,
            bytes_of(&other_source, &toolchain),
            "the MSL source must reach identity",
        );

        let mut other_sdk = request();
        other_sdk.target = MetalTarget::new(
            ApplePlatform::IOsSimulator,
            DeploymentMinimum::new(17, 0),
            MslVersion::Metal3_1,
        )
        .expect("the alternate target is valid");
        assert_ne!(
            baseline,
            bytes_of(&other_sdk, &toolchain),
            "the selected SDK must reach identity",
        );

        let mut other_minimum = request();
        other_minimum.target = MetalTarget::new(
            ApplePlatform::MacOs,
            DeploymentMinimum::new(15, 0),
            MslVersion::Metal3_1,
        )
        .expect("the alternate target is valid");
        assert_ne!(
            baseline,
            bytes_of(&other_minimum, &toolchain),
            "the deployment minimum must reach identity",
        );

        let mut other_standard = request();
        other_standard.target = MetalTarget::new(
            ApplePlatform::MacOs,
            DeploymentMinimum::new(14, 0),
            MslVersion::Metal3_0,
        )
        .expect("the alternate target is valid");
        assert_ne!(
            baseline,
            bytes_of(&other_standard, &toolchain),
            "the language standard must reach identity",
        );

        let mut other_optimization = request();
        other_optimization.optimization = OptimizationLevel::Aggressive;
        assert_ne!(
            baseline,
            bytes_of(&other_optimization, &toolchain),
            "the optimization level must reach identity",
        );
    }

    /// Each numerical realization flag independently reaches the subject.
    ///
    /// Asserted per dimension rather than through one relaxed realization: the
    /// three permissions are independent, and a subject that folded them into
    /// one bit would let two differently rounded compilations share a key.
    #[test]
    fn every_numerical_dimension_reaches_the_subject() {
        let baseline = baseline();
        let toolchain = toolchain();
        let strict = NumericalRealization::strict_baseline();

        for realization in [
            NumericalRealization::new(MathMode::Relaxed, strict.fp32_functions, strict.fp_contract),
            NumericalRealization::new(MathMode::Fast, strict.fp32_functions, strict.fp_contract),
            NumericalRealization::new(strict.math_mode, Fp32Functions::Fast, strict.fp_contract),
            NumericalRealization::new(strict.math_mode, strict.fp32_functions, FpContract::On),
            NumericalRealization::new(strict.math_mode, strict.fp32_functions, FpContract::Fast),
        ] {
            let mut altered = request();
            altered.numerical = realization;
            assert_ne!(
                baseline,
                bytes_of(&altered, &toolchain),
                "{realization:?} must not share a key with the strict baseline",
            );
        }
    }

    /// Every portable toolchain facet reaches the subject.
    ///
    /// This is the half a request-only key would omit. Two identical requests
    /// compiled by different toolchains are different compilations.
    #[test]
    fn every_portable_toolchain_facet_reaches_the_subject() {
        let baseline = baseline();
        let request = request();

        let mut other_metal = toolchain();
        other_metal.metal.version = "Metal 32024.1".to_owned();
        assert_ne!(
            baseline,
            bytes_of(&request, &other_metal),
            "the metal compiler version must reach identity",
        );

        let mut other_metallib = toolchain();
        other_metallib.metallib.version = "metallib 32024.1".to_owned();
        assert_ne!(
            baseline,
            bytes_of(&request, &other_metallib),
            "the metallib linker version must reach identity",
        );

        let mut other_sdk_version = toolchain();
        other_sdk_version.sdk.version = "26.6".to_owned();
        assert_ne!(
            baseline,
            bytes_of(&request, &other_sdk_version),
            "the SDK version must reach identity",
        );

        let mut other_sdk_build = toolchain();
        other_sdk_build.sdk.build = "25G01".to_owned();
        assert_ne!(
            baseline,
            bytes_of(&request, &other_sdk_build),
            "the SDK build identifier must reach identity",
        );

        let mut other_sdk_name = toolchain();
        other_sdk_name.sdk.canonical_name = "iphoneos".to_owned();
        assert_ne!(
            baseline,
            bytes_of(&request, &other_sdk_name),
            "the SDK canonical name must reach identity",
        );
    }

    /// Local filesystem paths are deliberately excluded from the subject.
    ///
    /// A path states where a host keeps a file, not what the file contains.
    /// Including it would split the cache across two hosts with identical
    /// toolchains at different paths, and would still not join two hosts with
    /// different toolchains at the same path — so it buys no soundness and
    /// costs every legitimate hit. The soundness it does not buy is what
    /// [`ToolchainEvidence`] states instead.
    ///
    /// The tool paths are the only local paths a resolution still carries: the
    /// SDK's path is excluded by its absence from [`SdkIdentity`] rather than by
    /// this test, and `push_sdk`'s irrefutable destructure is what makes
    /// re-admitting one a compile error rather than a silent identity change.
    #[test]
    fn local_paths_are_excluded_from_the_subject() {
        let baseline = baseline();
        let request = request();

        let mut moved = toolchain();
        moved.metal.path = PathBuf::from("/opt/tools/metal");
        moved.metallib.path = PathBuf::from("/opt/tools/metallib");
        assert_eq!(
            baseline,
            bytes_of(&request, &moved),
            "relocating an identical toolchain must not change the key",
        );
    }

    /// Adjacent runs cannot be re-split into a different subject.
    ///
    /// Without the length prefixes, a source ending in a tool version and a
    /// tool version beginning with the tail of a source could concatenate to
    /// the same bytes.
    #[test]
    fn length_prefixes_keep_adjacent_runs_unambiguous() {
        let mut left = request();
        left.source = "ab".to_owned();
        let mut left_chain = toolchain();
        left_chain.metal.version = "cd".to_owned();

        let mut right = request();
        right.source = "a".to_owned();
        let mut right_chain = toolchain();
        right_chain.metal.version = "bcd".to_owned();

        assert_ne!(bytes_of(&left, &left_chain), bytes_of(&right, &right_chain));
    }

    /// The evidence class reaches the subject.
    ///
    /// Only one class is inhabited today, so this asserts the mechanism rather
    /// than a difference between two classes: the tag is written, it is written
    /// from a stated constant, and it sits where a second class would move the
    /// bytes of every entry.
    #[test]
    fn the_evidence_class_reaches_the_subject() {
        let identity = CompilationIdentity::new(&request(), &toolchain());
        assert_eq!(identity.evidence(), ToolchainEvidence::ReportedVersions);
        assert_eq!(
            identity.as_bytes()[COMPILATION_DOMAIN.len()],
            ToolchainEvidence::ReportedVersions.tag(),
            "the evidence tag follows the domain tag",
        );
    }

    /// Reported versions bound reuse to one host, and asking for more is refused.
    ///
    /// This is the answer to the cross-host question `prototype-apple-aot-driver`
    /// deferred: the driver cannot certify a toolchain across hosts, so it
    /// refuses rather than returning a key that looks complete.
    #[test]
    fn reported_versions_do_not_license_cross_host_reuse() {
        let identity = CompilationIdentity::new(&request(), &toolchain());
        assert_eq!(identity.reuse_scope(), IdentityReuseScope::SameHost);
        assert_eq!(
            identity.require_cross_host_reuse(),
            Err(IdentityError::CrossHostReuseUnsupported {
                evidence: ToolchainEvidence::ReportedVersions,
            }),
        );
    }

    /// The refusal names the class and the scope rather than a bare message.
    #[test]
    fn the_refusal_states_the_evidence_class_and_the_scope() {
        let rendered = IdentityError::CrossHostReuseUnsupported {
            evidence: ToolchainEvidence::ReportedVersions,
        }
        .to_string();
        assert!(rendered.contains("reported-versions"), "{rendered}");
        assert!(rendered.contains("same-host"), "{rendered}");
    }

    /// Every evidence class states a reuse scope, and today's is the narrow one.
    ///
    /// One class is inhabited, so the coverage claim rests on the match rather
    /// than on iterating a list: it is exhaustive over [`ToolchainEvidence`]
    /// with no wildcard, so a class added without deciding its scope fails to
    /// compile here rather than inheriting `SameHost` by default. The assertion
    /// is against a second statement of the mapping, not against
    /// `reuse_scope`'s own result, so this cannot agree with a wrong scope by
    /// repeating it.
    #[test]
    fn every_evidence_class_states_its_reuse_scope() {
        let evidence = ToolchainEvidence::ReportedVersions;
        let expected = match evidence {
            ToolchainEvidence::ReportedVersions => IdentityReuseScope::SameHost,
        };
        assert_eq!(evidence.reuse_scope(), expected, "{evidence}");
    }
}
