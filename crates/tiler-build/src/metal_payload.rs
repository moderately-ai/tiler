//! Correspondence between prepared Metal compilation facts and carried payloads.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{PayloadMetadata, ToolComponent};
use tiler_metal_aot::driver::PreparedCompilation;

/// Governed representation key of retained Metal source.
pub(crate) const SOURCE_REPRESENTATION: &str = "metal-source";
/// Governed Apple offline Metal toolchain identity.
pub(crate) const TOOLCHAIN: &str = "tiler.toolchain.apple-metal";
/// Canonical role of the compiler component.
pub(crate) const COMPILER_ROLE: &str = "compiler";
/// Canonical role of the linker component.
pub(crate) const LINKER_ROLE: &str = "linker";

/// One compilation fact whose carried payload spelling may disagree.
///
/// Deliberately exhaustive: adding a compared fact must stop every diagnostic
/// renderer that promises to classify all producer/protocol defects.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MetalPayloadFact {
    /// Governed source-representation key.
    SourceRepresentation,
    /// Exact MSL source bytes.
    Source,
    /// Governed toolchain-family identity.
    Toolchain,
    /// Normalized compilation target triple.
    Target,
    /// Apple artifact platform family.
    Family,
    /// MSL language standard.
    Language,
    /// Requested deployment minimum.
    DeploymentMinimum,
    /// Canonically ordered compiler and linker versions.
    Components,
    /// Canonical SDK name, version, and build.
    Sdk,
    /// Exact ordered compiler flags.
    CompileFlags,
    /// Exact ordered linker flags.
    LinkFlags,
}

impl MetalPayloadFact {
    /// Returns the stable diagnostic spelling of this fact.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceRepresentation => "source-representation",
            Self::Source => "source",
            Self::Toolchain => "toolchain",
            Self::Target => "target",
            Self::Family => "family",
            Self::Language => "language",
            Self::DeploymentMinimum => "deployment-minimum",
            Self::Components => "components",
            Self::Sdk => "sdk",
            Self::CompileFlags => "compile-flags",
            Self::LinkFlags => "link-flags",
        }
    }
}

impl fmt::Display for MetalPayloadFact {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A carried payload does not describe the prepared compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetalPayloadMismatch {
    fact: MetalPayloadFact,
}

impl MetalPayloadMismatch {
    /// Returns the first fact that disagreed in contract order.
    #[must_use]
    pub const fn fact(self) -> MetalPayloadFact {
        self.fact
    }
}

impl fmt::Display for MetalPayloadMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "carried Metal payload disagrees with prepared compilation fact `{}`",
            self.fact
        )
    }
}

impl Error for MetalPayloadMismatch {}

/// Validates a carried payload against the exact prepared compilation.
///
/// The token supplies both the request and the provenance derived from the
/// single toolchain resolution that produced its cache identity. Validation is
/// allocation-free and compares ordered flag lists positionally.
///
/// Entry mappings and target obligations are emission facts rather than AOT
/// compilation facts and are deliberately outside this check.
///
/// # Errors
///
/// Returns the first mismatched fact in stable contract order. A mismatch is a
/// producer/protocol defect, not a cache miss: rebuilding under the same
/// mismatched subject would repeat the defect.
pub fn validate_prepared_metal_payload(
    prepared: &PreparedCompilation<'_>,
    metadata: &PayloadMetadata,
) -> Result<(), MetalPayloadMismatch> {
    let request = prepared.request();
    let expected = prepared.provenance();
    validate_metal_payload_facts(
        &MetalPayloadFacts {
            source_representation: SOURCE_REPRESENTATION,
            source: request.source.as_bytes(),
            toolchain: TOOLCHAIN,
            target: &expected.target_triple,
            family: expected.platform.as_str(),
            language: expected.msl_version.semantic_name(),
            deployment_major: expected.deployment_minimum.major(),
            deployment_minor: expected.deployment_minimum.minor(),
            components: ExpectedComponents::Prepared(expected),
            sdk_name: &expected.sdk.canonical_name,
            sdk_version: &expected.sdk.version,
            sdk_build: &expected.sdk.build,
            compile_flags: &expected.compile_flags,
            link_flags: &expected.link_flags,
        },
        metadata,
    )
}

