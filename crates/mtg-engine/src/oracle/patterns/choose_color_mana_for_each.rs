//! "{T}: Choose a color. Add one mana of that color for each [counted thing]." (Selvala,
//! Eager Trailblazer; Loot, the Nexus; Astral Cornucopia): that many mana of one color the
//! player chooses as the mana ability resolves — the same as "Add X mana of any one
//! color" (CR 106.1b, 605.1a). The count is taken as the ability resolves (CR 608.2h); a
//! count of 0 adds no mana.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn choose_color_mana_for_each(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cost_s, eff) = crate::oracle::split_cost(block.trim())?;
    let eff = eff.to_lowercase();
    let each = eff.strip_prefix("choose a color. add one mana of that color for each ")?;
    let n = super::statics::parse_for_each(end(each), None)?;
    let (cost, loyalty) = crate::oracle::costs::parse_cost(cost_s)?;
    if loyalty {
        return None;
    }
    let mut act = ActivatedAbility::new(
        cost,
        Body::effect(Effect::AddMana {
            who: PlayerRef::You,
            mana: ManaProduction::AnyOneColor(n),
            restriction: None,
        }),
    );
    act.is_mana_ability = true;
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "choose a color. add one mana of that color for each", priority: 60, parse: choose_color_mana_for_each } }
