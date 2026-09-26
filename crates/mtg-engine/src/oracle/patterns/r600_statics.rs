//! Oracle patterns for static abilities that function outside the battlefield (CR 604.3,
//! 604.6): color-defining characteristic-defining abilities ("~ is all colors") and
//! "You may cast ~ from your graveyard [as long as ...]".

use super::{AbilityPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::statics::parse_condition;
use crate::oracle::CompileContext;
use crate::types::ColorSet;

fn color_cda(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let colors = match l {
        "~ is all colors" => ColorSet::ALL,
        "~ is colorless" => ColorSet::NONE,
        _ => return None,
    };
    // CR 604.3: a characteristic-defining ability functions in all zones.
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::SetColors(colors)],
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// "You may cast ~ from your graveyard [as long as (condition)]": functions only while the
/// card is in the graveyard (CR 604.6).
fn cast_self_from_graveyard(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("you may cast ~ from your graveyard")?;
    let condition = if r.is_empty() {
        None
    } else {
        Some(parse_condition(r.strip_prefix(" as long as ")?, ctx)?)
    };
    let mut s = StaticAbility::new(StaticEffect::PlayPermission(PlayPermission {
        who: PlayerRel::You,
        zone: ZoneKind::Graveyard,
        top_only: false,
        what: Filter::Source,
        lands: false,
        spells: true,
        cost: None,
    }));
    s.zone = FunctionZone::Graveyard;
    s.condition = condition;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// "Cast ~ only during combat" etc.: a restriction that functions in every zone the card
/// could be cast from (CR 604.6).
fn cast_only_during(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let allowed = match l.strip_prefix("cast ~ only ")? {
        "during combat" => Condition::Phase(PhaseCond::Combat),
        "during your turn" => Condition::YourTurn,
        "during an opponent's turn" => Condition::NotYourTurn,
        "during your upkeep" => Condition::And(vec![
            Condition::YourTurn,
            Condition::Phase(PhaseCond::Upkeep),
        ]),
        _ => return None,
    };
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::CantCast {
        who: PlayerFilter::Any,
        what: Filter::Source,
    }));
    s.zone = FunctionZone::Anywhere;
    s.condition = Some(Condition::Not(Box::new(allowed)));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

/// The same, on instants and sorceries (whose lines aren't parsed as static abilities).
fn cast_only_during_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    cast_only_during(lower.strip_suffix('.')?, t, ctx)
}

inventory::submit! { StaticPattern { name: "cast only during", priority: 0, parse: cast_only_during } }
inventory::submit! { AbilityPattern { name: "cast only during", priority: 0, parse: cast_only_during_block } }
inventory::submit! { StaticPattern { name: "color-defining ability", priority: 0, parse: color_cda } }
inventory::submit! { StaticPattern { name: "cast this from your graveyard", priority: 0, parse: cast_self_from_graveyard } }
