//! "Each player gains control of all [permanents] they own" (Trostani Discordant): each
//! player gains control of their own permanents of that kind (CR 108.4, 613.1b). A
//! permanent its owner already controls is unaffected in practice.

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;

fn each_player_regains(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player gains control of all ")?;
    let r = r.strip_suffix(" they own")?;
    let (filter, plural, tail) = parse_object_phrase(r)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::GainControl {
            what: Sel::All(Filter::and(vec![
                filter,
                Filter::InZone(ZoneKind::Battlefield),
                Filter::OwnedBy(PlayerRel::Iterated),
                Filter::not(Filter::ControlledBy(PlayerRel::Iterated)),
            ])),
            who: PlayerRef::Iterated,
            duration: Duration::Permanent,
        }),
    })
}

inventory::submit! { EffectPattern { name: "r108 each player gains control of all X they own", priority: 100, parse: each_player_regains } }
