//! Revealing (CR 701.20): "Your opponents play with their hands revealed." (Telepathy),
//! "Players play with their hands revealed." (Revelation).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

fn hands_revealed(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let who = match end(l) {
        "your opponents play with their hands revealed" => "opponents",
        "players play with their hands revealed"
        | "each player plays with their hand revealed" => "each",
        "you play with your hand revealed" | "play with your hand revealed" => "you",
        _ => return None,
    };
    let name = SmolStr::new(format!("{}{who}", crate::reveal::HANDS_REVEALED));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 hands revealed", priority: 100, parse: hands_revealed } }
