//! Replacement effects that change how many counters are put ([`ReplacementEvent::
//! PutCountersMatching`], CR 614.1a, 122.6): "If an effect would put one or more counters
//! on a permanent you control, it puts twice that many of those counters on that
//! permanent instead", "If you would put one or more counters on a permanent you control,
//! put that many plus one of each of those kinds of counters on that permanent instead".

use super::nouns::Det;
use super::*;

impl Renderer<'_> {
    pub(crate) fn counters_matching_replacement(
        &mut self,
        on_objects: Option<&Filter>,
        on_players: Option<&PlayerFilter>,
        kind: Option<&CounterKind>,
        by: Option<PlayerRel>,
        effect_only: bool,
        action: &ReplacementAction,
    ) -> String {
        let (one_or_more, those) = match kind {
            Some(k) => {
                let c = plural(&counter_name(k));
                (format!("one or more {c}"), c)
            }
            None => (
                "one or more counters".to_string(),
                "{alt:those counters|of each of those kinds of counters}".to_string(),
            ),
        };
        // Where: "on a permanent you control", "on a creature or planeswalker you control
        // or on yourself".
        let mut on = Vec::new();
        let mut that = Vec::new();
        if let Some(f) = on_objects {
            on.push(format!("on {}", self.noun_det(f, Det::A)));
            that.push("permanent");
        }
        if let Some(pf) = on_players {
            let p = match pf {
                PlayerFilter::You | PlayerFilter::Controller => "yourself".to_string(),
                other => with_article(&self.player_filter_noun(other, Num::One)),
            };
            on.push(format!("on {p}"));
            that.push("player");
        }
        let on = join_list(&on, "or");
        let that = format!("{{alt:it|that {}}}", that.join(" or "));
        let amount = match action {
            ReplacementAction::Multiply(2) => "twice that many".to_string(),
            ReplacementAction::Multiply(n) => format!("{} times that many", number_word(*n)),
            ReplacementAction::Add(v) => {
                let v = match v {
                    Value::Const(n) => number_word(*n),
                    other => self.value(other),
                };
                format!("that many plus {v}")
            }
            _ => return self.gap("a counter replacement"),
        };
        let what = if kind.is_none() && matches!(action, ReplacementAction::Add(_)) {
            "of each of those kinds of counters".to_string()
        } else if kind.is_none() {
            "of those counters".to_string()
        } else {
            those
        };
        let (subj, verb, doer) = if effect_only {
            ("an effect".to_string(), "put", "it puts".to_string())
        } else {
            match by {
                Some(PlayerRel::You) => ("you".to_string(), "put", "{opt:you} put".to_string()),
                Some(r) => {
                    let w = self.rel_subject(r);
                    (w.clone(), "put", format!("{w} puts"))
                }
                None => {
                    return format!(
                    "if {one_or_more} would be put {on}, {amount} {what} are put on {that} instead"
                )
                }
            }
        };
        let _ = verb;
        format!("if {subj} would put {one_or_more} {on}, {doer} {amount} {what} on {that} instead")
    }
}
