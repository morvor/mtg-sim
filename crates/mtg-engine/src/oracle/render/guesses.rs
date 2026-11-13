//! Guesses (`patterns/choice_grammar_hidden.rs`): "Defending player guesses whether that
//! card's mana value is greater than the number of artifacts you control. If they guessed
//! wrong, ...": the player answers yes or no ([`GUESS`]), and whether the answer was right
//! is decided by the proposition as they guess ([`GUESSED_RIGHT`]).

use super::players::Case;
use super::*;
use crate::oracle::patterns::choice_grammar_hidden::{GUESS, GUESSED_RIGHT};

impl Renderer<'_> {
    /// "[player] guesses whether [proposition]": the yes-or-no choice, whether it was
    /// right, and the event of guessing, as one instruction (how many effects it takes).
    pub(crate) fn guess_seq(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let (opp, rest) = match v {
            [Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Opponent,
            }, rest @ ..] => (true, rest),
            rest => (false, rest),
        };
        let [Effect::ChooseOne { who, options }, Effect::StoreValue { var: right, value }, Effect::Custom(g), ..] =
            rest
        else {
            return None;
        };
        if *right != GUESSED_RIGHT || g != crate::choices::GUESSED {
            return None;
        }
        let yes_no = matches!(options.as_slice(), [(y, Effect::StoreValue { var: a, value: Value::Const(1) }), (n, Effect::StoreValue { var: b, value: Value::Const(0) })]
            if y == "yes" && n == "no" && *a == GUESS && *b == GUESS);
        if !yes_no {
            return None;
        }
        // The proposition (inside the condition that it's about anything at all).
        let value = match value {
            Value::If(_, inner, zero) if matches!(zero.as_ref(), Value::Const(0)) => inner.as_ref(),
            v => v,
        };
        let Value::If(cond, a, _) = value else {
            return None;
        };
        if !matches!(a.as_ref(), Value::Var(x) if *x == GUESS) {
            return None;
        }
        let prop = self.proposition(cond);
        let w = if opp && matches!(who, PlayerRef::ChosenOpponent) {
            "an opponent".to_string()
        } else {
            self.player(who, Case::Subj)
        };
        let used = if opp { 4 } else { 3 };
        // "... whether the top card of your library is the chosen kind. Reveal that card."
        let about_top = prop.starts_with("the top card of your library ");
        if let (
            true,
            Some(Effect::Dig {
                who: PlayerRef::You,
                n: Value::Const(1),
                reveal: true,
                take: Value::Const(0),
                ..
            }),
        ) = (about_top, v.get(used))
        {
            return Some((
                format!("{w} guesses whether {prop}. Reveal {{alt:that card|the top card of your library}}"),
                used + 1,
            ));
        }
        Some((format!("{w} guesses whether {prop}"), used))
    }

    /// What a guess is about: "the top card of your library is the chosen kind" (land or
    /// nonland), or a condition as an "if" clause words it.
    fn proposition(&mut self, c: &Condition) -> String {
        // "a card with that name is in your hand".
        if let Condition::Exists(Filter::And(v)) = c {
            let in_hand = v
                .iter()
                .any(|x| matches!(x, Filter::InZone(ZoneKind::Hand)))
                && v.iter()
                    .any(|x| matches!(x, Filter::OwnedBy(PlayerRel::You)));
            if in_hand {
                let rest = Filter::and(
                    v.iter()
                        .filter(|x| !matches!(x, Filter::InZone(_) | Filter::OwnedBy(_)))
                        .cloned()
                        .collect(),
                );
                let rest = Filter::and(vec![Filter::Card, rest]);
                let n = self.noun_det(&rest, super::nouns::Det::A);
                return format!("{n} is in your hand");
            }
        }
        if let Condition::Or(v) = c {
            if let [Condition::And(a), Condition::And(b)] = v.as_slice() {
                if let (
                    [Condition::Chose(l), Condition::SelMatches(s, _)],
                    [Condition::Chose(n), Condition::SelMatches(s2, _)],
                ) = (a.as_slice(), b.as_slice())
                {
                    if l.as_str() == "land"
                        && n.as_str() == "nonland"
                        && format!("{s:?}") == format!("{s2:?}")
                    {
                        let s = self.sel(s, Case::Subj);
                        return format!("{s} is the chosen kind");
                    }
                }
            }
        }
        self.condition(c)
    }

    /// "they guessed wrong", "the player guessed right".
    pub(crate) fn guessed(&mut self, a: &Value, cmp: Cmp, b: &Value) -> Option<String> {
        let (Value::Var(x), Cmp::Eq, Value::Const(k)) = (a, cmp, b) else {
            return None;
        };
        // "If you don't cast a spell this way, investigate."
        if *x == crate::oracle::patterns::choice_grammar_hidden::CAST_THIS_WAY {
            return Some(if *k == 0 {
                "you don't cast a spell this way".into()
            } else {
                "you cast a spell this way".into()
            });
        }
        if *x != GUESSED_RIGHT {
            return None;
        }
        let r = if *k == 1 { "right" } else { "wrong" };
        Some(format!(
            "{{alt:they|the player|that player|your opponent}} guessed {r}"
        ))
    }
}
