//! "You may cast that card by paying life equal to the spell's mana value rather than
//! paying its mana cost." (Bismuth Mindrender, after "that player exiles cards from the top
//! of their library until they exile a nonland card"): as the ability resolves, its
//! controller may cast the card (CR 608.2g) for an alternative cost (CR 118.9) of life
//! equal to the mana value of the spell it would become (CR 601.3e; X is 0, CR 107.3b).
//! That's the only way to cast it this way — not for its mana cost nor for another
//! alternative cost (CR 118.9a); additional costs may (or, if mandatory, must) be paid.
//! Life can be paid only by a player whose life total is at least that amount (CR 119.4).
//! The card is `vars::IT` ("you may" is the `Effect::May` around it); the oracle phrase is
//! parsed in
//! `oracle/patterns/cast_it_paying_life.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Cost, CostPart, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::CastMethod;
use crate::types::{Entity, ObjectId};

/// The `Effect::Custom` name.
pub const CAST_IT_PAYING_LIFE: &str = "cast that card paying life equal to its mana value";
/// The `CastMethod::Alternative` id of paying life equal to the spell's mana value.
const PAY_LIFE_METHOD: u64 = 0x11FE_0118_0019;

pub struct CastItPayingLife;

impl KeywordRules for CastItPayingLife {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CAST_IT_PAYING_LIFE {
            return false;
        }
        let cards: Vec<ObjectId> = ctx.var_objects(vars::IT);
        cast_paying_life(g, ctx, cards);
        true
    }
}

fn cast_paying_life(g: &mut Game, ctx: &mut Ctx, cards: Vec<ObjectId>) {
    let p = ctx.controller;
    ctx.prev_happened = false;
    // CR 702.61a: no spell can be cast while a spell with split second is on the stack.
    if g.split_second_on_stack() {
        return;
    }
    g.recompute();
    let Some(card) = cards.into_iter().find(|c| g.is_live(*c)) else {
        return;
    };
    let mut ways: Vec<_> = crate::casting::free_cast_options(g, p, card, |_| true)
        .into_iter()
        .map(|(label, mut opt)| {
            let chars = g.option_characteristics(card, &opt);
            let mv = chars
                .mana_cost
                .as_ref()
                .map_or(0, |m| m.mana_value_with_x(0));
            if opt.method == CastMethod::Free {
                opt.method = CastMethod::Alternative(PAY_LIFE_METHOD);
            }
            opt.alt_cost = Some(Cost::free().with(CostPart::PayLife(Value::c(mv as i32))));
            (label, opt)
        })
        .collect();
    if ways.is_empty() {
        return;
    }
    let i = if ways.len() > 1 {
        let labels = ways
            .iter()
            .map(|(label, _)| format!("Cast {label}"))
            .collect();
        g.ask_option(p, Some(card), "Choose which spell to cast", labels)
    } else {
        0
    };
    let (_, opt) = ways.swap_remove(i.min(ways.len() - 1));
    if let Ok(spell) = g.cast_with_option(p, card, opt) {
        ctx.prev_happened = true;
        // CR 400.7h: other parts of the effect can find the spell cast this way.
        ctx.set_var(vars::IT, vec![Entity::Object(spell)]);
    }
}

inventory::submit! { KeywordRegistration(&CastItPayingLife) }
