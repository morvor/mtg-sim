//! Damping Engine: "A player who controls more permanents than each other player can't
//! play lands or cast artifact, creature, or enchantment spells. That player may sacrifice
//! a permanent of their choice for that player to ignore this effect until end of turn."
//!
//! The sacrifice is a special action (CR 116.2d) only the affected player can take, once
//! each turn (rulings).

use super::{active, marker, ManualAbility};
use crate::ability::{Cost, CostPart, Filter, Value};
use crate::casting::Illegal;
use crate::decision::{Action, SpecialAction};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Characteristics;
use crate::special_actions::ignores;
use crate::types::{CardType, ObjectId, PlayerId};

const DAMPING: &str = "card:Damping Engine:the player with the most permanents can't play lands or cast permanent spells";
const IGNORE: &str = "card:Damping Engine:sacrifice a permanent to ignore it";
const TEXT: &str = "A player who controls more permanents than each other player can't play lands or cast artifact, creature, or enchantment spells. That player may sacrifice a permanent of their choice for that player to ignore this effect until end of turn.";

inventory::submit! { ManualAbility {
    card: "Damping Engine",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(DAMPING, TEXT)],
    reason: "the player with the most permanents can't play lands or cast permanent spells unless they sacrifice: unique",
} }

/// Whether `p` controls more permanents than each other player.
fn most_permanents(g: &Game, p: PlayerId) -> bool {
    let n = |q: PlayerId| g.permanents_controlled_by(q).len();
    let mine = n(p);
    g.players_in_game()
        .into_iter()
        .filter(|q| *q != p)
        .all(|q| n(q) < mine)
}

/// The Damping Engines whose effect applies to `p`.
fn dampers(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    if !most_permanents(g, p) {
        return vec![];
    }
    active(g, DAMPING)
        .into_iter()
        .map(|(s, _)| s)
        .filter(|s| !ignores(g, *s, p))
        .collect()
}

fn sacrifice_cost() -> Cost {
    Cost::free().with(CostPart::Sacrifice {
        filter: Filter::Permanent,
        count: Value::Const(1),
    })
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn cast_prohibited(
        &self,
        g: &Game,
        p: PlayerId,
        _card: ObjectId,
        chars: &Characteristics,
    ) -> bool {
        (chars.card_types.contains(CardType::Artifact)
            || chars.card_types.contains(CardType::Creature)
            || chars.card_types.contains(CardType::Enchantment))
            && !dampers(g, p).is_empty()
    }
    fn land_play_prohibited(&self, g: &Game, p: PlayerId, _card: ObjectId) -> bool {
        !dampers(g, p).is_empty()
    }
    fn special_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        if !g.has_priority(p) {
            return vec![];
        }
        dampers(g, p)
            .into_iter()
            .filter(|s| g.can_pay_cost(p, &sacrifice_cost(), Some(*s), &Ctx::new(Some(*s), p)))
            .map(|s| {
                Action::Special(SpecialAction::Other {
                    name: IGNORE.to_string(),
                    obj: Some(s),
                })
            })
            .collect()
    }
    fn perform_special_action(
        &self,
        g: &mut Game,
        p: PlayerId,
        sa: &SpecialAction,
    ) -> Option<Result<(), Illegal>> {
        let SpecialAction::Other {
            name,
            obj: Some(src),
        } = sa
        else {
            return None;
        };
        if name != IGNORE {
            return None;
        }
        if !dampers(g, p).contains(src) {
            return Some(Err(Illegal("that effect doesn't apply to you".into())));
        }
        let ctx = Ctx::new(Some(*src), p);
        if !crate::special_actions::pay(g, p, &sacrifice_cost(), Some(*src), &ctx) {
            return Some(Err(Illegal("can't pay the cost".into())));
        }
        let turn = g.turn.number;
        g.special.ignoring.push((*src, p, turn));
        g.dirty = true;
        Some(Ok(()))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
