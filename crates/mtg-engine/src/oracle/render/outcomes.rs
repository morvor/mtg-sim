//! Instructions whose outcome later parts of the effect refer to (CR 608.2c):
//!
//! * Revealing cards from a hand (CR 701.20a): the cards chosen are stored, then revealed
//!   (`kw::reveal_from_hand`); later parts say "them", "one of them", "the number of cards
//!   revealed this way".
//! * Conditions about what an earlier instruction did: "If a land card was milled this
//!   way, ...", "If you discarded a card this way, ...".
//!
//! Compiled as a snapshot of the objects the instruction affected (a `Store` of the
//! instruction's variable), the matching ones remembered for "it" in the effect (a `Store`
//! of `All(... In(snapshot))`), and an `If` whose condition says some object of the
//! snapshot matches: `SelNonEmpty(snapshot)` and not every object fails the filter.

use super::nouns::Det;
use super::players::Case;
use super::*;

/// How an instruction's outcome is described: the participles of the passive form ("a land
/// card was milled"), and the player who performed it with the verb's present and past
/// forms ("that player discards / discarded an artifact card").
#[derive(Clone, Debug)]
pub(crate) struct OutcomeVerb {
    passive: &'static [&'static str],
    actor: Option<(PlayerRef, &'static str, &'static str)>,
}

/// The outcome verb of an instruction, looking into optional and per-player parts.
fn outcome_verb(e: &Effect) -> Option<OutcomeVerb> {
    let v = |passive, actor| Some(OutcomeVerb { passive, actor });
    match e {
        Effect::May { effect, .. } => outcome_verb(effect),
        Effect::If { then, .. } => outcome_verb(then),
        Effect::Seq(v) => v.iter().rev().find_map(outcome_verb),
        Effect::ForEachPlayer { who, effect } => {
            let mut o = outcome_verb(effect)?;
            if let Some((p, _, _)) = &mut o.actor {
                if matches!(p, PlayerRef::Iterated) {
                    *p = who.clone();
                }
            }
            Some(o)
        }
        Effect::Mill { who, .. } => v(&["milled"], Some((who.clone(), "mills", "milled"))),
        Effect::Discard { who, .. } => {
            v(&["discarded"], Some((who.clone(), "discards", "discarded")))
        }
        Effect::Sacrifice { who, .. } => v(
            &["sacrificed"],
            Some((who.clone(), "sacrifices", "sacrificed")),
        ),
        Effect::Exile { .. } => v(
            &["exiled", "put into exile"],
            Some((PlayerRef::You, "exiles", "exiled")),
        ),
        Effect::Move { to, .. } if to.zone == ZoneKind::Exile => v(
            &["exiled", "put into exile"],
            Some((PlayerRef::You, "exiles", "exiled")),
        ),
        Effect::Move { to, .. } if to.zone == ZoneKind::Hand => {
            v(&["returned"], Some((PlayerRef::You, "returns", "returned")))
        }
        Effect::Destroy { .. } => v(
            &["destroyed"],
            Some((PlayerRef::You, "destroys", "destroyed")),
        ),
        Effect::DealDamage { .. } => v(&["dealt damage"], None),
        Effect::RevealHand { who } => v(&["revealed"], Some((who.clone(), "reveals", "revealed"))),
        Effect::Dig {
            who, reveal: true, ..
        } => v(&["revealed"], Some((who.clone(), "reveals", "revealed"))),
        _ => None,
    }
}

/// A snapshot of what an instruction affected: the variable it's stored in. An optional
/// instruction's snapshot is empty when it wasn't performed.
fn snapshot_var(e: &Effect) -> Option<Var> {
    match e {
        Effect::Store {
            var,
            sel: Sel::Var(_),
        } => Some(*var),
        Effect::If {
            cond: Condition::PrevHappened,
            then,
            otherwise,
        } => match (then.as_ref(), otherwise.as_ref()) {
            (
                Effect::Store {
                    var: a,
                    sel: Sel::Var(_),
                },
                Effect::Store {
                    var: b,
                    sel: Sel::None,
                },
            ) if a == b => Some(*a),
            _ => None,
        },
        _ => None,
    }
}

