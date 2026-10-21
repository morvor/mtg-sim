//! Secret choices (hidden information until revealed; see `choices.rs`):
//!
//! - "secretly choose an opponent", "each player secretly chooses a number", "each
//!   opponent secretly chooses silence or snitch": the choice is made without showing it
//!   to the other players ([`crate::choices::SECRET_NEXT`] before the choice);
//! - "then those choices are revealed", "then those numbers are revealed": every secret
//!   choice made for the source becomes public ([`crate::choices::REVEAL_CHOICES`]);
//! - the costs "Reveal the player you chose" and "Reveal the creature type you chose",
//!   which only the player who made the choice can pay (Emissary of Grudges's and A
//!   Killer Among Us's rulings).

use super::{CostPattern, EffectPattern};
use crate::ability::*;
use crate::choices::{REVEAL_CHOICES, REVEAL_CHOSEN_PLAYER, REVEAL_CHOSEN_TYPE, SECRET_NEXT};
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

fn custom(name: &str) -> Effect {
    Effect::Custom(SmolStr::new(name))
}

/// Whether `e` makes a choice for the source.
fn is_choice(e: &Effect) -> bool {
    match e {
        Effect::Choose { .. } => true,
        Effect::Seq(v) => !v.is_empty() && v.iter().all(is_choice),
        _ => false,
    }
}

/// "secretly choose an opponent", "secretly choose a number", "secretly choose 1, 2, or 3".
fn secretly_choose(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("secretly choose ")?;
    let targets = b.targets.len();
    let e = parse_clause(&format!("choose {r}"), b);
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    let e = e.filter(is_choice)?;
    Some(Effect::seq(vec![custom(SECRET_NEXT), e]))
}

inventory::submit! { EffectPattern { name: "secretly choose", priority: 60, parse: secretly_choose } }

/// Makes each choice of a value ([`Effect::Choose`]) in `v` secret.
fn mark_secret(v: serde_json::Value, n: &mut usize) -> serde_json::Value {
    use serde_json::Value as J;
    match v {
        J::Object(m) => {
            let is_choose = m.len() == 1
                && m.get("Choose")
                    .is_some_and(|x| x.get("kind").is_some() && x.get("who").is_some());
            let m: serde_json::Map<String, J> =
                m.into_iter().map(|(k, x)| (k, mark_secret(x, n))).collect();
            if is_choose {
                *n += 1;
                let secret = serde_json::to_value(custom(SECRET_NEXT)).unwrap_or(J::Null);
                let mut seq = serde_json::Map::new();
                seq.insert("Seq".into(), J::Array(vec![secret, J::Object(m)]));
                J::Object(seq)
            } else {
                J::Object(m)
            }
        }
        J::Array(a) => J::Array(a.into_iter().map(|x| mark_secret(x, n)).collect()),
        x => x,
    }
}

/// "each player secretly chooses a number", "each opponent secretly chooses silence or
/// snitch", "you and target opponent each secretly choose 1, 2, or 3": the same
/// instruction without "secretly", each value chosen in secret. (Objects chosen
/// secretly, "each player secretly chooses a creature they control", are chosen as
/// usual: nothing about them is shown until the instruction that reveals them.)
fn subject_secretly_chooses(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let at = l.find(" secretly choose")?;
    if l.starts_with("secretly ") || l[at + 1..].contains(" secretly ") {
        return None;
    }
    let plain = format!("{}{}", &l[..at + 1], &l[at + " secretly ".len()..]);
    // "..., then those choices are revealed": once every player has chosen.
    let (plain, reveal) = match REVEALED
        .iter()
        .find_map(|r| plain.strip_suffix(&format!(", then {r}")))
    {
        Some(p) => (p.to_string(), true),
        None => (plain, false),
    };
    let targets = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&plain, b);
    let Some(e) = e else {
        b.targets.truncate(targets);
        return None;
    };
    let mut n = 0;
    let v = mark_secret(serde_json::to_value(&e).ok()?, &mut n);
    let e = if n == 0 {
        e
    } else {
        serde_json::from_value(v).ok()?
    };
    Some(if reveal {
        Effect::seq(vec![e, custom(REVEAL_CHOICES)])
    } else {
        e
    })
}

