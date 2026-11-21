//! Kaya, Ghost Assassin: "0: Exile ~ or up to one target creature. Return that card to the
//! battlefield under its owner's control at the beginning of your next upkeep. You lose 2
//! life." With a target, you choose as it resolves whether to exile Kaya or that creature;
//! without one, Kaya is exiled (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "0: Exile ~ or up to one target creature. Return that card to the battlefield under its owner's control at the beginning of your next upkeep. You lose 2 life.";

fn exile_kaya_or_target(e: Effect) -> Effect {
    match e {
        Effect::Seq(v) => Effect::Seq(v.into_iter().map(exile_kaya_or_target).collect()),
        Effect::Exile {
            what: Sel::Target(0),
            face_down,
            link,
        } => {
            let exile = |what| Effect::Exile {
                what,
                face_down,
                link,
            };
            Effect::If {
                cond: Condition::SelNonEmpty(Sel::Target(0)),
                then: Box::new(Effect::ChooseOne {
                    who: PlayerRef::You,
                    options: vec![
                        ("Exile ~".into(), exile(Sel::This)),
                        ("Exile the target creature".into(), exile(Sel::Target(0))),
                    ],
                }),
                otherwise: Box::new(exile(Sel::This)),
            }
        }
        e => e,
    }
}

inventory::submit! { ManualAbility {
    card: "Kaya, Ghost Assassin",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "0: Exile up to one target creature. Return that card to the battlefield under its owner's control at the beginning of your next upkeep. You lose 2 life.")
            .iter()
            .map(|a| map_effect(a, TEXT, exile_kaya_or_target))
            .collect()
    },
    reason: "\"exile ~ or up to one target creature\": the choice between the source and the target is made as it resolves; only this card",
} }
