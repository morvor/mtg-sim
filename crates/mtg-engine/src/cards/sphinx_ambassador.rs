//! Sphinx Ambassador: "Whenever ~ deals combat damage to a player, search that player's
//! library for a card, then that player chooses a card name. If you searched for a creature
//! card that doesn't have that name, you may put it onto the battlefield under your
//! control. Then that player shuffles." Any card may be chosen, unrevealed; the player
//! names a card afterwards (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{Entity, ObjectId};

const GUESS: &str = "card:Sphinx Ambassador:search, then they name a card";
const TEXT: &str = "Whenever ~ deals combat damage to a player, search that player's library for a card, then that player chooses a card name. If you searched for a creature card that doesn't have that name, you may put it onto the battlefield under your control. Then that player shuffles.";

inventory::submit! { ManualAbility {
    card: "Sphinx Ambassador",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever ~ deals combat damage to a player, that player shuffles.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(GUESS.into())))
            .collect()
    },
    reason: "a hidden search checked against a card name the opponent chooses afterwards: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != GUESS {
            return false;
        }
        let Some(Entity::Player(them)) = g.eval_sel(&Sel::Players(PlayerRef::TriggerPlayer), ctx).first().copied()
        else {
            return true;
        };
        let you = ctx.controller;
        let src = ctx.source;
        // CR 701.23: search that player's library for a card (any card).
        let lib: Vec<ObjectId> = g.player(them).library.clone();
        let found = g.ask_objects(you, src, "Search their library for a card", lib, 0, 1);
        if crate::search_rules::begin(g, you, them, ctx) {
            g.emit(crate::events::Event::Searched { player: you });
        }
        let named = match g.ask(
            them,
            Decision::NameCard {
                source: src,
                prompt: "Choose a card name".into(),
            },
        ) {
            Answer::Text(t) if crate::choices::valid_card_name(t.trim(), None) => {
                Some(t.trim().to_string())
            }
            _ => None,
        };
        if let Some(c) = found.first().copied() {
            let o = g.obj(c);
            let creature = o.chars.is_creature();
            let has_name = named.as_deref().is_some_and(|n| o.chars.has_name(n));
            if creature
                && !has_name
                && g.obj(c).zone == Zone::Library(them)
                && g.ask_yes_no(you, src, "Put it onto the battlefield under your control?", true)
            {
                g.move_object_ev(MoveEv {
                    obj: c,
                    to: Zone::Battlefield,
                    pos: LibraryPosition::Top,
                    cause: MoveCause::Search,
                    by: Some(you),
                    etb: EtbInfo {
                        controller: Some(you),
                        ..Default::default()
                    },
                    source: src,
                });
            }
        }
        g.shuffle_library(them);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
