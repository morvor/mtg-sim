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
        // "Whenever a spell or ability causes a player to shuffle their library"
        // (`trigger_grammar_4.rs`).
        if let Some(r) = self.caused_by_spell(trigger, cond) {
            return Some(r);
        }
        // "Whenever one or more opponents each lose exactly 1 life" (`trigger_grammar_4.rs`).
        if let (
            TriggerCond::Batched {
                trigger: inner,
                per: BatchPer::AnyPlayer,
            },
            Condition::Compare(Value::EventAmount, cmp, Value::Const(n)),
        ) = (trigger, cond)
        {
            if let TriggerCond::LosesLife { who } = inner.as_ref() {
                let who = match who {
                    PlayerRel::Opponent => "one or more opponents",
                    _ => "one or more players",
                };
                let amount = match cmp {
                    Cmp::Eq => format!("exactly {}", number_word(*n)),
                    _ => format!("{} or more", number_word(*n)),
                };
                return Some((who.to_string(), format!("each lose {amount} life")));
            }
        }
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
        // The card is described as it was in the library. Its owner is the player who
        // milled it (a library holds only its owner's cards, CR 400.3, 701.17a).
        let atoms: Vec<Filter> = match filter {
            Filter::And(v) => v.clone(),
            other => vec![other.clone()],
        };
        let owner = atoms.iter().find_map(|x| match x {
            Filter::OwnedBy(r) => Some(*r),
            _ => None,
        });
        if owner.is_some() && who.is_some() {
            return None;
        }
        let f = Filter::and(
            atoms
                .into_iter()
                .filter(|x| !matches!(x, Filter::InZone(_) | Filter::OwnedBy(_)))
                .collect(),
        );
        let many = matches!(det, Det::OneOrMore);
        let n = self.noun_det(&f, det.clone());
        let player = match (&who, owner) {
            (Some(pf), _) => {
                let p = self.player_filter_noun(pf, Num::One);
                with_article(&p)
            }
            (None, Some(r)) => self.rel_subject(r),
            (None, None) => "a player".to_string(),
        };
        let _ = Case::Obj;
        if n.contains(['{', '|', '}']) || who.is_some() || owner.is_some() {
            return Some((player, format!("mills {n}")));
        }
        let be = if many { "are" } else { "is" };
        Some((
            String::new(),
            format!("{{alt:{n} {be} milled|{player} mills {n}}}"),
        ))
    }

    /// "[a spell or ability] causes [subject] to [verb phrase]".
    fn caused_by_spell(
        &mut self,
        trigger: &TriggerCond,
        cond: &Condition,
    ) -> Option<(String, String)> {
        use crate::kw::trigger_events as te;
        let (cause, own) = match cond {
            Condition::And(v) if v.len() == 2 && is_custom(&v[1], te::CAUSED_BY_EVENT_PLAYERS) => {
                (&v[0], true)
            }
            c => (c, false),
        };
        let cause = if is_custom(cause, te::CAUSED_BY_SPELL_OR_ABILITY) {
            "a spell or ability".to_string()
        } else if is_custom(cause, te::CAUSED_BY_OPPONENTS) {
            "a spell or ability an opponent controls".to_string()
        } else if let Condition::Compare(Value::CountSel(sel), Cmp::Ge, Value::Const(1)) = cause {
            let Sel::All(Filter::And(parts)) = sel.as_ref() else {
                return None;
            };
            let is_cause = |f: &Filter| matches!(f, Filter::Custom(n) if n == te::THE_CAUSE);
            if !parts.iter().any(is_cause) {
                return None;
            }
            let rest: Vec<Filter> = parts.iter().filter(|f| !is_cause(f)).cloned().collect();
            self.noun_det(&Filter::and(rest), Det::A)
        } else {
            return None;
        };
        let (subj, vp) = match trigger {
            // "a land to be put into your graveyard from the battlefield".
            TriggerCond::Dies(Filter::And(v))
                if v.iter()
                    .any(|f| matches!(f, Filter::OwnedBy(PlayerRel::You))) =>
            {
                let rest: Vec<Filter> = v
                    .iter()
                    .filter(|f| !matches!(f, Filter::OwnedBy(PlayerRel::You)))
                    .cloned()
                    .collect();
                (
                    self.noun_det(&Filter::and(rest), Det::A),
                    "is put into your graveyard from the battlefield".to_string(),
                )
            }
            _ => self.trigger_event_parts(trigger),
        };
        let vp = if vp == "shuffles" || vp == "shuffle" {
            format!("{vp} their library")
        } else {
            vp
        };
        let subj = if own {
            "its controller".to_string()
        } else {
            subj
        };
        // The verb phrase in the infinitive: "shuffles their library" → "shuffle their
        // library", "is put into" → "be put into".
        let vp = if let Some(r) = vp.strip_prefix("is ") {
            format!("be {r}")
        } else if subj == "you" {
            vp
        } else {
            let (v, r) = vp.split_once(' ').unwrap_or((vp.as_str(), ""));
            let v = v
                .strip_suffix("shes")
                .map(|x| format!("{x}sh"))
                .or_else(|| v.strip_suffix('s').map(str::to_string))
                .unwrap_or_else(|| v.to_string());
            format!("{v} {r}").trim_end().to_string()
        };
        Some((cause, format!("causes {subj} to {vp}")))
    }
}
