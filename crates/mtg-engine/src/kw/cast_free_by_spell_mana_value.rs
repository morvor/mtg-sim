//! Casting a card without paying its mana cost as a "whenever [a player] casts a spell"
//! ability resolves (CR 608.2g), if the spell it would become has a mana value compared
//! with that of the spell that triggered the ability — as that spell last existed on the
//! stack, with the X chosen for it (CR 202.3e):
//!
//! * "Whenever an opponent casts a spell, you may reveal the top card of your library. If
//!   you do, you may cast that card without paying its mana cost if the two spells have
//!   the same mana value." (Powerbalance): [`CAST_IT_FREE_IF_SAME_MANA_VALUE`], for the
//!   card named by "that card" (`vars::IT`).
//! * "Whenever you cast a Kraken, ... spell from your hand, look at the top X cards of
//!   your library, where X is that spell's mana value. You may cast a spell with mana
//!   value less than X from among them without paying its mana cost." (Kiora, Sovereign
//!   of the Deep): [`CAST_ONE_LOOKED_AT_FREE_WITH_LESSER_MANA_VALUE`], for one of the
//!   cards looked at (`vars::REVEALED`) still in the library.
//!
//! The card is cast for the alternative cost of nothing (CR 118.9; X is 0, CR 107.3b),
//! ignoring timing permissions. The spell it would become is what's compared (CR 601.3e):
//! one half of a split card by its own mana value (CR 709.3a), a prototype card cast as a
//! prototyped spell by its prototype mana cost (CR 718.3a).
//!
//! The oracle phrases are parsed in `oracle/patterns/r601_cast_free_by_spell_mana_value.rs`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{vars, Sel, Value};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::{Entity, ObjectId};

/// The `Effect::Custom` name of "you may cast that card without paying its mana cost if
/// the two spells have the same mana value".
pub const CAST_IT_FREE_IF_SAME_MANA_VALUE: &str =
    "cast it without paying its mana cost if its spell has the triggering spell's mana value";

/// The `Effect::Custom` name of "you may cast a spell with mana value less than X from
/// among them without paying its mana cost", X being the triggering spell's mana value.
pub const CAST_ONE_LOOKED_AT_FREE_WITH_LESSER_MANA_VALUE: &str =
    "cast one of the cards looked at without paying its mana cost if its spell has lesser mana value than the triggering spell";

pub struct CastFreeBySpellManaValue;

impl KeywordRules for CastFreeBySpellManaValue {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let (cards, same): (Vec<ObjectId>, bool) = match name {
            CAST_IT_FREE_IF_SAME_MANA_VALUE => (objects(ctx, vars::IT), true),
            CAST_ONE_LOOKED_AT_FREE_WITH_LESSER_MANA_VALUE => {
                let p = ctx.controller;
                let looked = objects(ctx, vars::REVEALED)
                    .into_iter()
                    .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Library(p))
                    .collect();
                (looked, false)
            }
            _ => return false,
        };
        ctx.prev_happened = false;
        let mv = g.eval_value(&Value::ManaValueOf(Box::new(Sel::TriggerSpell)), ctx);
        let Ok(mv) = u32::try_from(mv) else {
            return true;
        };
        let ok = |v: u32| if same { v == mv } else { v < mv };
        cast_one_free(g, ctx, cards, ok, same);
        true
    }
}

fn objects(ctx: &Ctx, var: crate::ability::Var) -> Vec<ObjectId> {
    ctx.vars
        .get(&var)
        .map(|v| v.iter().filter_map(|e| e.object()).collect())
        .unwrap_or_default()
}

/// The controller may cast one of `cards` without paying its mana cost as a spell whose
/// mana value satisfies `ok`: asked yes or no for a single card (`ask_yes_no`), or asked
/// to choose one or none of several.
fn cast_one_free(
    g: &mut Game,
    ctx: &mut Ctx,
    cards: Vec<ObjectId>,
    ok: impl Fn(u32) -> bool + Copy,
    ask_yes_no: bool,
) {
    let p = ctx.controller;
    // CR 702.61a: no spell can be cast while a spell with split second is on the stack.
    if g.split_second_on_stack() {
        return;
    }
    g.recompute();
    let mut options = Vec::new();
    for card in cards.into_iter().filter(|c| g.is_live(*c)) {
        for (label, opt) in crate::casting::free_cast_options(g, p, card, ok) {
            options.push((card, label, opt));
        }
    }
    let mut castable: Vec<ObjectId> = Vec::new();
    for (card, _, _) in &options {
        if !castable.contains(card) {
            castable.push(*card);
        }
    }
    let card = if ask_yes_no {
        match castable.first() {
            Some(c) if g.ask_yes_no(p, Some(*c), "Cast it without paying its mana cost?", true) => {
                *c
            }
            _ => return,
        }
    } else {
        // Choosing none (the default) is declining: it's "you may".
        let prompt = "Cast one of them without paying its mana cost (or none)";
        match g.ask_objects(p, ctx.source, prompt, castable, 0, 1).first() {
            Some(c) => *c,
            None => return,
        }
    };
    let mut ways: Vec<_> = options
        .into_iter()
        .filter(|(c, _, _)| *c == card)
        .map(|(_, label, opt)| (label, opt))
        .collect();
    if ways.is_empty() {
        return;
    }
    let i = if ways.len() > 1 {
        let labels = ways.iter().map(|(n, _)| format!("Cast {n}")).collect();
        g.ask_option(p, Some(card), "Choose what to cast", labels)
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

inventory::submit! { KeywordRegistration(&CastFreeBySpellManaValue) }
