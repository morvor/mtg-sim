//! Imprint (an ability word): cards exiled with a permanent, and its other abilities that
//! refer to "the exiled card" or "a card exiled with ~" (CR 607.2a).
//!
//! * "copy the exiled card", "copy a card exiled with ~": a copy of the card, created in
//!   exile (CR 707.12); "If you do, you may cast the copy without paying its mana cost." is
//!   handled by `r707_copy_cards`.
//! * "create a token that's a copy of the exiled card / a card exiled with ~" (CR 111.4,
//!   707.2).
//! * "return each other card exiled with ~ to its owner's graveyard" after exiling a card
//!   with it.
//!
//! Only a permanent's abilities are linked this way: on an instant or sorcery, "the exiled
//! card" is the one that spell exiled (Surge to Victory), which these patterns don't handle.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Cards exiled with the source that are still in exile (CR 400.7: a card that left exile
/// is a new object, no longer exiled with it).
fn exiled_with_source() -> Filter {
    Filter::And(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ])
}

/// "the exiled card" (all of them, usually one), "a card exiled with ~" (one of them, the
/// controller's choice).
fn exiled_card_ref(s: &str) -> Option<Sel> {
    Some(match s {
        "the exiled card" => Sel::All(exiled_with_source()),
        "a card exiled with ~" => Sel::Choose {
            chooser: PlayerRef::You,
            filter: exiled_with_source(),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        _ => return None,
    })
}

/// "copy the exiled card", "copy a card exiled with ~".
fn copy_exiled_card(l: &str, b: &mut Builder) -> Option<Effect> {
    if b.ctx.is_spell() {
        return None;
    }
    let what = exiled_card_ref(end(l).strip_prefix("copy ")?)?;
    Some(Effect::CopyCard { what, named: None })
}

inventory::submit! { EffectPattern { name: "imprint: copy the exiled card", priority: 0, parse: copy_exiled_card } }

/// "create a token that's a copy of the exiled card", "create a token that's a copy of a
/// card exiled with ~".
fn token_copy_of_exiled_card(l: &str, b: &mut Builder) -> Option<Effect> {
    if b.ctx.is_spell() {
        return None;
    }
    let of = exiled_card_ref(end(l).strip_prefix("create a token that's a copy of ")?)?;
    Some(Effect::CreateTokenCopy {
        of,
        count: Value::c(1),
        controller: PlayerRef::You,
        tapped: false,
        attacking: false,
        mods: vec![],
    })
}

inventory::submit! { EffectPattern { name: "imprint: token copy of the exiled card", priority: 0, parse: token_copy_of_exiled_card } }

/// "return each other card exiled with ~ to its owner's graveyard": the cards exiled with
/// the source other than the one just exiled ("it").
fn return_other_exiled_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    if b.ctx.is_spell() || end(l) != "return each other card exiled with ~ to its owner's graveyard" {
        return None;
    }
    const V: Var = vars::USER + 81;
    Some(Effect::ForEach {
        sel: Sel::All(Filter::And(vec![
            exiled_with_source(),
            Filter::not(Filter::In(Box::new(Sel::Var(vars::IT)))),
        ])),
        var: V,
        effect: Box::new(Effect::Move {
            what: Sel::Var(V),
            to: Destination::zone(ZoneKind::Graveyard),
        }),
    })
}

inventory::submit! { EffectPattern { name: "imprint: return each other exiled card", priority: 0, parse: return_other_exiled_cards } }
