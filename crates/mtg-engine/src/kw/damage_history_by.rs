//! "Creatures dealt damage by [a particular creature] this turn" (CR 120: the turn's
//! damage history) in an effect created by a resolving ability, after the creature was
//! locked in (`prevention::lock_damage_history`): Runesword's "If a creature dealt damage
//! by the targeted creature would die this turn, exile that creature instead".

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom` prefix: dealt damage this turn by one of the listed objects (ids).
pub const DEALT_DAMAGE_BY: &str = "damage history:dealt damage this turn by:";

/// The `Filter::Custom` name for objects dealt damage this turn by one of `ids`.
pub fn dealt_damage_by(ids: &[ObjectId]) -> String {
    let v: Vec<String> = ids.iter().map(|o| o.0.to_string()).collect();
    format!("{DEALT_DAMAGE_BY}{}", v.join(","))
}

/// `Effect::Custom` prefix: the controller chooses one of the objects a selection of the
/// turn's history names (JSON of the variable to store it in and the selection), such as
/// the spells a player cast this turn, which may have left the stack since: "the damage
/// dealt by one of those sorcery spells this turn" (Backdraft). With several, the choice is
/// made as the effect happens (CR 608.2d).
pub const CHOOSE_ONE_OF: &str = "damage history:choose one of:";

/// The effect choosing one of the objects `sel` names into `var`.
pub fn choose_one_of(var: crate::ability::Var, sel: &crate::ability::Sel) -> Option<crate::ability::Effect> {
    let json = serde_json::to_string(&(var, sel)).ok()?;
    Some(crate::ability::Effect::Custom(
        format!("{CHOOSE_ONE_OF}{json}").into(),
    ))
}

struct DamageHistoryBy;

impl KeywordRules for DamageHistoryBy {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(json) = name.strip_prefix(CHOOSE_ONE_OF) else {
            return false;
        };
        let Ok((var, sel)) =
            serde_json::from_str::<(crate::ability::Var, crate::ability::Sel)>(json)
        else {
            return true;
        };
        let objects: Vec<ObjectId> = g
            .eval_sel(&sel, ctx)
            .into_iter()
            .filter_map(|e| e.object())
            .collect();
        let pick = match objects.len() {
            0 => None,
            1 => Some(objects[0]),
            _ => {
                let labels = objects.iter().map(|o| g.describe(*o)).collect();
                let i = g.ask_option(ctx.controller, ctx.source, "Choose one of them", labels);
                objects.get(i).or(objects.first()).copied()
            }
        };
        ctx.prev_happened = pick.is_some();
        ctx.set_var(var, pick.map(Entity::Object).into_iter().collect());
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let list = name.strip_prefix(DEALT_DAMAGE_BY)?;
        let sources: Vec<u32> = list.split(',').filter_map(|x| x.parse().ok()).collect();
        Some(
            g.history
                .damage_by_source
                .iter()
                .any(|(s, t)| *t == id && sources.contains(&s.0)),
        )
    }
}

inventory::submit! { KeywordRegistration(&DamageHistoryBy) }