/// "Some object of the snapshot matches `f`": (the snapshot, `f`).
fn some_matches(c: &Condition) -> Option<(Var, Filter)> {
    match c {
        Condition::SelNonEmpty(Sel::Var(s)) => Some((*s, Filter::Any)),
        Condition::And(v) => match v.as_slice() {
            [Condition::SelNonEmpty(Sel::Var(s)), Condition::Not(inner)] => match inner.as_ref() {
                Condition::SelMatches(Sel::Var(s2), fails) if s == s2 => {
                    let f = match fails {
                        Filter::Not(x) => (**x).clone(),
                        other => Filter::Not(Box::new(other.clone())),
                    };
                    Some((*s, f))
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// Whether a filter is about the objects of a snapshot (`In(Var(s))`).
fn mentions_snapshot(f: &Filter, s: Var) -> bool {
    match f {
        Filter::In(sel) => matches!(sel.as_ref(), Sel::Var(x) if *x == s),
        Filter::And(v) => v.iter().any(|x| mentions_snapshot(x, s)),
        _ => false,
    }
}

fn owned_by_you(f: &Filter) -> bool {
    matches!(f, Filter::OwnedBy(PlayerRel::You))
}

impl Renderer<'_> {
    /// At `v[i]` of a sequence: renders an instruction (or several) whose outcome later
    /// parts refer to, returning how many instructions it covers and its text.
    pub(crate) fn outcome_seq_part(
        &mut self,
        v: &[Effect],
        i: usize,
        known: &mut Vec<(Var, OutcomeVerb)>,
    ) -> Option<(usize, String)> {
        self.reveal_part(v, i)
            .or_else(|| self.each_player_part(v, i))
            .or_else(|| self.outcome_part(v, i, known))
    }

    /// "Reveal [cards] from your hand", "target player reveals three cards from their
    /// hand", "reveal it and put it into your hand": the chosen cards stored, then
    /// revealed.
    fn reveal_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        use crate::kw::reveal_from_hand::{REVEALED, REVEAL_CHOSEN};
        let (Effect::Store { var: REVEALED, sel }, Some(Effect::Custom(c))) = (&v[i], v.get(i + 1))
        else {
            return None;
        };
        if c != REVEAL_CHOSEN {
            return None;
        }
        let text = match sel {
            Sel::Choose { chooser, .. } => {
                let c = self.player(chooser, Case::Subj);
                let what = Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: match sel {
                        Sel::Choose { filter, .. } => filter.clone(),
                        _ => Filter::Any,
                    },
                    count: match sel {
                        Sel::Choose { count, .. } => count.clone(),
                        _ => Value::Const(1),
                    },
                    up_to: matches!(sel, Sel::Choose { up_to: true, .. }),
                    store: None,
                };
                // "reveals a number of cards from their hand equal to ...".
                let s = match &what {
                    Sel::Choose {
                        filter,
                        count,
                        up_to: false,
                        ..
                    } if !Self::is_simple(count)
                        && !matches!(count, Value::Var(_) | Value::Prev) =>
                    {
                        let n = self.noun_det(filter, Det::Plural);
                        let k = self.value(count);
                        format!("a number of {n} equal to {k}")
                    }
                    _ => self.sel(&what, Case::Obj),
                };
                if c == "you" {
                    format!("reveal {s}")
                } else {
                    format!("{c} reveals {s}")
                }
            }
            other => {
                let s = self.sel(other, Case::Obj);
                format!("reveal {s}")
            }
        };
        Some((2, text))
    }

    /// At `v[i]` of a sequence: renders a part of an outcome condition, returning how many
    /// instructions it covers and its text (empty for bookkeeping). `known`: the
    /// snapshots seen so far in the sequence, with the verb of the instruction each one
    /// is about.
    pub(crate) fn outcome_part(
        &mut self,
        v: &[Effect],
        i: usize,
        known: &mut Vec<(Var, OutcomeVerb)>,
    ) -> Option<(usize, String)> {
        let asked_about = |s: Var, from: usize| {
            v[from..].iter().any(|e| {
                matches!(e, Effect::If { cond, .. } if some_matches(cond).is_some_and(|(x, _)| x == s))
            })
        };
        // The snapshot: bookkeeping, once the instruction it's about is known.
        if let Some(s) = snapshot_var(&v[i]) {
            if asked_about(s, i + 1) {
                let verb = v[..i].iter().rev().find_map(outcome_verb)?;
                known.push((s, verb));
                return Some((1, String::new()));
            }
            return None;
        }
        match &v[i] {
            // What "it" refers to in the effect: the matching objects.
            Effect::Store {
                sel: Sel::All(f), ..
            } if known.iter().any(|(s, _)| mentions_snapshot(f, *s)) => {
                let next_asks = v.get(i + 1).is_some_and(|e| {
                    matches!(e, Effect::If { cond, .. }
                        if some_matches(cond).is_some_and(|(x, _)| mentions_snapshot(f, x)))
                });
                next_asks.then(|| (1, String::new()))
            }
            Effect::If {
                cond,
                then,
                otherwise,
            } => {
                let (s, f) = some_matches(cond)?;
                let verb = known.iter().find(|(x, _)| *x == s)?.1.clone();
                let c = self.outcome_condition(&f, &verb);
                let text = if matches!(otherwise.as_ref(), Effect::Noop) {
                    let t = self.effect(then);
                    format!("if {c} this way, {t}")
                } else {
                    let o = self.effect(otherwise);
                    let t = self.effect(then);
                    format!("{o}. If {c} this way, {t} instead")
                };
                Some((1, text))
            }
            _ => None,
        }
    }

    /// "a land card was milled", "you discarded a card", "that player discards an artifact
    /// card" (without "this way").
    fn outcome_condition(&mut self, f: &Filter, verb: &OutcomeVerb) -> String {
        // Cards the player who discarded or milled them owns: "you discarded a card".
        let (by_you, f) = match f {
            Filter::And(v) if v.iter().any(owned_by_you) => {
                let rest: Vec<Filter> = v.iter().filter(|x| !owned_by_you(x)).cloned().collect();
                (true, Filter::and(rest))
            }
            other => (false, other.clone()),
        };
        let noun = match &f {
            Filter::Any => "a card".to_string(),
            f => self.noun_det(f, Det::A),
        };
        if by_you {
            if let Some((_, _, past)) = verb.actor {
                return format!("you {past} {noun}");
            }
        }
        let passive = verb.passive.join("|");
        if noun.contains(['{', '|']) {
            return format!("{noun} {{alt:is|was}} {{alt:{passive}}}");
        }
        let mut alts = Vec::new();
        for p in verb.passive {
            for be in ["is", "was"] {
                alts.push(format!("{noun} {be} {p}"));
                if let Some(rest) = noun.strip_prefix("a ").or(noun.strip_prefix("an ")) {
                    alts.push(format!("at least one {rest} {be} {p}"));
                }
            }
        }
        if let Some((who, present, past)) = &verb.actor {
            let subj = self.player(who, Case::Subj);
            if subj == "you" {
                alts.push(format!("you {past} {noun}"));
            } else if !subj.contains(['{', '|']) && !subj.starts_with("each ") {
                alts.push(format!("{subj} {present} {noun}"));
                alts.push(format!("{subj} {past} {noun}"));
            }
        }
        format!("{{alt:{}}}", alts.join("|"))
    }
}
