//! Camouflage: "This turn, instead of declaring blockers, each defending player chooses any
//! number of creatures they control and divides them into a number of piles equal to the
//! number of attacking creatures for whom that player is the defending player. Creatures
//! those players control that can block additional creatures may likewise be put into
//! additional piles. Assign each pile to a different one of those attacking creatures at
//! random. Each creature in a pile that can block the creature that pile is assigned to does
//! so." (Its "Cast this spell only during your declare attackers step." compiles.)

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{ObjectId, PlayerId};

const PILES_INSTEAD: &str = "card:Camouflage:blockers are put into piles assigned at random";
const TEXT: &str = "This turn, instead of declaring blockers, each defending player chooses any number of creatures they control and divides them into a number of piles equal to the number of attacking creatures for whom that player is the defending player. Creatures those players control that can block additional creatures may likewise be put into additional piles. Assign each pile to a different one of those attacking creatures at random. Each creature in a pile that can block the creature that pile is assigned to does so.";

inventory::submit! { ManualAbility {
    card: "Camouflage",
    face: 0,
    text: TEXT,
    build: |_| vec![spell(vec![], Effect::Custom(PILES_INSTEAD.into()), TEXT)],
    reason: "replaces the declaration of blockers with piles assigned to attackers at random: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (turn, Camouflage).
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PILES_INSTEAD {
            return false;
        }
        let turn = i64::from(g.turn.number);
        g.cards.retain(PILES_INSTEAD, |r| r[0] == turn);
        let src = ctx.source.map_or(-1, |s| i64::from(s.0));
        g.cards.push(PILES_INSTEAD, vec![turn, src]);
        true
    }
    fn blocks_instead(&self, g: &mut Game, group: &[PlayerId]) -> Option<Vec<(ObjectId, ObjectId)>> {
        let turn = i64::from(g.turn.number);
        let row = g.cards.get(PILES_INSTEAD).iter().find(|r| r[0] == turn)?.clone();
        let mut blocks = Vec::new();
        for &p in group {
            let attackers: Vec<ObjectId> = g
                .combat
                .as_ref()
                .map(|c| {
                    c.attackers
                        .iter()
                        .filter(|a| a.defending_player == Some(p))
                        .map(|a| a.id)
                        .collect()
                })
                .unwrap_or_default();
            let k = attackers.len();
            if k == 0 {
                continue;
            }
            let creatures: Vec<ObjectId> = g
                .permanents_controlled_by(p)
                .into_iter()
                .filter(|o| g.can_block_at_all(*o))
                .collect();
            // Each creature goes into up to as many piles as it can block creatures.
            let mut piles: Vec<Vec<ObjectId>> = vec![vec![]; k];
            for c in creatures {
                let most = g.max_blocks(c).map_or(k, |m| (m as usize).min(k));
                for _ in 0..most {
                    let mut labels = vec!["No (more) piles".to_string()];
                    labels.extend((1..=k).map(|i| format!("Pile {i}")));
                    let prompt = format!("Put {} into a pile?", g.describe(c));
                    let i = g.ask_option(p, Some(c), &prompt, labels);
                    if i == 0 || i > k || piles[i - 1].contains(&c) {
                        break;
                    }
                    piles[i - 1].push(c);
                }
            }
            g.emit(crate::events::Event::Custom {
                name: crate::piles::GROUPED_INTO_PILES.into(),
                player: Some(p),
                obj: (row[1] >= 0).then_some(ObjectId(row[1] as u32)),
                amount: 0,
            });
            // Assign each pile to a different attacker at random.
            let mut order: Vec<usize> = (0..k).collect();
            for i in (1..k).rev() {
                let j = g.random_range(0, i as u32) as usize;
                order.swap(i, j);
            }
            for (pile, &ai) in piles.iter().zip(&order) {
                let a = attackers[ai];
                for &c in pile {
                    if g.can_block(c, a) {
                        blocks.push((c, a));
                    }
                }
            }
        }
        Some(blocks)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
