//! Validated ordered-binary-tree topology for one contraction output.
//!
//! The semantic result set of `tiler::tensor-contraction-f32@1` under
//! permitted effective reassociation is the set of all full ordered binary
//! trees over the unchanged canonical contributor sequence. Membership alone
//! is not checkable — a body claiming tree A must be checked against A even
//! when its wrong result happens to equal a value tree B can produce — so a
//! plan-owned witness names **one** legal tree, and this module is its
//! validated representation.
//!
//! [`ContributorPartition`](super::ContributorPartition) remains the accepted
//! compact carrier for a regular physical split; it is deliberately not the
//! general result-set witness, because a regular contiguous partition cannot
//! spell an arbitrary full ordered binary tree.

use std::error::Error;
use std::fmt;

/// One node of a postorder-encoded full ordered binary tree.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContractionF32TreeNode {
    /// One leaf product, named by its contributor ordinal in the canonical
    /// contributor sequence.
    Leaf {
        /// Zero-based contributor ordinal.
        contributor: u64,
    },
    /// One reducer combine of two earlier nodes.
    Add {
        /// Node index of the left subtree's root.
        left: u32,
        /// Node index of the right subtree's root.
        right: u32,
    },
}

/// Caller-owned structural bounds on one topology.
///
/// No `Default`: the caller states its bounds, and the crate never chooses
/// them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContractionF32TopologyLimits {
    max_nodes: usize,
    max_depth: usize,
}

/// Why one limits value is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidContractionF32TopologyLimits {
    /// The node allowance is zero.
    ZeroNodes,
    /// The depth allowance is zero.
    ZeroDepth,
    /// The node allowance exceeds the `u32` node-index capacity.
    NodeIndexCapacity {
        /// The maximum admissible node allowance.
        maximum: usize,
    },
}

impl fmt::Display for InvalidContractionF32TopologyLimits {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroNodes => formatter.write_str("topology node allowance is zero"),
            Self::ZeroDepth => formatter.write_str("topology depth allowance is zero"),
            Self::NodeIndexCapacity { maximum } => write!(
                formatter,
                "topology node allowance exceeds the node-index capacity {maximum}"
            ),
        }
    }
}

impl Error for InvalidContractionF32TopologyLimits {}

impl ContractionF32TopologyLimits {
    /// Builds one bounds value.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidContractionF32TopologyLimits`] for a zero allowance or
    /// a node allowance above `u32::MAX`, because node indices are `u32`.
    pub const fn new(
        max_nodes: usize,
        max_depth: usize,
    ) -> Result<Self, InvalidContractionF32TopologyLimits> {
        if max_nodes == 0 {
            return Err(InvalidContractionF32TopologyLimits::ZeroNodes);
        }
        if max_depth == 0 {
            return Err(InvalidContractionF32TopologyLimits::ZeroDepth);
        }
        if max_nodes > u32::MAX as usize {
            return Err(InvalidContractionF32TopologyLimits::NodeIndexCapacity {
                maximum: u32::MAX as usize,
            });
        }
        Ok(Self {
            max_nodes,
            max_depth,
        })
    }

    /// Returns the node allowance.
    #[must_use]
    pub const fn max_nodes(&self) -> usize {
        self.max_nodes
    }

    /// Returns the depth allowance.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }
}

/// A typed refusal of one offered topology.
///
/// Exhaustive. The interval rules — children strictly earlier, exact
/// left-then-right adjacency, root coverage — jointly reject cycles, DAG
/// sharing, disconnected nodes, gaps, overlap, reversal, and permutation, so
/// each malformation is refused under a name rather than approximated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractionF32TreeError {
    /// The contributor count is zero; the unseeded fold has no empty result.
    EmptyContributors,
    /// The required `2K - 1` node count overflows the host `usize`.
    NodeCountOverflow,
    /// The required node count exceeds the caller's node allowance.
    NodeLimit {
        /// The caller's allowance.
        limit: usize,
        /// The required node count.
        actual: usize,
    },
    /// The validated tree is deeper than the caller's depth allowance.
    DepthLimit {
        /// The caller's allowance.
        limit: usize,
        /// The validated depth.
        actual: usize,
    },
    /// The offered node count is not exactly `2K - 1`.
    NodeCount {
        /// The required count.
        expected: usize,
        /// The offered count.
        actual: usize,
    },
    /// The last node is referenced by another node, so it is not the root.
    RootNotLast {
        /// Index of the last node.
        root: u32,
    },
    /// An `Add` references a child at or after its own index.
    ChildNotEarlier {
        /// The offending `Add`'s index.
        node: u32,
        /// The referenced child index.
        child: u32,
    },
    /// A non-root node is not referenced exactly once.
    ReferenceCount {
        /// The mis-referenced node's index.
        node: u32,
        /// The required reference count.
        expected: u32,
        /// The observed reference count.
        actual: u32,
    },
    /// A leaf names a contributor ordinal outside `0..K`.
    ContributorOutOfRange {
        /// The out-of-range ordinal.
        contributor: u64,
        /// The contributor count `K`.
        count: u64,
    },
    /// A contributor ordinal appears in more than one leaf.
    ContributorMultiplicity {
        /// The duplicated ordinal.
        contributor: u64,
        /// The observed leaf count for it.
        actual: u32,
    },
    /// An `Add`'s left interval does not end exactly where its right begins.
    NonAdjacentChildren {
        /// The offending `Add`'s index.
        node: u32,
    },
    /// The root does not cover the contributor interval `0..K` exactly.
    RootCoverage {
        /// The contributor count `K`.
        expected: u64,
        /// The root interval's start.
        actual_start: u64,
        /// The root interval's exclusive end.
        actual_end: u64,
    },
}

