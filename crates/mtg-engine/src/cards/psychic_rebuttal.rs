//! Psychic Rebuttal: "Spell mastery — If there are two or more instant and/or sorcery cards
//! in your graveyard, you may copy the spell countered this way. You may choose new targets
//! for the copy." The copy is of the spell as it last existed on the stack (CR 707.10,
//! 608.2h), only if Psychic Rebuttal countered it (ruling).

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::Entity;

const COPY: &str = "card:Psychic Rebuttal:you may copy the spell countered this way";
const TEXT: &str = "Spell mastery — If there are two or more instant and/or sorcery cards in your graveyard, you may copy the spell countered this way. You may choose new targets for the copy.";

fn copy_instead(e: Effect) -> Effect {
    match e {
        Effect::If { cond, .. } => Effect::If {
            cond,
            then: Box::new(Effect::Custom(COPY.into())),
            otherwise: Box::new(Effect::Noop),
        },
        e => e,
    }
}

inventory::submit! { ManualAbility {
    card: "Psychic Rebuttal",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "If there are two or more instant and/or sorcery cards in your graveyard, draw a card.")
            .iter()
            .map(|a| map_effect(a, TEXT, copy_instead))
            .collect()
    },
    reason: "copies the spell its other ability countered (a spell that has left the stack): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != COPY {
            return false;
        }
        // The spell its first ability targeted (the first target slot of the spell).
        let Some(Entity::Object(spell)) = ctx.targets.first().and_then(|t| t.first()).copied()
        else {
            return true;
        };
        // Countered this way: it was on the stack as Psychic Rebuttal resolved and has
        // left it (a spell that can't be countered is still there).
        let o = g.obj(spell);
        if g.is_live(spell) || o.zone != Zone::Stack {
            return true;
        }
        if !g.ask_yes_no(ctx.controller, ctx.source, "Copy the countered spell?", true) {
            return true;
        }
        let copies: Vec<_> = crate::copy::copy_spell(g, spell, ctx.controller, true)
            .into_iter()
            .collect();
        crate::copy::order_copies(g, ctx.controller, &copies);
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
