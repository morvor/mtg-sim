//! More instructions and instruction sequences worded as a whole.

use super::players::Case;
use super::*;

/// "Choose a number from 0 to N" stored for a following instruction that uses it.
fn chosen_number(e: &Effect) -> bool {
    matches!(
        e,
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Number { min: 0, .. },
        }
    )
}

impl Renderer<'_> {
    /// An instruction worded as a whole, before the general wording.
    pub(crate) fn effect_more(&mut self, e: &Effect) -> Option<String> {
        match e {
            // "Move any number of +1/+1 counters from other permanents you control onto
            // ~": for each of them, a number chosen and that many moved (moving more
            // counters than it has moves all of them).
            Effect::ForEach {
                sel: Sel::All(f),
                var,
                effect,
            } => {
                let Effect::Seq(v) = effect.as_ref() else {
                    return None;
                };
                let [c, Effect::MoveCounters {
                    from: Sel::Var(x),
                    to,
                    kind: Some(k),
                    n: Some(Value::Chosen),
                }] = v.as_slice()
                else {
                    return None;
                };
                if !chosen_number(c) || x != var || matches!(to, Sel::Var(t) if t == var) {
                    return None;
                }
                let from = self.noun(f, Num::Many);
                let t = self.sel(to, Case::Obj);
                Some(format!(
                    "move any number of {} from {from} onto {t}",
                    plural(&counter_name(k))
                ))
            }
            _ => None,
        }
    }

    /// At the start of `v`: a sequence printed as one sentence, as (text, how many
    /// instructions).
    pub(crate) fn seq_more(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        if let Some(s) = self.shuffle_hands_then_draw(v) {
            return Some((s, 3));
        }
        if let Some(r) = self.another_one(v) {
            return Some(r);
        }
        // "Move any number of +1/+1 counters from target creature onto another target
        // creature": a number chosen, then that many moved.
        if let [c, Effect::MoveCounters {
            kind: Some(k),
            n: Some(Value::Chosen),
            from,
            to,
        }, ..] = v
        {
            if chosen_number(c) {
                let f = self.sel(from, Case::Obj);
                let t = self.sel(to, Case::Obj);
                return Some((
                    format!(
                        "move any number of {} from {f} onto {t}",
                        plural(&counter_name(k))
                    ),
                    2,
                ));
            }
        }
        None
    }

    /// "Each player shuffles the cards from their hand into their library, then draws
    /// that many cards": the cards shuffled in are remembered, and each player draws as
    /// many as they own among them (CR 401.2: a card goes to its owner's library).
    fn shuffle_hands_then_draw(&mut self, v: &[Effect]) -> Option<String> {
        let [Effect::ShuffleIntoLibrary {
            what: Sel::All(Filter::And(f)),
            library: PlayerRef::EachPlayer,
        }, Effect::Store {
            var,
            sel: Sel::Var(vars::IT),
        }, Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect,
        }, ..] = v
        else {
            return None;
        };
        if !matches!(
            f.as_slice(),
            [
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(PlayerRel::Any)
            ]
        ) {
            return None;
        }
        let Effect::Draw {
            who: PlayerRef::Iterated,
            n: Value::Count(Filter::And(c)),
        } = effect.as_ref()
        else {
            return None;
        };
        let counted = matches!(c.as_slice(), [Filter::InZone(ZoneKind::Library), Filter::In(s), Filter::OwnedBy(PlayerRel::Iterated)]
            if matches!(s.as_ref(), Sel::Var(x) if x == var));
        counted.then(|| {
            "each player shuffles the cards from their hand into their library, then draws that many cards"
                .to_string()
        })
    }

    /// "Put a +1/+1 counter on ~. Then if ..., put another +1/+1 counter on ~", "Draw a
    /// card. Draw another card if ...": the same instruction again, under a condition.
    fn another_one(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [first, Effect::If {
            then, otherwise, ..
        }, ..] = v
        else {
            return None;
        };
        if !matches!(otherwise.as_ref(), Effect::Noop) {
            return None;
        }
        let again = match (first, then.as_ref()) {
            (
                Effect::AddCounters {
                    what: a,
                    kind: k,
                    n: Value::Const(1),
                },
                Effect::AddCounters {
                    what: b,
                    kind: l,
                    n: Value::Const(1),
                },
            ) if k == l && format!("{a:?}") == format!("{b:?}") => {
                Some(format!("put a {} on", counter_name(k)))
            }
            (
                Effect::Draw {
                    who: a,
                    n: Value::Const(1),
                },
                Effect::Draw {
                    who: b,
                    n: Value::Const(1),
                },
            ) if format!("{a:?}") == format!("{b:?}") => Some("draw a card".to_string()),
            _ => None,
        }?;
        let one = self.effect(first);
        let two = self.effect(&v[1]);
        let another = again.replacen(" a ", " {alt:another|a} ", 1);
        if !two.contains(&again) {
            return None;
        }
        let two = two.replacen(&again, &another, 1);
        Some((format!("{one}. {two}"), 2))
    }

    /// A permanent you control tapped for mana, as "you tap [noun]" (only a permanent's
    /// controller can activate its mana abilities, CR 602.2): the noun without "you
    /// control".
    pub(crate) fn yours_tapped(&mut self, filter: &Filter) -> Option<String> {
        let Filter::And(v) = filter else {
            return None;
        };
        if !v
            .iter()
            .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You)))
        {
            return None;
        }
        let rest = Filter::and(
            v.iter()
                .filter(|x| !matches!(x, Filter::ControlledBy(PlayerRel::You)))
                .cloned()
                .collect(),
        );
        Some(self.noun_det(&rest, super::nouns::Det::A))
    }
}
