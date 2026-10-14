//! Consecutive rendered abilities that cards print as one sentence.

/// "Prevent all damage that would be dealt to X." + "Prevent all damage that would be
/// dealt by X." = "Prevent all damage that would be dealt to and dealt by X." (two
/// prevention effects, one for each direction, CR 615.1).
pub(super) fn join_lines(a: &str, b: &str) -> Option<String> {
    let a = a.trim_end_matches('.');
    let b = b.trim_end_matches('.');
    let (head, to) = a.split_once(" that would be dealt to ")?;
    if !head.starts_with("Prevent all ") {
        return None;
    }
    let rest = b
        .strip_prefix(head)?
        .strip_prefix(" that would be dealt ")?;
    let by = rest
        .strip_prefix("{opt:to a permanent or player} by ")
        .or_else(|| rest.strip_prefix("by "))?;
    (by == to).then(|| format!("{head} that would be dealt to and dealt by {to}."))
}
