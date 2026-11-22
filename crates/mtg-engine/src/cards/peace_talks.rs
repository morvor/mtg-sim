//! Peace Talks: "This turn and next turn, creatures can't attack, and players and
//! permanents can't be the targets of spells or activated abilities."
//!
//! "Next turn" is the one turn after this one, whoever's it is (rulings).

use super::{spell, ManualAbility};
use crate::ability::{Effect, TargetSpec};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{StackKind, Zone};
use crate::types::{Entity, ObjectId};

const RESOLVE: &str = "card:Peace Talks:no attacks and no targeting this turn and next turn";
/// Rows `[turn]`: the turns Peace Talks resolved in.
const TURNS: &str = "card:Peace Talks:turns";
const TEXT: &str = "This turn and next turn, creatures can't attack, and players and permanents can't be the targets of spells or activated abilities.";

inventory::submit! { ManualAbility {
    card: "Peace Talks",
    face: 0,
    text: TEXT,
    build: |_| vec![spell(vec![], Effect::Custom(RESOLVE.into()), TEXT)],
    reason: "two-turn duration ('this turn and next turn') for attack and targeting bans: unique",
} }

fn in_effect(g: &Game) -> bool {
    let now = g.turn.number as i64;
    g.cards
        .get(TURNS)
        .iter()
        .any(|r| r[0] == now || r[0] + 1 == now)
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, _ctx: &mut Ctx) -> bool {
        if name != RESOLVE {
            return false;
        }
        let now = g.turn.number as i64;
        g.cards.push(TURNS, vec![now]);
        true
    }
    fn attack_declaration_ok(&self, g: &Game, decl: &[(ObjectId, Entity)]) -> bool {
        decl.is_empty() || !in_effect(g)
    }
    /// Spells and activated abilities (not triggered abilities) can't target players or
    /// permanents.
    fn target_forbidden(
        &self,
        g: &Game,
        _spec: &TargetSpec,
        e: Entity,
        source: Option<ObjectId>,
    ) -> bool {
        if !in_effect(g) {
            return false;
        }
        let Some(s) = source else {
            return false;
        };
        let by_spell_or_activated = match g.obj(s).stack.as_deref() {
            Some(si) => !matches!(si.kind, StackKind::Triggered { .. }),
            None => false,
        };
        by_spell_or_activated
            && match e {
                Entity::Player(_) => true,
                Entity::Object(o) => g.obj(o).zone == Zone::Battlefield,
            }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