impl fmt::Display for ContractionF32TreeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyContributors => formatter
                .write_str("contributor count is zero, and the unseeded fold has no empty result"),
            Self::NodeCountOverflow => {
                formatter.write_str("required node count 2K-1 overflows the host word")
            }
            Self::NodeLimit { limit, actual } => write!(
                formatter,
                "required node count {actual} exceeds the caller allowance {limit}"
            ),
            Self::DepthLimit { limit, actual } => write!(
                formatter,
                "tree depth {actual} exceeds the caller allowance {limit}"
            ),
            Self::NodeCount { expected, actual } => write!(
                formatter,
                "offered {actual} nodes where a full ordered binary tree needs exactly {expected}"
            ),
            Self::RootNotLast { root } => write!(
                formatter,
                "last node {root} is referenced by another node, so it is not the root"
            ),
            Self::ChildNotEarlier { node, child } => write!(
                formatter,
                "node {node} references child {child}, which is not strictly earlier"
            ),
            Self::ReferenceCount {
                node,
                expected,
                actual,
            } => write!(
                formatter,
                "node {node} is referenced {actual} times where exactly {expected} is required"
            ),
            Self::ContributorOutOfRange { contributor, count } => write!(
                formatter,
                "leaf contributor {contributor} is outside 0..{count}"
            ),
            Self::ContributorMultiplicity {
                contributor,
                actual,
            } => write!(
                formatter,
                "contributor {contributor} appears in {actual} leaves where exactly one is required"
            ),
            Self::NonAdjacentChildren { node } => write!(
                formatter,
                "node {node}'s left interval does not end exactly where its right begins"
            ),
            Self::RootCoverage {
                expected,
                actual_start,
                actual_end,
            } => write!(
                formatter,
                "root covers {actual_start}..{actual_end} where 0..{expected} is required"
            ),
        }
    }
}

impl Error for ContractionF32TreeError {}

/// One validated full ordered binary tree over `K` contributors.
///
/// Opaque: holding one is evidence that the postorder node vector passed every
/// structural rule under the caller's limits. The in-order leaf traversal of a
/// validated tree is exactly the canonical contributor sequence `0..K` — the
/// interval rules make grouping the only degree of freedom, which is precisely
/// the reassociation-only result class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedContractionF32Tree {
    contributor_count: u64,
    nodes: Vec<ContractionF32TreeNode>,
    depth: usize,
}

