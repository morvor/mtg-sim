//! "Spells you cast from anywhere other than your hand cost {N} less to cast." (Sage of the
//! Beyond, Bilbo, Fortune Teller's Talent), "Spells you cast from exile cost {N} less
//! ...": a cost modifier (CR 601.2f) for spells by where they're cast from. A card being
//! considered for casting is judged by the zone it's in; a spell being cast remembers
//! the zone it was cast from (CR 601.2a). A generic reduction can't reduce colored mana
//! (CR 118.7a).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "from anywhere other than your hand", "from exile", "from your graveyard".
fn from_zone(z: &str) -> Option<Filter> {
    let zone = |k: ZoneKind| Filter::Or(vec![Filter::InZone(k), Filter::CastFrom(k)]);
    Some(match z {
        "anywhere other than your hand" => Filter::not(zone(ZoneKind::Hand)),
        "exile" => zone(ZoneKind::Exile),
        "your graveyard" => zone(ZoneKind::Graveyard),
        _ => return None,
    })
}

fn spells_cast_from_zone_cost(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("spells you cast from ")?;
    let (zone, rest) = r.split_once(" cost ")?;
    let filter = from_zone(zone)?;
    let (amount, tail) = rest.split_once('}')?;
    let n: i32 = amount.strip_prefix('{')?.parse().ok()?;
    let change = match end(tail) {
        "less to cast" => CostChange::ReduceGeneric(Value::c(n)),
        "more to cast" => CostChange::IncreaseGeneric(Value::c(n)),
        _ => return None,
    };
    let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Spells(filter),
        who: PlayerRel::You,
        change,
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "spells you cast from [zone] cost {N} less", priority: 100, parse: spells_cast_from_zone_cost } }
