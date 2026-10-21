//! Objects and players described by what happened to them ([`Filter::History`],
//! [`PlayerFilter::History`]): "creature that fought this turn", "card in a graveyard that
//! was put there from a library this turn", "player who lost life this turn". The history
//! condition is worded about "~" (the object) or "you" (the player) and made a relative
//! clause.

use super::*;

/// The condition with `from` (a JSON string such as `"Iterated"`) replaced by `to`
/// wherever it appears as a plain string, and the tested object as the source.
fn reworded(c: &Condition, tested_as_source: bool, iterated_as_you: bool) -> Option<Condition> {
    let mut json = serde_json::to_value(c).ok()?;
    let tested = serde_json::to_value(Filter::In(Box::new(Sel::Var(vars::TESTED)))).ok()?;
    fn walk(v: &mut serde_json::Value, tested: &serde_json::Value, t: bool, i: bool) {
        use serde_json::Value as J;
        if t && *v == *tested {
            *v = J::String("Source".into());
            return;
        }
        match v {
            J::String(s) if i && s == "Iterated" => *s = "You".into(),
            J::Array(a) => a.iter_mut().for_each(|x| walk(x, tested, t, i)),
            J::Object(m) => m.values_mut().for_each(|x| walk(x, tested, t, i)),
            _ => {}
        }
    }
    walk(&mut json, &tested, tested_as_source, iterated_as_you);
    serde_json::from_value(json).ok()
}

impl Renderer<'_> {
    /// "that fought this turn" for [`Filter::History`].
    pub(crate) fn history_filter_clause(&mut self, c: &Condition) -> String {
        let Some(c2) = reworded(c, true, false) else {
            return self.gap("history filter");
        };
        let s = self.condition(&c2);
        match s.strip_prefix("~ ") {
            Some(r) => format!("that {r}"),
            None => format!("such that {s}"),
        }
    }

    /// "who lost life this turn" for [`PlayerFilter::History`].
    pub(crate) fn history_player_clause(&mut self, c: &Condition) -> String {
        let Some(c2) = reworded(c, false, true) else {
            return self.gap("history player filter");
        };
        let s = self.condition(&c2);
        if let Some(r) = s.strip_prefix("you've ") {
            return format!("who has {r}");
        }
        if let Some(r) = s.strip_prefix("you were ") {
            return format!("who was {r}");
        }
        match s.strip_prefix("you ") {
            Some(r) => format!("who {r}"),
            None => format!("such that {s}"),
        }
    }
}
