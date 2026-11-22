//! Niv-Mizzet Reborn: "When ~ enters, reveal the top ten cards of your library. For each
//! color pair, choose a card that's exactly those colors from among them. Put the chosen
//! cards into your hand and the rest on the bottom of your library in a random order."
//! A color pair is exactly two colors; with fewer pairs among them, as many cards as you
//! can are chosen (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{Color, ColorSet, ObjectId};

const PAIRS: &str = "card:Niv-Mizzet Reborn:a card of each color pair from the top ten";
const TEXT: &str = "When ~ enters, reveal the top ten cards of your library. For each color pair, choose a card that's exactly those colors from among them. Put the chosen cards into your hand and the rest on the bottom of your library in a random order.";

inventory::submit! { ManualAbility {
    card: "Niv-Mizzet Reborn",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ enters, draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(PAIRS.into())))
            .collect()
    },
    reason: "a card of each two-color pair from the revealed cards: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PAIRS {
            return false;
        }
        let you = ctx.controller;
        let src = ctx.source;
        let cards = crate::library::top_cards(g, you, 10);
        crate::reveal::reveal(g, you, &cards, src);
        let mut chosen: Vec<ObjectId> = Vec::new();
        for (i, a) in Color::ALL.iter().enumerate() {
            for b in &Color::ALL[i + 1..] {
                let mut pair = ColorSet::NONE;
                pair.insert(*a);
                pair.insert(*b);
                let cands: Vec<ObjectId> = cards
                    .iter()
                    .copied()
                    .filter(|c| !chosen.contains(c) && g.obj(*c).chars.colors == pair)
                    .collect();
                if let Some(c) = g
                    .ask_objects(you, src, &format!("Choose a {a:?}-{b:?} card"), cands, 1, 1)
                    .first()
                {
                    chosen.push(*c);
                }
            }
        }
        let mut moves: Vec<MoveEv> = chosen
            .iter()
            .map(|c| MoveEv {
                obj: *c,
                to: Zone::Hand(you),
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(you),
                etb: EtbInfo::default(),
                source: src,
            })
            .collect();
        g.move_objects(std::mem::take(&mut moves));
        // The rest on the bottom in a random order.
        let mut rest: Vec<ObjectId> = cards
            .into_iter()
            .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Library(you))
            .collect();
        for i in (1..rest.len()).rev() {
            let j = g.random_range(0, i as u32) as usize;
            rest.swap(i, j);
        }
        crate::library::put_on_bottom(g, you, &rest);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
