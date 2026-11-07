//! Abilities that modify the deck construction rules (CR 113.6n), which function before
//! the game begins: "[This card] can be your commander." (CR 903.3a, checked by
//! `kw::partner::can_be_commander`) and, on instants and sorceries (whose other lines are
//! spell abilities), "A deck can have any number of cards named ~." (see `crate::deck`).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn deck_static(name: &str, text: &str) -> Vec<Ability> {
    let mut s = StaticAbility::new(StaticEffect::Custom(name.into()));
    s.zone = FunctionZone::Anywhere;
    vec![AbilityDef::new(AbilityKind::Static(s), text)]
}

/// "~ can be your commander." — on any kind of card (sorceries and lands have it too).
fn can_be_your_commander(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    (end(&t.to_lowercase()) == "~ can be your commander")
        .then(|| deck_static(crate::kw::partner::CAN_BE_YOUR_COMMANDER, t))
}

inventory::submit! { AbilityPattern { name: "misc: ~ can be your commander", priority: 100, parse: can_be_your_commander } }

/// "A deck can have any number of cards named ~." on an instant or sorcery (permanents'
/// static lines are compiled by `r113_abilities.rs`).
fn any_number_on_spells(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    (ctx.is_spell() && end(&t.to_lowercase()) == "a deck can have any number of cards named ~")
        .then(|| deck_static(crate::deck::ANY_NUMBER, t))
}

inventory::submit! { AbilityPattern { name: "misc: any number of cards named ~ (spells)", priority: 100, parse: any_number_on_spells } }