/// The ways the text says the secret choices are revealed.
const REVEALED: [&str; 5] = [
    "those choices are revealed",
    "the choices are revealed",
    "those numbers are revealed",
    "the numbers are revealed",
    // Wheel of Misfortune.
    "all players reveal those numbers simultaneously and determine the highest and lowest numbers revealed this way",
];

inventory::submit! { EffectPattern { name: "[players] secretly choose", priority: 60, parse: subject_secretly_chooses } }

/// "those choices are revealed", "the choices are revealed", "those numbers are revealed".
fn choices_revealed(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    REVEALED.contains(&l).then(|| custom(REVEAL_CHOICES))
}

inventory::submit! { EffectPattern { name: "secret choices are revealed", priority: 60, parse: choices_revealed } }

/// "reveal the player you chose", "reveal the creature type you chose".
fn reveal_chosen_cost(s: &str) -> Option<CostPart> {
    let name = match end(s) {
        "reveal the player you chose" => REVEAL_CHOSEN_PLAYER,
        "reveal the creature type you chose" => REVEAL_CHOSEN_TYPE,
        _ => return None,
    };
    Some(CostPart::Effect(Box::new(custom(name))))
}

inventory::submit! { CostPattern { name: "reveal the secret choice", priority: 60, parse: reveal_chosen_cost } }

// ---------------------------------------------------------------------------
// Guesses
// ---------------------------------------------------------------------------

/// What a player guessed (1: yes, 0: no) and whether the guess was right (1) or wrong
/// (0), for the instructions after "[player] guesses whether [proposition]".
pub const GUESS: Var = vars::USER + 30523;
pub const GUESSED_RIGHT: Var = vars::USER + 30524;

/// The condition a proposition states, read the way a leading "if" reads it (with the
/// text's referents: "that card's mana value is greater than the number of artifacts you
/// control"), and the condition for it to be about anything ("the top card of your
/// library" with an empty library: no guess is right, Gollum's rulings).
fn proposition(p: &str, b: &mut Builder) -> Option<(Condition, Option<Condition>)> {
    if let Some(x) = p.strip_suffix(" is the chosen kind") {
        let land = plain_proposition(&format!("{x} is a land card"), b)?;
        let nonland = plain_proposition(&format!("{x} is a nonland card"), b)?;
        let defined = match (x, &land) {
            ("the top card of your library", Condition::SelMatches(s, _)) => {
                Some(Condition::SelNonEmpty(s.clone()))
            }
            _ => None,
        };
        let cond = Condition::Or(vec![
            Condition::And(vec![Condition::Chose("land".into()), land]),
            Condition::And(vec![Condition::Chose("nonland".into()), nonland]),
        ]);
        return Some((cond, defined));
    }
    Some((plain_proposition(p, b)?, None))
}

fn plain_proposition(p: &str, b: &mut Builder) -> Option<Condition> {
    // "a card with that name is in your hand" (the name chosen before).
    if p == "a card with that name is in your hand" {
        return Some(Condition::Exists(Filter::and(vec![
            Filter::ChosenName,
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::You),
        ])));
    }
    let p = p.replace("the card's ", "that card's ");
    let targets = b.targets.len();
    let e = crate::oracle::effects::parse_sentence(&format!("if {p}, you gain 1 life"), b);
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    match e? {
        Effect::If {
            cond,
            then,
            otherwise,
        } if matches!(*then, Effect::GainLife { .. }) && matches!(*otherwise, Effect::Noop) => {
            Some(cond)
        }
        _ => None,
    }
}

