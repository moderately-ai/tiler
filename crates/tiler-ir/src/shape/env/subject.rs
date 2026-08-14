//! Canonical encode and decode of the identity-bearing shape-environment subject.
//!
//! The bytes are [`super::ShapeEnvIdentity`]'s preimage: every declaration and
//! root binding, then every semantic input constraint, in the order `build`
//! established. Variant guards and solver-derived state stay out.

use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;

use crate::identity::{push_len, push_slice};
use crate::program::abi::{AvailabilityPhase, TargetPropertyKey};
use crate::semantic::InputKey;
use crate::shape::{Axis, Extent};

use super::constraint::{ExtentRelation, ExtentTerm, SemanticInputConstraint};
use super::{
    BindingSource, FactProvenance, InterfaceParameterKey, RootBinding, SHAPE_ENV_DOMAIN,
    ShapeSymbol, SymbolScope,
};

/// Largest number of declarations or constraints one encoded subject may name.
const MAX_SUBJECT_ROWS: usize = 4_096;
/// Largest factorization the encoder writes and the decoder admits.
const MAX_FACTORS: usize = 64;

/// Why one identity-bearing shape-environment subject could not be decoded.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ShapeEnvSubjectError {
    /// The encoding ran out of bytes before a field was complete.
    Truncated,
    /// The subject did not open with the governed shape-environment domain.
    BadDomain,
    /// Bytes remained after the last canonical field.
    TrailingBytes,
    /// A counted run exceeded the governed subject budget.
    Limit {
        /// Attempted quantity.
        actual: u64,
        /// Governed maximum.
        limit: usize,
    },
    /// A text field was not valid UTF-8.
    InvalidText,
    /// A scoped symbol, key, or constructor the encoding named is unwritable.
    Malformed {
        /// Why the constructor refused.
        reason: String,
    },
    /// A binding-source tag is outside the admitted vocabulary.
    UnknownBindingSource {
        /// Unrecognized tag.
        tag: u8,
    },
    /// An availability-phase tag is outside the admitted vocabulary.
    UnknownPhase {
        /// Unrecognized tag.
        tag: u8,
    },
    /// A provenance tag is outside the admitted vocabulary.
    UnknownProvenance {
        /// Unrecognized tag.
        tag: u8,
    },
    /// An extent-term tag is outside the admitted vocabulary.
    UnknownTerm {
        /// Unrecognized tag.
        tag: u8,
    },
    /// An extent-relation tag is outside the admitted vocabulary.
    UnknownRelation {
        /// Unrecognized tag.
        tag: u8,
    },
    /// A known binding source has no authoritative ABI binding yet.
    UnsupportedBindingSource {
        /// The symbol bound to the unsupported source.
        symbol: ShapeSymbol,
        /// Rendered source.
        source: String,
    },
    /// A divisibility relation named divisor zero.
    ZeroDivisor,
    /// Declarations were not in canonical symbol order, or a symbol repeated.
    UnorderedBindings,
    /// Constraints were not in canonical order.
    UnorderedConstraints,
    /// A constraint named a symbol the subject does not declare.
    ConstraintOnUndeclaredSymbol {
        /// The symbol the relation named.
        symbol: ShapeSymbol,
    },
    /// Re-encoding the decoded subject did not reproduce the carried bytes.
    EncodingMismatch,
}

impl fmt::Display for ShapeEnvSubjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => formatter.write_str("shape-env-subject.truncated"),
            Self::BadDomain => formatter.write_str("shape-env-subject.bad-domain"),
            Self::TrailingBytes => formatter.write_str("shape-env-subject.trailing-bytes"),
            Self::Limit { actual, limit } => {
                write!(
                    formatter,
                    "shape-env-subject.limit: {actual} exceeds {limit}"
                )
            }
            Self::InvalidText => formatter.write_str("shape-env-subject.invalid-text"),
            Self::Malformed { reason } => {
                write!(formatter, "shape-env-subject.malformed: {reason}")
            }
            Self::UnknownBindingSource { tag } => {
                write!(
                    formatter,
                    "shape-env-subject.unknown-binding-source: {tag:#04x}"
                )
            }
            Self::UnknownPhase { tag } => {
                write!(formatter, "shape-env-subject.unknown-phase: {tag:#04x}")
            }
            Self::UnknownProvenance { tag } => {
                write!(
                    formatter,
                    "shape-env-subject.unknown-provenance: {tag:#04x}"
                )
            }
            Self::UnknownTerm { tag } => {
                write!(formatter, "shape-env-subject.unknown-term: {tag:#04x}")
            }
            Self::UnknownRelation { tag } => {
                write!(formatter, "shape-env-subject.unknown-relation: {tag:#04x}")
            }
            Self::UnsupportedBindingSource { symbol, source } => write!(
                formatter,
                "shape-env-subject.unsupported-binding-source: {symbol} ({source})"
            ),
            Self::ZeroDivisor => formatter.write_str("shape-env-subject.zero-divisor"),
            Self::UnorderedBindings => formatter.write_str("shape-env-subject.unordered-bindings"),
            Self::UnorderedConstraints => {
                formatter.write_str("shape-env-subject.unordered-constraints")
            }
            Self::ConstraintOnUndeclaredSymbol { symbol } => write!(
                formatter,
                "shape-env-subject.constraint-on-undeclared-symbol: {symbol}"
            ),
            Self::EncodingMismatch => formatter.write_str("shape-env-subject.encoding-mismatch"),
        }
    }
}

