//! Player verbs and filters of older Oracle wordings:
//!
//! - a player as the subject of "destroy", choosing what to destroy among what they
//!   control: "You destroy four lands you control, then target opponent destroys four
//!   lands they control." (Burning of Xinye: chosen as the spell resolves; a regeneration
//!   shield applies, unlike a sacrifice, and a permanent with indestructible can't be
//!   chosen, its rulings);
//! - the player who didn't pay as the subject of a follow-up: "Counter target spell
//!   unless its controller pays {X}. If that player doesn't, they tap all lands with mana
//!   abilities they control and lose all unspent mana." (Power Sink);
//! - "you both own and control" / "you own and control" as a target qualifier (Obelisk of
//!   Undoing).

use super::{FilterSuffixPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;

/// The relation for "[objects] they control" after a player subject.
fn rel_of(p: &PlayerRef) -> Option<PlayerRel> {
    match p {
        PlayerRef::You => Some(PlayerRel::You),
        PlayerRef::Target(n) => Some(PlayerRel::Target(*n)),
        _ => None,
    }
}

/// "you destroy four lands you control", "target opponent destroys four lands they
/// control": that player chooses that many of them as the instruction is performed.
fn player_destroys(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r, poss) = if let Some(r) = l.strip_prefix("you destroy ") {
        (PlayerRef::You, r.to_string(), "you")
    } else {
        let saved = b.targets.len();
        let (who, rest) = player_ref(l, b)?;
        let Some(r) = rest.trim_start().strip_prefix("destroys ") else {
            b.targets.truncate(saved);
            return None;
        };
        if matches!(who, PlayerRef::You) {
            b.targets.truncate(saved);
            return None;
        }
        (who, r.trim_end().to_string(), "they")
    };
    let rel = rel_of(&who)?;
    let r = r.strip_suffix(&format!(" {poss} control"))?;
    let (n, objs) = parse_number(r)?;
    let Value::Const(1..) = n else {
        return None;
    };
    let (f, plural, tail) = parse_object_phrase(objs.trim_start())?;
    if !end(tail).is_empty() || (plural != !matches!(n, Value::Const(1))) {
        return None;
    }
    Some(Effect::Destroy {
        what: Sel::Choose {
            chooser: who,
            filter: Filter::and(vec![
                f,
                Filter::InZone(ZoneKind::Battlefield),
                Filter::ControlledBy(rel),
                // A player can't choose to destroy a permanent with indestructible
                // (Burning of Xinye's ruling; interpretation `destroy-choice-indestructible`).
                Filter::not(Filter::HasKeyword(crate::keywords::KeywordKind::Indestructible)),
            ]),
            count: n,
            up_to: false,
            store: None,
        },
        no_regen: false,
    })
}

inventory::submit! { super::EffectPattern { name: "[player] destroys N [permanents] they control", priority: 300, parse: player_destroys } }

/// Whether the last instruction of an effect is "[instruction] unless [player] pays
/// [cost]", which records whether that player paid.
fn ends_with_unless_paid(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(ends_with_unless_paid),
        Effect::PayOptional { who, then, .. } => {
            matches!(**then, Effect::Noop) && !matches!(who, PlayerRef::You)
        }
        _ => false,
    }
}

