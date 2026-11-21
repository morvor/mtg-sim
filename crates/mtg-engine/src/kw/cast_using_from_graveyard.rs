//! "You may cast this card from your graveyard using its [keyword] ability." (Tenacious
//! Underdog's blitz, Brokkos's mutate): a static ability functioning in its owner's
//! graveyard. The card may be cast from there only for that keyword's alternative cost
//! (Brokkos rulings), with the normal timing of casting it.

use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;

/// The `StaticEffect::Custom` name of the ability for `kind`.
pub fn permission_name(kind: KeywordKind) -> &'static str {
    match kind {
        KeywordKind::Blitz => "you may cast this card from your graveyard using its blitz ability",
        KeywordKind::Mutate => {
            "you may cast this card from your graveyard using its mutate ability"
        }
        _ => "you may cast this card from your graveyard using its keyword ability",
    }
}

/// Whether `card` is in `p`'s graveyard with the ability letting them cast it from there
/// using its `kind` ability.
pub fn allows(g: &Game, p: PlayerId, card: ObjectId, kind: KeywordKind) -> bool {
    let o = g.obj(card);
    let name = permission_name(kind);
    o.zone == Zone::Graveyard(p)
        && o.owner == p
        && o.chars.abilities.iter().any(|a| {
            matches!(&a.kind, AbilityKind::Static(s)
                if s.zone == FunctionZone::Graveyard
                    && matches!(&s.effect, StaticEffect::Custom(n) if n.as_str() == name))
        })
}
