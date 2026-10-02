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
    /// "If it's a creature card, you may reveal it and put it into your hand. If you don't
    /// put the card into your hand, you may put it into your graveyard.": compiled as
    /// `If { c, [A, if not done: B], otherwise: B }` — B happens if the condition doesn't
    /// hold or A wasn't done.
    pub(crate) fn if_or_else_part(&mut self, e: &Effect) -> Option<String> {
        let Effect::If {
            cond,
            then,
            otherwise,
        } = e
        else {
            return None;
        };
        let Effect::Seq(v) = then.as_ref() else {
            return None;
        };
        let [a, Effect::If {
            cond: Condition::Not(np),
            then: b,
            otherwise: none,
        }] = v.as_slice()
        else {
            return None;
        };
        if !matches!(np.as_ref(), Condition::PrevHappened)
            || !matches!(none.as_ref(), Effect::Noop)
            || format!("{b:?}") != format!("{otherwise:?}")
        {
            return None;
        }
        let c = self.condition(cond);
        let a_text = self.effect(a);
        // What wasn't done: "put the card into your hand".
        let done = match a {
            Effect::May { effect, .. } => match effect.as_ref() {
                Effect::Seq(steps) => steps.last(),
                other => Some(other),
            },
            _ => None,
        };
        let what = match done {
            Some(Effect::Move { to, .. }) if to.zone == ZoneKind::Hand => {
                " {opt:put it into your hand}"
            }
            _ => "",
        };
        let b_text = self.effect(b);
        Some(format!("if {c}, {a_text}. If you don't{what}, {b_text}"))
    }

    /// At `v[i]` of a sequence: renders an instruction (or several) whose outcome later
    /// parts refer to, returning how many instructions it covers and its text.
    pub(crate) fn outcome_seq_part(
        &mut self,
        v: &[Effect],
        i: usize,
        known: &mut Vec<(Var, OutcomeVerb)>,
    ) -> Option<(usize, String)> {
        self.look_then_exile_part(v, i)
            .or_else(|| self.named_group_part(v, i))
            .or_else(|| self.distribute_part(v, i))
            .or_else(|| self.each_chooses_part(v, i))
            .or_else(|| self.random_pick_part(v, i))
            .or_else(|| self.reveal_part(v, i))
            .or_else(|| self.each_player_part(v, i))
            .or_else(|| self.outcome_part(v, i, known))
    }

    /// "Look at the top card of that player's library, then exile it face down": exiled
    /// face down, and you may look at it (CR 406.3, 708.2).
    fn look_then_exile_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        let (
            Effect::Exile {
                what,
                face_down: true,
                ..
            },
            Some(Effect::Custom(c)),
        ) = (&v[i], v.get(i + 1))
        else {
            return None;
        };
        if c != "zones:may look at exiled" {
            return None;
        }
        let s = self.sel(what, Case::Obj);
        let pron = if super::effects::is_plural_sel(what) {
            "them"
        } else {
            "it"
        };
        Some((2, format!("look at {s}, then exile {pron} face down")))
    }

    /// "Untap target creature and each other creature that shares a color with it. Those
    /// creatures get +2/+0": a group remembered as the next instruction names it; later
    /// mentions are "them".
    fn named_group_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        let Effect::Store {
            var,
            sel: sel @ Sel::Union(parts),
        } = &v[i]
        else {
            return None;
        };
        let named = format!("{sel:?}");
        if parts.len() < 2
            || !v
                .get(i + 1)
                .is_some_and(|n| format!("{n:?}").contains(&named))
        {
            return None;
        }
        self.plural_vars.push(*var);
        Some((1, String::new()))
    }

    /// "Put one of them into your hand, one on top of your library, and one on the bottom
    /// of your library": each step picks one of the group not picked yet.
    fn distribute_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        let (
            Effect::Store {
                var: done,
                sel: Sel::None,
            },
            Some(Effect::Store {
                var: group,
                sel: Sel::Var(_),
            }),
        ) = (&v[i], v.get(i + 1))
        else {
            return None;
        };
        let mut j = i + 2;
        let mut items = Vec::new();
        while let (
            Some(Effect::Store {
                var: pick,
                sel:
                    Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::And(f),
                        count: Value::Const(1),
                        up_to: false,
                        ..
                    },
            }),
            Some(Effect::Store {
                var: d2,
                sel: Sel::Union(u),
            }),
            Some(act),
        ) = (v.get(j), v.get(j + 1), v.get(j + 2))
        {
            let from_group = matches!(f.as_slice(), [Filter::In(a), Filter::Not(b)]
                if matches!(a.as_ref(), Sel::Var(x) if x == group)
                    && matches!(b.as_ref(), Filter::In(c) if matches!(c.as_ref(), Sel::Var(x) if x == done)));
            let adds = d2 == done
                && matches!(u.as_slice(), [Sel::Var(a), Sel::Var(b)] if a == done && b == pick);
            if !from_group || !adds {
                break;
            }
            let one = if items.is_empty() {
                "one of them"
            } else {
                "one {opt:of them}"
            };
            let text = match act {
                Effect::Move {
                    what: Sel::Var(x),
                    to,
                } if x == pick => {
                    let m = self.move_effect(&Sel::Var(*pick), to);
                    let (verb, rest) = m.split_once(" it ")?;
                    if items.is_empty() {
                        format!("{verb} {one} {rest}")
                    } else {
                        format!("{{opt:{verb}}} {one} {rest}")
                    }
                }
                Effect::Exile {
                    what: Sel::Var(x), ..
                } if x == pick => format!("exile {one}"),
                _ => return None,
            };
            items.push(text);
            j += 3;
        }
        if items.len() < 2 {
            return None;
        }
        Some((j - i, join_list(&items, "and")))
    }

    /// "Target opponent chooses a creature they control", "for each opponent, choose a
    /// creature with the greatest power among creatures that player controls": the choices
    /// are collected; later mentions are "it" or "them".
    fn each_chooses_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        // The collected choices may start empty.
        let (start, var0) = match &v[i] {
            Effect::Store {
                var,
                sel: Sel::Union(empty),
            } if empty.is_empty() => (1, Some(*var)),
            _ => (0, None),
        };
        let Some(Effect::ForEachPlayer { who, effect }) = v.get(i + start) else {
            return None;
        };
        // Each player chooses as themselves ("chooses a creature card in their
        // graveyard"), or the choice is the iterated player's.
        let (as_player, inner) = match effect.as_ref() {
            Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect,
            } => (true, effect.as_ref()),
            other => (false, other),
        };
        let Effect::Store {
            var,
            sel: Sel::Union(u),
        } = inner
        else {
            return None;
        };
        let [Sel::Var(again), choice @ Sel::Choose { chooser, .. }] = u.as_slice() else {
            return None;
        };
        if again != var || var0.is_some_and(|v0| v0 != *var) {
            return None;
        }
        let single = matches!(who, PlayerRef::You | PlayerRef::Target(_));
        if !single {
            self.plural_vars.push(*var);
        }
        let what = match choice {
            Sel::Choose {
                filter,
                count,
                up_to,
                store,
                ..
            } => Sel::Choose {
                chooser: PlayerRef::You,
                filter: filter.clone(),
                count: count.clone(),
                up_to: *up_to,
                store: *store,
            },
            other => other.clone(),
        };
        let w = self.player(who, Case::Subj);
        let s = self.sel(&what, Case::Obj);
        let text = match (chooser, as_player) {
            (PlayerRef::Iterated, _) | (PlayerRef::You, true) => {
                let s = format!(" {s} ").replace(" your ", " their ");
                format!(
                    "{w} {}",
                    super::effects::third_person(&format!("choose {}", s.trim()))
                )
            }
            (PlayerRef::You, false) if !single => format!("for {w}, choose {s}"),
            _ => return None,
        };
        Some((start + 1, text))
    }

    /// "Return a Zombie creature card at random from your graveyard to the battlefield":
    /// the candidates and how many are remembered, then picked at random
    /// (`kw/zone_moves.rs`); the instruction after names the pick.
    fn random_pick_part(&mut self, v: &[Effect], i: usize) -> Option<(usize, String)> {
        use crate::oracle::patterns::zone_move_grammar::{RANDOM_COUNT, RANDOM_PICK, RANDOM_POOL};
        let (
            Effect::Store {
                var: RANDOM_POOL,
                sel: Sel::All(f),
            },
            Some(Effect::StoreValue {
                var: RANDOM_COUNT,
                value: n,
            }),
            Some(Effect::Custom(c)),
        ) = (&v[i], v.get(i + 1), v.get(i + 2))
        else {
            return None;
        };
        if c != crate::kw::zone_moves::PICK_AT_RANDOM {
            return None;
        }
        // The objects without where they are, then "at random", then where they are.
        let (here, rest): (Vec<Filter>, Vec<Filter>) = match f {
            Filter::And(v) => v.iter().cloned().partition(|x| {
                matches!(x, Filter::InZone(_) | Filter::OwnedBy(_))
                    || matches!(x, Filter::In(s) if matches!(s.as_ref(), Sel::Linked))
            }),
            other => (vec![], vec![other.clone()]),
        };
        let linked = here
            .iter()
            .any(|x| matches!(x, Filter::In(s) if matches!(s.as_ref(), Sel::Linked)));
        let det = match n {
            Value::Const(1) => super::nouns::Det::A,
            Value::Const(k) => super::nouns::Det::Count(number_word(*k)),
            other => super::nouns::Det::Count(self.value(other)),
        };
        let noun = self.noun_det(&Filter::and(rest), det);
        let zone = here.iter().find_map(|x| match x {
            Filter::InZone(z) => Some(*z),
            _ => None,
        });
        let owner = here.iter().find_map(|x| match x {
            Filter::OwnedBy(r) => Some(*r),
            _ => None,
        });
        let phrase = if linked {
            // "a card at random exiled with ~", "a card exiled with it at random".
            let m = self.me();
            if noun.contains(['{', '|', '}']) {
                format!("{noun} at random exiled with {m}")
            } else {
                format!(
                    "{{alt:{noun} at random exiled with {m}|{noun} at random that was exiled with {m}|{noun} exiled with {m} at random}}"
                )
            }
        } else {
            let from = match (zone, owner) {
                (Some(ZoneKind::Graveyard), Some(r)) => {
                    format!(" from {} graveyard", self.rel_possessive(r, Num::One))
                }
                (Some(ZoneKind::Exile), None) => " from exile".into(),
                (None, None) => String::new(),
                _ => return None,
            };
            format!("{noun} at random{from}")
        };
        // Named by the next instruction ("return a creature card at random from your
        // graveyard to your hand"), or chosen first ("choose a card at random in your
        // graveyard. Put it into your hand").
        let pick = format!("{:?}", Sel::Var(RANDOM_PICK));
        let named = v.get(i + 3).is_some_and(|e| {
            let d = format!("{e:?}");
            d.contains(&pick) && !matches!(e, Effect::Store { .. })
        });
        if named {
            self.target_vars.push((RANDOM_PICK, phrase, false));
            return Some((3, String::new()));
        }
        Some((3, format!("choose {phrase}")))
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
                // "reveals a number of cards from their hand equal to ...": a number the
                // ability remembered first is that number.
                let what = match &what {
                    Sel::Choose {
                        chooser,
                        filter,
                        count: Value::Var(v),
                        up_to: false,
                        store,
                    } => match self.stored_values.iter().find(|(x, _, _)| x == v) {
                        Some((_, value, _)) => Sel::Choose {
                            chooser: chooser.clone(),
                            filter: filter.clone(),
                            count: value.clone(),
                            up_to: false,
                            store: *store,
                        },
                        None => what.clone(),
                    },
                    other => other.clone(),
                };
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