pub(crate) fn validate_metal_payload_metadata(
    expected: &PayloadMetadata,
    actual: &PayloadMetadata,
) -> Result<(), MetalPayloadMismatch> {
    let provenance = &expected.provenance;
    validate_metal_payload_facts(
        &MetalPayloadFacts {
            source_representation: expected.source_representation.as_str(),
            source: &expected.source,
            toolchain: &provenance.toolchain,
            target: &provenance.target,
            family: &provenance.family,
            language: &provenance.language,
            deployment_major: provenance.deployment_major,
            deployment_minor: provenance.deployment_minor,
            components: ExpectedComponents::Metadata(&provenance.components),
            sdk_name: &provenance.sdk.name,
            sdk_version: &provenance.sdk.version,
            sdk_build: &provenance.sdk.build,
            compile_flags: &provenance.compile_flags,
            link_flags: &provenance.link_flags,
        },
        actual,
    )
}

struct MetalPayloadFacts<'facts> {
    source_representation: &'facts str,
    source: &'facts [u8],
    toolchain: &'facts str,
    target: &'facts str,
    family: &'facts str,
    language: &'facts str,
    deployment_major: u16,
    deployment_minor: u16,
    components: ExpectedComponents<'facts>,
    sdk_name: &'facts str,
    sdk_version: &'facts str,
    sdk_build: &'facts str,
    compile_flags: &'facts [String],
    link_flags: &'facts [String],
}

