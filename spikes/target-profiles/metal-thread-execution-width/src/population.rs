//! The pipeline matrix frozen before any `threadExecutionWidth` is read.

/// How many independent pipeline constructions are retained per identity.
pub const REPETITIONS: usize = 3;

/// The exact property this spike reads.
pub const METRIC: &str = "MTLComputePipelineState.threadExecutionWidth";

/// JSON schema id written into every retained record.
pub const RECORD_SCHEMA: &str = "tiler.spike.target-profiles.metal-thread-execution-width/v1";

/// One retained kernel source, compiled in isolation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KernelId {
    /// Independent store of a launch index.
    StoreU32,
    /// Elementwise f32 add.
    AddF32,
    /// Elementwise f16 add.
    AddF16,
    /// Elementwise bf16 add.
    AddBf16,
    /// Elementwise i32 add.
    AddI32,
    /// Elementwise f64 add. Optional: Apple GPUs typically reject `double`.
    AddF64,
    /// Authorized F32 in-range XOR-shuffle butterfly.
    XorShuffleF32,
    /// F16 XOR-shuffle control. Profile-silent arithmetic.
    XorShuffleF16,
    /// Authorized-family BF16 candidate. MSL Table 6.14 excludes `bfloat`.
    XorShuffleBf16,
    /// F64 XOR-shuffle control. Optional.
    XorShuffleF64,
    /// Negative: refused subgroup collective.
    SimdSumF32,
    /// Negative: refused descending-stride tree.
    ShuffleDownF32,
    /// Control: quad-group shuffle. MSL fixes quad width at 4.
    QuadShuffleF32,
    /// Divergent control flow on the launch index.
    DivergentCfF32,
    /// Loop-carried f32 arithmetic.
    LoopF32,
    /// 16 KiB threadgroup tile plus a workgroup barrier.
    ThreadgroupMemF32,
    /// Many simultaneously live f32 values.
    HighRegF32,
    /// Source-side `[[threads_per_threadgroup(8, 8, 1)]]`.
    ConstrainedTg8x8,
    /// Source-side `[[max_total_threads_per_threadgroup(32)]]`.
    SourceMaxTg32,
}

impl KernelId {
    /// Every kernel this matrix names. Length is `variant_count`.
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::StoreU32,
        Self::AddF32,
        Self::AddF16,
        Self::AddBf16,
        Self::AddI32,
        Self::AddF64,
        Self::XorShuffleF32,
        Self::XorShuffleF16,
        Self::XorShuffleBf16,
        Self::XorShuffleF64,
        Self::SimdSumF32,
        Self::ShuffleDownF32,
        Self::QuadShuffleF32,
        Self::DivergentCfF32,
        Self::LoopF32,
        Self::ThreadgroupMemF32,
        Self::HighRegF32,
        Self::ConstrainedTg8x8,
        Self::SourceMaxTg32,
    ];

    /// File stem, entry point, and identity component.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::StoreU32 => "store_u32",
            Self::AddF32 => "add_f32",
            Self::AddF16 => "add_f16",
            Self::AddBf16 => "add_bf16",
            Self::AddI32 => "add_i32",
            Self::AddF64 => "add_f64",
            Self::XorShuffleF32 => "xor_shuffle_f32",
            Self::XorShuffleF16 => "xor_shuffle_f16",
            Self::XorShuffleBf16 => "xor_shuffle_bf16",
            Self::XorShuffleF64 => "xor_shuffle_f64",
            Self::SimdSumF32 => "simd_sum_f32",
            Self::ShuffleDownF32 => "shuffle_down_f32",
            Self::QuadShuffleF32 => "quad_shuffle_f32",
            Self::DivergentCfF32 => "divergent_cf_f32",
            Self::LoopF32 => "loop_f32",
            Self::ThreadgroupMemF32 => "threadgroup_mem_f32",
            Self::HighRegF32 => "high_reg_f32",
            Self::ConstrainedTg8x8 => "constrained_tg_8x8",
            Self::SourceMaxTg32 => "source_max_tg_32",
        }
    }

    /// Arithmetic label retained with the pipeline identity.
    #[must_use]
    pub const fn arithmetic(self) -> &'static str {
        match self {
            Self::StoreU32 | Self::ConstrainedTg8x8 | Self::SourceMaxTg32 => "u32",
            Self::AddI32 => "i32",
            Self::AddF16 | Self::XorShuffleF16 => "f16",
            Self::AddBf16 | Self::XorShuffleBf16 => "bf16",
            Self::AddF64 | Self::XorShuffleF64 => "f64",
            Self::AddF32
            | Self::XorShuffleF32
            | Self::SimdSumF32
            | Self::ShuffleDownF32
            | Self::QuadShuffleF32
            | Self::DivergentCfF32
            | Self::LoopF32
            | Self::ThreadgroupMemF32
            | Self::HighRegF32 => "f32",
        }
    }

    /// Operation-family label retained with the pipeline identity.
    #[must_use]
    pub const fn operation_family(self) -> &'static str {
        match self {
            Self::StoreU32 | Self::ConstrainedTg8x8 | Self::SourceMaxTg32 => "independent-store",
            Self::AddF32 | Self::AddF16 | Self::AddBf16 | Self::AddI32 | Self::AddF64 => {
                "elementwise-add"
            }
            Self::XorShuffleF32
            | Self::XorShuffleF16
            | Self::XorShuffleBf16
            | Self::XorShuffleF64 => "in-range-xor-shuffle",
            Self::SimdSumF32 => "subgroup-collective",
            Self::ShuffleDownF32 => "shuffle-down-tree",
            Self::QuadShuffleF32 => "quad-shuffle",
            Self::DivergentCfF32 => "divergent-control-flow",
            Self::LoopF32 => "loop",
            Self::ThreadgroupMemF32 => "threadgroup-memory",
            Self::HighRegF32 => "high-register-pressure",
        }
    }
}

