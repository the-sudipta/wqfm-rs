//! One level of the divide-and-conquer recursion.

use crate::quartet::Quartet;

/// A taxon set (real and dummy taxa) with the weighted quartets on it.
pub(crate) struct Level {
    pub(crate) taxa: Vec<u32>,
    pub(crate) quartets: Vec<Quartet>,
}