/// "defending player guesses whether that card's mana value is greater than the number of
/// artifacts you control", "an opponent guesses whether the top card of your library is
/// the chosen kind", "target opponent guesses whether a card with that name is in your
/// hand": the player answers yes or no; whether they guessed right is decided by the
/// proposition as they guess.
fn guesses_whether(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let at = l.find(" guesses whether ")?;
    let (who_s, prop) = (&l[..at], &l[at + " guesses whether ".len()..]);
    let targets = b.targets.len();
    let mut pre = Vec::new();
    let who = if who_s == "an opponent" {
        pre.push(Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Opponent,
        });
        PlayerRef::ChosenOpponent
    } else {
        let (who, rest) = crate::oracle::effects::player_ref(who_s, b)?;
        if !rest.trim().is_empty()
            || matches!(
                who,
                PlayerRef::You | PlayerRef::EachPlayer | PlayerRef::EachOpponent
            )
        {
            b.targets.truncate(targets);
            return None;
        }
        who
    };
    let Some((cond, defined)) = proposition(prop, b) else {
        b.targets.truncate(targets);
        return None;
    };
    b.it_player = who.clone();
    let store = |n| Effect::StoreValue {
        var: GUESS,
        value: Value::c(n),
    };
    pre.push(Effect::ChooseOne {
        who,
        options: vec![("yes".into(), store(1)), ("no".into(), store(0))],
    });
    let mut right = Value::If(
        Box::new(cond),
        Box::new(Value::Var(GUESS)),
        Box::new(Value::Diff(Box::new(Value::c(1)), Box::new(Value::Var(GUESS)))),
    );
    if let Some(d) = defined {
        right = Value::If(Box::new(d), Box::new(right), Box::new(Value::c(0)));
    }
    pre.push(Effect::StoreValue {
        var: GUESSED_RIGHT,
        value: right,
    });
    pre.push(custom(crate::choices::GUESSED));
    Some(Effect::seq(pre))
}

inventory::submit! { EffectPattern { name: "[player] guesses whether [proposition]", priority: 60, parse: guesses_whether } }

/// "they guessed wrong", "the player guessed right", "your opponent guessed wrong".
fn guessed(c: &str) -> Option<Condition> {
    let r = ["they ", "the player ", "your opponent ", "that player "]
        .iter()
        .find_map(|p| c.strip_prefix(p))?;
    let right = match r {
        "guessed right" => 1,
        "guessed wrong" => 0,
        _ => return None,
    };
    Some(Condition::Compare(
        Value::Var(GUESSED_RIGHT),
        Cmp::Eq,
        Value::c(right),
    ))
}

/// "If they guessed wrong, [instruction]", "If you do and your opponent guessed wrong,
/// [instruction]".
fn if_guessed(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (c, rest) = r.split_once(", ")?;
    let (prev, c) = match c.strip_prefix("you do and ") {
        Some(c) => (true, c),
        None => (false, c),
    };
    let g = guessed(c)?;
    let cond = if prev {
        Condition::And(vec![Condition::PrevHappened, g])
    } else {
        g
    };
    let rest = rest.replace("the card", "it");
    let targets = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_effect_text(&rest, b) else {
        b.targets.truncate(targets);
        return None;
    };
    Some(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if they guessed right/wrong", priority: 60, parse: if_guessed } }

/// Whether a spell was cast by the instruction before "If you don't cast a spell this
/// way".
const CAST_THIS_WAY: Var = vars::USER + 30525;

/// Puts a record of whether it cast a spell after each [`Effect::CastCard`] in `e`.
fn record_casts(e: &mut Effect) -> usize {
    match e {
        Effect::CastCard { .. } => {
            let cast = std::mem::replace(e, Effect::Noop);
            *e = Effect::seq(vec![
                cast,
                Effect::StoreValue {
                    var: CAST_THIS_WAY,
                    value: Value::If(
                        Box::new(Condition::PrevHappened),
                        Box::new(Value::c(1)),
                        Box::new(Value::c(0)),
                    ),
                },
            ]);
            1
        }
        Effect::Seq(v) => v.iter_mut().map(record_casts).sum(),
        Effect::If { then, otherwise, .. } => record_casts(then) + record_casts(otherwise),
        _ => 0,
    }
}

/// "If you don't cast a spell this way, [instruction]" after an instruction that may cast
/// a spell.
fn if_you_dont_cast_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you don't cast a spell this way, ") else {
        return false;
    };
    let mut first = prev.clone();
    if record_casts(&mut first) == 0 {
        return false;
    }
    let targets = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_effect_text(r, b) else {
        b.targets.truncate(targets);
        return false;
    };
    *prev = Effect::seq(vec![
        Effect::StoreValue {
            var: CAST_THIS_WAY,
            value: Value::c(0),
        },
        first,
        Effect::If {
            cond: Condition::Compare(Value::Var(CAST_THIS_WAY), Cmp::Eq, Value::c(0)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "if you don't cast a spell this way", priority: 60, apply: if_you_dont_cast_this_way } }
