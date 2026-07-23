//! The bounded offline Apple Metal compiler driver.
//!
//! [`Toolchain`](crate::driver::Toolchain) wraps the Apple offline toolchain,
//! reached through an `xcrun`-compatible launcher.
//! [`Toolchain::resolve`](crate::driver::Toolchain::resolve) captures the
//! fingerprint and provenance for one selected SDK;
//! [`Toolchain::compile`](crate::driver::Toolchain::compile) compiles MSL into
//! `metallib` bytes with that provenance. Both fail closed with a typed
//! [`DriverError`](crate::diagnostic::DriverError).

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::diagnostic::{CompileStage, DriverError};
use crate::input::{AppleSdk, CompileRequest};
use crate::record::{
    ArtifactProvenance, CompiledArtifact, ResolvedTool, ResolvedToolchain, SdkIdentity,
};

/// The four magic bytes that begin every Metal library file.
const METALLIB_MAGIC: [u8; 4] = *b"MTLB";

/// Disambiguates scratch directories created within one process.
static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A handle to the Apple offline Metal toolchain, invoked through `xcrun`.
///
/// The launcher path is explicit so fail-closed behaviour is testable without a
/// live toolchain: pointing it at a nonexistent path exercises the same
/// [`DriverError::ToolchainUnavailable`] path a non-macOS host produces.
#[derive(Debug, Clone)]
pub struct Toolchain {
    launcher: PathBuf,
}

impl Toolchain {
    /// Uses the system `xcrun` launcher, resolved from `PATH`.
    #[must_use]
    pub fn system() -> Self {
        Self {
            launcher: PathBuf::from("xcrun"),
        }
    }

    /// Uses an explicit `xcrun`-compatible launcher path.
    #[must_use]
    pub fn with_launcher(launcher: impl Into<PathBuf>) -> Self {
        Self {
            launcher: launcher.into(),
        }
    }

    /// Resolves the tool and SDK identity for one selected SDK without
    /// compiling.
    ///
    /// This is the preflight every compilation runs first: it locates `metal`
    /// and `metallib`, reads their versions, and reads the SDK identity, so the
    /// compiler fingerprint and provenance are captured before any host work.
    ///
    /// # Errors
    ///
    /// Returns [`DriverError::ToolchainUnavailable`] when `metal` or `metallib`
    /// cannot be located or does not report a version, and
    /// [`DriverError::SdkUnavailable`] when the SDK identity cannot be read.
    pub fn resolve(&self, sdk: AppleSdk) -> Result<ResolvedToolchain, DriverError> {
        let metal_path = self.find_tool(sdk, "metal")?;
        let metallib_path = self.find_tool(sdk, "metallib")?;
        let metal_version = self.tool_version(sdk, "metal")?;
        let metallib_version = self.tool_version(sdk, "metallib")?;
        let sdk_path = self.sdk_field(sdk, "--show-sdk-path")?;
        let sdk_version = self.sdk_field(sdk, "--show-sdk-version")?;
        let sdk_build = self.sdk_field(sdk, "--show-sdk-build-version")?;

        Ok(ResolvedToolchain {
            sdk: SdkIdentity {
                canonical_name: sdk.selector().to_owned(),
                version: sdk_version,
                build: sdk_build,
                path: PathBuf::from(sdk_path),
            },
            metal: ResolvedTool {
                path: metal_path,
                version: metal_version,
            },
            metallib: ResolvedTool {
                path: metallib_path,
                version: metallib_version,
            },
        })
    }

