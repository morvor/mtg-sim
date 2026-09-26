//! CR 701.48: learn.
//!
//! "Learn" means "You may discard a card. If you do, draw a card. If you didn't discard a
//! card, you may reveal a Lesson card you own from outside the game and put it into your
//! hand" (CR 701.48a). Cards outside the game are those in the player's sideboard
//! (CR 400.11b).

use super::*;
use crate::events::MoveCause;

/// `Event::Custom` name reported when a player learns.
pub const LEARNED: &str = "learn";

/// What a player did when they learned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Learned {
    Rummaged,
    Lesson(ObjectId),
    Nothing,
}

fn lessons(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.player(p)
        .sideboard
        .iter()
        .copied()
        .filter(|c| g.obj(*c).is_card() && g.obj(*c).chars.has_subtype("Lesson"))
        .collect()
}

/// `p` learns (CR 701.48a).
pub fn learn(g: &mut Game, p: PlayerId, source: Option<ObjectId>) -> Learned {
    let can_discard = !g.player(p).hand.is_empty();
    let lessons = lessons(g, p);
    let mut options = vec!["Do nothing".to_string()];
    if can_discard {
        options.push("Discard a card, then draw a card".into());
    }
    if !lessons.is_empty() {
        options.push("Reveal a Lesson card from outside the game and put it into your hand".into());
    }
    let pick = g.ask_option(p, source, "Learn", options.clone());
    let result = match options.get(pick).map(String::as_str) {
        Some(s) if s.starts_with("Discard") => {
            let hand = g.player(p).hand.clone();
            let card = g
                .ask_objects(p, source, "Learn: discard a card", hand.clone(), 1, 1)
                .first()
                .copied()
                .unwrap_or(hand[0]);
            if g.discard(p, card, source).is_some() {
                g.draw_cards(p, 1);
                Learned::Rummaged
            } else {
                Learned::Nothing
            }
        }
        Some(s) if s.starts_with("Reveal") => {
            let card = g
                .ask_objects(
                    p,
                    source,
                    "Learn: choose a Lesson card",
                    lessons.clone(),
                    1,
                    1,
                )
                .first()
                .copied()
                .unwrap_or(lessons[0]);
            g.log(|g| format!("{p} reveals {} from outside the game", g.describe(card)));
            match g.move_object(card, Zone::Hand(p), MoveCause::Effect, Some(p)) {
                Some(new) => Learned::Lesson(new),
                None => Learned::Nothing,
            }
        }
        _ => Learned::Nothing,
    };
    emit(g, LEARNED, p, None, 0);
    result
}

pub struct Learn;

impl KeywordActionRules for Learn {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Learn]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        for p in g.eval_players(a.who, ctx) {
            learn(g, p, ctx.source);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Learn) }
