//! Casting a card for free if its spell's mana value compares with the triggering
//! spell's (see `kw/cast_free_by_spell_mana_value.rs`):
//!
//! * "... If you do, you may cast that card without paying its mana cost if the two
//!   spells have the same mana value." (Powerbalance, after revealing the top card of your
//!   library in a "whenever an opponent casts a spell" trigger).
//! * "Look at the top X cards of your library, where X is that spell's mana value. You may
//!   cast a spell with mana value less than X from among them without paying its mana
//!   cost. Put the rest on the bottom of your library in a random order." (Kiora,
//!   Sovereign of the Deep, in a "whenever you cast a [kind of] spell" trigger).

use super::r406_exile_until::THE_REST;
use super::EffectPattern;
use crate::ability::*;
use crate::kw::cast_free_by_spell_mana_value::{
    CAST_IT_FREE_IF_SAME_MANA_VALUE, CAST_ONE_LOOKED_AT_FREE_WITH_LESSER_MANA_VALUE,
};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The name (in `Builder::named`) noting that the top X cards of the library were looked
/// at, X being the triggering spell's mana value.
const LOOKED_AT_X: &str = "the top x cards, where x is that spell's mana value";

fn cast_free_same_mana_value(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l.trim())
        != "you may cast that card without paying its mana cost if the two spells have the same mana value"
    {
        return None;
    }
    if !matches!(b.it, Sel::Var(vars::IT)) {
        return None;
    }
    Some(Effect::Custom(CAST_IT_FREE_IF_SAME_MANA_VALUE.into()))
}

inventory::submit! { EffectPattern { name: "r601 cast that card free if the two spells have the same mana value", priority: 100, parse: cast_free_same_mana_value } }

/// "look at the top X cards of your library, where X is that spell's mana value" in a
/// "whenever you cast a [kind of] spell" trigger.
fn look_at_top_x_spell_mana_value(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l.trim())
        != "look at the top x cards of your library, where x is that spell's mana value"
    {
        return None;
    }
    if !b.in_trigger || !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    b.it = Sel::Var(vars::IT);
    b.named.push((LOOKED_AT_X.into(), Sel::None));
    b.named.push((
        THE_REST.into(),
        Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(vars::REVEALED))),
            Filter::InZone(ZoneKind::Library),
        ])),
    ));
    let mut in_place = Destination::library_top();
    in_place.position = LibraryPosition::FromTop(0);
    Some(Effect::Dig {
        who: PlayerRef::You,
        n: Value::ManaValueOf(Box::new(Sel::TriggerSpell)),
        reveal: false,
        filter: Filter::Any,
        take: Value::c(0),
        take_up_to: true,
        take_to: Destination::zone(ZoneKind::Hand),
        rest_to: in_place,
    })
}

inventory::submit! { EffectPattern { name: "r601 look at the top X cards of your library, where X is that spell's mana value", priority: 60, parse: look_at_top_x_spell_mana_value } }

/// "You may cast a spell with mana value less than X from among them without paying its
/// mana cost." after looking at the top X cards.
fn cast_one_with_lesser_mana_value(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let l = l.strip_prefix("you may ").unwrap_or(l);
    if l != "cast a spell with mana value less than x from among them without paying its mana cost"
    {
        return None;
    }
    if !b.named.iter().any(|(n, _)| n == LOOKED_AT_X) {
        return None;
    }
    Some(Effect::Custom(
        CAST_ONE_LOOKED_AT_FREE_WITH_LESSER_MANA_VALUE.into(),
    ))
}

inventory::submit! { EffectPattern { name: "r601 cast a spell with mana value less than X from among them for free", priority: 100, parse: cast_one_with_lesser_mana_value } }
