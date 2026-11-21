//! Sylvan Library: "At the beginning of your draw step, you may draw two additional cards.
//! If you do, choose two cards in your hand drawn this turn. For each of those cards, pay 4
//! life or put the card on top of your library." Any cards drawn this turn count (the normal
//! draw too); with only one drawn, that one is chosen (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::ObjectId;

const PAY_OR_PUT_BACK: &str = "card:Sylvan Library:pay 4 life or put back two cards drawn this turn";
const TEXT: &str = "At the beginning of your draw step, you may draw two additional cards. If you do, choose two cards in your hand drawn this turn. For each of those cards, pay 4 life or put the card on top of your library.";

fn then_choose(e: Effect) -> Effect {
    match e {
        Effect::Seq(v) => Effect::Seq(
            v.into_iter()
                .map(|e| match e {
                    Effect::If { cond, otherwise, .. } => Effect::If {
                        cond,
                        then: Box::new(Effect::Custom(PAY_OR_PUT_BACK.into())),
                        otherwise,
                    },
                    e => e,
                })
                .collect(),
        ),
        e => e,
    }
}

inventory::submit! { ManualAbility {
    card: "Sylvan Library",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of your draw step, you may draw two additional cards. If you do, you gain 1 life.")
            .iter()
            .map(|a| map_effect(a, TEXT, then_choose))
            .collect()
    },
    reason: "chooses cards in hand drawn this turn, each paid for with 4 life or put back: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PAY_OR_PUT_BACK {
            return false;
        }
        let you = ctx.controller;
        let src = ctx.source;
        let drawn: Vec<ObjectId> = g
            .turn_events
            .iter()
            .chain(g.events.iter())
            .filter_map(|e| match e {
                Event::Drew { player, card, .. } if *player == you => Some(*card),
                _ => None,
            })
            .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Hand(you))
            .collect();
        let n = drawn.len().min(2) as u32;
        let chosen = g.ask_objects(you, src, "Choose two cards drawn this turn", drawn, n, n);
        let mut back = Vec::new();
        for c in chosen {
            let opts = vec!["Put it on top of your library".to_string(), "Pay 4 life".to_string()];
            let prompt = format!("{}: put it back or pay 4 life?", g.describe(c));
            let pay = g.can_pay_life(you, 4)
                && g.ask_option(you, Some(c), &prompt, opts) == 1
                && g.pay_life(you, 4);
            if !pay {
                back.push(c);
            }
        }
        for c in back {
            g.move_object_ev(MoveEv {
                obj: c,
                to: Zone::Library(you),
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(you),
                etb: EtbInfo::default(),
                source: src,
            });
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
