//! "~ is also a Cleric, Rogue, Warrior, and Wizard." (Stonework Packbeast, Tajuru Paragon,
//! Veteran Adventurer): a characteristic-defining ability (CR 604.3a: it defines the
//! object's subtypes, is printed on the card it affects and isn't conditional), so it
//! functions in every zone (CR 604.3; their rulings: "applies in all zones, not just the
//! battlefield") and applies first in layer 4 (CR 613.3).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;
use crate::types::CardType;

fn is_also_subtypes_cda(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let l = t.to_lowercase();
    let r = l.strip_suffix('.')?.strip_prefix("~ is also ")?;
    let subj = super::statics::Subject {
        filter: Filter::Source,
        it: Some(Sel::This),
        hint: CardType::Creature,
        lands: false,
        creatures: true,
    };
    let mods = super::statics::type_predicate_mods(&format!("also {r}"), &subj)?;
    if !matches!(mods.as_slice(), [Modification::AddSubtypes(_)]) {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods,
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), t)])
}

inventory::submit! { AbilityPattern { name: "r604.3: ~ is also a [subtypes] (a CDA)", priority: 100, parse: is_also_subtypes_cda } }
