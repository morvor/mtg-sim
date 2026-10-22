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
    /// Where the objects went, which cards may say ("returned to your hand").
    to: Option<&'static str>,
}

/// The outcome verb of an instruction, looking into optional and per-player parts.
fn outcome_verb(e: &Effect) -> Option<OutcomeVerb> {
    let v = |passive, actor| {
        Some(OutcomeVerb {
            passive,
            actor,
            to: None,
        })
    };
    match e {
        Effect::May { effect, .. } => outcome_verb(effect),
        // "Target opponent reveals the top card of their library", "that player exiles
        // that card": done as that player.
        Effect::AsPlayer { who, effect } => {
            let mut o = outcome_verb(effect)?;
            if let Some((p, _, _)) = &mut o.actor {
                if matches!(p, PlayerRef::You) {
                    *p = who.clone();
                }
            }
            Some(o)
        }
        // "exile up to one other target creature until ~ leaves the battlefield".
        Effect::ExileUntil { .. } => v(
            &["exiled", "put into exile"],
            Some((PlayerRef::You, "exiles", "exiled")),
        ),
        // "Each player reveals the top card of their library" (`kw/parley.rs`).
        Effect::Custom(n) if n == crate::kw::parley::REVEAL_TOPS => v(&["revealed"], None),
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
        Effect::Move { to, .. } if to.zone == ZoneKind::Hand => Some(OutcomeVerb {
            passive: &["returned"],
            actor: Some((PlayerRef::You, "returns", "returned")),
            to: Some("to your hand"),
        }),
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

/// What a condition says about the objects of a snapshot.
#[derive(Clone, Debug)]
enum OutcomeCond {
    /// "a land card was milled".
    Some(Filter),
    /// "four or more nontoken permanents you control were returned".
    AtLeast(i32, Filter),
    /// "no creature cards were revealed".
    NoneOf(Filter),
    /// "two nonland cards that share a color were milled" (the qualities, and whether
    /// it's a color rather than a card type they share): two of them have one in common.
    PairSharing(Filter, bool),
}

/// The snapshot a count is about, looking through "as they were before" (`Before`).
fn snapshot_count(v: &Value) -> Option<(Var, Filter)> {
    let Value::CountSel(s) = v else {
        return None;
    };
    let Sel::Matching(inner, f) = s.as_ref() else {
        return None;
    };
    let inner = match inner.as_ref() {
        Sel::Before(x) => x.as_ref(),
        x => x,
    };
    match inner {
        Sel::Var(x) => Some((*x, f.clone())),
        _ => None,
    }
}

/// "Two cards that share a card type": for each card type (each color), two or more of
/// the objects have it.
fn pair_sharing(v: &[Condition]) -> Option<(Var, Filter, bool)> {
    let mut snap = None;
    let mut base: Option<Vec<Filter>> = None;
    let mut colors = Vec::new();
    let mut types = Vec::new();
    for c in v {
        let Condition::Compare(n, Cmp::Ge, two) = c else {
            return None;
        };
        if !matches!(two, Value::Const(2)) {
            return None;
        }
        let (s, f) = snapshot_count(n)?;
        if snap.is_some_and(|x| x != s) {
            return None;
        }
        snap = Some(s);
        let Filter::And(parts) = f else {
            return None;
        };
        let (rest, kind): (Vec<Filter>, Vec<Filter>) = parts
            .into_iter()
            .partition(|x| !matches!(x, Filter::Color(_)) && !matches!(x, Filter::Type(_)));
        let [k] = kind.as_slice() else {
            return None;
        };
        match k {
            Filter::Color(c) => colors.push(*c),
            Filter::Type(t) => types.push(*t),
            _ => return None,
        }
        if base
            .as_ref()
            .is_some_and(|b| format!("{b:?}") != format!("{rest:?}"))
        {
            return None;
        }
        base = Some(rest);
    }
    // Every color, or every card type.
    let by_color = if types.is_empty() && colors == crate::types::Color::ALL {
        true
    } else if colors.is_empty() && types == crate::types::CardType::ALL {
        false
    } else {
        return None;
    };
    Some((snap?, Filter::and(base?), by_color))
}

/// A condition about a snapshot: (the snapshot, what it says).
fn outcome_cond(c: &Condition) -> Option<(Var, OutcomeCond)> {
    if let Some((s, f)) = some_matches(c) {
        return Some((s, OutcomeCond::Some(f)));
    }
    match c {
        // "Destroy target artifact or land. If an artifact is destroyed this way, ...":
        // the snapshot is the one target, if it was destroyed.
        Condition::And(v) => match v.as_slice() {
            [Condition::SelNonEmpty(Sel::Var(s)), Condition::SelMatches(Sel::Target(_), f)] => {
                Some((*s, OutcomeCond::Some(f.clone())))
            }
            _ => None,
        },
        Condition::Compare(n, Cmp::Ge, Value::Const(k)) if *k >= 2 => {
            let (s, f) = snapshot_count(n)?;
            Some((s, OutcomeCond::AtLeast(*k, f)))
        }
        Condition::Compare(n, Cmp::Eq, Value::Const(0)) => {
            let (s, f) = snapshot_count(n)?;
            Some((s, OutcomeCond::NoneOf(f)))
        }
        Condition::Or(v) => {
            let (s, f, by_color) = pair_sharing(v)?;
            Some((s, OutcomeCond::PairSharing(f, by_color)))
        }
        _ => None,
    }
}

/// "If you control a Squirrel or returned a Squirrel card to your hand this way": another
/// condition, or one about a snapshot.
fn or_outcome(c: &Condition) -> Option<(&Condition, Var, OutcomeCond)> {
    let Condition::Or(v) = c else {
        return None;
    };
    let [other, about] = v.as_slice() else {
        return None;
    };
    if outcome_cond(other).is_some() {
        return None;
    }
    let (s, oc) = outcome_cond(about)?;
    Some((other, s, oc))
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
        self.tail_seq_part(v, i)
            .or_else(|| self.attempt(|r| r.remembered_group_part(v, i)))
            .or_else(|| self.block_pair_phase_part(v, i))
            .or_else(|| self.chosen_from_targets_part(v, i))
            .or_else(|| self.any_opponent_sacrifices_part(v, i))
            .or_else(|| self.cast_then_rest_part(v, i))
            .or_else(|| self.look_then_exile_part(v, i))
            .or_else(|| self.named_group_part(v, i))
            .or_else(|| self.distribute_part(v, i))
            .or_else(|| self.each_chooses_part(v, i))
            .or_else(|| self.random_pick_part(v, i))
            .or_else(|| self.reveal_part(v, i))
            .or_else(|| self.attempt(|r| r.opt_in_part(v, i)))
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
        // CR 406.3: a player told to look at a card and then exile it face down may
        // look at it for as long as it remains exiled, the same permission as "exile it
        // face down. You may look at it for as long as it remains exiled".
        let (that, remains) = if pron == "them" {
            ("{alt:those cards|them}", "they remain")
        } else {
            ("{alt:that card|it}", "it remains")
        };
        Some((
            2,
            format!(
                "{{alt:look at {s}, then exile {pron} face down|exile {s} face down and look at {pron}|exile {s} face down. You may look at {that} for as long as {remains} exiled}}"
            ),
        ))
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
                // Only when chosen as that player is "your" that player's.
                let s = if as_player {
                    format!(" {s} ").replace(" your ", " their ")
                } else {
                    format!(" {s} ")
                };
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
        let noun = self.noun_det(&Filter::and(rest.clone()), det);
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
                // "an exiled card used to craft ~ at random": the quality says where.
                (Some(ZoneKind::Exile), None)
                    if rest.iter().any(
                        |x| matches!(x, Filter::Custom(n) if n == crate::kw::craft::USED_TO_CRAFT),
                    ) =>
                {
                    String::new()
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
        use crate::kw::reveal_from_hand::{REVEALED, REVEAL_CHOSEN, REVEAL_CHOSEN_ANY};
        let (Effect::Store { var: REVEALED, sel }, Some(Effect::Custom(c))) = (&v[i], v.get(i + 1))
        else {
            return None;
        };
        if c != REVEAL_CHOSEN && c != REVEAL_CHOSEN_ANY {
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
                        let mut k = self.value(count);
                        // A number remembered before the reveal, performed as the
                        // revealing player: its "you" is still the controller.
                        if self.in_as_player {
                            k = format!(" {k}")
                                .replace(" you", &format!(" {}you", super::KEEP_YOU))
                                .replace(" your", &format!(" {}your", super::KEEP_YOU))
                                .trim_start()
                                .to_string();
                        }
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
        let about = |c: &Condition| {
            outcome_cond(c)
                .map(|(x, _)| x)
                .or_else(|| or_outcome(c).map(|(_, x, _)| x))
        };
        let asked_about = |s: Var, from: usize| {
            v[from..]
                .iter()
                .any(|e| matches!(e, Effect::If { cond, .. } if about(cond) == Some(s)))
        };
        // "You may choose a nonland card from it. If you do, that player exiles that card.
        // If a nonblack card is exiled this way, ...": the snapshot starts empty, and the
        // instruction (done only if the condition held) takes it.
        if let (
            Effect::Store {
                var: s,
                sel: Sel::None,
            },
            Some(Effect::If { then, .. }),
        ) = (&v[i], v.get(i + 1))
        {
            let takes = matches!(then.as_ref(), Effect::Seq(t)
                if matches!(t.last(), Some(Effect::Store { var, sel: Sel::Var(_) }) if var == s));
            if takes && asked_about(*s, i + 2) {
                let verb = outcome_verb(then)?;
                known.push((*s, verb));
                return Some((1, String::new()));
            }
        }
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
                        if about(cond).is_some_and(|x| mentions_snapshot(f, x)))
                });
                next_asks.then(|| (1, String::new()))
            }
            Effect::If {
                cond,
                then,
                otherwise,
            } => {
                let (other, s, oc) = match or_outcome(cond) {
                    Some((o, s, oc)) => (Some(o), s, oc),
                    None => {
                        let (s, oc) = outcome_cond(cond)?;
                        (None, s, oc)
                    }
                };
                let verb = known.iter().find(|(x, _)| *x == s)?.1.clone();
                let c = match other {
                    Some(o) => {
                        // "If you control a Squirrel or returned a Squirrel card to your
                        // hand this way": the subject said once.
                        let elide = matches!(o, Condition::Exists(Filter::And(v))
                            if v.iter().any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You))));
                        let o = self.condition(o);
                        let c = self.outcome_cond_text(&oc, &verb, elide);
                        format!("{o} or {c}")
                    }
                    None => self.outcome_cond_text(&oc, &verb, false),
                };
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

    /// The words of a condition about what an instruction did (without "this way").
    /// `elide_you`: after another condition about you ("you control a Squirrel or
    /// returned a Squirrel card"), the subject "you" may go unsaid.
    fn outcome_cond_text(
        &mut self,
        oc: &OutcomeCond,
        verb: &OutcomeVerb,
        elide_you: bool,
    ) -> String {
        let noun = |r: &mut Self, f: &Filter| match f {
            Filter::Any => "cards".to_string(),
            f => r.noun_det(f, Det::Plural),
        };
        let np = match oc {
            OutcomeCond::Some(f) => return self.outcome_condition(f, verb, elide_you),
            OutcomeCond::AtLeast(k, f) => {
                let n = noun(self, f);
                format!("{} or more {n}", number_word(*k))
            }
            OutcomeCond::NoneOf(f) => {
                let n = noun(self, f);
                format!("no {n}")
            }
            OutcomeCond::PairSharing(f, by_color) => {
                let n = noun(self, f);
                let what = if *by_color { "a color" } else { "a card type" };
                format!("two {n} that share {what}")
            }
        };
        let alts = self.outcome_alts(&np, true, verb, elide_you);
        format!("{{alt:{}}}", alts.join("|"))
    }

    /// "[noun] was milled", "you discarded [noun]", ... for a noun phrase.
    fn outcome_alts(
        &mut self,
        noun: &str,
        plural: bool,
        verb: &OutcomeVerb,
        elide_you: bool,
    ) -> Vec<String> {
        let bes: &[&str] = if plural {
            &["are", "were"]
        } else {
            &["is", "was"]
        };
        let tos: Vec<String> = match verb.to {
            Some(to) => vec![String::new(), format!(" {to}")],
            None => vec![String::new()],
        };
        let mut alts = Vec::new();
        for p in verb.passive {
            for be in bes {
                for to in &tos {
                    alts.push(format!("{noun} {be} {p}{to}"));
                }
            }
        }
        if let Some((who, present, past)) = &verb.actor {
            let subj = self.player(who, Case::Subj);
            for to in &tos {
                if subj == "you" {
                    let base = present.strip_suffix('s').unwrap_or(present);
                    let you = if elide_you { "{opt:you}" } else { "you" };
                    alts.push(format!("{you} {past} {noun}{to}"));
                    alts.push(format!("{you} {base} {noun}{to}"));
                } else if !subj.contains(['{', '|']) && !subj.starts_with("each ") {
                    alts.push(format!("{subj} {present} {noun}{to}"));
                    alts.push(format!("{subj} {past} {noun}{to}"));
                }
            }
        }
        alts
    }

    /// "a land card was milled", "you discarded a card", "that player discards an artifact
    /// card" (without "this way").
    fn outcome_condition(&mut self, f: &Filter, verb: &OutcomeVerb, elide_you: bool) -> String {
        // Cards the player who discarded or milled them owns: "you discarded a card"; a
        // player sacrifices only permanents they control (CR 701.21a): "you sacrificed a
        // creature".
        let sacrificed = verb.passive == ["sacrificed"];
        let yours = |x: &Filter| {
            owned_by_you(x) || (sacrificed && matches!(x, Filter::ControlledBy(PlayerRel::You)))
        };
        let (by_you, f) = match f {
            Filter::And(v) if v.iter().any(yours) => {
                let rest: Vec<Filter> = v.iter().filter(|x| !yours(x)).cloned().collect();
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
        let mut alts = self.outcome_alts(&noun, false, verb, elide_you);
        if let Some(rest) = noun.strip_prefix("a ").or(noun.strip_prefix("an ")) {
            for p in verb.passive {
                for be in ["is", "was"] {
                    alts.push(format!("at least one {rest} {be} {p}"));
                }
            }
        }
        format!("{{alt:{}}}", alts.join("|"))
    }
}
