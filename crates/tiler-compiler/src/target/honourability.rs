//! Per-dimension, per-dtype numerical honourability, a peer of the capability
//! authority.
//!
//! ADR 0076 item 3. A target profile declares, for each dimension of the
//! resolved numerical contract it can be asked about *and for each arithmetic
//! type it can be asked about it in*, which behaviour it honours and *by what
//! means*. This module owns that vocabulary; the composition of a declaration and
//! a caller requirement into one ADR 0043 outcome lives beside the capability
//! assessment in [`crate::target::feasibility`], because a candidate has exactly one
//! feasibility verdict and the two kinds of predicate contribute to it together.
//!
//! # Why the key carries an arithmetic type
//!
//! **Measurement.** On one Apple row — same GPU, same math modes, modules
//! declaring `air.compile.denorms_disable` identically — `f32` arithmetic flushes
//! subnormals, `f16` arithmetic preserves them, and `bf16` flushes. So on that
//! one profile, [`NumericalDimension::InputSubnormals`] is honoured
//! [`HonouringMeans::SupportedExactly`] for `f16` and
//! [`HonouringMeans::Unsupported`] for `f32`.
//!
//! **Inference.** A declaration keyed by dimension alone therefore has to state
//! one of those two wrongly, and a preset assuming one behaviour per dimension
//! per profile assumes something already known to be false. The key is
//! `(dimension, arithmetic type)` for that reason, not for symmetry; an
//! arithmetic type a profile does not speak about is `Unknown` in ADR 0043's
//! exact sense and fails closed, exactly as an unenumerated dimension does.
//!
//! # Why this is not a `CapabilityAxis`
//!
//! [`crate::target::feasibility::CapabilityAxis`] is a quantitative space: a `u64`
//! bound, a [`crate::explain::Quantity`] unit, and an `AtMost`/`Exact`/`Implies`
//! relation. Numerical honourability is not a quantity, and the decisive point
//! is that [`HonouringMeans::SupportedWithExactEmulation`] has no representation
//! as a bound comparison — emulation is honoured by *emitting different
//! operations*, so it changes the program rather than the verdict, and encoding
//! it as a satisfied `Implies` predicate would discard exactly the outcome that
//! carries work.
//!
//! # The honesty rule this vocabulary exists to enforce
//!
//! No authority may narrow, weaken, or substitute the caller's stated numerical
//! contract in order to make a target feasible (ADR 0076 item 5). Nothing here
//! computes a *nearest honourable* behaviour, and nothing ranks one behaviour
//! against another: a required behaviour is either declared honourable, declared
//! unhonourable, or not spoken to at all. The consequence is that the numerical
//! contract is not a search dimension — cost may rank implementations of one
//! contract and may never rank contracts against each other, because that would
//! price meaning.

use std::sync::{Arc, OnceLock};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::schedule::ArithmeticType;
use tiler_ir::semantic::ResolvedValueType;

use crate::target::feasibility::{FactProvenance, TargetProfileIdentity};

// The scalar-arithmetic policy vocabulary is `tiler_ir::numerics`, not this
// module's. Two crates each declared a dimension set and they had drifted to
// four cases and eleven; naming the one authority by re-export is what keeps a
// widened vocabulary a build error at every consumer rather than a silent
// disagreement between siblings. The re-export is deliberate rather than a
// convenience: every `crate::target::honourability::X` path in this crate keeps
// resolving, so the relocation is invisible to the assessment logic that is
// this module's actual subject.
pub(crate) use tiler_ir::numerics::{
    CANONICAL_DIMENSIONS, CompilerBuildIdentity, CompilerBuildRole, DimensionBehaviour,
    ExecutionEnvironmentIdentity, FactAuthority, FactEvidenceBasis, FactSourceProvenance,
    FactValidityScope, HonouringMeans, MAX_COMPILER_BUILDS_PER_CONTEXT,
    MAX_MEASUREMENT_CONTEXTS_PER_SOURCE, MAX_PROVENANCE_TEXT_BYTES, MeasurementContext,
    NumericalDimension, ProvenanceIdentity,
};
pub(crate) use tiler_ir::program::abi::AvailabilityPhase;

