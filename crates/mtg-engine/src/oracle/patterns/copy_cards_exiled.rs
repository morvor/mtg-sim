//! Copying a card that the same ability just exiled (CR 707.12): "exile target instant or
//! sorcery card from a graveyard and copy it. You may cast the copy without paying its
//! mana cost." (Narset, Enlightened Exile), "... from your graveyard. Copy it, then you may
//! cast the copy without paying its mana cost." (Shiko, Paragon of the Way). The copy is
//! created in exile, where the card is; "cast the copy" is `r707_copy_cards`.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Whether "it" is a card the ability targets in a graveyard (which the ability exiles
/// before copying it).
fn it_is_a_targeted_graveyard_card(b: &Builder) -> bool {
    let Sel::Target(slot) = b.it else {
        return false;
    };
    b.targets.get(slot as usize).is_some_and(|t| match &t.what {
        TargetKind::Object(f) => f.zone() == Some(ZoneKind::Graveyard),
        _ => false,
    })
}

/// "copy it" after exiling a targeted card from a graveyard: a copy of the card in exile
/// (the object it became, CR 400.7).
fn copy_exiled_target(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "copy it" || !it_is_a_targeted_graveyard_card(b) {
        return None;
    }
    Some(Effect::CopyCard {
        what: Sel::Var(vars::IT),
        named: None,
    })
}

inventory::submit! { EffectPattern { name: "copy it (the card just exiled)", priority: 100, parse: copy_exiled_target } }
