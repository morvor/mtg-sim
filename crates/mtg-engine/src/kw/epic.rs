//! CR 702.50 Epic: two spell abilities, "For the rest of the game, you can't cast spells,"
//! and "At the beginning of each of your upkeeps for the rest of the game, copy this spell
//! except for its epic ability. If the spell has any targets, you may choose new targets
//! for the copy." (CR 702.50a). Copies can still be put onto the stack (CR 702.50b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::{ContinuousEffect, DelayedTrigger, Game, RuleEffect};
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// Epic's spell abilities, performed as the spell resolves.
pub const EPIC: &str = "epic:can't cast spells and copy each upkeep";
/// The delayed triggered ability's effect: copy the epic spell.
pub const EPIC_COPY: &str = "epic:copy the spell";
/// The variable holding the epic spell (as it last existed on the stack).
const EPIC_SPELL: Var = vars::USER + 93;

/// Puts a copy of the spell `lki` (as it last existed on the stack, CR 608.2h) onto the
/// stack under `p`'s control, without its epic ability (CR 702.50a, 707.10).
pub fn copy_without_epic(g: &mut Game, lki: ObjectId, p: PlayerId) -> Option<ObjectId> {
    let has_targets = g
        .obj(lki)
        .stack
        .as_deref()
        .is_some_and(|s| s.chosen.iter().any(|c| c.targets.iter().any(|t| !t.is_empty())));
    let id = crate::copy::copy_spell(g, lki, p, has_targets)?;
    let eid = g.new_effect_id();
    let ts = g.new_timestamp();
    g.effects.push(ContinuousEffect {
        id: eid,
        source: Some(id),
        controller: p,
        timestamp: ts,
        duration: Duration::Permanent,
        affected: crate::game::Affected::Objects(vec![id]),
        mods: vec![Modification::RemoveKeyword(KeywordKind::Epic)],
        layer1: None,
        created_turn: g.turn.number,
    });
    g.dirty = true;
    g.recompute();
    g.log(|g| format!("{p} copies {} (epic)", g.describe(id)));
    Some(id)
}

pub struct Epic;

impl KeywordRules for Epic {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Epic]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Spell(SpellAbility {
                body: Body::effect(Effect::Custom(SmolStr::new(EPIC))),
            }),
            "Epic",
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            EPIC => {
                let p = ctx.controller;
                let Some(spell) = ctx.source else {
                    return true;
                };
                // "For the rest of the game, you can't cast spells."
                let id = g.new_effect_id();
                let ts = g.new_timestamp();
                g.rule_effects.push(RuleEffect {
                    id,
                    source: Some(spell),
                    controller: p,
                    timestamp: ts,
                    duration: Duration::Permanent,
                    restriction: Restriction::CantCast {
                        who: PlayerFilter::Is(p),
                        what: Filter::Any,
                    },
                    objects: None,
                });
                // "At the beginning of each of your upkeeps for the rest of the game, copy
                // this spell except for its epic ability."
                g.flush_events();
                let mut dctx = Ctx::new(Some(spell), p);
                dctx.set_var(EPIC_SPELL, vec![Entity::Object(spell)]);
                let id = g.new_effect_id();
                g.delayed_triggers.push(DelayedTrigger {
                    id,
                    source: Some(spell),
                    controller: p,
                    trigger: TriggerCond::BeginningOf {
                        step: TriggerStep::Upkeep,
                        whose: PlayerRel::You,
                    },
                    body: Body::effect(Effect::Custom(SmolStr::new(EPIC_COPY))),
                    once: false,
                    ctx: dctx,
                    created_turn: g.turn.number,
                    created_step: Some(g.turn.step),
                    for_rest_of_game: true,
                });
                g.dirty = true;
                true
            }
            EPIC_COPY => {
                if let Some(spell) = ctx.var_objects(EPIC_SPELL).first().copied() {
                    copy_without_epic(g, spell, ctx.controller);
                }
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&Epic) }
