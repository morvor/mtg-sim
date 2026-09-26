//! CR 702.123 Fabricate: "When this permanent enters, you may put N +1/+1 counters on it.
//! If you don't, create N 1/1 colorless Servo artifact creature tokens." (CR 702.123a).
//! Each instance triggers separately (CR 702.123b).
//!
//! The choice is made as the ability resolves. If the permanent is no longer on the
//! battlefield then, counters can't be put on it, and the tokens are created.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// `Condition::Custom`: the source of the resolving ability is still on the battlefield
/// (as the same object), so counters can be put on it.
const ON_BATTLEFIELD: &str = "fabricate:this permanent is on the battlefield";

/// A 1/1 colorless Servo artifact creature token.
pub fn servo() -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: vec![Subtype::new("Servo")],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
    }
}

pub struct Fabricate;

impl KeywordRules for Fabricate {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Fabricate]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        let effect = Effect::Seq(vec![
            Effect::If {
                cond: Condition::Custom(ON_BATTLEFIELD.into()),
                then: Box::new(Effect::May {
                    who: PlayerRef::You,
                    effect: Box::new(Effect::AddCounters {
                        what: Sel::This,
                        kind: counters::PLUS1.into(),
                        n: Value::c(n),
                    }),
                }),
                otherwise: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: Condition::Not(Box::new(Condition::PrevHappened)),
                then: Box::new(Effect::CreateToken {
                    spec: servo(),
                    count: Value::c(n),
                    controller: PlayerRef::You,
                    tapped: false,
                    attacking: false,
                }),
                otherwise: Box::new(Effect::Noop),
            },
        ]);
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::EntersBattlefield(Filter::Source),
                Body::effect(effect),
            )),
            format!("Fabricate {n}"),
        )])
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == ON_BATTLEFIELD).then(|| {
            ctx.source
                .is_some_and(|s| g.is_live(s) && g.obj(s).zone == Zone::Battlefield)
        })
    }
}

inventory::submit! { KeywordRegistration(&Fabricate) }
