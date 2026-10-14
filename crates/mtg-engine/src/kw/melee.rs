//! CR 702.121 Melee: "Whenever this creature attacks, it gets +1/+1 until end of turn for
//! each opponent you attacked with a creature this combat." (CR 702.121a). Each instance
//! triggers separately (CR 702.121b).
//!
//! The bonus is determined as the ability resolves, from the attackers declared this
//! combat (`CombatState::declared_attackers`): only opponents (not planeswalkers or
//! battles) count, each once however many creatures attacked them, whether or not those
//! creatures are still attacking or on the battlefield, or the opponent still in the game.
//! Creatures put onto the battlefield attacking weren't declared as attackers: they
//! neither count nor trigger it.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// `Value::Custom`: the number of opponents the ability's controller attacked with a
/// creature this combat.
pub const OPPONENTS_ATTACKED: &str = "melee:opponents you attacked with a creature this combat";

/// The opponents of `p` that `p` attacked with one or more creatures this combat.
pub fn opponents_attacked_this_combat(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    let mut out = Vec::new();
    let Some(c) = g.combat.as_ref() else {
        return out;
    };
    for (id, target) in &c.declared_attackers {
        if let Entity::Player(q) = target {
            if g.obj(*id).controller == p && g.are_opponents(p, *q) && !out.contains(q) {
                out.push(*q);
            }
        }
    }
    out
}

pub struct Melee;

impl KeywordRules for Melee {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Melee]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let n = Value::Custom(OPPONENTS_ATTACKED.into());
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Attacks(Filter::Source),
                Body::effect(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::ModifyPT(n.clone(), n)],
                    duration: Duration::EndOfTurn,
                }),
            )),
            KeywordKind::Melee.name(),
        )])
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        (name == OPPONENTS_ATTACKED)
            .then(|| opponents_attacked_this_combat(g, ctx.controller).len() as i64)
    }
}

inventory::submit! { KeywordRegistration(&Melee) }
