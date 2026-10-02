//! Trigger events with a condition about how they happened (`TriggerCond::Where`):
//! "Whenever an opponent mills a nonland card" (a card leaving a library because it was
//! milled, CR 701.17a), "whenever one or more lands enter without being played" (CR
//! 305.1), "whenever an opponent draws a card except the first one they draw in each of
//! their draw steps" (CR 504.1).

use super::nouns::Det;
use super::players::Case;
use super::*;
use crate::kw::trigger_event_causes::{MILLED, NOT_PLAYED};

const NOT_FIRST_DRAW: &str = "draw step:not the first card drawn in a draw step";

/// Whether a condition is the named custom one.
fn is_custom(c: &Condition, name: &str) -> bool {
    matches!(c, Condition::Custom(n) if n == name)
}

impl Renderer<'_> {
    /// (subject, verb phrase) of such a trigger event, if it's one.
    pub(crate) fn trigger_with_cause(
        &mut self,
        trigger: &TriggerCond,
        cond: &Condition,
        det: &Det,
    ) -> Option<(String, String)> {
        // "Whenever an opponent draws a card except the first one they draw in each of
        // their draw steps"; "Whenever you draw your first card during each of your draw
        // steps".
        if let TriggerCond::Draws { who } = trigger {
            let w = self.rel_subject(*who);
            let you = w == "you";
            if is_custom(cond, NOT_FIRST_DRAW) {
                let (pron, poss, verb) = if you {
                    ("you", "your", "draw")
                } else {
                    ("they", "their", "draws")
                };
                return Some((
                    w,
                    format!("{verb} a card except the first one {pron} draw in each of {poss} draw steps"),
                ));
            }
            if matches!(cond, Condition::Not(c) if is_custom(c, NOT_FIRST_DRAW)) {
                let (poss, verb) = if you {
                    ("your", "draw")
                } else {
                    ("their", "draws")
                };
                return Some((
                    w,
                    format!("{verb} {poss} first card during each of {poss} draw steps"),
                ));
            }
            return None;
        }
        // "When ~ enters during the declare attackers step".
        if matches!(cond, Condition::Phase(_)) {
            let c = self.condition(cond);
            if let Some(rest) = c.strip_prefix("it's ") {
                if !rest.contains(['{', '|', '}']) {
                    let (s, vp) = self.trigger_event_parts(trigger);
                    return Some((
                        s,
                        format!("{vp} {{alt:during {rest}|while it's {rest}|if it's {rest}}}"),
                    ));
                }
            }
            return None;
        }
        // "Whenever one or more lands enter under an opponent's control without being
        // played".
        if is_custom(cond, NOT_PLAYED) {
            let (s, vp) = self.trigger_event_parts(trigger);
            return Some((s, format!("{vp} without being played")));
        }
        // Milled cards: the condition, maybe with who milled them ("an opponent").
        let (milled, who) = match cond {
            c if is_custom(c, MILLED) => (true, None),
            Condition::And(v) if v.len() == 2 && is_custom(&v[0], MILLED) => match &v[1] {
                Condition::PlayerMatches(PlayerRef::TriggerPlayer, pf) => (true, Some(pf.clone())),
                _ => return None,
            },
            _ => return None,
        };
        if !milled {
            return None;
        }
        let TriggerCond::ZoneChange {
            filter,
            from: Some(ZoneKind::Library),
            ..
        } = trigger
        else {
            return None;
        };
        // The card is described as it was in the library.
        let f = match filter {
            Filter::And(v) => Filter::and(
                v.iter()
                    .filter(|x| !matches!(x, Filter::InZone(_) | Filter::OwnedBy(_)))
                    .cloned()
                    .collect(),
            ),
            other => other.clone(),
        };
        let many = matches!(det, Det::OneOrMore);
        let n = self.noun_det(&f, det.clone());
        let player = match &who {
            Some(pf) => {
                let p = self.player_filter_noun(pf, Num::One);
                with_article(&p)
            }
            None => "a player".to_string(),
        };
        let _ = Case::Obj;
        if n.contains(['{', '|', '}']) || who.is_some() {
            return Some((player, format!("mills {n}")));
        }
        let be = if many { "are" } else { "is" };
        Some((
            String::new(),
            format!("{{alt:{n} {be} milled|{player} mills {n}}}"),
        ))
    }
}
