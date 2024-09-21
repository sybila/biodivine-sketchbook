//! Modified versions of algorithms adapted from [AEON](https://biodivine.fi.muni.cz/aeon/).
//! These algorithms can be used to compute attractor states and optimize some procedures.

/// Wrappers for AEON algorithms for attractor search.
pub mod compute_attractors;

/// Xie-Beerel TSCC algorithm
mod algo_xie_beerel;
/// Interleaved transition guided reduction quickly eliminates most non-attractor states.
mod itgr;
/// Reachability algorithms that use saturation for improved efficiency.
mod saturated_reachability;
