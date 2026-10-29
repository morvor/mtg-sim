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
//! * The number of counters is determined as the ability resolves, from the tapped
//!   creature as it currently exists or as it last existed on the battlefield; a negative
//!   power puts none. Static abilities may make a creature station using another
//!   characteristic (CR 702.184c): "Each creature you control with toughness greater than
//!   its power stations permanents using its toughness rather than its power", "Each
//!   creature you control crews Vehicles and stations permanents as though its power were
//!   2 greater". Those give the creatures the `kw/crew.rs` statics for station
//!   ([`crate::kw::crew::uses_toughness`], [`crate::kw::crew::power_bonus`]), read from
//!   the creature's abilities as it resolves or as it last existed.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

/// The variable holding the creature tapped to pay the station cost (the objects a
/// cost tapped, see `Game::activate_ability`).
const TAPPED: Var = vars::USER + 91;

/// `Value::Custom`: the number of charge counters the resolving station ability puts
/// (CR 702.184a, 702.184c).
const STATION_COUNTERS: &str = "station:counters";

/// The number `creature` stations permanents for: its power, or its toughness, plus any
/// bonus, from the statics it has (or had as it last existed on the battlefield).
pub fn stationing_power(g: &Game, creature: ObjectId) -> i64 {
    let o = g.obj(creature);
    let toughness = crate::kw::crew::uses_toughness(KeywordKind::Station);
    let prefix = format!("tap_total_power:{}:+", KeywordKind::Station.name());
    let mut base = o.power() as i64;
    let mut bonus = 0i64;
    for a in &o.chars.abilities {
        let AbilityKind::Static(st) = &a.kind else {
            continue;
        };
        let StaticEffect::Custom(name) = &st.effect else {
            continue;
        };
        if *name == toughness {
            base = o.toughness() as i64;
        } else if let Some(n) = name.strip_prefix(prefix.as_str()) {
            bonus += n.parse::<i64>().unwrap_or(0);
        }
    }
    base + bonus
}

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
                n: Value::Custom(STATION_COUNTERS.into()),
            }),
        );
        act.timing = ActivationTiming::Sorcery;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            KeywordKind::Station.name(),
        )])
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != STATION_COUNTERS {
            return None;
        }
        Some(
            ctx.vars
                .get(&TAPPED)
                .into_iter()
                .flatten()
                .filter_map(|e| e.object())
                .map(|c| stationing_power(g, c).max(0))
                .sum(),
        )
    }
}

inventory::submit! { KeywordRegistration(&Station) }