/// Compile-profile measurement provenance for one exact build and environment.
///
/// The three parameters are the three knobs a test varies to change *only* the
/// evidence behind a refusal: which authority made the measurement, on which
/// compiler build, in which execution environment. Nothing here touches the
/// declared behaviour or the means, so a difference observed downstream is
/// attributable to provenance alone.
#[cfg(test)]
pub(crate) fn measured_profile_source(
    authority_key: &str,
    compiler_version: &str,
    platform_build: &str,
) -> Arc<FactSourceProvenance> {
    Arc::new(FactSourceProvenance::measured(
        AvailabilityPhase::CompileProfile,
        FactAuthority::MeasuredProfile,
        FactValidityScope::MeasuredEnvironment,
        ProvenanceIdentity::new(authority_key, 1),
        vec![MeasurementContext::new(
            vec![CompilerBuildIdentity::new(
                CompilerBuildRole::CodeGenerator,
                "test-offline-compiler",
                compiler_version,
                None,
            )],
            ExecutionEnvironmentIdentity::new(
                "test-platform",
                "1.0",
                platform_build,
                "test-architecture",
                "test-hardware",
            ),
        )],
    ))
}

pub(crate) fn governed_profile_source() -> Arc<FactSourceProvenance> {
    static SOURCE: OnceLock<Arc<FactSourceProvenance>> = OnceLock::new();
    Arc::clone(SOURCE.get_or_init(|| {
        Arc::new(FactSourceProvenance::governed(
            ProvenanceIdentity::new("tiler.governed-target-profile-authority.v1", 1),
            ProvenanceIdentity::new("tiler.prototype-target-neutral-baseline.v1", 1),
        ))
    }))
}

/// One line of a target profile's honourability declaration.
///
/// Rows share one immutable structured source record; a checked profile then
/// attributes each row to the declaring profile's identity.
/// [`NumericalHonourabilityFact`] is that attributed form. The split mirrors how a
/// [`crate::target::feasibility::CapabilityFact`]'s provenance is bound at checking time
/// rather than restated by every declarer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeclaredBehaviour {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    resolved_type: ResolvedValueType,
    behaviour: DimensionBehaviour,
    means: HonouringMeans,
    source: Arc<FactSourceProvenance>,
}

impl DeclaredBehaviour {
    /// Declares how a target honours one behaviour of one dimension, in one
    /// arithmetic type.
    pub(crate) fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        resolved_type: ResolvedValueType,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
        source: Arc<FactSourceProvenance>,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            resolved_type,
            behaviour,
            means,
            source,
        }
    }

    /// Declares a governed-profile honourability fixture.
    ///
    /// Tests use this shorthand for a portable profile fact known before any
    /// artifact exists. Production profile builders supply their fact source
    /// explicitly through [`Self::new`].
    #[allow(
        dead_code,
        reason = "test-fixture shorthand for a governed compile-profile row; production profile builders supply explicit fact sources through DeclaredBehaviour::new"
    )]
    pub(crate) fn compile_profile(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        resolved_type: ResolvedValueType,
        behaviour: DimensionBehaviour,
        means: HonouringMeans,
    ) -> Self {
        Self::new(
            dimension,
            arithmetic,
            resolved_type,
            behaviour,
            means,
            governed_profile_source(),
        )
    }

    /// Binds this declaration to the profile that declared it.
    pub(crate) fn attributed_to(
        &self,
        profile: impl Into<TargetProfileIdentity>,
    ) -> NumericalHonourabilityFact {
        NumericalHonourabilityFact {
            declaration: self.clone(),
            provenance: FactProvenance::declared_by(profile),
        }
    }

    /// Appends this declaration's canonical bytes.
    ///
    /// The one encoding of a declared behaviour in this crate. A checked profile
    /// descriptor and a request subject both reach it, so a widened vocabulary
    /// cannot change one and leave the other reading the old shape.
    fn encode_declaration_body(&self, bytes: &mut Vec<u8>, subject_index: usize) {
        bytes.push(self.dimension.tag());
        encode_compact_index(bytes, subject_index);
        self.behaviour.encode(bytes);
        self.means.encode(bytes);
    }

    fn subject_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.arithmetic.tag()];
        push_slice(
            &mut bytes,
            self.resolved_type.canonical_encoding().as_bytes(),
        );
        bytes
    }
}

