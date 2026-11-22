//! Combat restrictions and requirements on the declaration as a whole (CR 508.1c-d,
//! 509.1b-c):
//!
//! * `Restriction::AttackOnlyWith` / `BlockOnlyWith`: "~ can't attack unless a black or
//!   green creature also attacks", "~ can't block unless a creature with greater power
//!   also blocks" — another creature matching the companion filter (relative to the
//!   source) attacks or blocks too (Scarred Puma ruling: "other than itself").
//! * `Restriction::PlayerMustAttack`: "Each opponent must attack [you or a planeswalker
//!   you control] with at least one creature each combat if able" — one requirement on
//!   the attacking player's declaration (Trove of Temptation rulings: one creature total;
//!   creatures that can't attack or would need a cost paid don't have to; attacking a
//!   teammate doesn't satisfy it).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::combat::AttackRequirement;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

pub struct CombatRequirements2;

/// Whether each declared creature under a companion restriction has a companion among the
/// other declared creatures.
fn companions_ok(
    g: &Game,
    declared: &[ObjectId],
    pick: impl Fn(&Restriction) -> Option<(&Filter, &Filter)>,
) -> bool {
    for (s, c, r, locked) in g.all_restrictions() {
        let Some((who, companion)) = pick(&r) else {
            continue;
        };
        let ctx = Ctx::new(s, c);
        for a in declared {
            if g.restriction_applies(*a, who, &ctx, &locked)
                && !declared
                    .iter()
                    .any(|o| o != a && g.matches(*o, companion, &ctx))
            {
                return false;
            }
        }
    }
    true
}

impl KeywordRules for CombatRequirements2 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn attack_requirements(&self, g: &Game) -> Vec<AttackRequirement> {
        let mut out = Vec::new();
        let attacking = crate::combat::attacking_players(g);
        for (s, c, r, _) in g.all_restrictions() {
            let Restriction::PlayerMustAttack {
                players,
                defender,
                planeswalkers,
            } = &r
            else {
                continue;
            };
            let ctx = Ctx::new(s, c);
            for p in &attacking {
                if !g.player_filter_matches(players, *p, &ctx) {
                    continue;
                }
                let targets = defender.as_ref().map(|d| {
                    crate::combat::attack_targets(g)
                        .into_iter()
                        .filter(|t| match t {
                            Entity::Player(q) => g.player_filter_matches(d, *q, &ctx),
                            Entity::Object(o) => {
                                *planeswalkers
                                    && g.obj(*o).is(CardType::Planeswalker)
                                    && g.player_filter_matches(d, g.obj(*o).controller, &ctx)
                            }
                        })
                        .collect::<Vec<_>>()
                });
                out.push(AttackRequirement::AttacksWithAny {
                    player: *p,
                    targets,
                });
            }
        }
        out
    }

    fn attack_declaration_ok(&self, g: &Game, decl: &[(ObjectId, Entity)]) -> bool {
        let declared: Vec<ObjectId> = decl.iter().map(|(a, _)| *a).collect();
        companions_ok(g, &declared, |r| match r {
            Restriction::AttackOnlyWith {
                attackers,
                companion,
            } => Some((attackers, companion)),
            _ => None,
        })
    }

    fn block_declaration_ok(
        &self,
        g: &Game,
        _options: &[(ObjectId, Vec<ObjectId>)],
        decl: &[(ObjectId, ObjectId)],
    ) -> bool {
        let mut declared: Vec<ObjectId> = decl.iter().map(|(b, _)| *b).collect();
        declared.sort();
        declared.dedup();
        companions_ok(g, &declared, |r| match r {
            Restriction::BlockOnlyWith {
                blockers,
                companion,
            } => Some((blockers, companion)),
            _ => None,
        })
    }
}

inventory::submit! { KeywordRegistration(&CombatRequirements2) }