/// "If that player doesn't, they [instructions]" after "... unless [player] pays [cost]":
/// what that player does if they didn't pay. "They [verb] ... they control" is the
/// player's own instruction, read as "[verb] ... you control" performed by them.
fn if_that_player_doesnt_they(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l
        .strip_prefix("if that player doesn't, they ")
        .or_else(|| l.strip_prefix("if they don't, they "))
    else {
        return false;
    };
    if r.starts_with("may ") || !ends_with_unless_paid(prev) {
        return false;
    }
    let who = b.it_player.clone();
    if matches!(who, PlayerRef::You)
        || super::oracle_hardening_referents::is_no_player_referent(&who)
    {
        return false;
    }
    // The plural verb form is the imperative; "they"/"their" are that player.
    let words: Vec<&str> = r.split(' ').collect();
    if words.iter().any(|w| matches!(*w, "you" | "your" | "them")) {
        return false;
    }
    let you: Vec<&str> = words
        .iter()
        .map(|w| match *w {
            "they" => "you",
            "their" => "your",
            w => w,
        })
        .collect();
    let text = you.join(" ");
    let saved = (b.it.clone(), b.it_player.clone());
    b.it_player = PlayerRef::You;
    let parsed = parse_clause(&text, b);
    (b.it, b.it_player) = saved;
    let Some(e) = parsed else {
        return false;
    };
    let p = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        p,
        Effect::If {
            cond: Condition::Not(Box::new(Condition::PrevHappened)),
            then: Box::new(Effect::AsPlayer {
                who,
                effect: Box::new(e),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if that player doesn't, they [instructions]", priority: 880, apply: if_that_player_doesnt_they } }

/// "you both own and control", "you own and control": owned and controlled by you.
fn own_and_control<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t
        .strip_prefix("you both own and control")
        .or_else(|| t.strip_prefix("you own and control"))?;
    if !(r.is_empty() || r.starts_with(' ') || r.starts_with(',')) {
        return None;
    }
    Some((
        Filter::and(vec![
            Filter::OwnedBy(PlayerRel::You),
            Filter::ControlledBy(PlayerRel::You),
        ]),
        r,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "you both own and control", priority: 100, parse: own_and_control } }

/// "lose all unspent mana" (you): the mana empties from your mana pool (CR 106.4). "That
/// player loses all unspent mana" is `r106_mana`'s.
fn lose_unspent_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    matches!(end(l), "lose all unspent mana" | "you lose all unspent mana").then_some(
        Effect::LoseUnspentMana {
            who: PlayerRef::You,
            to: None,
        },
    )
}

inventory::submit! { super::EffectPattern { name: "lose all unspent mana", priority: 300, parse: lose_unspent_mana } }

/// "the [type] [player] controls with the greatest mana value", chosen by that player
/// among those tied (Juxtapose: "their controller chooses one of them").
fn greatest_of(noun: &str, who: &PlayerRef, b: &mut Builder) -> Option<Sel> {
    let (sel, rest) =
        crate::oracle::effects::object_ref(&format!("the {noun} you control with the greatest mana value"), b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    let Sel::Choose { .. } = &sel else {
        return None;
    };
    if matches!(who, PlayerRef::You) {
        return Some(sel);
    }
    // The same selection for the other player: their permanents, their choice.
    let PlayerRef::Target(n) = who else {
        return None;
    };
    let mut j = serde_json::to_value(&sel).ok()?;
    fn swap(v: &mut serde_json::Value, n: u8) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, x) in m.iter_mut() {
                    if (k == "ControlledBy" || k == "chooser") && *x == serde_json::json!("You") {
                        *x = if k == "ControlledBy" {
                            serde_json::json!({ "Target": n })
                        } else {
                            serde_json::json!({ "Target": n })
                        };
                    } else {
                        swap(x, n);
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| swap(x, n)),
            _ => {}
        }
    }
    swap(&mut j, *n);
    serde_json::from_value(j).ok()
}

/// The exchange of the greatest-mana-value [noun] each of two players controls.
fn exchange_greatest(noun: &str, other: &PlayerRef, b: &mut Builder) -> Option<Effect> {
    Some(Effect::ExchangeControl {
        a: greatest_of(noun, &PlayerRef::You, b)?,
        b: greatest_of(noun, other, b)?,
    })
}

/// "You and target player exchange control of the creature you each control with the
/// greatest mana value." (Juxtapose): each player's own greatest (ties chosen by that
/// player); nothing is exchanged if either controls none (its rulings).
fn you_and_player_exchange(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("you and ")?;
    let (subj, r) = r.split_once(" exchange control of the ")?;
    let noun = r.strip_suffix(" you each control with the greatest mana value")?;
    let first = b.targets.len();
    let (who, rest) = player_ref(subj, b)?;
    if !rest.trim().is_empty() || !matches!(who, PlayerRef::Target(_)) {
        b.targets.truncate(first);
        return None;
    }
    let e = exchange_greatest(noun, &who, b);
    if e.is_none() {
        b.targets.truncate(first);
    }
    e
}

inventory::submit! { super::EffectPattern { name: "you and target player exchange control of the [type] you each control with the greatest mana value", priority: 300, parse: you_and_player_exchange } }

/// The other player of the last "you and [player] exchange ... greatest" in an effect.
fn last_greatest_exchange(e: &Effect) -> Option<PlayerRef> {
    match e {
        Effect::Seq(v) => v.last().and_then(last_greatest_exchange),
        Effect::ExchangeControl {
            a: Sel::Choose { chooser: PlayerRef::You, .. },
            b: Sel::Choose { chooser: other @ PlayerRef::Target(_), .. },
        } => Some(other.clone()),
        _ => None,
    }
}

/// "Then exchange control of artifacts the same way." after the above.
fn exchange_the_same_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(plural) = l
        .strip_prefix("exchange control of ")
        .and_then(|r| r.strip_suffix(" the same way"))
    else {
        return false;
    };
    let Some(other) = last_greatest_exchange(prev) else {
        return false;
    };
    let noun = match plural {
        "artifacts" => "artifact",
        "creatures" => "creature",
        "lands" => "land",
        "enchantments" => "enchantment",
        _ => return false,
    };
    let Some(e) = exchange_greatest(noun, &other, b) else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "then exchange control of [type] the same way", priority: 60, apply: exchange_the_same_way } }

/// "If two or more permanents a player controls are tied for greatest, their controller
/// chooses one of them." after those exchanges: how they're already chosen.
fn tied_for_greatest(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    end(l) == "if two or more permanents a player controls are tied for greatest, their controller chooses one of them"
        && last_greatest_exchange(prev).is_some()
}

inventory::submit! { FollowupPattern { name: "if two or more permanents a player controls are tied for greatest", priority: 60, apply: tied_for_greatest } }
