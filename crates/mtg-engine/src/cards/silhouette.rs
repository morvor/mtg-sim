//! Silhouette: "Choose target creature. If a spell or ability that targets that creature
//! would cause a source to deal damage to that creature this turn, prevent that damage."
//! (CR 615).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, Sel};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules, KeywordShield};
use crate::types::{Entity, ObjectId};

const CHOOSE: &str = "card:Silhouette:protect the creature from targeted damage";
/// Rows `[creature, turn, controller, silhouette]`.
const SHIELDED: &str = "card:Silhouette:shielded creatures";
const TEXT: &str = "Choose target creature. If a spell or ability that targets that creature would cause a source to deal damage to that creature this turn, prevent that damage.";

inventory::submit! { ManualAbility {
    card: "Silhouette",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Choose target creature. Prevent all damage that would be dealt to that creature this turn.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(CHOOSE.into())))
            .collect()
    },
    reason: "prevents damage caused by spells and abilities that target the creature: unique",
} }

struct Rules;

/// The spell or ability that's resolving, if any (it stays on the stack while it
/// resolves, CR 608.2).
fn resolving(g: &Game) -> Option<ObjectId> {
    if g.turn.priority.is_some() {
        return None;
    }
    g.stack.last().copied()
}

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CHOOSE {
            return false;
        }
        let turn = g.turn.number as i64;
        let me = ctx.source.map_or(-1, |s| s.0 as i64);
        for o in g.eval_sel_objects(&Sel::Target(0), ctx) {
            g.cards.push(
                SHIELDED,
                vec![o.0 as i64, turn, ctx.controller.0 as i64, me],
            );
        }
        true
    }
    fn damage_prevention(&self, g: &Game, _source: ObjectId, target: Entity) -> Vec<KeywordShield> {
        let Entity::Object(o) = target else {
            return vec![];
        };
        let turn = g.turn.number as i64;
        let Some(row) = g
            .cards
            .get(SHIELDED)
            .iter()
            .find(|r| r[0] == o.0 as i64 && r[1] == turn)
        else {
            return vec![];
        };
        let Some(by) = resolving(g) else {
            return vec![];
        };
        let targets_it = g.obj(by).stack.as_deref().is_some_and(|si| {
            si.chosen
                .iter()
                .any(|c| c.targets.iter().flatten().any(|e| *e == target))
        });
        if !targets_it {
            return vec![];
        }
        vec![KeywordShield {
            holder: target,
            id: row[3] as u64,
            controller: crate::types::PlayerId(row[2] as u8),
            text: "Silhouette: prevent damage from a spell or ability that targets it".into(),
        }]
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
