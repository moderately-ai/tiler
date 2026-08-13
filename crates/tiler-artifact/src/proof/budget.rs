//! Exact encoded-size projection for the proof sidecar.
//!
//! Every producer path that would clone, hash, reserve, or append bytes
//! proportional to a governed bound asks this module for the exact encoded
//! length first. The arithmetic is checked: a sum or product that does not fit
//! `usize` is [`ProofBudgetError::Unrepresentable`] rather than a wrapped or
//! saturated length that could pass a later bound by accident.

use crate::program::DIGEST_BYTES;

use super::codec::{
    HEADER_BYTES, IDENTITY_DOMAIN, MANIFEST_DOMAIN, ProofLimitExceeded, ProofLimitKind, proof_limit,
};
use super::model::ProofSidecarData;
use super::{MAX_PROOF_CASES, MAX_PROOF_INTERFACE_ENTRIES, MAX_PROOF_PAYLOAD_BYTES};

/// Width of the canonical length prefix `push_len` writes.
const LENGTH_BYTES: usize = 8;
/// Width of a payload's canonical ordinal on the wire.
const ORDINAL_BYTES: usize = 4;
/// Manifest / identity schema version: two `u16`s.
const SCHEMA_BYTES: usize = 4;
/// Bytes one framed payload writes ahead of its content: ordinal then length.
const PAYLOAD_FRAME_PREFIX: usize = ORDINAL_BYTES + LENGTH_BYTES;
/// Bytes one manifest payload descriptor writes: ordinal, length, digest.
const PAYLOAD_DESCRIPTOR_BYTES: usize = ORDINAL_BYTES + LENGTH_BYTES + DIGEST_BYTES;

/// Why a projected encoding could not be admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProofBudgetError {
    /// A representable size exceeded its governed bound.
    Limit(ProofLimitExceeded),
    /// Combining two sizes overflowed this host's `usize`.
    Unrepresentable {
        /// The bound whose projected size could not be represented.
        kind: ProofLimitKind,
    },
}

impl From<ProofLimitExceeded> for ProofBudgetError {
    fn from(cause: ProofLimitExceeded) -> Self {
        Self::Limit(cause)
    }
}

/// Exact encoded lengths of one sidecar, derived before any proportional write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ProjectedSizes {
    pub(super) identity: usize,
    pub(super) manifest: usize,
    pub(super) framed_payloads: usize,
    pub(super) sidecar: usize,
    pub(super) payload_count: usize,
}

impl ProjectedSizes {
    /// Refuses any projected length that exceeds its governed bound.
    pub(super) fn check(self) -> Result<Self, ProofBudgetError> {
        for (kind, attempted) in [
            (ProofLimitKind::IdentityBytes, self.identity),
            (ProofLimitKind::ManifestBytes, self.manifest),
            (ProofLimitKind::SidecarBytes, self.sidecar),
        ] {
            let limit = kind
                .byte_budget()
                .expect("identity, manifest, and sidecar are byte resources");
            proof_limit(attempted, limit, kind)?;
        }
        proof_limit(self.payload_count, max_payloads(), ProofLimitKind::Payloads)?;
        Ok(self)
    }
}

/// One case's contribution to the encoded sizes, as lengths only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CaseLens {
    pub(super) key_len: usize,
    pub(super) input_lens: Vec<usize>,
    pub(super) expected_lens: Vec<usize>,
}

/// The largest framed payload count any admitted sidecar can reach.
///
/// Derived from the case and interface bounds rather than declared, so the
/// framing bound and the structural bounds cannot disagree.
pub(super) const fn max_payloads() -> usize {
    MAX_PROOF_CASES * (2 * MAX_PROOF_INTERFACE_ENTRIES)
}

/// Adds two sizes or names the bound whose projection overflowed.
pub(super) fn add(
    left: usize,
    right: usize,
    kind: ProofLimitKind,
) -> Result<usize, ProofBudgetError> {
    left.checked_add(right)
        .ok_or(ProofBudgetError::Unrepresentable { kind })
}

fn mul(left: usize, right: usize, kind: ProofLimitKind) -> Result<usize, ProofBudgetError> {
    left.checked_mul(right)
        .ok_or(ProofBudgetError::Unrepresentable { kind })
}

fn framed(len: usize, kind: ProofLimitKind) -> Result<usize, ProofBudgetError> {
    add(LENGTH_BYTES, len, kind)
}

/// Projects every encoded byte length from already-owned sidecar data.
///
/// `carried_identity` is the identity the encoder will actually write. The
/// derived identity length is always computed from `data`; a forge that stamps
/// a stale identity of a different length must size the manifest from the
/// carried bytes, not from the identity that would be derived.
pub(super) fn project_from_data(
    data: &ProofSidecarData,
) -> Result<ProjectedSizes, ProofBudgetError> {
    project_from_data_with_identity(data, None)
}

/// Projects encoding sizes using a carried identity length when one is known.
pub(super) fn project_from_data_with_identity(
    data: &ProofSidecarData,
    carried_identity: Option<usize>,
) -> Result<ProjectedSizes, ProofBudgetError> {
    let input_key_lens: Vec<usize> = data
        .input_keys
        .iter()
        .map(|key| key.as_str().len())
        .collect();
    let output_key_lens: Vec<usize> = data
        .output_keys
        .iter()
        .map(|key| key.as_str().len())
        .collect();
    let cases: Vec<CaseLens> = data
        .cases
        .iter()
        .map(|case| CaseLens {
            key_len: case.key.as_str().len(),
            input_lens: case.inputs.iter().map(Vec::len).collect(),
            expected_lens: case.expected.iter().map(Vec::len).collect(),
        })
        .collect();
    project_layout(
        &Layout {
            artifact_identity: data.artifact_identity.len(),
            semantic: data.subjects.semantic.as_bytes().len(),
            numerical: data.subjects.numerical.as_bytes().len(),
            reference: data.subjects.reference.as_bytes().len(),
            input_key_lens: &input_key_lens,
            output_key_lens: &output_key_lens,
            cases: &cases,
        },
        carried_identity,
    )
}