/// Encodes declared rows with one canonical, deduplicated source table.
#[allow(
    dead_code,
    reason = "test-facing raw-declaration identity projection; production encodes attributed facts after profile validation"
)]
pub(crate) fn encode_declared_behaviours(bytes: &mut Vec<u8>, declarations: &[DeclaredBehaviour]) {
    let declarations: Vec<_> = declarations.iter().collect();
    encode_declaration_table(bytes, &declarations);
}

/// Encodes checked facts with one canonical, deduplicated source table.
pub(crate) fn encode_honourability_facts(
    bytes: &mut Vec<u8>,
    facts: &[NumericalHonourabilityFact],
) {
    let declarations: Vec<_> = facts.iter().map(|fact| &fact.declaration).collect();
    encode_declaration_table(bytes, &declarations);
}

fn encode_declaration_table(bytes: &mut Vec<u8>, declarations: &[&DeclaredBehaviour]) {
    let mut sources: Vec<_> = declarations
        .iter()
        .map(|declaration| {
            let source = declaration.source.as_ref();
            (source.canonical_bytes(), source)
        })
        .collect();
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    sources.dedup_by(|left, right| left.0 == right.0);

    push_len(bytes, sources.len());
    for (_, source) in &sources {
        source.encode(bytes);
    }

    let mut subjects: Vec<_> = declarations
        .iter()
        .map(|declaration| declaration.subject_bytes())
        .collect();
    subjects.sort();
    subjects.dedup();
    push_len(bytes, subjects.len());
    for subject in &subjects {
        push_slice(bytes, subject);
    }

    let mut rows = Vec::with_capacity(declarations.len());
    for declaration in declarations {
        let source_key = declaration.source.canonical_bytes();
        let source_index = sources
            .binary_search_by(|candidate| candidate.0.cmp(&source_key))
            .expect("every declaration source was collected into the source table");
        let subject_key = declaration.subject_bytes();
        let subject_index = subjects
            .binary_search(&subject_key)
            .expect("every declaration subject was collected into the subject table");
        let mut row = Vec::new();
        declaration.encode_declaration_body(&mut row, subject_index);
        encode_compact_index(&mut row, source_index);
        rows.push(row);
    }
    rows.sort_unstable();
    push_len(bytes, rows.len());
    for row in rows {
        bytes.extend_from_slice(&row);
    }
}

fn encode_compact_index(bytes: &mut Vec<u8>, mut value: usize) {
    loop {
        let low = u8::try_from(value & 0x7f).expect("seven masked bits fit in u8");
        value >>= 7;
        if value == 0 {
            bytes.push(low);
            break;
        }
        bytes.push(low | 0x80);
    }
}

/// A typed honourability fact: how one behaviour of one dimension is honoured,
/// in one arithmetic type.
///
/// It carries the same provenance discipline a
/// [`crate::target::feasibility::CapabilityFact`] does — an availability phase, a fact
/// authority, a validity scope, and the declaring profile's identity — so a
/// rejection can name where the claim came from (ADR 0076 item 3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NumericalHonourabilityFact {
    declaration: DeclaredBehaviour,
    provenance: FactProvenance,
}

impl NumericalHonourabilityFact {
    /// The dimension this fact speaks about.
    pub(crate) const fn dimension(&self) -> NumericalDimension {
        self.declaration.dimension
    }

    /// The arithmetic type this fact speaks about.
    pub(crate) const fn arithmetic(&self) -> ArithmeticType {
        self.declaration.arithmetic
    }