/// One offline compiler flag vector.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CompilerSelection {
    /// The first macOS Metal compile profile's numerical realization.
    ProfileStrict,
    /// `-fmetal-math-mode=fast` in place of `safe`.
    MathFast,
    /// `-fmetal-math-mode=relaxed` in place of `safe`.
    MathRelaxed,
    /// `-ffp-contract=fast` in place of `off`.
    ContractFast,
    /// Profile-strict plus `-O0`.
    OptO0,
    /// Profile-strict plus `-Os`.
    OptOs,
    /// Profile-strict with `-std=metal3.1`.
    StdMetal3_1,
}

impl CompilerSelection {
    /// Every compilation-selection identity this matrix names.
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::ProfileStrict,
        Self::MathFast,
        Self::MathRelaxed,
        Self::ContractFast,
        Self::OptO0,
        Self::OptOs,
        Self::StdMetal3_1,
    ];

    /// Identity component written into the pipeline id.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ProfileStrict => "profile_strict",
            Self::MathFast => "math_fast",
            Self::MathRelaxed => "math_relaxed",
            Self::ContractFast => "contract_fast",
            Self::OptO0 => "opt_O0",
            Self::OptOs => "opt_Os",
            Self::StdMetal3_1 => "std_metal3.1",
        }
    }

    /// `xcrun metal` flags after the sdk selector and before `-c`.
    #[must_use]
    pub const fn metal_flags(self) -> &'static [&'static str] {
        match self {
            Self::ProfileStrict => &[
                "-std=metal4.0",
                "-target",
                "air64-apple-macos26.0",
                "-fmetal-math-mode=safe",
                "-fmetal-math-fp32-functions=precise",
                "-ffp-contract=off",
            ],
            Self::MathFast => &[
                "-std=metal4.0",
                "-target",
                "air64-apple-macos26.0",
                "-fmetal-math-mode=fast",
                "-fmetal-math-fp32-functions=precise",
                "-ffp-contract=off",
            ],
            Self::MathRelaxed => &[
                "-std=metal4.0",
                "-target",
                "air64-apple-macos26.0",
                "-fmetal-math-mode=relaxed",
                "-fmetal-math-fp32-functions=precise",
                "-ffp-contract=off",
            ],
            Self::ContractFast => &[
                "-std=metal4.0",
                "-target",
                "air64-apple-macos26.0",
                "-fmetal-math-mode=safe",
                "-fmetal-math-fp32-functions=precise",
                "-ffp-contract=fast",
            ],
            Self::OptO0 => &[
                "-std=metal4.0",
                "-target",
                "air64-apple-macos26.0",
                "-fmetal-math-mode=safe",
                "-fmetal-math-fp32-functions=precise",
                "-ffp-contract=off",
                "-O0",
            ],
            Self::OptOs => &[
                "-std=metal4.0",
                "-target",
                "air64-apple-macos26.0",
                "-fmetal-math-mode=safe",
                "-fmetal-math-fp32-functions=precise",
                "-ffp-contract=off",
                "-Os",
            ],
            Self::StdMetal3_1 => &[
                "-std=metal3.1",
                "-target",
                "air64-apple-macos26.0",
                "-fmetal-math-mode=safe",
                "-fmetal-math-fp32-functions=precise",
                "-ffp-contract=off",
            ],
        }
    }
}

