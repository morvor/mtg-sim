//! Magnetic Web: "Whenever a creature with a magnet counter on it attacks, all creatures
//! with magnet counters on them block that creature this turn if able." (CR 509.1c). Its
//! "If a creature with a magnet counter on it attacks, all creatures with magnet counters
//! on them attack if able." is compiled (`Restriction::AttackTogether`).

use super::{parse, ManualAbility};
use crate::ability::*;

const BLOCK_TEXT: &str = "Whenever a creature with a magnet counter on it attacks, all creatures with magnet counters on them block that creature this turn if able.";

fn magnetized() -> Filter {
    Filter::And(vec![
        Filter::Type(crate::types::CardType::Creature),
        Filter::HasCounter(Some("magnet".into())),
    ])
}

inventory::submit! { ManualAbility {
    card: "Magnetic Web",
    face: 0,
    text: BLOCK_TEXT,
    build: |ctx| {
        parse(ctx, "Whenever a creature with a magnet counter on it attacks, target creature blocks it this turn if able.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Triggered(t) = &mut kind {
                    t.body.targets.clear();
                    if let Effect::AddRestriction { restriction, .. } = &mut t.body.effect {
                        if let Restriction::MustBlockAttacker { blocker, .. } = restriction {
                            // The creatures with magnet counters as it resolves
                            // (locked in then, CR 608.2h).
                            *blocker = Filter::In(Box::new(Sel::All(magnetized())));
                        }
                    }
                }
                AbilityDef::new(kind, BLOCK_TEXT)
            })
            .collect()
    },
    reason: "block requirement for all creatures with magnet counters: unique pair of abilities",
} }
