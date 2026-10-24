//! Settings of the amalgamation (the defaults of wQFM version 1.4).

/// Settings of `wQFM-rs`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    /// Weight of violated quartets in the partition score (wQFM default 1).
    pub beta: f64,
    /// Limit of FM iterations per level (wQFM: 1,000,000).
    pub max_iterations: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            beta: 1.0,
            max_iterations: 1_000_000,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn defaults_match_wqfm() {
        let o = super::Options::default();
        assert_eq!((o.beta, o.max_iterations), (1.0, 1_000_000));
    }
}