/// Pipeline-descriptor shape applied at prepare time.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DescriptorShape {
    /// Unset max; `threadGroupSizeIsMultipleOfThreadExecutionWidth` left false.
    Default,
    /// `maxTotalThreadsPerThreadgroup = 1`.
    Max1,
    /// `maxTotalThreadsPerThreadgroup = 32`.
    Max32,
    /// `maxTotalThreadsPerThreadgroup = 256`.
    Max256,
    /// `maxTotalThreadsPerThreadgroup = 1024`.
    Max1024,
    /// `threadGroupSizeIsMultipleOfThreadExecutionWidth = true`.
    MultipleOfWidth,
    /// Max 1024 and multiple-of-width together.
    Max1024Multiple,
}

impl DescriptorShape {
    /// Every descriptor identity this matrix names.
    pub const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::Default,
        Self::Max1,
        Self::Max32,
        Self::Max256,
        Self::Max1024,
        Self::MultipleOfWidth,
        Self::Max1024Multiple,
    ];

    /// Identity component written into the pipeline id.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Max1 => "max_1",
            Self::Max32 => "max_32",
            Self::Max256 => "max_256",
            Self::Max1024 => "max_1024",
            Self::MultipleOfWidth => "multiple_of_width",
            Self::Max1024Multiple => "max_1024_multiple",
        }
    }

    /// Optional `maxTotalThreadsPerThreadgroup` applied to the descriptor.
    #[must_use]
    pub const fn max_total_threads(self) -> Option<u64> {
        match self {
            Self::Max1 => Some(1),
            Self::Max32 => Some(32),
            Self::Max256 => Some(256),
            Self::Max1024 | Self::Max1024Multiple => Some(1024),
            Self::Default | Self::MultipleOfWidth => None,
        }
    }

    /// Whether the descriptor asserts a multiple-of-width threadgroup.
    #[must_use]
    pub const fn multiple_of_width(self) -> bool {
        matches!(self, Self::MultipleOfWidth | Self::Max1024Multiple)
    }
}

/// Why this identity is in the matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PipelineRole {
    /// F32 `InRangeXorShuffle`, which the Apple9 profile could later authorize.
    AuthorizedF32XorShuffle,
    /// BF16 `InRangeXorShuffle` candidate. Compile may fail; that is a row.
    AuthorizedFamilyBf16Candidate,
    /// Isolating control, not an authorized family.
    Control,
    /// Explicitly refused construct (`simd_sum` or `shuffle_down`).
    Negative,
}

impl PipelineRole {
    /// Stable label written into the record.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AuthorizedF32XorShuffle => "authorized-f32-xor-shuffle",
            Self::AuthorizedFamilyBf16Candidate => "authorized-family-bf16-candidate",
            Self::Control => "control",
            Self::Negative => "negative",
        }
    }
}

/// One frozen pipeline identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PipelineSpec {
    /// Kernel compiled for this identity.
    pub kernel: KernelId,
    /// Offline flag vector.
    pub compiler: CompilerSelection,
    /// Prepare-time descriptor shape.
    pub descriptor: DescriptorShape,
    /// Why the identity is present.
    pub role: PipelineRole,
    /// When true, compile or prepare failure aborts the run.
    pub required: bool,
}

impl PipelineSpec {
    /// `{kernel}/{compiler}/{descriptor}`.
    #[must_use]
    pub fn id(self) -> String {
        format!(
            "{}/{}/{}",
            self.kernel.name(),
            self.compiler.name(),
            self.descriptor.name()
        )
    }
}

const fn spec(
    kernel: KernelId,
    compiler: CompilerSelection,
    descriptor: DescriptorShape,
    role: PipelineRole,
    required: bool,
) -> PipelineSpec {
    PipelineSpec {
        kernel,
        compiler,
        descriptor,
        role,
        required,
    }
}

