//! Flaming Gambit: "~ deals X damage to target player or planeswalker. That player or that
//! planeswalker's controller may choose a creature they control and have ~ deal that damage
//! to it instead." (Its flashback compiles.)

use super::{map_effect, parse, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{Entity, ObjectId};

const GAMBIT: &str = "card:Flaming Gambit:X damage the player may redirect to a creature they control";
const TEXT: &str = "~ deals X damage to target player or planeswalker. That player or that planeswalker's controller may choose a creature they control and have ~ deal that damage to it instead.";

inventory::submit! { ManualAbility {
    card: "Flaming Gambit",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "~ deals X damage to target player or planeswalker.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(GAMBIT.into())))
            .collect()
    },
    reason: "the damaged player may have the damage dealt to a creature they control instead: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != GAMBIT {
            return false;
        }
        let (Some(src), Some(t)) = (ctx.source, ctx.targets.first().and_then(|v| v.first()).copied())
        else {
            return true;
        };
        let x = ctx.x.max(0) as u32;
        let chooser = match t {
            Entity::Player(p) => p,
            Entity::Object(o) => g.obj(o).controller,
        };
        let creatures: Vec<ObjectId> = g
            .permanents_controlled_by(chooser)
            .into_iter()
            .filter(|o| g.obj(*o).is_creature())
            .collect();
        let mut to = t;
        if !creatures.is_empty()
            && g.ask_yes_no(
                chooser,
                Some(src),
                "Have a creature you control be dealt the damage instead?",
                false,
            )
        {
            if let Some(c) = g
                .ask_objects(chooser, Some(src), "Choose a creature", creatures, 1, 1)
                .first()
            {
                to = Entity::Object(*c);
            }
        }
        if x > 0 {
            g.deal_damage(src, to, x, false);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
