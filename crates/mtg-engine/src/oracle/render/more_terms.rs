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
}