impl Error for ShapeEnvSubjectError {}

/// One decoded identity-bearing shape-environment subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapeEnvSubject {
    /// Canonical declarations and root bindings.
    pub bindings: Vec<(ShapeSymbol, RootBinding)>,
    /// Canonical semantic input constraints.
    pub constraints: Vec<SemanticInputConstraint>,
}

/// Encodes one bound environment canonically.
///
/// Domain-separated and length-prefixed per ADR 0074, over the entries and
/// constraints in the canonical order `build` established, so the bytes are a
/// function of the environment rather than of authoring order. Guards are not
/// encoded; the retained proof summary is not an identity component.
pub(super) fn encode_environment(
    entries: &[(ShapeSymbol, RootBinding)],
    constraints: &[SemanticInputConstraint],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, SHAPE_ENV_DOMAIN);
    push_len(&mut bytes, entries.len());
    for (symbol, binding) in entries {
        symbol.encode(&mut bytes);
        binding.encode(&mut bytes);
    }
    push_len(&mut bytes, constraints.len());
    for constraint in constraints {
        constraint.encode(&mut bytes);
    }
    bytes
}

/// Encodes the identity-bearing subject of one verified shape environment.
#[must_use]
pub fn encode_shape_env_subject(
    entries: &[(ShapeSymbol, RootBinding)],
    constraints: &[SemanticInputConstraint],
) -> Vec<u8> {
    encode_environment(entries, constraints)
}

/// Decodes one identity-bearing shape-environment subject and revalidates it.
///
/// Canonical order, table closure, and exact re-encoding are proved before a
/// view is returned, so the carried bytes stay the one authority.
///
/// # Errors
///
/// Returns [`ShapeEnvSubjectError`] when the bytes are not the canonical
/// encoding of one closed, ordered subject.
pub fn decode_shape_env_subject(bytes: &[u8]) -> Result<ShapeEnvSubject, ShapeEnvSubjectError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.slice()? != SHAPE_ENV_DOMAIN {
        return Err(ShapeEnvSubjectError::BadDomain);
    }
    let binding_count = cursor.count(MAX_SUBJECT_ROWS)?;
    let mut bindings = Vec::with_capacity(binding_count);
    for _ in 0..binding_count {
        let symbol = decode_symbol(&mut cursor)?;
        let binding = decode_binding(&mut cursor)?;
        bindings.push((symbol, binding));
    }
    let constraint_count = cursor.count(MAX_SUBJECT_ROWS)?;
    let mut constraints = Vec::with_capacity(constraint_count);
    for _ in 0..constraint_count {
        constraints.push(decode_constraint(&mut cursor)?);
    }
    if cursor.remaining() != 0 {
        return Err(ShapeEnvSubjectError::TrailingBytes);
    }

    if bindings.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(ShapeEnvSubjectError::UnorderedBindings);
    }
    if constraints.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ShapeEnvSubjectError::UnorderedConstraints);
    }
    for constraint in &constraints {
        let mut undeclared = None;
        constraint.relation().for_each_symbol(|symbol| {
            if undeclared.is_none() && bindings.iter().all(|(declared, _)| declared != symbol) {
                undeclared = Some(symbol.clone());
            }
        });
        if let Some(symbol) = undeclared {
            return Err(ShapeEnvSubjectError::ConstraintOnUndeclaredSymbol { symbol });
        }
    }

    let encoded = encode_environment(&bindings, &constraints);
    if encoded != bytes {
        return Err(ShapeEnvSubjectError::EncodingMismatch);
    }
    Ok(ShapeEnvSubject {
        bindings,
        constraints,
    })
}

fn decode_symbol(cursor: &mut Cursor<'_>) -> Result<ShapeSymbol, ShapeEnvSubjectError> {
    let scope = SymbolScope::new(cursor.slice()?).map_err(malformed)?;
    let name = cursor.text()?;
    ShapeSymbol::new(scope, name).map_err(malformed)
}

fn decode_binding(cursor: &mut Cursor<'_>) -> Result<RootBinding, ShapeEnvSubjectError> {
    let source = decode_source(cursor)?;
    let phase_tag = cursor.u8()?;
    let phase = AvailabilityPhase::from_tag(phase_tag)
        .ok_or(ShapeEnvSubjectError::UnknownPhase { tag: phase_tag })?;
    let provenance_tag = cursor.u8()?;
    let provenance = FactProvenance::from_tag(provenance_tag).ok_or(
        ShapeEnvSubjectError::UnknownProvenance {
            tag: provenance_tag,
        },
    )?;
    RootBinding::new(source, phase, provenance).map_err(malformed)
}

