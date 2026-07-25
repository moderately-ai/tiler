//! Filling the neutral carried payload from a real emission and compilation.
//!
//! `tiler-artifact` owns the *shape* of a carried payload and deliberately
//! knows nothing about Metal: a compilation subject is a governed source
//! representation, exact source bytes, provenance, entry mappings, and recorded
//! obligations. This module is the Metal half of that correspondence — it reads
//! a `MetalTranslationUnit` and an `ArtifactProvenance` and produces the neutral
//! record. It lives in the producer because nothing else sees both.
//!
//! # What is deliberately excluded, and why
//!
//! **Absolute paths.** `ResolvedTool::path` and `SdkIdentity::path` are marked
//! in their own documentation as *local* provenance rather than portable
//! identity, and a payload identity that folded one would differ between two
//! hosts running the same toolchain. Only the reported versions travel.
//!
//! **The `metallib` bytes.** They are the payload's *object*, carried in their
//! own section, and are not part of the compilation subject. That is the
//! identity decision `prototype-metal-bundle-assembly` made: a payload is
//! content-addressed over its compilation inputs, so relinking the same source
//! yields the same artifact identity and a different envelope digest.
//!
//! # Ordering
//!
//! Entry mappings and obligations are sorted, because the codec proves both are
//! in canonical content order and rejects a non-canonical spelling rather than
//! normalizing it. Compile and link flags are **not** sorted: a compiler
//! resolves repeated or conflicting flags positionally, so their order is
//! meaning, and the codec deliberately exempts them from the canonical-order
//! rule.

use tiler_artifact::program::{
    BackendEntryKey, PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadProvenance,
    PayloadSdkIdentity, PayloadTargetObligation, RepresentationKey, ToolComponent,
};
use tiler_metal::record::MetalTranslationUnit;
use tiler_metal_aot::record::ArtifactProvenance;

/// Governed representation key of the retained Metal source.
const SOURCE_REPRESENTATION: &str = "metal-source";
/// Governed role key of the offline Metal compiler.
const COMPILER_ROLE: &str = "compiler";
/// Governed role key of the offline Metal linker.
const LINKER_ROLE: &str = "linker";
/// Governed key of the Apple offline Metal toolchain family.
const TOOLCHAIN_KEY: &str = "tiler.toolchain.apple-metal";
/// Governed obligation key naming a numerical gap the target cannot honour.
const NUMERICAL_GAP_OBLIGATION: &str = "tiler.numerical.unhonoured-gap";
/// Governed obligation key naming a numerical requirement the emission carries.
const NUMERICAL_REQUIREMENT_OBLIGATION: &str = "tiler.numerical.emission-requirement";

/// Assembles the neutral compilation subject and object of one carried payload.
///
/// # Errors
///
/// Returns the governed key that was rejected when a value read from the
/// emission or the toolchain is not a valid representation key.
pub fn carried_payload(
    unit: &MetalTranslationUnit,
    provenance: &ArtifactProvenance,
    metallib: &[u8],
) -> Result<PayloadContent, String> {
    let source_representation = RepresentationKey::new(SOURCE_REPRESENTATION)
        .map_err(|_| format!("{SOURCE_REPRESENTATION} is not a governed representation key"))?;

    let mut components = vec![
        ToolComponent {
            role: COMPILER_ROLE.to_owned(),
            version: provenance.fingerprint.metal_version.clone(),
        },
        ToolComponent {
            role: LINKER_ROLE.to_owned(),
            version: provenance.fingerprint.metallib_version.clone(),
        },
    ];
    components.sort();

    // The neutral entry key is the kernel's canonical identity, not the emitted
    // symbol. The symbol is a bounded digest and presentation only; the identity
    // is what an artifact's executable entry names when it says which stage this
    // entry realizes, so keying on it is what lets a loader tie the two together.
    let mut entries = Vec::with_capacity(unit.entry_points().len());
    for entry in unit.entry_points() {
        entries.push(PayloadEntryMapping {
            entry_key: BackendEntryKey::from_bytes(entry.kernel_identity().as_bytes()).map_err(
                |cause| {
                    format!(
                        "the {}-byte kernel identity is not a valid backend entry key: {cause:?}",
                        entry.kernel_identity().as_bytes().len(),
                    )
                },
            )?,
            symbol: entry.symbol().to_owned(),
            transports: entry
                .buffers()
                .iter()
                .map(|binding| binding.index())
                .collect(),
        });
    }
    entries.sort_by(|left, right| left.entry_key.as_bytes().cmp(right.entry_key.as_bytes()));

    // Both the requirements the emission carries and the gaps it could not
    // honour are recorded. A gap reaching a *packaged* payload should be
    // impossible — `require_declared_realization` refuses first — but recording
    // it rather than dropping it means a future path that skips that check
    // produces a legible artifact instead of a silent one.
    let mut obligations: Vec<PayloadTargetObligation> = unit
        .numerical_requirements()
        .iter()
        .map(|requirement| PayloadTargetObligation {
            key: NUMERICAL_REQUIREMENT_OBLIGATION.to_owned(),
            value: requirement.to_string(),
        })
        .chain(
            unit.numerical_gaps()
                .iter()
                .map(|gap| PayloadTargetObligation {
                    key: NUMERICAL_GAP_OBLIGATION.to_owned(),
                    value: gap.rule().to_owned(),
                }),
        )
        .collect();
    obligations.sort();
    obligations.dedup();

    Ok(PayloadContent {
        metadata: PayloadMetadata {
            source_representation,
            source: unit.source().as_bytes().to_vec(),
            provenance: PayloadProvenance {
                toolchain: TOOLCHAIN_KEY.to_owned(),
                target: provenance.target_triple.clone(),
                family: provenance.platform.as_str().to_owned(),
                language: provenance.msl_version.std_token().to_owned(),
                deployment_major: provenance.deployment_minimum.major(),
                deployment_minor: provenance.deployment_minimum.minor(),
                components,
                sdk: PayloadSdkIdentity {
                    name: provenance.sdk.canonical_name.clone(),
                    version: provenance.sdk.version.clone(),
                    build: provenance.sdk.build.clone(),
                },
                compile_flags: provenance.compile_flags.clone(),
                link_flags: provenance.link_flags.clone(),
            },
            entries,
            obligations,
        },
        code: metallib.to_vec(),
    })
}
