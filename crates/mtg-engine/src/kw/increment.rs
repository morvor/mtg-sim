//! CR 702.191 Increment: "Whenever you cast a spell, if this permanent is a creature and
//! the amount of mana spent to cast that spell is greater than this creature's power or
//! this creature's toughness, put a +1/+1 counter on this creature." (CR 702.191a). The
//! condition is an intervening "if" clause (CR 603.4), checked as the spell is cast and
//! again as the ability resolves. Each instance triggers separately (CR 702.191b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// `Value::Custom`: the amount of mana spent to cast the spell that triggered the ability.
const MANA_SPENT_ON_THAT_SPELL: &str = "increment:mana spent to cast that spell";

pub struct Increment;

impl KeywordRules for Increment {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Increment]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let spent = || Value::Custom(MANA_SPENT_ON_THAT_SPELL.into());
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Any,
            },
            Body::effect(Effect::AddCounters {
                what: Sel::This,
                kind: counters::PLUS1.into(),
                n: Value::c(1),
            }),
        );
        t.intervening_if = Some(Condition::And(vec![
            Condition::SelMatches(Sel::This, Filter::Type(CardType::Creature)),
            Condition::Or(vec![
                Condition::Compare(spent(), Cmp::Gt, Value::PowerOf(Box::new(Sel::This))),
                Condition::Compare(spent(), Cmp::Gt, Value::ToughnessOf(Box::new(Sel::This))),
            ]),
        ]));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Increment.name(),
        )])
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != MANA_SPENT_ON_THAT_SPELL {
            return None;
        }
        let spell = ctx.event.as_ref().and_then(|e| e.spell.or(e.object));
        Some(spell.map_or(0, |s| {
            g.obj(s)
                .stack
                .as_deref()
                .map_or(0, |si| si.cast.mana_spent.len() as i64)
        }))
    }
}

inventory::submit! { KeywordRegistration(&Increment) }
