//! "Once during each of your turns, you may cast a creature spell from your graveyard."
//! (Karador, Ghost Chieftain), "... an Aura or Equipment spell ..." (Danitha, New
//! Benalia's Light), "... a permanent spell with mana value 2 or less ..." (Lurrus of the
//! Dream-Den). See `kw/once_each_turn_cast.rs`.

use super::StaticPattern;
use crate::ability::*;
use crate::kw::once_each_turn_cast::ONCE_UNUSED;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;
use crate::types::CardType;

fn once_each_turn_cast(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let r = end(l)
        .strip_prefix("once during each of your turns, you may cast ")?
        .strip_suffix(" from your graveyard")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    // "[quality] spell [with ...]": a card that would be cast as such a spell.
    let (before, after) = r.split_once("spell")?;
    let desc = format!("{before}card{after}");
    let (quality, _, tail) = parse_object_phrase(desc.trim())?;
    if !end(tail).trim().is_empty() {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::PlayPermission(PlayPermission {
        who: PlayerRel::You,
        zone: ZoneKind::Graveyard,
        top_only: false,
        what: Filter::and(vec![
            Filter::Not(Box::new(Filter::Type(CardType::Land))),
            quality,
        ]),
        lands: false,
        spells: true,
        cost: None,
    }));
    s.condition = Some(Condition::And(vec![
        Condition::YourTurn,
        Condition::Custom(ONCE_UNUSED.into()),
    ]));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "once during each of your turns, you may cast a [quality] spell from your graveyard", priority: 100, parse: once_each_turn_cast } }
