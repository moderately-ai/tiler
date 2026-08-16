//! Kani harnesses over *copies* of four Tiler identity encoders.
//!
//! # Read this before citing any result from this crate
//!
//! `crates/tiler-ir` does not compile under Kani 0.67.0's bundled rustc
//! (1.93.0-nightly, 2025-11-21). The diagnostic is recorded in the README and in
//! `docs/research/verification/kani-bounded-encoder-verification.md`. Every encoder
//! below is therefore a **token-content copy under `guard.sh`'s normalization**
//! of a function that lives in the crates, not the function itself.
//!
//! **A proof here proves the copy, not the source.** The two are tied by exactly
//! one thing: `guard.sh` re-extracts each named function from its source file and
//! compares normalized token content against the copy here, so a semantic source
//! edit that is not mirrored here fails the guard instead of silently leaving a
//! stale proof standing. That is a *text-derived token* tie, and it is weaker
//! than compiling the real crate in three ways worth naming:
//!
//! - it does not tie the *types*. `SubnormalMode` below is a copy too, and a
//!   variant added to the real enum widens the real domain without touching this
//!   file's text. The guard would stay green.
//! - it does not tie the *callers*. Fixed width and prefix-freeness matter only
//!   relative to what a record writes next, and no caller is copied here.
//! - it runs only when someone runs it. No `make` target reaches this directory,
//!   by the standing spikes discipline.
//!
//! The type-drift hole is the serious one, and `guard.sh` closes as much of it as
//! text comparison can by guarding the copied type definitions as well as the
//! copied functions. What remains is that nothing forces the guard to be run.
//!
//! # What the harnesses establish
//!
//! For each encoder, that no two distinct inputs produce equal bytes — over the
//! **whole** input type where the type is finite-width, which is the thing an
//! enumerated test could not reach for a `u32` field. Kani discharges these
//! symbolically through CBMC, so `push_resources_injective` really does quantify
//! over all ~2^299 ordered input pairs rather than sampling them.
//!
//! Where a domain is not finite-width — `push_numerical` carries a `String` — the
//! proof is bounded, the bound is in the harness name, and what lies outside it
//! is stated at the harness.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Copied type definitions. Guarded by `guard.sh` against their sources.
// ---------------------------------------------------------------------------

/// @source: crates/tiler-ir/src/schedule/numerics.rs :: FlushedZeroSign
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FlushedZeroSign {
    PreservesSign,
    AlwaysPositive,
}

/// @source: crates/tiler-ir/src/schedule/numerics.rs :: SubnormalMode
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SubnormalMode {
    Preserve,
    FlushToZero { zero_sign: FlushedZeroSign },
}

/// @source: crates/tiler-ir/src/schedule/numerics.rs :: NumericalPermission
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumericalPermission {
    Forbidden,
    Permitted,
}

/// @source: crates/tiler-ir/src/schedule/numerics.rs :: ValueDomainProvenance
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValueDomainProvenance {
    CompilerProven,
    RuntimeValidated,
    CallerDeclaredUnvalidated,
}

/// @source: crates/tiler-ir/src/schedule/numerics.rs :: ExceptionalValueAssumption
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExceptionalValueAssumption {
    MakeNoAssumption,
    AssumeAbsent { provenance: ValueDomainProvenance },
}

/// @source: crates/tiler-ir/src/schedule/synchronization.rs :: SynchronizationKind
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SynchronizationKind {
    ControlBarrier,
    AsynchronousCopy,
    SplitPhaseBarrier,
    Collective,
    Atomic,
    InterDispatchDependency,
}

/// @source: crates/tiler-ir/src/schedule/synchronization.rs :: SynchronizationScope
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SynchronizationScope {
    Subgroup,
    Workgroup,
    Device,
}

/// @source: crates/tiler-ir/src/schedule/synchronization.rs :: MemoryOrdering
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryOrdering {
    Relaxed,
    AcquireRelease,
    SequentiallyConsistent,
}

/// @source: crates/tiler-ir/src/schedule/synchronization.rs :: FencedSpaces
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FencedSpaces {
    pub workgroup: bool,
    pub device: bool,
}

