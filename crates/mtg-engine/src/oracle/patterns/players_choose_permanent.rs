//! "Each opponent chooses a creature they control. Tap and goad the chosen creatures."
//! (Fell Beast's Shriek), "Target opponent chooses a creature they control. Destroy that
//! creature." (Imperial Edict): as the effect happens, each of those players chooses one
//! permanent they control (not targeted, CR 115.10), in APNAP order (CR 101.4). Later
//! sentences name the chosen permanents ("the chosen creatures", "those creatures", "that
//! creature").
//!
//! Also "tap and goad [objects]" / "untap and goad [objects]" (CR 701.26, 701.15).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

/// The variable holding the permanents chosen this way.
const CHOSEN: Var = vars::USER + 2231;

fn players_choose_permanent(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "For each opponent, choose a creature with the greatest power among creatures that
    // player controls." (Highcliff Felidar): you choose, for each of them.
    let mut you_choose = false;
    let (who, r) = if let Some(r) = l.strip_prefix("for each opponent, choose ") {
        you_choose = true;
        (PlayerRef::EachOpponent, r)
    } else if let Some(r) = l.strip_prefix("each opponent chooses ") {
        (PlayerRef::EachOpponent, r)
    } else if let Some(r) = l.strip_prefix("each player chooses ") {
        (PlayerRef::EachPlayer, r)
    } else if let Some(r) = l.strip_prefix("target opponent chooses ") {
        let text = "target opponent";
        let slot = b.add_target(TargetSpec::player(PlayerFilter::Opponent, text), text);
        (PlayerRef::Target(slot), r)
    } else if let Some(r) = l.strip_prefix("target player chooses ") {
        let text = "target player";
        let slot = b.add_target(TargetSpec::player(PlayerFilter::Any, text), text);
        (PlayerRef::Target(slot), r)
    } else {
        return None;
    };
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (noun, f) = match r.strip_suffix(" they control").filter(|n| !n.contains(' ')) {
        // A single word naming the kind of permanent ("creature", "artifact", "permanent").
        Some(noun) => {
            let (f, false, tail) = parse_object_phrase(noun)? else {
                return None;
            };
            if !end(tail).is_empty() {
                return None;
            }
            (noun, f)
        }
        // "a creature with the greatest mana value among creatures they control", "a
        // permanent they control that shares a card type with the sacrificed permanent"
        // (`filters_relational`).
        None => super::filters_relational::their_object(r, b)?,
    };
    if you_choose && !matches!(noun, "creature" | "permanent") {
        return None;
    }
    if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    let choose = Effect::Store {
        var: CHOSEN,
        sel: Sel::Union(vec![
            Sel::Var(CHOSEN),
            Sel::Choose {
                chooser: if you_choose {
                    PlayerRef::You
                } else {
                    PlayerRef::Iterated
                },
                filter: Filter::and(vec![
                    f,
                    Filter::InZone(ZoneKind::Battlefield),
                    Filter::ControlledBy(PlayerRel::Iterated),
                ]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        ]),
    };
    for name in [
        format!("the chosen {noun}s"),
        format!("the chosen {noun}"),
        format!("those {noun}s"),
        format!("that {noun}"),
    ] {
        b.named.push((name, Sel::Var(CHOSEN)));
    }
    b.it = Sel::Var(CHOSEN);
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN,
            sel: Sel::Union(vec![]),
        },
        Effect::ForEachPlayer {
            who,
            effect: Box::new(choose),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "[players] chooses a [permanent] they control", priority: 120, parse: players_choose_permanent } }

/// "Tap and goad the chosen creatures." / "Untap and goad that creature."
fn tap_and_goad(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (untap, r) = if let Some(r) = l.strip_prefix("tap and goad ") {
        (false, r)
    } else {
        (true, l.strip_prefix("untap and goad ")?)
    };
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let first = if untap {
        Effect::Untap { what: what.clone() }
    } else {
        Effect::Tap { what: what.clone() }
    };
    Some(Effect::seq(vec![
        first,
        Effect::KeywordAction {
            action: KeywordAction::Goad,
            who: PlayerRef::You,
            what,
            n: Value::c(1),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "[un]tap and goad [objects]", priority: 100, parse: tap_and_goad } }
