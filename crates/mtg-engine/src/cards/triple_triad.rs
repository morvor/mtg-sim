//! Triple Triad: "At the beginning of your upkeep, each player exiles the top card of their
//! library. Until end of turn, you may play the card you own exiled this way and each other
//! card exiled this way with lesser mana value than it without paying their mana costs."
//! Timing rules still apply; X is 0 in mana values (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{Entity, ObjectId};

const TRIAD: &str = "card:Triple Triad:play your card and those of lesser mana value";
const TEXT: &str = "At the beginning of your upkeep, each player exiles the top card of their library. Until end of turn, you may play the card you own exiled this way and each other card exiled this way with lesser mana value than it without paying their mana costs.";

inventory::submit! { ManualAbility {
    card: "Triple Triad",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of your upkeep, each player exiles the top card of their library.")
            .iter()
            .map(|a| map_effect(a, TEXT, |e| Effect::Seq(vec![e, Effect::Custom(TRIAD.into())])))
            .collect()
    },
    reason: "free plays of your exiled card and the others with lesser mana value: only this card",
} }

struct Rules;

fn mv(g: &Game, c: ObjectId) -> u32 {
    g.obj(c)
        .chars
        .mana_cost
        .as_ref()
        .map_or(0, |m| m.mana_value_with_x(0))
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != TRIAD {
            return false;
        }
        let you = ctx.controller;
        let exiled: Vec<ObjectId> = ctx
            .vars
            .get(&vars::IT)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(Entity::object)
            .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Exile)
            .collect();
        let Some(mine) = exiled.iter().copied().find(|c| g.obj(*c).owner == you) else {
            return true;
        };
        let top = mv(g, mine);
        let mut playable = vec![mine];
        playable.extend(exiled.into_iter().filter(|c| *c != mine && mv(g, *c) < top));
        crate::casting::grant_play_permission(g, you, playable, Duration::EndOfTurn, true, ctx.source);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