/// @source: crates/tiler-ir/src/schedule/synchronization.rs :: SynchronizationSubject
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SynchronizationSubject {
    pub kind: SynchronizationKind,
    pub execution_scope: SynchronizationScope,
    pub visibility_scope: SynchronizationScope,
    pub fenced_spaces: FencedSpaces,
    pub ordering: MemoryOrdering,
}

/// @source: crates/tiler-ir/src/semantic/types.rs :: EncodedComponentRole
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EncodedComponentRole(u32);

impl EncodedComponentRole {
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// @source: crates/tiler-ir/src/schedule/model.rs :: TensorRole
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TensorRole {
    Input,
    Intermediate,
    Output,
}

/// @source: crates/tiler-ir/src/schedule/model.rs :: IndexArithmetic
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexArithmetic {
    CompleteU64,
}

// Support vocabulary copied only so the guarded `ResourceRequirements` shape
// remains constructible. `push_resources` deliberately ignores this reserved
// field, so these are not encoder proof subjects and carry no `@source` marker.
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArithmeticType {
    F16,
    Bf16,
    F32,
    F64,
}

#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubgroupWidth(u32);

#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SubgroupTransfer {
    InRangeXorShuffle,
}

#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubgroupRealizationSubject {
    width: SubgroupWidth,
    arithmetic: ArithmeticType,
    transfer: SubgroupTransfer,
}

/// @source: crates/tiler-ir/src/schedule/model.rs :: ResourceRequirements
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceRequirements {
    pub buffer_bindings: u32,
    pub threads_per_workgroup: u32,
    pub local_memory_bytes: u64,
    pub requires_device_memory: bool,
    pub index_arithmetic: IndexArithmetic,
    pub synchronization: Option<SynchronizationSubject>,
    pub subgroup: Option<SubgroupRealizationSubject>,
    pub input_subnormals: SubnormalMode,
    pub result_subnormals: SubnormalMode,
    pub contraction: NumericalPermission,
    pub reassociation: NumericalPermission,
    pub permutation: NumericalPermission,
    pub signed_zero: NumericalPermission,
    pub nan_assumptions: ExceptionalValueAssumption,
    pub infinity_assumptions: ExceptionalValueAssumption,
}

/// @source: crates/tiler-artifact/src/program/codec/model.rs :: NumericalFacts
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumericalFacts {
    pub profile_key: String,
    pub canonical_arithmetic_nan_bits: u32,
    pub input_subnormals: SubnormalMode,
    pub result_subnormals: SubnormalMode,
    pub contraction: NumericalPermission,
    pub reassociation: NumericalPermission,
    pub permutation: NumericalPermission,
    pub signed_zero: NumericalPermission,
    pub nan_assumptions: ExceptionalValueAssumption,
    pub infinity_assumptions: ExceptionalValueAssumption,
}

// ---------------------------------------------------------------------------
// Copied encoders. Every body below is token-equivalent to its source under
// `guard.sh`'s documented normalization; the guard is what says so.
// ---------------------------------------------------------------------------

