//! Dina's Guidance: "Search your library for a creature card, reveal it, put it into your
//! hand or graveyard, then shuffle." (CR 701.23).
//!
//! The search and reveal compile; the found card then goes where you choose (hand or
//! graveyard), and your library is shuffled after that.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "Search your library for a creature card, reveal it, put it into your hand or graveyard, then shuffle.";

inventory::submit! { ManualAbility {
    card: "Dina's Guidance",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Search your library for a creature card and reveal it.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |search| {
                    let put = |zone| Effect::Move {
                        what: Sel::Var(vars::IT),
                        to: Destination::zone(zone),
                    };
                    Effect::seq(vec![
                        search,
                        Effect::ChooseOne {
                            who: PlayerRef::You,
                            options: vec![
                                ("Put it into your hand".into(), put(ZoneKind::Hand)),
                                ("Put it into your graveyard".into(), put(ZoneKind::Graveyard)),
                            ],
                        },
                        Effect::Shuffle { who: PlayerRef::You },
                    ])
                })
            })
            .collect()
    },
    reason: "a choice between two destinations for a found card: unique",
} }