    /// The complete resolved semantic type this fact speaks about.
    pub(crate) const fn resolved_type(&self) -> &ResolvedValueType {
        &self.declaration.resolved_type
    }

    /// The behaviour of that dimension this fact speaks about.
    pub(crate) const fn behaviour(&self) -> DimensionBehaviour {
        self.declaration.behaviour
    }

    /// The means by which the behaviour is honoured, if it is.
    pub(crate) fn means(&self) -> HonouringMeans {
        self.declaration.means.clone()
    }

    /// The phase from which this fact is available.
    pub(crate) fn phase(&self) -> AvailabilityPhase {
        self.declaration.source.phase()
    }

    /// The authority vouching for this fact.
    pub(crate) fn authority(&self) -> FactAuthority {
        self.declaration.source.authority()
    }

    /// The scope over which this fact remains valid.
    pub(crate) fn validity(&self) -> FactValidityScope {
        self.declaration.source.validity()
    }

    /// The structured source statement supplied by the declaring authority.
    pub(crate) fn source(&self) -> &FactSourceProvenance {
        &self.declaration.source
    }

    /// Where this fact came from.
    pub(crate) const fn provenance(&self) -> &FactProvenance {
        &self.provenance
    }

    /// The canonical sort key: dimension, arithmetic type, behaviour, phase.
    ///
    /// The behaviour contributes its canonical bytes rather than a fixed-width
    /// tag, because one behaviour space is variable-width: two distinct accuracy
    /// envelopes would tie under a tag, and the duplicate check this key feeds
    /// would then reject a profile that declared both.
    pub(crate) fn sort_key(&self) -> (u8, u8, Vec<u8>, Vec<u8>, AvailabilityPhase) {
        (
            self.declaration.dimension.tag(),
            self.declaration.arithmetic.tag(),
            self.declaration
                .resolved_type
                .canonical_encoding()
                .as_bytes()
                .to_vec(),
            self.declaration.behaviour.canonical_key(),
            self.declaration.source.phase(),
        )
    }

    /// Whether this fact declares the behaviour honoured without conditions.
    ///
    /// A conditional means is deliberately excluded: whether it honours anything
    /// depends on the request, so it is not an alternative the profile *offers*.
    pub(crate) const fn is_unconditionally_honoured(&self) -> bool {
        matches!(
            self.declaration.means,
            HonouringMeans::SupportedExactly | HonouringMeans::SupportedWithExactEmulation
        )
    }
}

/// A candidate requirement: the behaviour the caller's contract needs on one
/// dimension, in one arithmetic type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NumericalRequirement {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    resolved_type: ResolvedValueType,
    behaviour: DimensionBehaviour,
}

impl NumericalRequirement {
    /// Requires `behaviour` on `dimension`, for `arithmetic`.
    pub(crate) fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        resolved_type: ResolvedValueType,
        behaviour: DimensionBehaviour,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            resolved_type,
            behaviour,
        }
    }

    /// The dimension this requirement ranges over.
    pub(crate) const fn dimension(&self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type this requirement is stated for.
    pub(crate) const fn arithmetic(&self) -> ArithmeticType {
        self.arithmetic
    }

    /// The complete resolved semantic type this requirement is stated for.
    pub(crate) const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    /// The behaviour the contract requires.
    pub(crate) const fn behaviour(&self) -> DimensionBehaviour {
        self.behaviour
    }

    /// The canonical key this requirement is unique under.
    pub(crate) fn subject(&self) -> (NumericalDimension, ArithmeticType, Vec<u8>) {
        (
            self.dimension,
            self.arithmetic,
            self.resolved_type.canonical_encoding().as_bytes().to_vec(),
        )
    }
}

/// A dimension whose required behaviour the target honours, and by what means.
///
/// The means is retained rather than collapsed to a boolean because it is what
/// an artifact record and a cost model both need: an emulated dimension is
/// honoured by emitted operations, which is work that a satisfied predicate
/// alone would hide.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HonouredDimension {
    fact: NumericalHonourabilityFact,
    canonical_key: Arc<[u8]>,
}

