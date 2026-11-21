//! CR 702.59 Recover. "Recover [cost]" means "When a creature is put into your graveyard
//! from the battlefield, you may pay [cost]. If you do, return this card from your
//! graveyard to your hand. Otherwise, exile this card." (CR 702.59a). The triggered
//! ability functions only while the card is in a graveyard; as a leaves-the-battlefield
//! ability it looks back in time (CR 603.10a), so a creature card with recover doesn't
//! trigger its own recover ability when it dies, nor does a card put into the graveyard
//! at the same time as the creature.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;

pub struct Recover;

impl KeywordRules for Recover {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Recover]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = kw.cost.clone().unwrap_or_default();
        // A creature card goes to its owner's graveyard: "your graveyard" is the
        // graveyard of this card's owner (who controls it there).
        let pay = Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(Effect::Move {
                what: Sel::This,
                to: Destination::zone(ZoneKind::Hand),
            }),
            otherwise: Box::new(Effect::Exile {
                what: Sel::This,
                face_down: false,
                link: false,
            }),
        };
        let mut t = TriggeredAbility::new(
            TriggerCond::Dies(Filter::And(vec![
                Filter::creature(),
                Filter::OwnedBy(PlayerRel::You),
            ])),
            // A card that has left the graveyard is a new object the ability can't find
            // (CR 400.7): there's nothing to return or exile, so nothing to pay for.
            Body::effect(Effect::If {
                cond: Condition::Custom(STILL_IN_GRAVEYARD.into()),
                then: Box::new(pay),
                otherwise: Box::new(Effect::Noop),
            }),
        );
        t.zone = FunctionZone::Graveyard;
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Recover.name(),
        )])
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == STILL_IN_GRAVEYARD).then(|| {
            ctx.source
                .is_some_and(|s| g.is_live(s) && matches!(g.obj(s).zone, Zone::Graveyard(_)))
        })
    }
}

/// `Condition::Custom`: the ability's source is still the card in the graveyard.
const STILL_IN_GRAVEYARD: &str = "recover:this card is still in the graveyard";

inventory::submit! { KeywordRegistration(&Recover) }
