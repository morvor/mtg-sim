//! "Other players can't play lands or cast spells from their graveyards this turn." and
//! "You may play lands and cast spells from other players' graveyards this turn as though
//! those cards were in your graveyard." (Shaman's Trance): a restriction on the other
//! players for the turn (CR 601.3, 305.2), checked for the cards in each restricted
//! player's own graveyard, and an "as though" permission (CR 609.4) for the turn
//! (`as_though.rs`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn graveyard_play_swap(l: &str, _b: &mut Builder) -> Option<Effect> {
    match end(l) {
        "other players can't play lands or cast spells from their graveyards this turn" => {
            // "their graveyards": the graveyard of the player the restriction is checked
            // for (`PlayerRel::Iterated` while a casting or land play is checked).
            let what = Filter::and(vec![
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::Iterated),
            ]);
            let who = PlayerFilter::NotYou;
            Some(Effect::Seq(vec![
                Effect::AddRestriction {
                    restriction: Restriction::CantPlayLandCards {
                        who: who.clone(),
                        what: what.clone(),
                    },
                    duration: Duration::EndOfTurn,
                },
                Effect::AddRestriction {
                    restriction: Restriction::CantCast { who, what },
                    duration: Duration::EndOfTurn,
                },
            ]))
        }
        "you may play lands and cast spells from other players' graveyards this turn as though those cards were in your graveyard" => {
            Some(Effect::AddPlayerEffect {
                who: PlayerRef::You,
                effect: PlayerModification::Custom(
                    crate::as_though::OTHER_GRAVEYARDS_AS_YOURS.into(),
                ),
                duration: Duration::EndOfTurn,
            })
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "other players can't play from their graveyards; you may play from theirs as though yours", priority: 100, parse: graveyard_play_swap } }

