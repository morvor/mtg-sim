//! CR 702.46 Soulshift: "Soulshift N" means "When this permanent is put into a graveyard
//! from the battlefield, you may return target Spirit card with mana value N or less from
//! your graveyard to your hand." (CR 702.46a). Each instance triggers separately
//! (CR 702.46b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use smol_str::SmolStr;

pub struct Soulshift;

impl KeywordRules for Soulshift {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Soulshift]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0);
        let target = TargetSpec::object(
            Filter::And(vec![
                Filter::Card,
                Filter::Subtype(SmolStr::new("Spirit")),
                Filter::ManaValue(Cmp::Le, Box::new(Value::c(n))),
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
            ]),
            format!("target Spirit card with mana value {n} or less from your graveyard"),
        );
        let effect = Effect::May {
            who: PlayerRef::You,
            effect: Box::new(Effect::Move {
                what: Sel::Target(0),
                to: Destination::zone(ZoneKind::Hand),
            }),
        };
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                // "Put into a graveyard from the battlefield" (CR 700.4).
                TriggerCond::Dies(Filter::Source),
                Body::simple(vec![target], effect),
            )),
            format!("Soulshift {n}"),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Soulshift) }
