//! Magnetic Web: "If a creature with a magnet counter on it attacks, all creatures with
//! magnet counters on them attack if able." (CR 508.1d) and "Whenever a creature with a
//! magnet counter on it attacks, all creatures with magnet counters on them block that
//! creature this turn if able." (CR 509.1c).

use super::{active, marker, parse, ManualAbility};
use crate::ability::*;
use crate::combat::{attacking_players, AttackRequirement};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const ATTACK_TOGETHER: &str = "card:Magnetic Web:creatures with magnet counters attack together";
const ATTACK_TEXT: &str = "If a creature with a magnet counter on it attacks, all creatures with magnet counters on them attack if able.";
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
    text: ATTACK_TEXT,
    build: |_| vec![marker(ATTACK_TOGETHER, ATTACK_TEXT)],
    reason: "attack requirement for all creatures with magnet counters once one attacks: unique pair of abilities",
} }

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
                            *blocker = magnetized();
                        }
                    }
                }
                AbilityDef::new(kind, BLOCK_TEXT)
            })
            .collect()
    },
    reason: "block requirement for all creatures with magnet counters: unique pair of abilities",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Each creature with a magnet counter attacks if able if another one attacks (one
    /// requirement per Magnetic Web).
    fn attack_requirements(&self, g: &Game) -> Vec<AttackRequirement> {
        let webs = active(g, ATTACK_TOGETHER).len();
        if webs == 0 {
            return vec![];
        }
        let players = attacking_players(g);
        let magnets: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.is_creature() && o.counters.get("magnet").is_some_and(|n| *n > 0))
            .filter(|o| players.contains(&o.controller))
            .map(|o| o.id)
            .collect();
        let mut out = Vec::new();
        for &c in &magnets {
            let others: Vec<ObjectId> = magnets.iter().copied().filter(|o| *o != c).collect();
            if others.is_empty() {
                continue;
            }
            for _ in 0..webs {
                out.push(AttackRequirement::AttacksIfAnyAttacks(c, others.clone()));
            }
        }
        out
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
