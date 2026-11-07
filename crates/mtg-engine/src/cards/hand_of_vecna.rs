//! Hand of Vecna: "At the beginning of combat on your turn, equipped creature or a creature
//! you control named Vecna gets +X/+X until end of turn, where X is the number of cards in
//! your hand." One creature, chosen as the ability resolves, from the equipped creature
//! and the creatures you control named Vecna.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::types::CardType;

const TEXT: &str = "At the beginning of combat on your turn, equipped creature or a creature you control named Vecna gets +X/+X until end of turn, where X is the number of cards in your hand.";

fn equipped_or_vecna() -> Sel {
    Sel::Choose {
        chooser: PlayerRef::You,
        filter: Filter::Or(vec![
            Filter::In(Box::new(Sel::AttachedTo)),
            Filter::And(vec![
                Filter::Type(CardType::Creature),
                Filter::ControlledBy(PlayerRel::You),
                Filter::Named("Vecna".into()),
            ]),
        ]),
        count: Value::Const(1),
        up_to: false,
        store: None,
    }
}

inventory::submit! { ManualAbility {
    card: "Hand of Vecna",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "At the beginning of combat on your turn, equipped creature gets +X/+X until end of turn, where X is the number of cards in your hand.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::Modify { mods, duration, .. } => Effect::Modify {
                        what: equipped_or_vecna(),
                        mods,
                        duration,
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "\"equipped creature or a creature you control named Vecna\": one creature chosen on resolution among two descriptions; only this card",
} }
