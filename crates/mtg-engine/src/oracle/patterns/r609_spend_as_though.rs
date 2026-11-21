//! "You may spend mana as though it were mana of any color." (Chromatic Orrery) and
//! "Players may spend mana as though it were mana of any color." (CR 609.4b): the player
//! may pay colored mana symbols with any mana. This changes only how costs may be paid,
//! not the costs or what mana was actually spent (see `as_though::payment_cost`).

use super::StaticPattern;
use crate::ability::*;
use crate::as_though::SPEND_AS_ANY_COLOR;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn spend_as_any_color(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let rest = " may spend mana as though it were mana of any color";
    let affected = match end(l).strip_suffix(rest)? {
        "you" => PlayerFilter::You,
        "players" | "each player" => PlayerFilter::Any,
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayerEffect {
            affected,
            effect: PlayerModification::Custom(SPEND_AS_ANY_COLOR.into()),
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r609 spend mana as though any color", priority: 60, parse: spend_as_any_color } }
