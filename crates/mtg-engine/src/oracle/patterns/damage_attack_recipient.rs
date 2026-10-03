//! "... deals N damage to the player or planeswalker it's attacking" (Hellrider, Scorch
//! Spitter, Mage Slayer, Fathom Fleet Swordjack, Rakdos Roustabout), "... to the player or
//! planeswalker that creature is attacking" (Raid Bombardment, Cavalcade of Calamity): the
//! player, planeswalker or battle the attacking creature is attacking (CR 506.2), not a
//! target. See `kw::attack_recipient`.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The variable holding the recipient.
const RECIPIENT: Var = u16::MAX - 4211;

/// The phrases naming what an attacking creature is attacking, and whether the creature
/// is the source itself ("~") rather than the pronoun's referent.
const PHRASES: &[(&str, bool)] = &[
    ("the player or planeswalker it's attacking", false),
    ("the player or planeswalker it is attacking", false),
    (
        "the player or planeswalker that creature is attacking",
        false,
    ),
    ("the player or planeswalker ~ is attacking", true),
];

fn replace_players(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
    n: &mut usize,
) -> serde_json::Value {
    use serde_json::Value as J;
    if v == *from {
        *n += 1;
        return to.clone();
    }
    match v {
        J::Array(a) => J::Array(
            a.into_iter()
                .map(|x| replace_players(x, from, to, n))
                .collect(),
        ),
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, x)| (k, replace_players(x, from, to, n)))
                .collect(),
        ),
        x => x,
    }
}

fn damage_to_attack_recipient(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l.contains("defending player") {
        return None;
    }
    let &(phrase, is_source) = PHRASES.iter().find(|(p, _)| l.contains(p))?;
    let attacker = if is_source {
        Sel::This
    } else if b.in_trigger {
        b.it.clone()
    } else {
        return None;
    };
    let text = l.replacen(phrase, "defending player", 1);
    let saved = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&text, b)
        .or_else(|| crate::oracle::effects::parse_sentence(&text, b));
    let Some(e) = e else {
        b.targets.truncate(saved);
        return None;
    };
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    let from = serde_json::to_value(Sel::Players(PlayerRef::DefendingPlayer)).ok()?;
    let to = serde_json::to_value(Sel::Var(RECIPIENT)).ok()?;
    let mut n = 0;
    let json = replace_players(serde_json::to_value(&e).ok()?, &from, &to, &mut n);
    if n == 0 {
        return None;
    }
    let e: Effect = serde_json::from_value(json).ok()?;
    let store = crate::kw::attack_recipient::attack_recipient_effect(RECIPIENT, &attacker)?;
    Some(Effect::Seq(vec![store, e]))
}

inventory::submit! { EffectPattern { name: "damage to the player or planeswalker it's attacking", priority: 120, parse: damage_to_attack_recipient } }