/// Projects every encoded byte length from field lengths alone.
///
/// The caller supplies lengths, never payload bytes, so a sidecar that would
/// exceed a bound can be refused without cloning or reserving those bytes.
pub(super) fn project_sidecar(
    artifact_identity: usize,
    semantic: usize,
    numerical: usize,
    reference: usize,
    input_key_lens: impl IntoIterator<Item = usize>,
    output_key_lens: impl IntoIterator<Item = usize>,
    cases: impl IntoIterator<Item = CaseLens>,
) -> Result<ProjectedSizes, ProofBudgetError> {
    let input_key_lens: Vec<usize> = input_key_lens.into_iter().collect();
    let output_key_lens: Vec<usize> = output_key_lens.into_iter().collect();
    let cases: Vec<CaseLens> = cases.into_iter().collect();
    project_layout(
        &Layout {
            artifact_identity,
            semantic,
            numerical,
            reference,
            input_key_lens: &input_key_lens,
            output_key_lens: &output_key_lens,
            cases: &cases,
        },
        None,
    )
}

fn project_layout(
    layout: &Layout<'_>,
    carried_identity: Option<usize>,
) -> Result<ProjectedSizes, ProofBudgetError> {
    let mut payload_count = 0_usize;
    let mut framed_payloads = 0_usize;
    for case in layout.cases {
        for &len in case.input_lens.iter().chain(&case.expected_lens) {
            proof_limit(len, MAX_PROOF_PAYLOAD_BYTES, ProofLimitKind::PayloadBytes)?;
            payload_count = add(payload_count, 1, ProofLimitKind::Payloads)?;
            let frame = add(PAYLOAD_FRAME_PREFIX, len, ProofLimitKind::SidecarBytes)?;
            framed_payloads = add(framed_payloads, frame, ProofLimitKind::SidecarBytes)?;
        }
    }

    let identity = identity_len(layout)?;
    let written_identity = carried_identity.unwrap_or(identity);
    let manifest = manifest_len(layout, written_identity)?;
    let sidecar = add(
        add(HEADER_BYTES, manifest, ProofLimitKind::SidecarBytes)?,
        framed_payloads,
        ProofLimitKind::SidecarBytes,
    )?;
    Ok(ProjectedSizes {
        identity,
        manifest,
        framed_payloads,
        sidecar,
        payload_count,
    })
}

struct Layout<'a> {
    artifact_identity: usize,
    semantic: usize,
    numerical: usize,
    reference: usize,
    input_key_lens: &'a [usize],
    output_key_lens: &'a [usize],
    cases: &'a [CaseLens],
}

fn encoded_prelude(
    layout: &Layout<'_>,
    kind: ProofLimitKind,
    domain_len: usize,
) -> Result<usize, ProofBudgetError> {
    let mut total = add(domain_len, SCHEMA_BYTES, kind)?;
    total = add(total, framed(layout.artifact_identity, kind)?, kind)?;
    total = add(total, DIGEST_BYTES, kind)?;
    total = add(total, framed(layout.semantic, kind)?, kind)?;
    total = add(total, framed(layout.numerical, kind)?, kind)?;
    total = add(total, framed(layout.reference, kind)?, kind)?;
    total = add(total, LENGTH_BYTES, kind)?;
    for &len in layout.input_key_lens {
        total = add(total, framed(len, kind)?, kind)?;
    }
    total = add(total, LENGTH_BYTES, kind)?;
    for &len in layout.output_key_lens {
        total = add(total, framed(len, kind)?, kind)?;
    }
    add(total, LENGTH_BYTES, kind)
}

fn identity_len(layout: &Layout<'_>) -> Result<usize, ProofBudgetError> {
    let kind = ProofLimitKind::IdentityBytes;
    let mut total = encoded_prelude(layout, kind, IDENTITY_DOMAIN.len())?;
    for case in layout.cases {
        total = add(total, framed(case.key_len, kind)?, kind)?;
        total = add(total, LENGTH_BYTES.saturating_mul(2), kind)?;
        let payloads = add(case.input_lens.len(), case.expected_lens.len(), kind)?;
        total = add(total, mul(payloads, DIGEST_BYTES, kind)?, kind)?;
    }
    Ok(total)
}

fn manifest_len(layout: &Layout<'_>, identity: usize) -> Result<usize, ProofBudgetError> {
    let kind = ProofLimitKind::ManifestBytes;
    let mut total = encoded_prelude(layout, kind, MANIFEST_DOMAIN.len())?;
    for case in layout.cases {
        total = add(total, framed(case.key_len, kind)?, kind)?;
        total = add(total, LENGTH_BYTES.saturating_mul(2), kind)?;
        let payloads = add(case.input_lens.len(), case.expected_lens.len(), kind)?;
        total = add(total, mul(payloads, PAYLOAD_DESCRIPTOR_BYTES, kind)?, kind)?;
    }
    add(total, framed(identity, kind)?, kind)
}
