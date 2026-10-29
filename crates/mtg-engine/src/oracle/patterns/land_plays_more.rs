//! More wordings of additional land plays (CR 305.2): "You may play up to three additional
//! lands this turn." (Summer Bloom; "up to" adds nothing: a land play need not be used)
//! and "Each player may play an additional land during each of their turns." (Storm
//! Cauldron, for every player).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;

/// "you may play up to N additional lands this turn".
fn up_to_additional_lands(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l).trim();
    let r = l
        .strip_prefix("you may play up to ")
        .or_else(|| l.strip_prefix("play up to "))?;
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?;
    if n < 1 || r.trim() != "additional lands this turn" {
        return None;
    }
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::AdditionalLandPlays(n as u32),
        duration: Duration::ThisTurn,
    })
}

/// "Each player may play an additional land during each of their turns."
fn each_player_additional_land(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let n = match end(l).trim() {
        "each player may play an additional land during each of their turns"
        | "each player may play an additional land on each of their turns" => 1,
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::AdditionalLandPlays(
            PlayerRel::Any,
            n,
        ))),
        text,
    )])
}

inventory::submit! { EffectPattern { name: "you may play up to N additional lands this turn", priority: 100, parse: up_to_additional_lands } }
inventory::submit! { StaticPattern { name: "each player may play an additional land during each of their turns", priority: 100, parse: each_player_additional_land } }
