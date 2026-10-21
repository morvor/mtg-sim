//! Mirror-Mad Phantasm: "{1}{U}: ~'s owner shuffles it into their library. If that player
//! does, they reveal cards from the top of that library until a card named ~ is revealed.
//! The player puts that card onto the battlefield and all other cards revealed this way
//! into their graveyard."
//!
//! Everything is done by the owner. The card looked for is named Mirror-Mad Phantasm even
//! when another object has the ability (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "{1}{U}: ~'s owner shuffles it into their library. If that player does, they reveal cards from the top of that library until a card named ~ is revealed. The player puts that card onto the battlefield and all other cards revealed this way into their graveyard.";

inventory::submit! { ManualAbility {
    card: "Mirror-Mad Phantasm",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let reveal = parse(ctx, "{1}{U}: Reveal cards from the top of your library until you reveal a creature card. Put that card onto the battlefield and the rest into your graveyard.");
        let reveal = reveal.first().and_then(|a| match &a.kind {
            AbilityKind::Activated(act) => match &act.body.effect {
                Effect::RevealUntil { who, found_to, rest_to, .. } => Some(Effect::RevealUntil {
                    who: who.clone(),
                    filter: Filter::And(vec![
                        Filter::Card,
                        Filter::Named("Mirror-Mad Phantasm".into()),
                    ]),
                    found_to: found_to.clone(),
                    rest_to: rest_to.clone(),
                }),
                _ => None,
            },
            _ => None,
        });
        parse(ctx, "{1}{U}: Shuffle ~ into its owner's library.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |_| Effect::AsPlayer {
                    who: PlayerRef::OwnerOf(Box::new(Sel::This)),
                    effect: Box::new(Effect::Seq(vec![
                        // "If that player does": recorded by this form of the shuffle.
                        Effect::ShuffleIntoLibrary {
                            what: Sel::This,
                            library: PlayerRef::You,
                        },
                        Effect::If {
                            cond: Condition::PrevHappened,
                            then: Box::new(reveal.clone().unwrap_or(Effect::Noop)),
                            otherwise: Box::new(Effect::Noop),
                        },
                    ])),
                })
            })
            .collect()
    },
    reason: "its owner shuffles it away and digs for a card with its name: only this card",
} }
