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
