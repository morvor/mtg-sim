//! "If you would pay life while your library has at least that many cards in it, exile
//! that many cards from the top of your library instead." (Ashiok, Wicked Manipulator):
//! see `kw/pay_life_exile_instead.rs`.

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

inventory::submit! { StaticPattern { name: "r119 pay life: exile that many cards from your library instead", priority: 100, parse: pay_life_exile_instead } }

fn pay_life_exile_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l)
        != "if you would pay life while your library has at least that many cards in it, exile that many cards from the top of your library instead"
    {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::Custom(
        crate::kw::pay_life_exile_instead::PAY_LIFE_EXILE_INSTEAD.into(),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}
