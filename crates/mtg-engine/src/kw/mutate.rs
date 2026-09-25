//! Mutate (CR 702.140): "You may pay [cost] rather than pay this spell's mana cost. If
//! you do, it becomes a mutating creature spell and targets a non-Human creature with the
//! same owner as this spell." A mutating creature spell whose target is legal as it
//! resolves merges with it (CR 702.140c, 730.2); otherwise it resolves as a creature
//! spell (CR 702.140b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;

pub struct Mutate;

/// Whether `spell` is on the stack cast for its mutate cost.
fn cast_mutating(g: &Game, spell: ObjectId) -> bool {
    let o = g.obj(spell);
    o.zone == Zone::Stack
        && o.stack
            .as_deref()
            .is_some_and(|si| matches!(si.cast.method, CastMethod::Keyword(KeywordKind::Mutate)))
}

/// The mutating spell's target: a non-Human creature with the same owner as the spell.
fn target_spec(g: &Game, spell: ObjectId) -> TargetSpec {
    let owner = g.obj(spell).owner;
    let owned: Vec<ObjectId> = g
        .battlefield
        .iter()
        .copied()
        .filter(|o| g.obj(*o).owner == owner)
        .collect();
    TargetSpec::object(
        Filter::And(vec![
            Filter::Type(CardType::Creature),
            Filter::Not(Box::new(Filter::Subtype("Human".into()))),
            Filter::Objects(owned),
        ]),
        "target non-Human creature with the same owner as this spell",
    )
}

/// The creature a mutating spell targets, if it's still a legal target.
fn legal_target(g: &Game, spell: ObjectId) -> Option<ObjectId> {
    let si = g.obj(spell).stack.as_deref()?;
    let t = *si.chosen.first()?.targets.first()?.first()?;
    let Entity::Object(t) = t else {
        return None;
    };
    let ctx = g.stack_ctx(spell);
    g.is_legal_target(&target_spec(g, spell), Entity::Object(t), &ctx, spell)
        .then_some(t)
}

impl KeywordRules for Mutate {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Mutate]
    }

    /// CR 702.140a: an alternative cost, from wherever the card could be cast.
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        if o.zone != Zone::Hand(p) && !g.permitted_cards(p).contains(&card) {
            return vec![];
        }
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Mutate);
        opt.alt_cost = Some(cost);
        opt.tag = Some("mutate");
        vec![opt]
    }

    /// A mutating creature spell targets a non-Human creature with the same owner.
    fn adjust_spell_body(&self, g: &Game, spell: ObjectId, _kw: &Keyword, mut body: Body) -> Body {
        if cast_mutating(g, spell) {
            body.targets.insert(0, target_spec(g, spell));
        }
        body
    }

    /// CR 702.140b: with an illegal target, it's no longer a mutating creature spell.
    fn is_mutating(&self, g: &Game, spell: ObjectId) -> bool {
        cast_mutating(g, spell) && legal_target(g, spell).is_some()
    }

    fn resolve_mutate(&self, g: &mut Game, spell: ObjectId) -> bool {
        if !cast_mutating(g, spell) {
            return false;
        }
        let Some(target) = legal_target(g, spell) else {
            return false;
        };
        crate::merge::mutate(g, spell, target);
        true
    }
}

inventory::submit! { KeywordRegistration(&Mutate) }
