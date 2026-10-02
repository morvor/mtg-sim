//! CR 702.175 Offspring: "Offspring [cost]" means "You may pay an additional [cost] as you
//! cast this spell" and "When this permanent enters, if its offspring cost was paid,
//! create a token that's a copy of it, except it's 1/1." (CR 702.175a). The cost is an
//! optional additional cost (CR 601.2b, 601.2f–h).
//!
//! Each instance is paid separately and its trigger looks only at the payment made for it
//! (CR 702.175b): the `i`th offspring ability of a spell (counting from 1) is recorded as
//! `"offspring#i"` in `CastInfo::paid`, and the `i`th "Offspring" triggered ability of the
//! permanent checks that name. The permanent sees how the spell it was cast as was paid
//! for (CR 400.7d); a token copy of it wasn't cast, so its own offspring abilities don't
//! trigger.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Condition::Custom`: the offspring cost of the resolving (or triggering) offspring
/// ability's instance was paid.
pub const PAID: &str = "offspring:its offspring cost was paid";
/// `Filter::Custom` prefix: "with base power N" (CR 208.4b), N following the prefix
/// ("other creatures you control with base power 1", Zinnia, Valley's Voice, which
/// counts the 1/1 offspring tokens).
pub const BASE_POWER: &str = "base power=";

/// The name recorded in `CastInfo::paid` for the payment of the `i`th (0-based) offspring
/// cost of a spell.
pub fn cost_name(i: usize) -> SmolStr {
    SmolStr::new(format!("offspring#{}", i + 1))
}

/// Which of `source`'s offspring triggered abilities (counting from 0) the ability `uid`
/// is. It's the instance that triggered, even if the permanent no longer has it as the
/// ability resolves (it became a copy of something else, or lost its abilities): its
/// offspring cost was paid all the same, so the printed instances are consulted then.
fn instance_index(g: &Game, source: ObjectId, uid: u64) -> Option<usize> {
    let position = |abilities: &mut dyn Iterator<Item = &Ability>| {
        abilities
            .filter(|a| {
                matches!(a.kind, AbilityKind::Triggered(_)) && a.text == KeywordKind::Offspring.name()
            })
            .position(|a| a.uid == uid)
    };
    let o = g.obj(source);
    position(&mut o.chars.abilities.iter()).or_else(|| {
        let printed = crate::keyword_impls::derived_by_keyword(&o.base);
        position(&mut printed.iter().map(|(_, a)| a))
    })
}

pub struct Offspring;

impl KeywordRules for Offspring {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Offspring]
    }

    /// "When this permanent enters, if its offspring cost was paid, create a token that's
    /// a copy of it, except it's 1/1."
    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut t = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::CreateTokenCopy {
                of: Sel::This,
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
                mods: vec![Modification::SetPT(Some(Value::c(1)), Some(Value::c(1)))],
            }),
        );
        t.intervening_if = Some(Condition::Custom(PAID.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Offspring.name(),
        )])
    }

    /// "You may pay an additional [cost] as you cast this spell", once for each instance
    /// (CR 702.175b).
    fn spell_optional_costs(&self, g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
        g.obj(spell)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Offspring)
            .enumerate()
            .map(|(i, k)| (cost_name(i), k.cost.clone().unwrap_or_default(), false))
            .collect()
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let n: i32 = name.strip_prefix(BASE_POWER)?.parse().ok()?;
        Some(super::base_pt::base_pt(g, id).0 == Some(n))
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != PAID {
            return None;
        }
        let Some(i) = ctx.source.and_then(|s| instance_index(g, s, ctx.ability_uid)) else {
            return Some(false);
        };
        let paid = cost_name(i);
        Some(
            g.cast_info(ctx)
                .is_some_and(|c| c.was_cast && c.paid.iter().any(|p| *p == paid)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Offspring) }