fn decode_source(cursor: &mut Cursor<'_>) -> Result<BindingSource, ShapeEnvSubjectError> {
    let tag = cursor.u8()?;
    match tag {
        0x01 => Ok(BindingSource::Static(Extent::new(cursor.u64()?))),
        0x02 => {
            let input = InputKey::new(cursor.text()?).map_err(malformed)?;
            let axis = Axis::new(cursor.u32()?);
            Ok(BindingSource::InputDimension { input, axis })
        }
        0x03 => {
            let key = InterfaceParameterKey::new(cursor.text()?).map_err(malformed)?;
            Ok(BindingSource::InterfaceParameter { key })
        }
        0x04 => {
            let key = TargetPropertyKey::new(cursor.text()?).map_err(malformed)?;
            Ok(BindingSource::TargetProperty { key })
        }
        tag => Err(ShapeEnvSubjectError::UnknownBindingSource { tag }),
    }
}

fn decode_constraint(
    cursor: &mut Cursor<'_>,
) -> Result<SemanticInputConstraint, ShapeEnvSubjectError> {
    let relation = decode_relation(cursor)?;
    let provenance_tag = cursor.u8()?;
    let provenance = FactProvenance::from_tag(provenance_tag).ok_or(
        ShapeEnvSubjectError::UnknownProvenance {
            tag: provenance_tag,
        },
    )?;
    Ok(SemanticInputConstraint::new(relation, provenance))
}

fn decode_relation(cursor: &mut Cursor<'_>) -> Result<ExtentRelation, ShapeEnvSubjectError> {
    let tag = cursor.u8()?;
    match tag {
        0x01 => Ok(ExtentRelation::equal(
            decode_term(cursor)?,
            decode_term(cursor)?,
        )),
        0x02 => {
            let dividend = decode_term(cursor)?;
            let divisor =
                NonZeroU64::new(cursor.u64()?).ok_or(ShapeEnvSubjectError::ZeroDivisor)?;
            Ok(ExtentRelation::divisible(dividend, divisor))
        }
        0x03 => Ok(ExtentRelation::non_negative_difference(
            decode_term(cursor)?,
            decode_term(cursor)?,
        )),
        0x04 => ExtentRelation::interval(decode_term(cursor)?, cursor.u64()?, cursor.u64()?)
            .map_err(malformed),
        0x05 => {
            let product = decode_term(cursor)?;
            let factor_count = cursor.count(MAX_FACTORS)?;
            let mut factors = Vec::with_capacity(factor_count);
            for _ in 0..factor_count {
                factors.push(decode_term(cursor)?);
            }
            ExtentRelation::factorization(product, factors).map_err(malformed)
        }
        0x06 => Ok(ExtentRelation::additive_equality(
            decode_term(cursor)?,
            decode_term(cursor)?,
            decode_term(cursor)?,
        )),
        tag => Err(ShapeEnvSubjectError::UnknownRelation { tag }),
    }
}

fn decode_term(cursor: &mut Cursor<'_>) -> Result<ExtentTerm, ShapeEnvSubjectError> {
    let tag = cursor.u8()?;
    match tag {
        0x01 => Ok(ExtentTerm::Symbol(decode_symbol(cursor)?)),
        0x02 => Ok(ExtentTerm::Constant(cursor.u64()?)),
        tag => Err(ShapeEnvSubjectError::UnknownTerm { tag }),
    }
}

fn malformed(error: impl fmt::Display) -> ShapeEnvSubjectError {
    ShapeEnvSubjectError::Malformed {
        reason: error.to_string(),
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    const fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], ShapeEnvSubjectError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(ShapeEnvSubjectError::Truncated)?;
        let taken = self
            .bytes
            .get(self.position..end)
            .ok_or(ShapeEnvSubjectError::Truncated)?;
        self.position = end;
        Ok(taken)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], ShapeEnvSubjectError> {
        Ok(self
            .take(N)?
            .try_into()
            .expect("a checked read of N bytes is an N-byte array"))
    }

    fn u8(&mut self) -> Result<u8, ShapeEnvSubjectError> {
        Ok(self.array::<1>()?[0])
    }

    fn u32(&mut self) -> Result<u32, ShapeEnvSubjectError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, ShapeEnvSubjectError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    fn count(&mut self, limit: usize) -> Result<usize, ShapeEnvSubjectError> {
        let declared = self.u64()?;
        let count = usize::try_from(declared).map_err(|_| ShapeEnvSubjectError::Limit {
            actual: declared,
            limit,
        })?;
        if count > limit {
            return Err(ShapeEnvSubjectError::Limit {
                actual: declared,
                limit,
            });
        }
        Ok(count)
    }

    fn slice(&mut self) -> Result<&'a [u8], ShapeEnvSubjectError> {
        let len = self.count(self.remaining())?;
        self.take(len)
    }

    fn text(&mut self) -> Result<String, ShapeEnvSubjectError> {
        let bytes = self.slice()?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ShapeEnvSubjectError::InvalidText)
    }
}