impl HonouredDimension {
    pub(crate) fn new(fact: NumericalHonourabilityFact) -> Self {
        let mut canonical_key = Vec::new();
        encode_honourability_facts(&mut canonical_key, std::slice::from_ref(&fact));
        push_slice(
            &mut canonical_key,
            fact.provenance().profile().key().as_bytes(),
        );
        Self {
            fact,
            canonical_key: Arc::from(canonical_key),
        }
    }

    /// The dimension honoured.
    pub(crate) const fn dimension(&self) -> NumericalDimension {
        self.fact.dimension()
    }

    /// The arithmetic type it is honoured in.
    pub(crate) const fn arithmetic(&self) -> ArithmeticType {
        self.fact.arithmetic()
    }

    /// The complete resolved semantic type it is honoured in.
    pub(crate) const fn resolved_type(&self) -> &ResolvedValueType {
        self.fact.resolved_type()
    }

    /// The behaviour the contract required.
    pub(crate) const fn behaviour(&self) -> DimensionBehaviour {
        self.fact.behaviour()
    }

    /// The means by which the target honours it.
    pub(crate) fn means(&self) -> HonouringMeans {
        self.fact.means()
    }

    /// The profile that declared the honouring means.
    pub(crate) const fn profile(&self) -> &TargetProfileIdentity {
        self.fact.provenance().profile()
    }

    /// The exact checked fact that justified selection.
    pub(crate) const fn fact(&self) -> &NumericalHonourabilityFact {
        &self.fact
    }

    /// Complete canonical evidence key, including the declaring profile.
    pub(crate) fn canonical_key(&self) -> &[u8] {
        &self.canonical_key
    }
}

/// A dimension the target declares it cannot honour as required.
///
/// This is the rejection shape ADR 0076 item 5 requires, and it is what replaces
/// `strict-f32: required 1, available 0`. It retains the **exact checked fact
/// that refused** rather than a summary copied out of it, so the dimension, the
/// arithmetic type, the behaviour the profile declared, the means it offers, the
/// declaring profile, and the whole of that fact's structured provenance —
/// authority, validity scope, compiler builds, execution environments — survive
/// every rejection hop to the diagnostic surfaces. A rejection rebuilt from
/// scalar means and a profile key cannot answer *who measured this, on what
/// build, in what environment*, and a caller deciding whether a refusal applies
/// to its own deployment needs exactly that.
///
/// The caller-required behaviour is kept beside the fact rather than read out of
/// it, because the two answer different questions: the fact states what the
/// target declares about a behaviour, and `required` states what the caller's
/// contract asked for. They coincide under today's resolution rule, which
/// matches a fact by required behaviour; collapsing them would make a change to
/// that rule silently misreport the caller's contract.
///
/// The fact is held by shared immutable ownership. A rejection is cloned at
/// every hop — feasibility to physical to frontier to opaque call to explain —
/// and the measured provenance it carries is unbounded-in-principle structure;
/// sharing it keeps those clones from duplicating measurement contexts and makes
/// "the same fact reached the diagnostic" checkable by pointer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnhonouredDimension {
    fact: Arc<NumericalHonourabilityFact>,
    required: DimensionBehaviour,
    honoured: Option<DimensionBehaviour>,
}

impl UnhonouredDimension {
    /// Records that `fact` refused `required`, offering `honoured` instead.
    ///
    /// There is no provenance-free constructor. A refusal names a fact a profile
    /// declared, and a synthetic one assembled from a dimension, a means, and a
    /// profile key would be a claim no authority made — indistinguishable, once
    /// it reached a diagnostic, from evidence.
    pub(crate) fn new(
        fact: NumericalHonourabilityFact,
        required: DimensionBehaviour,
        honoured: Option<DimensionBehaviour>,
    ) -> Self {
        Self {
            fact: Arc::new(fact),
            required,
            honoured,
        }
    }

    /// The dimension the contract could not be honoured on.
    pub(crate) fn dimension(&self) -> NumericalDimension {
        self.fact.dimension()
    }

