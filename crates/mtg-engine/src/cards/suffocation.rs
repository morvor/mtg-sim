//! Suffocation: "~ deals 4 damage to the controller of the last red instant or sorcery
//! spell that dealt damage to you this turn." The spells that dealt damage to you this
//! turn are found by the compiled trigger condition "a red instant or sorcery spell deals
//! damage to you" over this turn's events (`Sel::ThisTurn`, in the order they dealt it,
//! as they were then); the last one's controller, as it last existed, is dealt the
//! damage. The damage isn't targeted.

use super::ManualAbility;
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::Entity;

const CONTROLLER_OF_LAST: &str =
    "card:Suffocation:the controller of the last of those spells";
const TEXT: &str = "~ deals 4 damage to the controller of the last red instant or sorcery spell that dealt damage to you this turn.";

fn build(_: &crate::oracle::CompileContext) -> Vec<Ability> {
    let Some((cond, _, _)) = crate::oracle::triggers::parse_trigger_condition(
        "whenever a red instant or sorcery spell deals damage to you",
    ) else {
        return vec![AbilityDef::new(
            AbilityKind::Unsupported(TEXT.into()),
            TEXT,
        )];
    };
    // Event by event, not by batch.
    let cond = match cond {
        TriggerCond::Batched { trigger, .. } => *trigger,
        c => c,
    };
    vec![super::spell(
        vec![],
        Effect::Seq(vec![
            Effect::Store {
                var: vars::IT,
                sel: Sel::ThisTurn(Box::new(cond)),
            },
            Effect::Custom(CONTROLLER_OF_LAST.into()),
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::Const(4),
                to: Sel::Var(vars::IT),
            },
        ]),
        TEXT,
    )]
}

inventory::submit! { ManualAbility {
    card: "Suffocation",
    face: 0,
    text: TEXT,
    build,
    reason: "damages the controller of the last spell of a kind that dealt damage to you this turn: no other legal card refers to the last damage source",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CONTROLLER_OF_LAST {
            return false;
        }
        let last = ctx
            .vars
            .get(&vars::IT)
            .and_then(|v| {
                v.iter().rev().find_map(|e| match e {
                    Entity::Object(o) => Some(*o),
                    _ => None,
                })
            })
            .map(|o| g.obj(o).controller);
        ctx.vars
            .insert(vars::IT, last.map(Entity::Player).into_iter().collect());
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
