//! Information about a card an instruction just put onto the battlefield, read from the
//! card as it last existed in the zone it left, not from the permanent it became (CR 400.7):
//! a Clone that enters as a copy of a 2-mana creature still has the mana value of 4 it had
//! in the graveyard.
//!
//! - "That card's" mana value, power or toughness: "Put target creature card from a
//!   graveyard onto the battlefield under your control. You lose life equal to that card's
//!   mana value." (Reanimate; its ruling: "The amount of life you lose is determined by the
//!   mana value of the card in your graveyard, not the creature once it's on the
//!   battlefield.").
//! - "Its" mana value, for a card a target or a choice found in a graveyard: "...return
//!   target creature card from your graveyard to the battlefield. You gain life equal to its
//!   mana value." (Sister Hospitaller; its ruling: "Use the mana value of the card as it last
//!   existed in the graveyard to determine the amount of life you gain."). Only the mana
//!   value: "its power" and "its toughness" after such a return still read the permanent
//!   (no ruling settles them), as "that creature's" does ("You gain life equal to that
//!   creature's toughness.", Cleric Class).
//!
//! CR 608.2h has an effect use an object's last known information once it's "no longer in"
//! the zone "it was expected to be in"; which zone that is for a card the effect itself
//! moved, the CR doesn't say, and this follows the rulings (provisional register entry
//! `returned-card-values-by-wording`).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;

fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        other => other,
    }
}

/// Whether `what` is a card a target or a choice finds in a graveyard ("target creature
/// card from your graveyard", "a creature card from your graveyard").
fn card_from_a_graveyard(what: &Sel, b: &Builder) -> bool {
    let f = match what {
        Sel::Target(i) => match b.targets.get(*i as usize).map(|t| &t.what) {
            Some(TargetKind::Object(f)) => f,
            _ => return false,
        },
        Sel::Choose { filter, .. } => filter,
        _ => return false,
    };
    f.zone() == Some(ZoneKind::Graveyard)
}

fn moved_card_info(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::Move { what, to } = last_instruction(prev) else {
        return false;
    };
    if to.zone != ZoneKind::Battlefield || !matches!(b.it, Sel::Var(vars::IT)) {
        return false;
    }
    let from_a_graveyard = card_from_a_graveyard(what, b);
    let l = crate::oracle::phrases::end(l);
    let (head, stat) = if let Some((head, stat)) = l.split_once(" equal to that card's ") {
        if !matches!(stat, "mana value" | "power" | "toughness") {
            return false;
        }
        (head, stat)
    } else if let Some((head, stat)) = l.split_once(" equal to its ") {
        if stat != "mana value" || !from_a_graveyard {
            return false;
        }
        (head, stat)
    } else {
        return false;
    };
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

inventory::submit! { FollowupPattern { name: "608.2h: that card's [stat], or its mana value for a card from a graveyard, after it was put onto the battlefield", priority: 80, apply: moved_card_info } }