    /// The arithmetic type it could not be honoured in.
    ///
    /// Reported because the same dimension can be honoured in one type and
    /// unhonourable in another on one profile: a rejection that named only the
    /// dimension would be false about the other type.
    pub(crate) fn arithmetic(&self) -> ArithmeticType {
        self.fact.arithmetic()
    }

    /// The complete resolved semantic type it could not be honoured in.
    pub(crate) fn resolved_type(&self) -> &ResolvedValueType {
        self.fact.resolved_type()
    }

    /// The behaviour the caller's contract required.
    pub(crate) const fn required(&self) -> DimensionBehaviour {
        self.required
    }

    /// The behaviour the refusing declaration speaks about.
    pub(crate) fn declared(&self) -> DimensionBehaviour {
        self.fact.behaviour()
    }

    /// The means the profile declares for the required behaviour.
    pub(crate) fn means(&self) -> HonouringMeans {
        self.fact.means()
    }

    /// The behaviour on this dimension the profile does honour unconditionally,
    /// in canonical order, when it honours one at all.
    ///
    /// It is reported so a caller can see what contract this target would
    /// accept. It is never substituted for the stated one: only the caller may
    /// change what its program means (ADR 0076 item 5).
    pub(crate) const fn honoured(&self) -> Option<DimensionBehaviour> {
        self.honoured
    }

    /// The profile that declared the means.
    pub(crate) fn profile(&self) -> &TargetProfileIdentity {
        self.fact.provenance().profile()
    }

    /// The exact checked fact that refused, with its complete provenance.
    pub(crate) fn evidence(&self) -> NumericalRefusalEvidence {
        NumericalRefusalEvidence(Arc::clone(&self.fact))
    }

    /// Appends the complete canonical evidence for this refusal.
    ///
    /// The one encoding of a refused dimension in this crate: the frontier's
    /// rejection identity, its opaque-call cause, and the explain record all
    /// reach it, so a widened fact cannot change one and leave the others
    /// encoding the old shape.
    pub(crate) fn encode(&self, bytes: &mut Vec<u8>) {
        let Self {
            fact,
            required,
            honoured,
        } = self;
        required.encode(bytes);
        match honoured {
            Some(honoured) => {
                bytes.push(0x01);
                honoured.encode(bytes);
            }
            None => bytes.push(0x00),
        }
        push_slice(
            bytes,
            &NumericalRefusalEvidence(Arc::clone(fact)).canonical_bytes(),
        );
    }
}

/// The exact checked fact behind one refusal, carried by shared ownership.
///
/// A read-only carrier rather than a second copy of the fact's fields: the
/// rejection pipeline and the explain record hold the same instance, so no stage
/// can reconstruct a plausible fact that the authority never declared. The
/// [`Arc`] is private and never crosses the public boundary; the session facade
/// reads through borrowed accessors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NumericalRefusalEvidence(Arc<NumericalHonourabilityFact>);

impl NumericalRefusalEvidence {
    /// The behaviour the declaration speaks about.
    pub(crate) fn declared(&self) -> DimensionBehaviour {
        self.0.behaviour()
    }

    /// The means the declaration offers for that behaviour.
    pub(crate) fn means(&self) -> HonouringMeans {
        self.0.means()
    }

    /// The phase from which the declaration is available.
    pub(crate) fn phase(&self) -> AvailabilityPhase {
        self.0.phase()
    }

    /// The authority vouching for the declaration.
    pub(crate) fn authority(&self) -> FactAuthority {
        self.0.authority()
    }

    /// The scope over which the declaration remains valid.
    pub(crate) fn validity(&self) -> FactValidityScope {
        self.0.validity()
    }

    /// The complete structured source statement the declarer supplied.
    pub(crate) fn source(&self) -> &FactSourceProvenance {
        self.0.source()
    }

    /// The profile that declared the fact.
    pub(crate) fn profile(&self) -> &TargetProfileIdentity {
        self.0.provenance().profile()
    }

