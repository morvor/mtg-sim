//! Permissions in the body of a delayed or reflexive triggered ability (cards3-cast-zones):
//! "When that creature dies this turn, exile a number of cards from the top of your library
//! equal to its power, then choose a card exiled this way. Until the end of your next turn,
//! you may play that card." (End-Blaze Epiphany), "When you do, target opponent exiles the
//! top five cards of their library. You may cast up to two spells from among the exiled
//! cards without paying their mana costs." (Doom Reigns Supreme).
//!
//! A permission for the cards an earlier instruction moved is about the cards that
//! instruction moves: when that instruction is the last one of a delayed triggered
//! ability's effect (CR 603.7) or of a reflexive one's (CR 603.12), the permission is
//! given (or the spells are cast) when that ability resolves, as part of its effect.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

use super::permission_grammar::{moved, Moved};

/// The body of the delayed or reflexive triggered ability the effect ends with.
fn last_trigger_body(e: &mut Effect) -> Option<&mut Body> {
    match e {
        Effect::Seq(v) => last_trigger_body(v.last_mut()?),
        Effect::DelayedTrigger { body, .. } => Some(body),
        // "When you do, ..." (CR 603.12).
        Effect::If {
            cond,
            then,
            otherwise,
        } if matches!(**otherwise, Effect::Noop)
            && matches!(cond, Condition::PrevHappened) =>
        {
            match &mut **then {
                Effect::Reflexive { body } => Some(body),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Whether the sentence is a permission ("you may play that card", "until end of turn,
/// you may cast ...").
fn is_permission(l: &str) -> bool {
    let s = match l.split_once(", you may ") {
        Some((lead, _)) if lead.starts_with("until ") || lead.starts_with("for as long as ") => {
            &l[lead.len() + 2..]
        }
        _ => l,
    };
    s.starts_with("you may play ") || s.starts_with("you may cast ")
}

fn f_permission_in_trigger_body(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l).trim();
    if !is_permission(l) {
        return false;
    }
    let Some(body) = last_trigger_body(prev) else {
        return false;
    };
    if matches!(moved(&body.effect), Moved::Unchanged) {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::IT);
    sub.it_player = b.it_player.clone();
    sub.sentences = 1;
    let mut effect = body.effect.clone();
    if crate::oracle_ext::apply_followup_ext(l, &mut effect, &mut sub) {
        if !sub.targets.is_empty() {
            return false;
        }
        body.effect = effect;
        return true;
    }
    // A permission of its own for the cards "it" means now.
    let Some(e) = parse_sentence(l, &mut sub).filter(|_| sub.targets.is_empty()) else {
        return false;
    };
    body.effect = Effect::seq(vec![std::mem::take(&mut body.effect), e]);
    true
}

inventory::submit! { FollowupPattern { name: "permissions 4: a permission for the cards a delayed or reflexive triggered ability moves", priority: 130, apply: f_permission_in_trigger_body } }
