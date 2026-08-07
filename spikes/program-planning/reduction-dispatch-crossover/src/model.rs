//! The launch topology of each reduction strategy, and the analytical cost
//! model fitted to it.
//!
//! Shared by both binaries on purpose. The sweep uses [`stages`] to **check** the
//! launch geometry the compiler published against the structure this model
//! assumes, and records the resulting `threads:work:depth` triples in the TSV;
//! the fit consumes those recorded triples. Had the two carried separate
//! spellings of a stage's shape, a change in the compiler's plan structure would
//! have moved one and not the other, and the fit would have gone on predicting a
//! topology nothing dispatches.

/// Which reduction strategy one retained alternative realizes.
///
/// Recognized by an observable each strategy alone has, never by a name: the
/// compiler publishes an alternative's kernels and its ABI and never its
/// reduction topology. The multi-pass split is the only alternative with three
/// stages; the single-workgroup tree is the only one declaring an entry wider
/// than one thread per workgroup; the serial fold declares neither. This is the
/// same rule `spikes/program-planning/reduction-crossover`,
/// `tiler_build::metal_plan`'s parallel-portfolio fixture, and
/// `prototypes/serial-sum-run` all use, deliberately rather than a fourth one.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Strategy {
    /// One dispatch chain with a single-threaded fold.
    SerialFold,
    /// A cooperative fold whose declared workgroup exceeds one thread.
    SingleWorkgroupTree,
    /// A three-stage program writing and then consuming partials.
    MultiPassSplit,
}

impl Strategy {
    /// The stable code naming this strategy in the recorded sweep.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::SerialFold => "serial-fold",
            Self::SingleWorkgroupTree => "single-workgroup-tree",
            Self::MultiPassSplit => "multi-pass-split",
        }
    }

    /// Parses one recorded key back into a strategy.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        match key {
            "serial-fold" => Some(Self::SerialFold),
            "single-workgroup-tree" => Some(Self::SingleWorkgroupTree),
            "multi-pass-split" => Some(Self::MultiPassSplit),
            _ => None,
        }
    }

    /// Classifies one alternative from its kernel count and widest workgroup.
    ///
    /// The split is tested first because a three-stage program is the one
    /// unambiguous structural signature; a cooperative fold and a serial fold
    /// both carry two stages and are told apart only by the declared width.
    #[must_use]
    pub const fn classify(kernels: usize, widest_workgroup: u64) -> Self {
        if kernels >= 3 {
            Self::MultiPassSplit
        } else if widest_workgroup > 1 {
            Self::SingleWorkgroupTree
        } else {
            Self::SerialFold
        }
    }
}

/// One dispatched stage, as the cost model sees it.
///
/// Three numbers and no more: how many invocations the stage launches, how many
/// fold steps it performs in total across all of them, and how many one
/// invocation performs in sequence. Everything the three strategies differ in is
/// a redistribution of one program's arithmetic between total work and critical
/// path, which is why those two are the model's whole input and the thread count
/// is carried only so a sweep can check the model against a published launch.
///
/// **`work` is not `threads * depth`, and conflating them is the mistake this
/// field exists to prevent.** In a single-workgroup tree the committing
/// participant folds the staged slots while every other participant idles, so
/// the stage's critical path is longer than its work per lane; charging every
/// invocation for the whole path overstates the tree by the participant count
/// and hides the crossover it is supposed to locate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stage {
    /// Invocations this stage launches along the grid axis.
    pub threads: u64,
    /// Fold steps this stage performs, summed over every invocation.
    pub work: u64,
    /// Fold steps on the longest sequential path through one invocation.
    pub depth: u64,
}

/// The balanced exact split the compiler's `governed_partition` chooses.
///
/// Reimplemented here rather than reached for, because `governed_partition` is
/// `pub(crate)` in `tiler-compiler` and this spike may not widen a compiler
/// boundary to measure it. The sweep does not trust this copy: it compares the
/// launch geometry derived from it against the geometry the compiler actually
/// published, and refuses the cell when they disagree — so a divergence between
/// the two is a hard failure rather than a silently wrong depth column.
#[must_use]
pub fn governed_partition(contributors: u64) -> Option<(u64, u64)> {
    if contributors < 4 {
        return None;
    }
    let mut candidate = contributors.isqrt();
    while candidate >= 2 {
        if contributors.is_multiple_of(candidate) {
            let partitions = contributors / candidate;
            if partitions >= 2 {
                return Some((partitions, candidate));
            }
        }
        candidate -= 1;
    }
    None
}