    /// Whether two refusals cite the exact same retained fact instance.
    ///
    /// Pointer equality rather than structural equality, and that is the point:
    /// it is the only check that distinguishes a fact carried through the
    /// rejection pipeline from one rebuilt later with equal contents, which is
    /// exactly the reconstruction this type exists to rule out.
    #[allow(
        dead_code,
        reason = "test-only pointer-identity witness that rejection pipelines retain the originating fact; production consumes the carried fact's contents"
    )]
    pub(crate) fn cites_same_fact(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// The complete canonical bytes of the refusing fact, with its declarer.
    ///
    /// Delegated to [`encode_honourability_facts`] rather than written again:
    /// a checked profile descriptor and a rejection identity must agree about
    /// what a fact *is*, and two encoders of one vocabulary drift.
    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_honourability_facts(&mut bytes, std::slice::from_ref(self.0.as_ref()));
        push_slice(&mut bytes, self.profile().key().as_bytes());
        bytes
    }

    /// Renders the complete declaration and provenance into an explanation.
    pub(crate) fn render(&self, output: &mut String) {
        use std::fmt::Write as _;
        let _ = write!(
            output,
            "declares={}:means={}:",
            self.declared().key(),
            self.means().label()
        );
        self.source().render(output);
    }
}

/// A dimension the profile does not speak to at all, in the required arithmetic
/// type.
///
/// ADR 0043's `Unknown` in its exact sense — no admissible proof or query path —
/// and the clause that makes an unenumerated dimension fail closed instead of
/// defaulting to honoured. A profile that enumerates the dimension but not the
/// *required behaviour* is the same case for the same reason, and so is one that
/// enumerates both but only for another arithmetic type: nothing declared says
/// how that behaviour would be realized in this one, and a neighbouring type's
/// fact is measurably not a substitute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UndeclaredDimension {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    resolved_type: Arc<ResolvedValueType>,
    required: DimensionBehaviour,
}

impl UndeclaredDimension {
    pub(crate) fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        resolved_type: ResolvedValueType,
        required: DimensionBehaviour,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            resolved_type: Arc::new(resolved_type),
            required,
        }
    }

    /// The dimension nothing available declares.
    pub(crate) const fn dimension(&self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type nothing available declares it for.
    pub(crate) const fn arithmetic(&self) -> ArithmeticType {
        self.arithmetic
    }

    /// The complete resolved semantic type nothing available declares.
    pub(crate) fn resolved_type(&self) -> &ResolvedValueType {
        self.resolved_type.as_ref()
    }

    /// The behaviour the caller's contract required.
    pub(crate) const fn required(&self) -> DimensionBehaviour {
        self.required
    }
}

/// A dimension whose declaration is admissible only from a later phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeferredDimension {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    resolved_type: Arc<ResolvedValueType>,
    required: DimensionBehaviour,
    phase: AvailabilityPhase,
}

impl DeferredDimension {
    pub(crate) fn new(
        dimension: NumericalDimension,
        arithmetic: ArithmeticType,
        resolved_type: ResolvedValueType,
        required: DimensionBehaviour,
        phase: AvailabilityPhase,
    ) -> Self {
        Self {
            dimension,
            arithmetic,
            resolved_type: Arc::new(resolved_type),
            required,
            phase,
        }
    }

    /// The dimension whose declaration is not yet available.
    pub(crate) const fn dimension(&self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type whose declaration is not yet available.
    pub(crate) const fn arithmetic(&self) -> ArithmeticType {
        self.arithmetic
    }

    /// The complete resolved semantic type whose declaration is deferred.
    pub(crate) fn resolved_type(&self) -> &ResolvedValueType {
        self.resolved_type.as_ref()
    }

    /// The behaviour the caller's contract required.
    pub(crate) const fn required(&self) -> DimensionBehaviour {
        self.required
    }

    /// The earliest phase that can supply the declaration.
    pub(crate) const fn phase(&self) -> AvailabilityPhase {
        self.phase
    }
}