impl OrderedContractionF32Tree {
    /// Validates one postorder node vector as a full ordered binary tree.
    ///
    /// Validation requires `K > 0`; a checked `2K - 1` node count; the root
    /// last; children strictly earlier than their `Add`; every non-root node
    /// referenced exactly once and the root never; every leaf ordinal in
    /// `0..K` exactly once and no other ordinal; and exact left-then-right
    /// interval adjacency with the root covering `0..K`. Limits are checked
    /// before the vector is retained, and every count uses checked arithmetic.
    ///
    /// # Errors
    ///
    /// Returns [`ContractionF32TreeError`] naming the first violated rule.
    ///
    /// # Panics
    ///
    /// Never panics: every internal index conversion runs after the node
    /// count is bounded by the `u32`-capped caller allowance.
    pub fn try_from_postorder(
        contributor_count: u64,
        nodes: Vec<ContractionF32TreeNode>,
        limits: ContractionF32TopologyLimits,
    ) -> Result<Self, ContractionF32TreeError> {
        if contributor_count == 0 {
            return Err(ContractionF32TreeError::EmptyContributors);
        }
        let expected = usize::try_from(contributor_count)
            .ok()
            .and_then(|count| count.checked_mul(2))
            .and_then(|count| count.checked_sub(1))
            .ok_or(ContractionF32TreeError::NodeCountOverflow)?;
        if expected > limits.max_nodes() {
            return Err(ContractionF32TreeError::NodeLimit {
                limit: limits.max_nodes(),
                actual: expected,
            });
        }
        if nodes.len() != expected {
            return Err(ContractionF32TreeError::NodeCount {
                expected,
                actual: nodes.len(),
            });
        }

        // Reference counting. `expected <= max_nodes <= u32::MAX`, so every
        // index below fits a `u32`.
        let mut references = vec![0_u32; nodes.len()];
        for (index, node) in nodes.iter().enumerate() {
            let index = u32::try_from(index).expect("node count is bounded by u32 above");
            if let ContractionF32TreeNode::Add { left, right } = node {
                for child in [*left, *right] {
                    if child >= index {
                        return Err(ContractionF32TreeError::ChildNotEarlier {
                            node: index,
                            child,
                        });
                    }
                    references[child as usize] += 1;
                }
            }
        }
        let root = u32::try_from(nodes.len() - 1).expect("node count is bounded by u32 above");
        if references[root as usize] != 0 {
            return Err(ContractionF32TreeError::RootNotLast { root });
        }
        for (index, count) in references.iter().enumerate().take(nodes.len() - 1) {
            if *count != 1 {
                return Err(ContractionF32TreeError::ReferenceCount {
                    node: u32::try_from(index).expect("node count is bounded by u32 above"),
                    expected: 1,
                    actual: *count,
                });
            }
        }

        // Contributor ordinals: each in range and each exactly once. Duplicate
        // detection over a bitmap of `K` entries; `K <= expected <= u32::MAX`.
        let mut seen = vec![
            false;
            usize::try_from(contributor_count)
                .expect("K is bounded by the node count above")
        ];
        // Interval derivation, in the same postorder pass: leaf `c` is
        // `[c, c+1)`, and an `Add` requires the left interval to end exactly
        // where the right begins. Depth is derived alongside.
        let mut intervals: Vec<(u64, u64)> = Vec::with_capacity(nodes.len());
        let mut depths: Vec<usize> = Vec::with_capacity(nodes.len());
        for (index, node) in nodes.iter().enumerate() {
            match node {
                ContractionF32TreeNode::Leaf { contributor } => {
                    if *contributor >= contributor_count {
                        return Err(ContractionF32TreeError::ContributorOutOfRange {
                            contributor: *contributor,
                            count: contributor_count,
                        });
                    }
                    let slot = usize::try_from(*contributor)
                        .expect("the range check above bounds the ordinal");
                    if seen[slot] {
                        return Err(ContractionF32TreeError::ContributorMultiplicity {
                            contributor: *contributor,
                            actual: 2,
                        });
                    }
                    seen[slot] = true;
                    intervals.push((*contributor, contributor + 1));
                    depths.push(1);
                }
                ContractionF32TreeNode::Add { left, right } => {
                    let node_index =
                        u32::try_from(index).expect("node count is bounded by u32 above");
                    let (left_start, left_end) = intervals[*left as usize];
                    let (right_start, right_end) = intervals[*right as usize];
                    if left_end != right_start {
                        return Err(ContractionF32TreeError::NonAdjacentChildren {
                            node: node_index,
                        });
                    }
                    intervals.push((left_start, right_end));
                    depths.push(
                        depths[*left as usize]
                            .max(depths[*right as usize])
                            .checked_add(1)
                            .ok_or(ContractionF32TreeError::NodeCountOverflow)?,
                    );
                }
            }
        }
        let (root_start, root_end) = intervals[root as usize];
        if root_start != 0 || root_end != contributor_count {
            return Err(ContractionF32TreeError::RootCoverage {
                expected: contributor_count,
                actual_start: root_start,
                actual_end: root_end,
            });
        }
        let depth = depths[root as usize];
        if depth > limits.max_depth() {
            return Err(ContractionF32TreeError::DepthLimit {
                limit: limits.max_depth(),
                actual: depth,
            });
        }
        Ok(Self {
            contributor_count,
            nodes,
            depth,
        })
    }

    /// Returns the contributor count `K`.
    #[must_use]
    pub const fn contributor_count(&self) -> u64 {
        self.contributor_count
    }

    /// Returns the validated postorder node vector.
    #[must_use]
    pub fn nodes(&self) -> &[ContractionF32TreeNode] {
        &self.nodes
    }

    /// Returns the root's node index, which is always the last node.
    ///
    /// # Panics
    ///
    /// Never panics: a validated tree's node count is bounded by the
    /// `u32`-capped caller allowance and is nonzero.
    #[must_use]
    pub fn root(&self) -> u32 {
        u32::try_from(self.nodes.len() - 1).expect("a validated tree's node count fits u32")
    }

    /// Returns the validated depth: the node count of the longest
    /// root-to-leaf path, so a single-leaf tree has depth one.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }
}