/// The stage shapes one strategy declares for one `rows x contributors` shape.
///
/// The structures are those `crates/tiler-compiler/src/physical.rs` documents:
///
/// - **serial fold** — the elementwise prologue over every element, then one
///   invocation per output position folding the whole contributor run;
/// - **single-workgroup tree** — the same prologue, then one workgroup per
///   output position holding `partitions` participants, each folding its
///   `contributors_per_partition` run before the committing participant folds
///   the `partitions` staged slots;
/// - **multi-pass split** — the same prologue, then one invocation per
///   partition folding its run into a materialized partial, then one invocation
///   per output position folding the `partitions` partials.
///
/// The prologue is identical in all three and is deliberately included rather
/// than subtracted: it is what every one of these plans actually dispatches,
/// and a model fitted to the difference alone would predict a quantity no
/// consumer pays.
#[must_use]
pub fn stages(strategy: Strategy, rows: u64, contributors: u64) -> Option<Vec<Stage>> {
    let elements = rows.checked_mul(contributors)?;
    let prologue = Stage {
        threads: elements,
        work: elements,
        depth: 1,
    };
    match strategy {
        Strategy::SerialFold => Some(vec![
            prologue,
            Stage {
                threads: rows,
                work: elements,
                depth: contributors,
            },
        ]),
        Strategy::SingleWorkgroupTree => {
            let (partitions, per_partition) = governed_partition(contributors)?;
            Some(vec![
                prologue,
                Stage {
                    threads: rows.checked_mul(partitions)?,
                    // Every participant folds its own run — `elements` steps
                    // over the whole launch — and then the committing
                    // participant of each output position folds the staged
                    // slots, which is `partitions` more per row.
                    work: elements.checked_add(rows.checked_mul(partitions)?)?,
                    // The critical path is the sum rather than the maximum: the
                    // second phase happens after the barrier, not beside it.
                    depth: per_partition.checked_add(partitions)?,
                },
            ])
        }
        Strategy::MultiPassSplit => {
            let (partitions, per_partition) = governed_partition(contributors)?;
            Some(vec![
                prologue,
                Stage {
                    threads: rows.checked_mul(partitions)?,
                    work: elements,
                    depth: per_partition,
                },
                Stage {
                    threads: rows,
                    work: rows.checked_mul(partitions)?,
                    depth: partitions,
                },
            ])
        }
    }
}

/// The three fitted parameters of the analytical cost model.
///
/// The smallest set that can express a crossover between these strategies at
/// all. Each is a quantity of the *machine*, not of a strategy, which is what
/// keeps the model from being a preference written as arithmetic: no parameter
/// names a strategy, and no strategy has a term of its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CostParameters {
    /// Fixed cost of encoding and retiring one dispatch, in seconds.
    pub encoder_seconds: f64,
    /// Fold steps the device retires at once when it is saturated.
    pub parallel_threads: f64,
    /// Seconds one fold step costs on the critical path.
    pub step_seconds: f64,
}

impl CostParameters {
    /// Predicted seconds for one submission of the given stages.
    ///
    /// `cost = sum over stages of ( encoder + max(work/P, depth) * step )`.
    ///
    /// The `max` is the whole of the model's physics, and it is the classical
    /// work-span bound rather than anything invented here: a stage cannot finish
    /// before its longest sequential path, and it cannot finish before its total
    /// work has passed through a machine that retires `P` steps at a time.
    ///
    /// That single term produces the crossover instead of asserting it. When a
    /// program's row count already saturates the device, `work / P` dominates
    /// every strategy and the cheapest is whichever does least total work — the
    /// serial fold, which stages nothing and folds each row once. When the row
    /// count does not, `depth` dominates, and the serial fold's path is the
    /// whole contributor run while a tree's is roughly its square root. **No
    /// parameter names a strategy and no strategy has a term of its own**, so a
    /// preference derived from this model is a consequence of three measured
    /// machine quantities rather than a thumb on the scale.
    #[must_use]
    pub fn predict(self, stages: &[Stage]) -> f64 {
        stages
            .iter()
            .map(|stage| {
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "work and depth counts here are at most 2^25; f64 represents every integer below 2^53 exactly, so no precision is lost"
                )]
                let work = stage.work as f64;
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "see above: depths here are at most 2^14"
                )]
                let depth = stage.depth as f64;
                self.encoder_seconds
                    + (work / self.parallel_threads).max(depth) * self.step_seconds
            })
            .sum()
    }
}
