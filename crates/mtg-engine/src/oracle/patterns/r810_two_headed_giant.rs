//! Oracle patterns for life totals and poison counters as teams share them in Two-Headed
//! Giant (CR 810.9, 810.10): "Redistribute any number of players' life totals." and
//! "You can't get poison counters."

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::multiplayer::two_headed::{CANT_GET_POISON, REDISTRIBUTE_LIFE};
use crate::oracle::effects::Builder;
use crate::oracle::CompileContext;

/// "Redistribute any number of players' life totals." (Reverse the Sands).
fn redistribute(l: &str, _b: &mut Builder) -> Option<Effect> {
    (l.trim_end_matches('.') == "redistribute any number of players' life totals")
        .then(|| Effect::Custom(REDISTRIBUTE_LIFE.into()))
}

/// "You can't get poison counters." (Melira, Sylvok Outcast).
fn cant_get_poison(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let who = match l {
        "you can't get poison counters" => PlayerFilter::You,
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayerEffect {
            affected: who,
            effect: PlayerModification::Custom(CANT_GET_POISON.into()),
        })),
        text,
    )])
}

inventory::submit! { EffectPattern { name: "r810 redistribute life totals", priority: 0, parse: redistribute } }
inventory::submit! { StaticPattern { name: "r810 can't get poison counters", priority: 0, parse: cant_get_poison } }