    /// Compiles MSL source into a `metallib` with full provenance.
    ///
    /// The single selected SDK from `request.target` is used for both the
    /// `metal` and `metallib` invocations. Both run with the explicit flags from
    /// the request and with `ZERO_AR_DATE=1` for reproducible archive metadata.
    ///
    /// # Errors
    ///
    /// Fails closed with a typed [`DriverError`] and produces no artifact when
    /// the toolchain or SDK is unavailable, when the scratch filesystem work
    /// fails, when either tool reports a nonzero status, or when the linker
    /// yields no usable Metal library.
    pub fn compile(&self, request: &CompileRequest) -> Result<CompiledArtifact, DriverError> {
        let resolved = self.resolve(request.target.sdk)?;

        let scratch = Scratch::create()?;
        let source_path = scratch.path.join("kernel.metal");
        let air_path = scratch.path.join("kernel.air");
        let metallib_path = scratch.path.join("kernel.metallib");

        fs::write(&source_path, request.source.as_bytes()).map_err(|error| DriverError::Host {
            detail: format!("could not write MSL source: {error}"),
        })?;

        let compile_flags = request.compile_flags();
        let link_flags = request.link_flags();

        let mut metal_args: Vec<OsString> = compile_flags.iter().map(OsString::from).collect();
        metal_args.push(OsString::from("-c"));
        metal_args.push(source_path.clone().into_os_string());
        metal_args.push(OsString::from("-o"));
        metal_args.push(air_path.clone().into_os_string());
        self.run_stage(request.target.sdk, CompileStage::Metal, &metal_args)?;

        let mut link_args: Vec<OsString> = link_flags.iter().map(OsString::from).collect();
        link_args.push(air_path.clone().into_os_string());
        link_args.push(OsString::from("-o"));
        link_args.push(metallib_path.clone().into_os_string());
        self.run_stage(request.target.sdk, CompileStage::Metallib, &link_args)?;

        let metallib = fs::read(&metallib_path).map_err(|error| DriverError::Host {
            detail: format!("could not read metallib output: {error}"),
        })?;
        if metallib.len() < METALLIB_MAGIC.len()
            || metallib[..METALLIB_MAGIC.len()] != METALLIB_MAGIC
        {
            return Err(DriverError::EmptyArtifact {
                detail: format!(
                    "linker output was not a Metal library ({} bytes)",
                    metallib.len()
                ),
            });
        }

        let fingerprint = resolved.fingerprint();
        let provenance = ArtifactProvenance {
            platform: request.target.platform(),
            target_triple: request.target.triple(),
            deployment_minimum: request.target.deployment_minimum,
            msl_version: request.target.msl_version,
            optimization: request.optimization,
            numerical: request.numerical,
            sdk: resolved.sdk,
            metal: resolved.metal,
            metallib: resolved.metallib,
            fingerprint,
            compile_flags,
            link_flags,
        };

        Ok(CompiledArtifact {
            metallib,
            provenance,
        })
    }

    /// Locates one offline tool via `xcrun --sdk <sdk> --find <tool>`.
    fn find_tool(&self, sdk: AppleSdk, tool: &str) -> Result<PathBuf, DriverError> {
        self.capture(sdk, &[OsStr::new("--find"), OsStr::new(tool)])
            .map(PathBuf::from)
            .map_err(|detail| DriverError::ToolchainUnavailable {
                tool: tool.to_owned(),
                detail,
            })
    }

    /// Reads one offline tool's version via `xcrun --sdk <sdk> <tool> --version`.
    fn tool_version(&self, sdk: AppleSdk, tool: &str) -> Result<String, DriverError> {
        self.capture(sdk, &[OsStr::new(tool), OsStr::new("--version")])
            .map_err(|detail| DriverError::ToolchainUnavailable {
                tool: tool.to_owned(),
                detail,
            })
    }

    /// Reads one SDK identity field via `xcrun --sdk <sdk> <flag>`.
    fn sdk_field(&self, sdk: AppleSdk, flag: &str) -> Result<String, DriverError> {
        self.capture(sdk, &[OsStr::new(flag)])
            .map_err(|detail| DriverError::SdkUnavailable {
                sdk: sdk.selector().to_owned(),
                detail,
            })
    }

