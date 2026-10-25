//! One level of the divide-and-conquer recursion.

use std::borrow::Cow;

use crate::quartet::Quartet;

/// A taxon set (real and dummy taxa) with the weighted quartets on it. The
/// top level borrows the caller's quartets instead of copying them.
pub(crate) struct Level<'a> {
    pub(crate) taxa: Vec<u32>,
    pub(crate) quartets: Cow<'a, [Quartet]>,
}