/// @source: crates/tiler-ir/src/identity.rs :: push_len
pub fn push_len(bytes: &mut Vec<u8>, len: usize) {
    bytes.extend_from_slice(
        &u64::try_from(len)
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
}

/// @source: crates/tiler-ir/src/identity.rs :: push_slice
pub fn push_slice(bytes: &mut Vec<u8>, value: &[u8]) {
    // Reserved as one request because the two `extend_from_slice` calls below
    // would otherwise each test capacity and each be able to trigger a separate
    // reallocation-and-move of the whole buffer. A sampling profile of the
    // compile loop put this function at 8.93% of active self time, spread over
    // twenty-odd encoders with no dominant caller, so the growth is systemic to
    // the primitive rather than to any one encoder. The reserved amount is
    // exact, not an estimate.
    bytes.reserve(8 + value.len());
    push_len(bytes, value.len());
    bytes.extend_from_slice(value);
}

/// @source: crates/tiler-ir/src/schedule/model.rs :: push_tensor_role
pub fn push_tensor_role(bytes: &mut Vec<u8>, role: TensorRole) {
    match role {
        TensorRole::Input => bytes.push(0x01),
        TensorRole::Intermediate => bytes.push(0x02),
        TensorRole::Output => bytes.push(0x03),
    }
}

/// @source: crates/tiler-ir/src/schedule/model.rs :: push_component_role
pub fn push_component_role(bytes: &mut Vec<u8>, role: Option<EncodedComponentRole>) {
    match role {
        None => bytes.push(0x00),
        Some(role) => {
            bytes.push(0x01);
            bytes.extend_from_slice(&role.get().to_be_bytes());
        }
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: subnormal_tag
pub const fn subnormal_tag(mode: SubnormalMode) -> u8 {
    match mode {
        SubnormalMode::Preserve => 0x01,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        } => 0x02,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        } => 0x03,
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: permission_tag
pub const fn permission_tag(permission: NumericalPermission) -> u8 {
    match permission {
        NumericalPermission::Forbidden => 0x01,
        NumericalPermission::Permitted => 0x02,
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: exceptional_assumption_tag
pub const fn exceptional_assumption_tag(assumption: ExceptionalValueAssumption) -> u8 {
    match assumption {
        ExceptionalValueAssumption::MakeNoAssumption => 0x01,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CompilerProven,
        } => 0x02,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::RuntimeValidated,
        } => 0x03,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
        } => 0x04,
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: synchronization_kind_tag
pub const fn synchronization_kind_tag(kind: SynchronizationKind) -> u8 {
    match kind {
        SynchronizationKind::ControlBarrier => 0x01,
        SynchronizationKind::AsynchronousCopy => 0x02,
        SynchronizationKind::SplitPhaseBarrier => 0x03,
        SynchronizationKind::Collective => 0x04,
        SynchronizationKind::Atomic => 0x05,
        SynchronizationKind::InterDispatchDependency => 0x06,
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: synchronization_scope_tag
pub const fn synchronization_scope_tag(scope: SynchronizationScope) -> u8 {
    match scope {
        SynchronizationScope::Subgroup => 0x01,
        SynchronizationScope::Workgroup => 0x02,
        SynchronizationScope::Device => 0x03,
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: memory_ordering_tag
pub const fn memory_ordering_tag(ordering: MemoryOrdering) -> u8 {
    match ordering {
        MemoryOrdering::Relaxed => 0x01,
        MemoryOrdering::AcquireRelease => 0x02,
        MemoryOrdering::SequentiallyConsistent => 0x03,
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: index_arithmetic_tag
pub const fn index_arithmetic_tag(index_arithmetic: IndexArithmetic) -> u8 {
    match index_arithmetic {
        IndexArithmetic::CompleteU64 => 0x01,
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: push_synchronization
pub fn push_synchronization(bytes: &mut Vec<u8>, subject: Option<SynchronizationSubject>) {
    match subject {
        None => bytes.push(0x00),
        Some(subject) => {
            bytes.push(0x01);
            bytes.push(synchronization_kind_tag(subject.kind));
            bytes.push(synchronization_scope_tag(subject.execution_scope));
            bytes.push(synchronization_scope_tag(subject.visibility_scope));
            bytes.push(u8::from(subject.fenced_spaces.workgroup));
            bytes.push(u8::from(subject.fenced_spaces.device));
            bytes.push(memory_ordering_tag(subject.ordering));
        }
    }
}

/// @source: crates/tiler-artifact/src/program/model.rs :: push_resources
pub fn push_resources(bytes: &mut Vec<u8>, resources: ResourceRequirements) {
    let ResourceRequirements {
        buffer_bindings,
        threads_per_workgroup,
        local_memory_bytes,
        requires_device_memory,
        index_arithmetic,
        synchronization,
        subgroup: _,
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
        permutation,
        signed_zero,
        nan_assumptions,
        infinity_assumptions,
    } = resources;
    bytes.extend_from_slice(&buffer_bindings.to_be_bytes());
    bytes.extend_from_slice(&threads_per_workgroup.to_be_bytes());
    bytes.extend_from_slice(&local_memory_bytes.to_be_bytes());
    bytes.push(u8::from(requires_device_memory));
    bytes.push(index_arithmetic_tag(index_arithmetic));
    push_synchronization(bytes, synchronization);
    bytes.push(subnormal_tag(input_subnormals));
    bytes.push(subnormal_tag(result_subnormals));
    bytes.push(permission_tag(contraction));
    bytes.push(permission_tag(reassociation));
    bytes.push(permission_tag(permutation));
    bytes.push(permission_tag(signed_zero));
    bytes.push(exceptional_assumption_tag(nan_assumptions));
    bytes.push(exceptional_assumption_tag(infinity_assumptions));
}

/// @source: crates/tiler-artifact/src/program/model.rs :: push_numerical
pub fn push_numerical(bytes: &mut Vec<u8>, numerical: &NumericalFacts) {
    let NumericalFacts {
        profile_key,
        canonical_arithmetic_nan_bits,
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
        permutation,
        signed_zero,
        nan_assumptions,
        infinity_assumptions,
    } = numerical;
    push_slice(bytes, profile_key.as_bytes());
    bytes.extend_from_slice(&canonical_arithmetic_nan_bits.to_be_bytes());
    bytes.push(subnormal_tag(*input_subnormals));
    bytes.push(subnormal_tag(*result_subnormals));
    bytes.push(permission_tag(*contraction));
    bytes.push(permission_tag(*reassociation));
    bytes.push(permission_tag(*permutation));
    bytes.push(permission_tag(*signed_zero));
    bytes.push(exceptional_assumption_tag(*nan_assumptions));
    bytes.push(exceptional_assumption_tag(*infinity_assumptions));
}

// ---------------------------------------------------------------------------
// Harnesses. Not guarded — this is the spike's own code, not copied text.
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod proofs {
    use super::*;

    /// Builds a symbolic `NumericalFacts` whose key is at most `N` ASCII bytes.
    ///
    /// The length is itself symbolic, so a framing defect that only shows up
    /// *across* two different key lengths is in the proof's domain. ASCII is
    /// assumed rather than proved: `String` requires UTF-8 and the real
    /// `profile_key` is a crate-chosen `&'static str`, so constraining to the
    /// single-byte range loses no reachable key while sparing CBMC the
    /// multi-byte validation automaton.
    fn any_facts<const N: usize>() -> NumericalFacts {
        let key: [u8; N] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= N);
        for byte in &key {
            kani::assume(*byte < 0x80);
        }
        NumericalFacts {
            profile_key: String::from_utf8(key[..len].to_vec()).expect("ascii is utf-8"),
            canonical_arithmetic_nan_bits: kani::any(),
            input_subnormals: kani::any(),
            result_subnormals: kani::any(),
            contraction: kani::any(),
            reassociation: kani::any(),
            permutation: kani::any(),
            signed_zero: kani::any(),
            nan_assumptions: kani::any(),
            infinity_assumptions: kani::any(),
        }
    }

    /// Injective over the whole three-value `TensorRole` domain, all pairs.
    ///
    /// **The input domain is not what needs bounding.** `TensorRole` is
    /// finite and `push_tensor_role` has no loop. The unwind bound below is about
    /// the *output*: comparing two `Vec<u8>` lowers to `memcmp`.
    ///
    /// **2 is complete, not a compromise.** The encoder writes exactly one tag
    /// byte, so no execution can reach a second `memcmp` iteration. Kani's
    /// unwinding assertion turns that from a claim into a check.
    #[kani::proof]
    #[kani::unwind(2)]
    fn push_tensor_role_injective() {
        let a: TensorRole = kani::any();
        let b: TensorRole = kani::any();
        let mut encoded_a = Vec::new();
        let mut encoded_b = Vec::new();
        push_tensor_role(&mut encoded_a, a);
        push_tensor_role(&mut encoded_b, b);
        if encoded_a == encoded_b {
            assert!(a == b, "two distinct tensor roles share an encoding");
        }
    }

    /// Injective over the whole `Option<EncodedComponentRole>` domain: 2^32 + 1.
    ///
    /// Unwind 6 for the reason `push_tensor_role_injective` records, and complete
    /// for the same reason: the encoder writes at most five bytes.
    #[kani::proof]
    #[kani::unwind(6)]
    fn push_component_role_injective() {
        let a: Option<EncodedComponentRole> = kani::any();
        let b: Option<EncodedComponentRole> = kani::any();
        let mut encoded_a = Vec::new();
        let mut encoded_b = Vec::new();
        push_component_role(&mut encoded_a, a);
        push_component_role(&mut encoded_b, b);
        if encoded_a == encoded_b {
            assert!(a == b, "two distinct component roles share an encoding");
        }
    }

    /// Injective over the whole `ResourceRequirements` domain, unbounded head included.
    ///
    /// The domain is 2^32 x 2^32 x 2^64 x 2 x 1 x 649 x 3^2 x 2^4 x 4^2,
    /// about 2^149.5 values and so about 2^299 ordered pairs. The type-derived
    /// `kani::Arbitrary` value covers all of `IndexArithmetic`; its current
    /// single variant scales the cardinality by one rather than narrowing the
    /// proof. The exhaustive-finite work could only reach the 1 495 296-value
    /// tail with the head held fixed; this covers every head.
    ///
    /// Unwind 34 bounds the output comparison, not the input: the record is at
    /// most 33 bytes (4 + 4 + 8 + 1 + 1 + 7 + 8), so no path reaches a 34th `memcmp`
    /// iteration and the unwinding assertion proves it. Nothing lies outside.
    #[kani::proof]
    #[kani::unwind(34)]
    fn push_resources_injective() {
        let a: ResourceRequirements = kani::any();
        let b: ResourceRequirements = kani::any();
        let mut encoded_a = Vec::new();
        let mut encoded_b = Vec::new();
        push_resources(&mut encoded_a, a);
        push_resources(&mut encoded_b, b);
        if encoded_a == encoded_b {
            assert!(
                a == b,
                "two distinct resource requirements share an encoding"
            );
        }
    }

    /// `push_resources` composes: its bytes determine where the record resumes.
    ///
    /// Injectivity alone does not give this. The encoder is variable-width — a
    /// `None` synchronization writes one byte where a `Some` writes seven — so a
    /// caller writing more fields after it needs the stronger property that
    /// `enc(a) ++ tail_a == enc(b) ++ tail_b` forces both `a == b` *and*
    /// `tail_a == tail_b`. That is prefix-freeness, and it is a statement about
    /// the encoder together with arbitrary following bytes, which no enumeration
    /// of the input domain can reach at all.
    ///
    /// **Genuinely bounded, unlike the harnesses above.** The trailing runs are
    /// exactly 4 bytes each. A defect that only shows up under a longer tail, or
    /// under two tails of *different* lengths, is outside this proof. Unwind 38
    /// covers the 33-byte record plus the 4-byte tail with one to spare.
    #[kani::proof]
    #[kani::unwind(38)]
    fn push_resources_prefix_free_tail_4() {
        let a: ResourceRequirements = kani::any();
        let b: ResourceRequirements = kani::any();
        let tail_a: [u8; 4] = kani::any();
        let tail_b: [u8; 4] = kani::any();
        let mut encoded_a = Vec::new();
        let mut encoded_b = Vec::new();
        push_resources(&mut encoded_a, a);
        push_resources(&mut encoded_b, b);
        encoded_a.extend_from_slice(&tail_a);
        encoded_b.extend_from_slice(&tail_b);
        if encoded_a == encoded_b {
            assert!(a == b, "two resource requirements share a framed encoding");
            assert!(
                tail_a == tail_b,
                "the record cannot tell where the resources end"
            );
        }
    }

    /// Injective over `NumericalFacts` with a key of at most 0 bytes.
    ///
    /// Outside the proof: every key of length 1 or more.
    ///
    /// Unwind is `21 + N`: the record is `8 + N + 4 + 8` bytes, and every loop
    /// `any_facts` adds — the ASCII assumption, the `to_vec`, the UTF-8
    /// validation — runs at most `N` times.
    #[kani::proof]
    #[kani::unwind(21)]
    fn push_numerical_injective_key_len_0() {
        check_numerical::<0>();
    }

    /// Injective over `NumericalFacts` with a key of at most 1 byte.
    ///
    /// Outside the proof: every key of length 2 or more.
    #[kani::proof]
    #[kani::unwind(22)]
    fn push_numerical_injective_key_len_1() {
        check_numerical::<1>();
    }

    /// Injective over `NumericalFacts` with a key of at most 2 bytes.
    ///
    /// Outside the proof: every key of length 3 or more.
    #[kani::proof]
    #[kani::unwind(23)]
    fn push_numerical_injective_key_len_2() {
        check_numerical::<2>();
    }

    /// Injective over `NumericalFacts` with a key of at most 4 bytes.
    ///
    /// **Outside the proof: every key of length 5 or more, which is every key
    /// the workspace actually constructs.** The `profile_key` literals in the
    /// crates measure 30 to 43 bytes — `tiler.test.scalar-host-profile` at 30,
    /// `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` at 43. So this harness
    /// proves nothing about any key in use; it establishes how the cost scales
    /// with the bound, and the scaling is what says the real lengths are out of
    /// reach. See the research record for the measured curve.
    #[kani::proof]
    #[kani::unwind(25)]
    fn push_numerical_injective_key_len_4() {
        check_numerical::<4>();
    }

    /// Injective over every `NumericalFacts` field *except* the key, held concrete.
    ///
    /// **This is a cost-attribution diagnostic, and its own claim is narrow.**
    /// `push_numerical_injective_key_len_0` exceeded a 900 s cap at the smallest
    /// symbolic-key bound that exists — an empty key. That result on its own does
    /// not say whether the obstacle is `push_numerical` or the `String` in its
    /// input type. This harness answers that: the key is a concrete 30-byte
    /// literal of the shape the crates actually use, and everything else stays
    /// symbolic — the `u32` NaN bits and all 2 304 tail combinations, so 2^32 x
    /// 2 304 values and the square of that in ordered pairs.
    ///
    /// **What it proves:** injectivity across the whole tail for two facts that
    /// share this one key. **What is outside it:** every pair differing in the
    /// key, which is the part that needs the key symbolic.
    ///
    /// A fast verdict here means the encoder is reachable and only the symbolic
    /// `String` is not, which makes the property recoverable by decomposition —
    /// prove the key's framing separately over a symbolic byte run. A slow one
    /// would mean the encoder itself is out of reach.
    #[kani::proof]
    #[kani::unwind(51)]
    fn push_numerical_injective_fixed_key() {
        let key = "tiler.test.scalar-host-profile";
        let a = NumericalFacts {
            profile_key: key.to_owned(),
            canonical_arithmetic_nan_bits: kani::any(),
            input_subnormals: kani::any(),
            result_subnormals: kani::any(),
            contraction: kani::any(),
            reassociation: kani::any(),
            permutation: kani::any(),
            signed_zero: kani::any(),
            nan_assumptions: kani::any(),
            infinity_assumptions: kani::any(),
        };
        let b = NumericalFacts {
            profile_key: key.to_owned(),
            canonical_arithmetic_nan_bits: kani::any(),
            input_subnormals: kani::any(),
            result_subnormals: kani::any(),
            contraction: kani::any(),
            reassociation: kani::any(),
            permutation: kani::any(),
            signed_zero: kani::any(),
            nan_assumptions: kani::any(),
            infinity_assumptions: kani::any(),
        };
        let mut encoded_a = Vec::new();
        let mut encoded_b = Vec::new();
        push_numerical(&mut encoded_a, &a);
        push_numerical(&mut encoded_b, &b);
        if encoded_a == encoded_b {
            assert!(a == b, "two distinct numerical facts share an encoding");
        }
    }

    fn check_numerical<const N: usize>() {
        let a = any_facts::<N>();
        let b = any_facts::<N>();
        let mut encoded_a = Vec::new();
        let mut encoded_b = Vec::new();
        push_numerical(&mut encoded_a, &a);
        push_numerical(&mut encoded_b, &b);
        if encoded_a == encoded_b {
            assert!(a == b, "two distinct numerical facts share an encoding");
        }
    }
}
