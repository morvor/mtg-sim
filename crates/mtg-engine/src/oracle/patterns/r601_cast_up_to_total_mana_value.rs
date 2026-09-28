//! "You may cast up to two instant and/or sorcery spells with total mana value 6 or less
//! from your graveyard and/or hand without paying their mana costs. If those spells would
//! be put into your graveyard, exile them instead." (Invoke Calamity). See
//! `kw/cast_up_to_total_mana_value.rs`.

use super::EffectPattern;
use crate::ability::*;
use crate::kw::cast_up_to_total_mana_value::effect_name;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

fn small_number(s: &str) -> Option<u32> {
    Some(match s {
        "two" => 2,
        "three" => 3,
        "four" => 4,
        _ => s.parse().ok()?,
    })
}

fn cast_up_to_total_mana_value(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let l = l.strip_prefix("you may ").unwrap_or(l);
    let r = l
        .strip_prefix("cast up to ")?
        .strip_suffix(" without paying their mana costs")?;
    let (n, r) = r.split_once(' ')?;
    let n = small_number(n)?;
    let (kinds, r) = r.split_once(" spells with total mana value ")?;
    let (total, zones) = r.split_once(" or less from ")?;
    let total: u32 = total.parse().ok()?;
    let zones = match zones {
        "your hand" => vec![ZoneKind::Hand],
        "your graveyard" => vec![ZoneKind::Graveyard],
        "your graveyard and/or hand" | "your hand and/or graveyard" => {
            vec![ZoneKind::Hand, ZoneKind::Graveyard]
        }
        _ => return None,
    };
    let types = kinds
        .split(" and/or ")
        .map(CardType::from_word)
        .collect::<Option<Vec<_>>>()?;
    Some(Effect::Custom(effect_name(n, total, &zones, &types).into()))
}

inventory::submit! { EffectPattern { name: "r601 cast up to N spells with total mana value N or less for free", priority: 100, parse: cast_up_to_total_mana_value } }

/// "If those spells would be put into your graveyard, exile them instead." after casting
/// spells: the replacement effect applies to the spells they became (CR 400.7h).
fn those_spells_exiled_instead(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l.trim()) != "if those spells would be put into your graveyard, exile them instead" {
        return None;
    }
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: Filter::In(Box::new(Sel::Var(vars::IT))),
                from: None,
                to: Some(ZoneKind::Graveyard),
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        },
        // Locked onto the spells: once they've left the stack, they're new objects the
        // effect can't apply to.
        duration: Duration::Permanent,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "r601 those spells are exiled instead", priority: 100, parse: those_spells_exiled_instead } }
