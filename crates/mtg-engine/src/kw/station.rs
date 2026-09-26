//! CR 702.184 Station, and station cards (CR 721).
//!
//! * "Station" means "Tap another untapped creature you control: Put a number of charge
//!   counters on this permanent equal to the tapped creature's power. Activate only as a
//!   sorcery." (CR 702.184a). A station card has it at all times (CR 721.4).
//! * A station symbol "{N+} [abilities] [P/T]" is a static ability: as long as the
//!   permanent has N or more charge counters, it has the abilities, and with a P/T box
//!   it's a creature with that base power and toughness in addition to its other types
//!   (CR 721.2a, 721.2b); compiled in `oracle/patterns/r721_station.rs`.
//! * Outside the battlefield a station card has no power or toughness (CR 721.2c): the
//!   P/T box belongs to its highest station symbol's striation (`card.rs`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// The variable holding the creature tapped to pay the station cost (the objects a
/// cost tapped, see `Game::activate_ability`).
const TAPPED: Var = vars::USER + 91;

pub struct Station;

impl KeywordRules for Station {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Station]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = Cost::free().with(CostPart::TapUntapped {
            filter: Filter::and(vec![
                Filter::Type(CardType::Creature),
                Filter::Other,
                Filter::ControlledBy(PlayerRel::You),
            ]),
            count: Value::c(1),
        });
        let mut act = ActivatedAbility::new(
            cost,
            Body::effect(Effect::AddCounters {
                what: Sel::This,
                kind: counters::CHARGE.into(),
                n: Value::PowerOf(Box::new(Sel::Var(TAPPED))),
            }),
        );
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Station.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Station) }