    /// Runs `xcrun --sdk <sdk> <args>` and returns trimmed, non-empty stdout.
    fn capture(&self, sdk: AppleSdk, args: &[&OsStr]) -> Result<String, String> {
        let output = Command::new(&self.launcher)
            .arg("--sdk")
            .arg(sdk.selector())
            .args(args)
            .env("ZERO_AR_DATE", "1")
            .output()
            .map_err(|error| format!("could not run {}: {error}", self.launcher.display()))?;
        if !output.status.success() {
            return Err(format!(
                "{} exited {}: {}",
                self.launcher.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if text.is_empty() {
            return Err(format!("{} produced empty output", self.launcher.display()));
        }
        Ok(text)
    }

    /// Runs one `xcrun --sdk <sdk> <tool> <args>` compile or link stage.
    fn run_stage(
        &self,
        sdk: AppleSdk,
        stage: CompileStage,
        args: &[OsString],
    ) -> Result<(), DriverError> {
        let output = Command::new(&self.launcher)
            .arg("--sdk")
            .arg(sdk.selector())
            .arg(stage.tool())
            .args(args)
            .env("ZERO_AR_DATE", "1")
            .output()
            .map_err(|error| DriverError::ToolchainUnavailable {
                tool: stage.tool().to_owned(),
                detail: format!("could not run {}: {error}", self.launcher.display()),
            })?;
        if !output.status.success() {
            return Err(DriverError::ToolFailure {
                stage,
                status: output.status.to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        Ok(())
    }
}

/// An owned scratch directory removed when the compilation finishes.
///
/// The driver needs local files to exchange with the offline tools; this is
/// ephemeral working space, not the expansion cache, which a later ticket owns.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    /// Creates a fresh, uniquely named scratch directory.
    fn create() -> Result<Self, DriverError> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let counter = SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("tiler-metal-aot-{}-{counter}-{nanos}", std::process::id());
        let path = std::env::temp_dir().join(name);
        fs::create_dir(&path).map_err(|error| DriverError::Host {
            detail: format!(
                "could not create scratch directory {}: {error}",
                path.display()
            ),
        })?;
        Ok(Self { path })
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::Toolchain;
    use crate::diagnostic::{CompileStage, DriverError};
    use crate::input::{
        AppleSdk, CompileRequest, DeploymentMinimum, MetalTarget, MslVersion, NumericalRealization,
        OptimizationLevel,
    };

    const TRIVIAL_MSL: &str = "#include <metal_stdlib>\n\
using namespace metal;\n\
kernel void copy_kernel(device const float* in [[buffer(0)]],\n\
                        device float* out [[buffer(1)]],\n\
                        uint gid [[thread_position_in_grid]]) {\n\
    out[gid] = in[gid];\n\
}\n";

    fn macos_request(source: &str) -> CompileRequest {
        CompileRequest::new(
            source,
            MetalTarget::new(
                AppleSdk::MacOs,
                DeploymentMinimum::new(13, 0),
                MslVersion::Metal3_1,
            ),
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
    }

    #[test]
    fn compile_fails_closed_when_launcher_is_absent() {
        let toolchain = Toolchain::with_launcher("/nonexistent/tiler-metal-aot-xcrun");
        let error = toolchain.compile(&macos_request(TRIVIAL_MSL)).unwrap_err();
        assert!(
            matches!(error, DriverError::ToolchainUnavailable { .. }),
            "expected ToolchainUnavailable, got {error:?}"
        );
    }

    #[test]
    fn resolve_fails_closed_when_launcher_is_absent() {
        let toolchain = Toolchain::with_launcher("/nonexistent/tiler-metal-aot-xcrun");
        let error = toolchain.resolve(AppleSdk::MacOs).unwrap_err();
        assert!(matches!(error, DriverError::ToolchainUnavailable { .. }));
    }

    // The following tests exercise the real Apple toolchain. They self-skip when
    // no qualified toolchain is present so the repository gate passes on hosts
    // and CI runners without one.

    #[test]
    fn compiles_trivial_kernel_when_toolchain_available() {
        let toolchain = Toolchain::system();
        let Ok(resolved) = toolchain.resolve(AppleSdk::MacOs) else {
            return;
        };
        assert!(!resolved.metal.version.is_empty());
        assert!(!resolved.metallib.version.is_empty());

        let artifact = toolchain
            .compile(&macos_request(TRIVIAL_MSL))
            .expect("trivial kernel should compile on a resolved toolchain");

        assert_eq!(&artifact.metallib[..4], b"MTLB");
        assert_eq!(artifact.provenance.target_triple, "air64-apple-macos13.0");
        assert_eq!(
            artifact.provenance.fingerprint.metal_version,
            resolved.metal.version
        );
        assert!(!artifact.provenance.metal.path.as_os_str().is_empty());
        assert!(!artifact.provenance.sdk.build.is_empty());
        assert_eq!(
            artifact
                .provenance
                .compile_flags
                .first()
                .map(String::as_str),
            Some("-target")
        );
    }

    #[test]
    fn rejects_invalid_source_when_toolchain_available() {
        let toolchain = Toolchain::system();
        if toolchain.resolve(AppleSdk::MacOs).is_err() {
            return;
        }
        let error = toolchain
            .compile(&macos_request("this is not valid Metal Shading Language;"))
            .unwrap_err();
        assert!(
            matches!(
                error,
                DriverError::ToolFailure {
                    stage: CompileStage::Metal,
                    ..
                }
            ),
            "expected a metal-stage ToolFailure, got {error:?}"
        );
    }
}
