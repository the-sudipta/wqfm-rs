//! What to do about the library's error messages, for the command line and
//! the GUI.

/// A short hint for an error message of this library, if one fits.
pub fn error_hint(message: &str) -> Option<&'static str> {
    const HINTS: [(&str, &str); 9] = [
        (
            "an unresolved 4-set",
            "lines like ((a,b,c,d)); n come from generate_wqrts.py on gene trees with \
             polytomies; make the file with `wqfm-rs gt2wq`, which leaves them out",
        ),
        (
            "quartets after an empty line",
            "remove the empty line; wQFM would silently ignore everything after it",
        ),
        (
            "not a finite number",
            "weights must be ordinary numbers such as 3, 0.5 or 2e-3",
        ),
        (
            "is not a number",
            "the last field of each line is the weight, e.g. ((a,b),(c,d)); 0.75",
        ),
        (
            "expected ((a,b),(c,d)); w",
            "each line needs four names and a weight: ((a,b),(c,d)); w",
        ),
        (
            "different taxa",
            "both trees must have exactly the same leaf names",
        ),
        (
            "unknown leaf",
            "the tree has a leaf that does not appear in the quartet file",
        ),
        (
            "a taxon appears twice",
            "each leaf name may appear only once per gene tree",
        ),
        (
            "invalid Newick",
            "each tree must be one line of Newick ending with ;",
        ),
    ];
    HINTS
        .iter()
        .find(|(pattern, _)| message.contains(pattern))
        .map(|(_, hint)| *hint)
}

#[cfg(test)]
mod tests {
    #[test]
    fn errors_get_hints() {
        let polytomy = "line 3: not a quartet ((a,b),(c,d)); w (an unresolved 4-set?)";
        assert!(super::error_hint(polytomy).is_some());
        assert!(super::error_hint("something else").is_none());
    }
}