enum ExpectedComponents<'facts> {
    Prepared(&'facts tiler_metal_aot::record::ArtifactProvenance),
    Metadata(&'facts [ToolComponent]),
}

fn validate_metal_payload_facts(
    expected: &MetalPayloadFacts<'_>,
    metadata: &PayloadMetadata,
) -> Result<(), MetalPayloadMismatch> {
    let actual = &metadata.provenance;
    require(
        metadata.source_representation.as_str() == expected.source_representation,
        MetalPayloadFact::SourceRepresentation,
    )?;
    require(
        metadata.source.as_slice() == expected.source,
        MetalPayloadFact::Source,
    )?;
    require(
        actual.toolchain == expected.toolchain,
        MetalPayloadFact::Toolchain,
    )?;
    require(actual.target == expected.target, MetalPayloadFact::Target)?;
    require(actual.family == expected.family, MetalPayloadFact::Family)?;
    require(
        actual.language == expected.language,
        MetalPayloadFact::Language,
    )?;
    require(
        actual.deployment_major == expected.deployment_major
            && actual.deployment_minor == expected.deployment_minor,
        MetalPayloadFact::DeploymentMinimum,
    )?;
    require(
        match expected.components {
            ExpectedComponents::Prepared(expected) => {
                components_match(&actual.components, expected)
            }
            ExpectedComponents::Metadata(expected) => actual.components == expected,
        },
        MetalPayloadFact::Components,
    )?;
    require(
        actual.sdk.name == expected.sdk_name
            && actual.sdk.version == expected.sdk_version
            && actual.sdk.build == expected.sdk_build,
        MetalPayloadFact::Sdk,
    )?;
    require(
        actual.compile_flags == expected.compile_flags,
        MetalPayloadFact::CompileFlags,
    )?;
    require(
        actual.link_flags == expected.link_flags,
        MetalPayloadFact::LinkFlags,
    )
}

fn components_match(
    actual: &[ToolComponent],
    expected: &tiler_metal_aot::record::ArtifactProvenance,
) -> bool {
    let [compiler, linker] = actual else {
        return false;
    };
    compiler.role == COMPILER_ROLE
        && compiler.version == expected.fingerprint.metal_version
        && linker.role == LINKER_ROLE
        && linker.version == expected.fingerprint.metallib_version
}

fn require(matches: bool, fact: MetalPayloadFact) -> Result<(), MetalPayloadMismatch> {
    if matches {
        Ok(())
    } else {
        Err(MetalPayloadMismatch { fact })
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tiler_artifact::program::{
        PayloadMetadata, PayloadProvenance, PayloadSdkIdentity, RepresentationKey, ToolComponent,
    };
    use tiler_metal_aot::driver::{PreparedCompilation, Toolchain};
    use tiler_metal_aot::input::{
        ApplePlatform, CompileRequest, DeploymentMinimum, MetalTarget, MslVersion,
        NumericalRealization, OptimizationLevel,
    };

    use super::{
        COMPILER_ROLE, LINKER_ROLE, MetalPayloadFact, SOURCE_REPRESENTATION, TOOLCHAIN,
        validate_prepared_metal_payload,
    };

    const SOURCE: &str = "kernel void main0() {}";

    fn write_executable(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::write(path, body).expect("the fake tool is writable");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("the fake tool is executable");
    }

    fn prepared<'request>(
        directory: &Path,
        request: &'request CompileRequest,
    ) -> PreparedCompilation<'request> {
        let metal = directory.join("metal");
        let metallib = directory.join("metallib");
        let launcher = directory.join("xcrun");
        write_executable(&metal, "#!/bin/sh\necho 'Metal build-v1'\n");
        write_executable(&metallib, "#!/bin/sh\necho 'metallib build-v1'\n");
        write_executable(
            &launcher,
            &format!(
                "#!/bin/sh\n\
                 shift 2\n\
                 case \"$1\" in\n\
                   --find) if [ \"$2\" = \"metal\" ]; then echo '{}'; else echo '{}'; fi ;;\n\
                   --show-sdk-path) echo /SDKs/MacOSX.sdk ;;\n\
                   --show-sdk-version) echo 26.5 ;;\n\
                   --show-sdk-build-version) echo 25F70 ;;\n\
                   *) exit 1 ;;\n\
                 esac\n",
                metal.display(),
                metallib.display(),
            ),
        );
        Toolchain::with_launcher(launcher)
            .prepare(request)
            .expect("the fake toolchain resolves")
    }

    fn request() -> CompileRequest {
        CompileRequest::new(
            SOURCE,
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

    fn metadata(prepared: &PreparedCompilation<'_>) -> PayloadMetadata {
        let provenance = prepared.provenance();
        PayloadMetadata {
            source_representation: RepresentationKey::new(SOURCE_REPRESENTATION).unwrap(),
            source: prepared.request().source.as_bytes().to_vec(),
            provenance: PayloadProvenance {
                toolchain: TOOLCHAIN.to_owned(),
                target: provenance.target_triple.clone(),
                family: provenance.platform.as_str().to_owned(),
                language: provenance.msl_version.semantic_name().to_owned(),
                deployment_major: provenance.deployment_minimum.major(),
                deployment_minor: provenance.deployment_minimum.minor(),
                components: vec![
                    ToolComponent {
                        role: COMPILER_ROLE.to_owned(),
                        version: provenance.fingerprint.metal_version.clone(),
                    },
                    ToolComponent {
                        role: LINKER_ROLE.to_owned(),
                        version: provenance.fingerprint.metallib_version.clone(),
                    },
                ],
                sdk: PayloadSdkIdentity {
                    name: provenance.sdk.canonical_name.clone(),
                    version: provenance.sdk.version.clone(),
                    build: provenance.sdk.build.clone(),
                },
                compile_flags: provenance.compile_flags.clone(),
                link_flags: provenance.link_flags.clone(),
            },
            entries: Vec::new(),
            obligations: Vec::new(),
        }
    }

    #[test]
    fn exact_prepared_compilation_metadata_is_accepted() {
        let directory = scratch("accept");
        let request = request();
        let prepared = prepared(&directory, &request);
        assert_eq!(
            validate_prepared_metal_payload(&prepared, &metadata(&prepared)),
            Ok(())
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn every_compilation_fact_mismatch_is_named() {
        let directory = scratch("mismatch");
        let request = request();
        let prepared = prepared(&directory, &request);

        for (expected, mutate) in mutations() {
            let mut altered = metadata(&prepared);
            mutate(&mut altered);
            assert_eq!(
                validate_prepared_metal_payload(&prepared, &altered)
                    .unwrap_err()
                    .fact(),
                expected,
            );
        }
        let _ = std::fs::remove_dir_all(directory);
    }

    type Mutation = fn(&mut PayloadMetadata);

    fn mutations() -> [(MetalPayloadFact, Mutation); 11] {
        [
            (MetalPayloadFact::SourceRepresentation, |metadata| {
                metadata.source_representation = RepresentationKey::new("other-source").unwrap();
            }),
            (MetalPayloadFact::Source, |metadata| {
                metadata.source.push(b'!');
            }),
            (MetalPayloadFact::Toolchain, |metadata| {
                metadata.provenance.toolchain.push_str(".other");
            }),
            (MetalPayloadFact::Target, |metadata| {
                metadata.provenance.target.push_str("-other");
            }),
            (MetalPayloadFact::Family, |metadata| {
                metadata.provenance.family.push_str("-other");
            }),
            (MetalPayloadFact::Language, |metadata| {
                metadata.provenance.language.push_str("-other");
            }),
            (MetalPayloadFact::DeploymentMinimum, |metadata| {
                metadata.provenance.deployment_minor += 1;
            }),
            (MetalPayloadFact::Components, |metadata| {
                metadata.provenance.components.swap(0, 1);
            }),
            (MetalPayloadFact::Sdk, |metadata| {
                metadata.provenance.sdk.build.push_str("-other");
            }),
            (MetalPayloadFact::CompileFlags, |metadata| {
                metadata.provenance.compile_flags.reverse();
            }),
            (MetalPayloadFact::LinkFlags, |metadata| {
                metadata.provenance.link_flags.push("-other".to_owned());
            }),
        ]
    }

    fn scratch(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tiler-build-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&path).expect("the scratch directory is creatable");
        path
    }
}
