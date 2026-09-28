//! "Until end of turn, you may look at the top card of your library any time, and you may
//! play lands and cast spells from the top of your library." (The Belligerent): what the
//! static abilities "You may look at the top card of your library any time." and "You may
//! play lands and cast spells from the top of your library." would say, given to the
//! player by a resolved effect for the rest of the turn (CR 611.2a). The permission is a
//! `PlayerModification::PlayPermission`, looking is `zones::LOOK_AT_TOP_CARD`. The cards
//! are played following all the usual costs and timing rules (CR 601.2, 305.1); the top
//! card can't be looked at while a spell is being cast (CR 401.5).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn play_from_top_this_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim()).strip_prefix("until end of turn, ")?;
    let (look, rest) = match l
        .strip_prefix("you may look at the top card of your library any time, and ")
    {
        Some(r) => (true, r),
        None => (false, l),
    };
    let rest = rest.strip_prefix("you may ")?;
    if !rest.ends_with(" from the top of your library") {
        return None;
    }
    let statics = crate::oracle::statics::parse_static(&format!("You may {rest}."), b.ctx)?;
    let [a] = statics.as_slice() else {
        return None;
    };
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    let StaticEffect::PlayPermission(pp) = &s.effect else {
        return None;
    };
    if s.condition.is_some() || pp.who != PlayerRel::You || pp.zone != ZoneKind::Library {
        return None;
    }
    let grant = |effect| Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect,
        duration: Duration::EndOfTurn,
    };
    let play = grant(PlayerModification::PlayPermission(pp.clone()));
    Some(if look {
        Effect::seq(vec![
            grant(PlayerModification::Custom(
                crate::zones::LOOK_AT_TOP_CARD.into(),
            )),
            play,
        ])
    } else {
        play
    })
}

inventory::submit! { EffectPattern { name: "r601 until end of turn, you may play cards from the top of your library", priority: 100, parse: play_from_top_this_turn } }
