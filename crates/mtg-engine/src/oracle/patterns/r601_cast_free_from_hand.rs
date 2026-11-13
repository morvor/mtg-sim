//! "You may cast spells from your hand without paying their mana costs." (Omniscience,
//! Tamiyo's emblem): a static permission to cast spells from the hand for an alternative
//! cost of nothing (CR 118.9). The spells keep their normal timing (CR 601.3); no other
//! alternative cost can be combined with it (CR 118.9a); additional costs are paid, and
//! X is 0 (CR 107.3b). See `Game::cast_options`.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn cast_free_from_hand(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you may cast ")?;
    let kind = r.strip_suffix(" from your hand without paying their mana costs")?;
    let what = if kind == "spells" {
        Filter::Any
    } else {
        let (f, _, tail) = parse_object_phrase(kind)?;
        // "instant spells", "spells with mana value less than or equal to the number of
        // creatures you control" (Omnipresence).
        if !end(tail).is_empty() || !(kind.ends_with(" spells") || kind.starts_with("spells with "))
        {
            return None;
        }
        f
    };
    let s = StaticAbility::new(StaticEffect::PlayPermission(PlayPermission {
        who: PlayerRel::You,
        zone: ZoneKind::Hand,
        top_only: false,
        what,
        lands: false,
        spells: true,
        cost: Some(Cost::free()),
        flash: false,
        terms: Default::default(),
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "you may cast spells from your hand without paying their mana costs", priority: 100, parse: cast_free_from_hand } }
