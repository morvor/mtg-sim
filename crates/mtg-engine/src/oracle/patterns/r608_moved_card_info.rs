//! Information about a card an instruction just put onto the battlefield, named as "that
//! card": "Put target creature card from a graveyard onto the battlefield under your
//! control. You lose life equal to that card's mana value." (Reanimate). The card was
//! expected in the graveyard and is no longer there, so its last known information there
//! is used (CR 608.2h), not the permanent it became (CR 400.7): a Clone that enters as a
//! copy of a 2-mana creature still costs its mana value of 4.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;

fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        other => other,
    }
}

fn moved_card_info(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let onto_battlefield = matches!(
        last_instruction(prev),
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield
    );
    if !onto_battlefield || !matches!(b.it, Sel::Var(vars::IT)) {
        return false;
    }
    let l = crate::oracle::phrases::end(l);
    let Some((head, stat)) = l.split_once(" equal to that card's ") else {
        return false;
    };
    if !matches!(stat, "mana value" | "power" | "toughness") {
        return false;
    }
    let saved = b.it.clone();
    b.it = Sel::Before(Box::new(Sel::Var(vars::IT)));
    let e = crate::oracle::effects::parse_sentence(&format!("{head} equal to its {stat}"), b);
    b.it = saved;
    let Some(e) = e else {
        return false;
    };
    // Only information: an instruction about the card itself would name the permanent.
    if !matches!(
        e,
        Effect::LoseLife { .. } | Effect::GainLife { .. } | Effect::Draw { .. }
    ) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "608.2h: that card's [stat] after it was put onto the battlefield", priority: 80, apply: moved_card_info } }
