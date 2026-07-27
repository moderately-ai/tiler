//! Provenance, fingerprint, and compiled-artifact records.
//!
//! These are the driver's outputs. They mirror the identity dimensions the
//! Apple artifact-compatibility research requires — platform family, deployment
//! minimum, language version, SDK identity, compiler and linker identity, flags,
//! and numerical realization — but they are the driver's own provenance record,
//! not the target-neutral artifact bundle, which a later ticket assembles.

use std::path::PathBuf;

use crate::input::{
    ApplePlatform, DeploymentMinimum, MslVersion, NumericalRealization, OptimizationLevel,
};

/// The resolved identity of one offline tool (`metal` or `metallib`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTool {
    /// The absolute path `xcrun` resolved for the tool. This is local
    /// provenance: on this host it encodes the resolved Metal toolchain
    /// component, but it is not portable identity across hosts.
    pub path: PathBuf,
    /// The tool's reported version string.
    pub version: String,
}

/// The resolved identity of the single selected SDK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdkIdentity {
    /// The canonical `--sdk` selector, for example `macosx`.
    pub canonical_name: String,
    /// The SDK's canonical version, for example `26.5`.
    pub version: String,
    /// The SDK's build identifier, for example `25F70`.
    pub build: String,
    /// The absolute SDK path. This is local provenance, not portable identity.
    pub path: PathBuf,
}

/// The portable compiler fingerprint: the `metal` and `metallib` component
/// versions.
///
/// Two component builds can report the same front-end version string, so this
/// portable fingerprint is paired with the resolved local tool paths in
/// [`ArtifactProvenance`]. A content digest of the tool binaries and SDK is a
/// deferred strengthening for cross-host identity; it belongs with the
/// expansion cache, not this bounded driver.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompilerFingerprint {
    /// The `metal` compiler version string.
    pub metal_version: String,
    /// The `metallib` linker version string.
    pub metallib_version: String,
}

/// The resolved toolchain for one selected SDK, captured before compiling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedToolchain {
    /// The resolved SDK identity.
    pub sdk: SdkIdentity,
    /// The resolved `metal` compiler.
    pub metal: ResolvedTool,
    /// The resolved `metallib` linker.
    pub metallib: ResolvedTool,
}

impl ResolvedToolchain {
    /// Returns the portable compiler fingerprint of this resolved toolchain.
    #[must_use]
    pub fn fingerprint(&self) -> CompilerFingerprint {
        CompilerFingerprint {
            metal_version: self.metal.version.clone(),
            metallib_version: self.metallib.version.clone(),
        }
    }
}

/// The full provenance of one compiled `metallib`.
///
/// It records every explicit semantic and compiler input plus the resolved
/// toolchain identity, so an artifact's identity is legible without re-running
/// the compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ArtifactProvenance {
    /// The artifact platform family.
    pub platform: ApplePlatform,
    /// The normalized `air64-apple-*` target triple.
    pub target_triple: String,
    /// The requested deployment minimum.
    pub deployment_minimum: DeploymentMinimum,
    /// The selected MSL language version.
    pub msl_version: MslVersion,
    /// The selected optimization level.
    pub optimization: OptimizationLevel,
    /// The explicit numerical realization flags.
    pub numerical: NumericalRealization,
    /// The resolved SDK identity.
    pub sdk: SdkIdentity,
    /// The resolved `metal` compiler.
    pub metal: ResolvedTool,
    /// The resolved `metallib` linker.
    pub metallib: ResolvedTool,
    /// The portable compiler fingerprint.
    pub fingerprint: CompilerFingerprint,
    /// The exact ordered `metal` flags used, excluding file paths.
    pub compile_flags: Vec<String>,
    /// The exact ordered `metallib` flags used, excluding file paths.
    pub link_flags: Vec<String>,
}

/// A compiled `metallib` and its complete provenance.
/// # Growth
///
/// `#[non_exhaustive]`, because this is a value the driver produces and a
/// consumer reads: `tiler-metal` and the serial-sum producer both read its
/// fields and neither builds one, so a further recorded fact is additive for
/// them and remains a compile error for every construction site *inside* this
/// crate.
///
/// An out-of-crate literal is refused:
///
/// ```compile_fail,E0639
/// use tiler_metal_aot::record::CompiledArtifact;
/// let artifact = CompiledArtifact { };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CompiledArtifact {
    /// The compiled Metal library bytes.
    pub metallib: Vec<u8>,
    /// The full provenance of this compilation.
    pub provenance: ArtifactProvenance,
}