/// The frozen population. Do not edit after the first width observation.
pub const PIPELINES: &[PipelineSpec] = &[
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::AuthorizedF32XorShuffle,
        true,
    ),
    spec(
        KernelId::XorShuffleBf16,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::AuthorizedFamilyBf16Candidate,
        false,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Max1,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Max32,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Max256,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Max1024,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::MultipleOfWidth,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Max1024Multiple,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::MathFast,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::MathRelaxed,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::ContractFast,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::OptO0,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::OptOs,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF32,
        CompilerSelection::StdMetal3_1,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::StoreU32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::AddF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::AddF16,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::AddBf16,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::AddI32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::AddF64,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        false,
    ),
    spec(
        KernelId::XorShuffleF16,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::XorShuffleF64,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        false,
    ),
    spec(
        KernelId::SimdSumF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Negative,
        true,
    ),
    spec(
        KernelId::ShuffleDownF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Negative,
        true,
    ),
    spec(
        KernelId::QuadShuffleF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::DivergentCfF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::LoopF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::ThreadgroupMemF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::ThreadgroupMemF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Max1024,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::HighRegF32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::ConstrainedTg8x8,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::SourceMaxTg32,
        CompilerSelection::ProfileStrict,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::AddF32,
        CompilerSelection::MathFast,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
    spec(
        KernelId::AddF32,
        CompilerSelection::OptO0,
        DescriptorShape::Default,
        PipelineRole::Control,
        true,
    ),
];

/// Declared population size. Equal to `PIPELINES.len()` by construction.
pub const PIPELINE_COUNT: usize = PIPELINES.len();

/// Looks up the frozen spec for a retained identity string.
#[must_use]
pub fn spec_by_id(id: &str) -> Option<&'static PipelineSpec> {
    PIPELINES.iter().find(|spec| spec.id() == id)
}

/// Lengths of the typed enumerations the freeze is required to cover.
#[must_use]
pub const fn enumeration_counts() -> (usize, usize, usize) {
    (
        KernelId::ALL.len(),
        CompilerSelection::ALL.len(),
        DescriptorShape::ALL.len(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;

    use super::*;

    #[test]
    fn the_declared_count_is_the_array_length() {
        assert_eq!(PIPELINE_COUNT, 34);
        assert_eq!(PIPELINES.len(), PIPELINE_COUNT);
    }

    #[test]
    fn every_identity_is_unique() {
        let mut seen = BTreeSet::new();
        for spec in PIPELINES {
            assert!(seen.insert(spec.id()), "duplicate identity {}", spec.id());
        }
    }

    #[test]
    fn every_kernel_file_is_named_and_every_named_kernel_has_a_file() {
        let root = crate::record::spike_root();
        let mut files = BTreeSet::new();
        for entry in fs::read_dir(root.join("kernels")).expect("kernels/ is readable") {
            let path = entry.expect("a directory entry").path();
            if path.extension().is_some_and(|ext| ext == "metal") {
                files.insert(
                    path.file_stem()
                        .expect("a metal file has a stem")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
        let named: BTreeSet<_> = KernelId::ALL
            .iter()
            .map(|kernel| kernel.name().to_owned())
            .collect();
        assert_eq!(files, named);
        for spec in PIPELINES {
            assert!(named.contains(spec.kernel.name()));
        }
        for kernel in KernelId::ALL {
            assert!(
                PIPELINES.iter().any(|spec| spec.kernel == kernel),
                "{} is never used",
                kernel.name()
            );
        }
    }

    #[test]
    fn every_compiler_selection_and_descriptor_shape_is_used() {
        for selection in CompilerSelection::ALL {
            assert!(
                PIPELINES.iter().any(|spec| spec.compiler == selection),
                "{} is never used",
                selection.name()
            );
        }
        for shape in DescriptorShape::ALL {
            assert!(
                PIPELINES.iter().any(|spec| spec.descriptor == shape),
                "{} is never used",
                shape.name()
            );
        }
    }

    #[test]
    fn authorized_families_and_negatives_are_present() {
        assert!(
            PIPELINES.iter().any(|spec| {
                spec.role == PipelineRole::AuthorizedF32XorShuffle && spec.required
            })
        );
        assert!(PIPELINES.iter().any(|spec| {
            spec.role == PipelineRole::AuthorizedFamilyBf16Candidate && !spec.required
        }));
        assert!(
            PIPELINES
                .iter()
                .any(|spec| spec.kernel == KernelId::SimdSumF32)
        );
        assert!(
            PIPELINES
                .iter()
                .any(|spec| spec.kernel == KernelId::ShuffleDownF32)
        );
    }
}
