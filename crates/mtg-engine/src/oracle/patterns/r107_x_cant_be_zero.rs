//! "X can't be 0." in an activated ability (CR 107.3a, 602.2b): "{X}{G}: Until end of
//! turn, this land becomes an X/X green Hydra creature. It's still a land. X can't be 0."
//! (Lair of the Hydra), or after each mode of a modal one (Marath, Will of the Wild:
//! "{X}, Remove X +1/+1 counters from Marath: Choose one — • Put X +1/+1 counters on
//! target creature. X can't be 0. • ..."). The value announced for X must be at least 1:
//! the ability gets the activation condition "X is 1 or more", which the engine checks
//! for the announced value (an ability with X in its cost can be activated if some value
//! of X satisfies its condition).

use crate::ability::*;

const SENTENCE: &str = "x can't be 0.";

/// The effect text without its "X can't be 0." sentences, if it has any: in the main
/// text, or after every mode of a modal ability. `None` if it has none, or if only some
/// modes say it (a restriction on choosing those modes, not expressed here).
pub(crate) fn strip(eff: &str) -> Option<String> {
    if !eff.to_lowercase().contains(SENTENCE) {
        return None;
    }
    let mut modes = 0;
    let mut said = 0;
    let mut lines = Vec::new();
    for line in eff.lines() {
        let is_mode = line.trim_start().starts_with('•');
        modes += is_mode as usize;
        let lower = line.to_lowercase();
        let t = lower.trim_end();
        let kept = match t.strip_suffix(SENTENCE) {
            Some(head)
                if (head.is_empty() || head.ends_with(' ')) && line.is_char_boundary(head.len()) =>
            {
                said += is_mode as usize;
                line[..head.len()].trim_end().to_string()
            }
            _ => line.to_string(),
        };
        // Nothing else may still say it ("X can't be 0. Draw X cards.").
        if kept.to_lowercase().contains(SENTENCE) {
            return None;
        }
        if !kept.trim().is_empty() {
            lines.push(kept);
        }
    }
    if said > 0 && said != modes {
        return None;
    }
    Some(lines.join("\n"))
}

/// The activation condition "X is 1 or more", added to the ability's own.
pub(crate) fn condition(existing: Option<Condition>) -> Condition {
    let x = Condition::Compare(Value::X, Cmp::Ge, Value::c(1));
    match existing {
        Some(c) => Condition::And(vec![c, x]),
        None => x,
    }
}
