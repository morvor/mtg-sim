//! A number chosen once per number and guessed by another player (cards6-per-player-results):
//!
//! - "secretly choose a number between 1 and 5 that hasn't been chosen" (The Toymaker's
//!   Trap): a number of the range not chosen before for this object
//!   ([`ChoiceKind::NumberNotChosen`], `modal_history.rs`), in secret (`choices.rs`). With
//!   every number already chosen nothing is chosen, so "If you do" is false.
//! - "an opponent guesses which number you chose, then you reveal the number you chose":
//!   the player names one of the numbers the choice could have been (the guess is theirs,
//!   a decision of that player); then the secret choice is revealed. "If they guessed wrong" / "If they guessed right"
//!   (`choice_grammar_hidden.rs`) compare the guess with the number chosen, and "the number
//!   they guessed" is the guess.

use super::choice_grammar_hidden::{GUESS, GUESSED_RIGHT};
use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::end;

/// The number a player guessed.
pub const GUESSED_NUMBER: Var = GUESS;

/// The most numbers a guess offers (the range of the choice).
const MAX_RANGE: i32 = 20;

/// "secretly choose a number between 1 and 5 that hasn't been chosen", "choose a number
/// between 1 and 5 that hasn't been chosen".
fn number_that_hasnt_been_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" that hasn't been chosen")?;
    if !(r.starts_with("secretly choose a number ") || r.starts_with("choose a number ")) {
        return None;
    }
    let targets = b.targets.len();
    let mut e = parse_clause(r, b)?;
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    let mut n = 0;
    unchosen(&mut e, &mut n);
    (n == 1).then_some(e)
}

/// Makes the number choices of `e` choices of a number that hasn't been chosen.
fn unchosen(e: &mut Effect, n: &mut usize) {
    match e {
        Effect::Choose {
            who: PlayerRef::You,
            kind: kind @ ChoiceKind::Number { .. },
        } => {
            let ChoiceKind::Number { min, max } = *kind else {
                return;
            };
            *kind = ChoiceKind::NumberNotChosen { min, max };
            *n += 1;
        }
        Effect::Seq(v) => v.iter_mut().for_each(|x| unchosen(x, n)),
        _ => {}
    }
}

inventory::submit! { super::EffectPattern { name: "choices 6: choose a number between N and N that hasn't been chosen", priority: 55, parse: number_that_hasnt_been_chosen } }

/// The range of the number choice `e` ends with, if it's your choice.
fn chosen_range(e: &Effect) -> Option<(i32, i32)> {
    match e {
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Number { min, max } | ChoiceKind::NumberNotChosen { min, max },
        } => Some((*min, *max)),
        Effect::Seq(v) => v.last().and_then(chosen_range),
        _ => None,
    }
}

/// "[if you do, ][player] guesses which number you chose[, then you reveal the number you
/// chose]" after you chose a number.
fn guesses_which_number(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (if_you_do, r) = match l.strip_prefix("if you do, ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let (r, reveal) = match r.strip_suffix(", then you reveal the number you chose") {
        Some(r) => (r, true),
        None => (r, false),
    };
    let Some(who_s) = r.strip_suffix(" guesses which number you chose") else {
        return false;
    };
    let Some((min, max)) = chosen_range(prev) else {
        return false;
    };
    if max < min || max - min >= MAX_RANGE {
        return false;
    }
    let mut steps = Vec::new();
    let who = if who_s == "an opponent" {
        steps.push(Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Opponent,
        });
        PlayerRef::ChosenOpponent
    } else {
        let targets = b.targets.len();
        let Some((who, rest)) = player_ref(who_s, b) else {
            return false;
        };
        if !rest.trim().is_empty()
            || matches!(
                who,
                PlayerRef::You
                    | PlayerRef::EachPlayer
                    | PlayerRef::EachOpponent
                    | PlayerRef::EachOtherPlayer
            )
        {
            b.targets.truncate(targets);
            return false;
        }
        who
    };
    // The guess: one of the numbers the choice could be.
    steps.push(Effect::ChooseOne {
        who: who.clone(),
        options: (min..=max)
            .map(|n| {
                (
                    n.to_string(),
                    Effect::StoreValue {
                        var: GUESSED_NUMBER,
                        value: Value::c(n),
                    },
                )
            })
            .collect(),
    });
    steps.push(Effect::StoreValue {
        var: GUESSED_RIGHT,
        value: Value::If(
            Box::new(Condition::Compare(
                Value::Var(GUESSED_NUMBER),
                Cmp::Eq,
                Value::Chosen,
            )),
            Box::new(Value::c(1)),
            Box::new(Value::c(0)),
        ),
    });
    steps.push(Effect::Custom(crate::choices::GUESSED.into()));
    if reveal {
        steps.push(Effect::Custom(crate::choices::REVEAL_CHOICES.into()));
    }
    b.it_player = who;
    let guess = Effect::seq(steps);
    let guess = if if_you_do {
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(guess),
            otherwise: Box::new(Effect::Noop),
        }
    } else {
        guess
    };
    // Neither "they guessed wrong" nor "they guessed right" without a guess.
    let choice = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::StoreValue {
            var: GUESSED_RIGHT,
            value: Value::c(-1),
        },
        choice,
        guess,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "choices 6: [player] guesses which number you chose", priority: 55, apply: guesses_which_number } }

/// "the number they guessed", "the number that player guessed" (the value grammar's atom),
/// on a card where a player guesses a number.
pub(crate) fn value(s: &str) -> Option<(Value, String)> {
    let rest = ["the number they guessed", "the number that player guessed"]
        .iter()
        .find_map(|p| s.strip_prefix(p))?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains(" guesses which number ") {
        return None;
    }
    Some((Value::Var(GUESSED_NUMBER), rest.to_string()))
}

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn guessed_numbers() {
        for text in [
            "At the beginning of your upkeep, secretly choose a number between 1 and 5 that hasn't been chosen. If you do, an opponent guesses which number you chose, then you reveal the number you chose. If they guessed wrong, they lose life equal to the number they guessed and you draw a card. If they guessed right, sacrifice this enchantment.",
            "At the beginning of your upkeep, secretly choose a number between 1 and 5. Target opponent guesses which number you chose, then you reveal the number you chose. If they guessed right, sacrifice this enchantment.",
            "At the beginning of your upkeep, choose a number between 1 and 3 that hasn't been chosen. If you do, draw a card.",
        ] {
            assert!(compiled("Enchantment", text).is_some(), "{text}");
        }
        for text in [
            // A guess of a number nobody chose.
            "At the beginning of your upkeep, an opponent guesses which number you chose.",
        ] {
            assert!(compiled("Enchantment", text).is_none(), "{text}");
        }
    }
}
