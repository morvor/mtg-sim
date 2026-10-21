//! Memories Returning: "Reveal the top five cards of your library. Put one of them into
//! your hand. Then choose an opponent. They put one on the bottom of your library. Then you
//! put one into your hand. Then they put one on the bottom of your library. Put the other
//! into your hand." (Its flashback compiles.)

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{Entity, ObjectId, PlayerId};

const PICKS: &str = "card:Memories Returning:alternate picks with an opponent";
const TEXT: &str = "Reveal the top five cards of your library. Put one of them into your hand. Then choose an opponent. They put one on the bottom of your library. Then you put one into your hand. Then they put one on the bottom of your library. Put the other into your hand.";

inventory::submit! { ManualAbility {
    card: "Memories Returning",
    face: 0,
    text: TEXT,
    build: |_| vec![spell(vec![], Effect::Custom(PICKS.into()), TEXT)],
    reason: "alternating picks from the revealed cards between you and a chosen opponent: only this card",
} }

struct Rules;

fn put(g: &mut Game, you: PlayerId, by: PlayerId, c: ObjectId, to_hand: bool, src: Option<ObjectId>) {
    if !g.is_live(c) || g.obj(c).zone != Zone::Library(you) {
        return;
    }
    if !to_hand {
        crate::library::put_on_bottom(g, you, &[c]);
        return;
    }
    g.move_object_ev(MoveEv {
        obj: c,
        to: Zone::Hand(you),
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(by),
        etb: EtbInfo::default(),
        source: src,
    });
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PICKS {
            return false;
        }
        let you = ctx.controller;
        let src = ctx.source;
        let mut cards = crate::library::top_cards(g, you, 5);
        crate::reveal::reveal(g, you, &cards, src);
        let opps: Vec<Entity> = g.opponents(you).into_iter().map(Entity::Player).collect();
        let mut opp: Option<PlayerId> = None;
        // You, (choose an opponent), they, you, they; the last card to your hand.
        for (step, to_hand) in [true, false, true, false].into_iter().enumerate() {
            if cards.is_empty() {
                break;
            }
            if step == 1 {
                opp = g
                    .ask_entities(you, src, "Choose an opponent", opps.clone(), 1, 1)
                    .first()
                    .and_then(|e| e.player());
            }
            let who = if to_hand { you } else { opp.unwrap_or(you) };
            let prompt = if to_hand {
                "Put one into your hand"
            } else {
                "Put one on the bottom of their library"
            };
            let c = g
                .ask_objects(who, src, prompt, cards.clone(), 1, 1)
                .first()
                .copied()
                .unwrap_or(cards[0]);
            cards.retain(|x| *x != c);
            put(g, you, who, c, to_hand, src);
        }
        for c in cards {
            put(g, you, you, c, true, src);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
