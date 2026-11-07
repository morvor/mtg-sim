//! Restrictions on attack and block declarations as a whole (CR 508.1c, 509.1b) that
//! the restriction grammar compiles: "~ can only attack alone" (CR 506.5), "No more than
//! one creature can attack you each combat", "No more than one creature can attack ~ each
//! combat", "Each opponent can't block with more than one creature this combat".

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Filter::Custom`: the creature is goaded (CR 701.15).
pub const GOADED: &str = "restrictions:goaded";

/// `Filter::Custom`: the object's mana value is even / odd (CR 202.3).
pub const EVEN_MANA_VALUE: &str = "restrictions:even_mana_value";
pub const ODD_MANA_VALUE: &str = "restrictions:odd_mana_value";

pub struct CombatLimits;

impl KeywordRules for CombatLimits {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        match name {
            GOADED => Some(!g.goaders(id).is_empty()),
            EVEN_MANA_VALUE | ODD_MANA_VALUE => {
                let even = g.mana_value_of(id) % 2 == 0;
                Some(even == (name == EVEN_MANA_VALUE))
            }
            _ => None,
        }
    }

    /// "If a creature you control attacks, ~ also attacks if able" and "[creatures] attack
    /// a player other than you if able" (CR 508.1d).
    fn attack_requirements(&self, g: &Game) -> Vec<crate::combat::AttackRequirement> {
        use crate::combat::AttackRequirement;
        let mut out = Vec::new();
        let rs: Vec<_> = g
            .all_restrictions()
            .into_iter()
            .filter(|(_, _, r, _)| {
                matches!(
                    r,
                    Restriction::AttackTogether { .. } | Restriction::MustAttackOtherThan { .. }
                )
            })
            .collect();
        if rs.is_empty() {
            return out;
        }
        let players = crate::combat::attacking_players(g);
        let creatures: Vec<ObjectId> = g
            .permanents()
            .filter(|o| o.is_creature() && players.contains(&o.controller))
            .map(|o| o.id)
            .collect();
        for (s, c, r, locked) in rs {
            let ctx = Ctx::new(s, c);
            match &r {
                Restriction::AttackTogether {
                    attackers,
                    triggers,
                    same_controller,
                } => {
                    for &a in &creatures {
                        if !g.restriction_applies(a, attackers, &ctx, &locked) {
                            continue;
                        }
                        let ac = g.obj(a).controller;
                        let others: Vec<ObjectId> = creatures
                            .iter()
                            .copied()
                            .filter(|o| *o != a && g.matches(*o, triggers, &ctx))
                            .filter(|o| !same_controller || g.obj(*o).controller == ac)
                            .collect();
                        if !others.is_empty() {
                            out.push(AttackRequirement::AttacksIfAnyAttacks(a, others));
                        }
                    }
                }
                Restriction::MustAttackOtherThan { attackers, players } => {
                    let ps: Vec<PlayerId> = g
                        .player_ids()
                        .into_iter()
                        .filter(|p| g.player_filter_matches(players, *p, &ctx))
                        .collect();
                    for &a in &creatures {
                        if g.restriction_applies(a, attackers, &ctx, &locked) {
                            out.push(AttackRequirement::AttacksPlayerOtherThan(a, ps.clone()));
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }

    fn attack_declaration_ok(&self, g: &Game, decl: &[(ObjectId, Entity)]) -> bool {
        if decl.is_empty() {
            return true;
        }
        for (s, c, r, locked) in g.all_restrictions() {
            let ctx = Ctx::new(s, c);
            match &r {
                // CR 506.5: it attacks alone or not at all.
                Restriction::AttackOnlyAlone(f) => {
                    if decl.len() > 1
                        && decl
                            .iter()
                            .any(|(a, _)| g.restriction_applies(*a, f, &ctx, &locked))
                    {
                        return false;
                    }
                }
                Restriction::MaxAttackersAgainst { player, object, n } => {
                    let count = decl
                        .iter()
                        .filter(|(_, t)| match (t, player, object) {
                            (Entity::Player(p), Some(pf), _) => {
                                g.player_filter_matches(pf, *p, &ctx)
                            }
                            (Entity::Object(o), _, Some(f)) => g.matches(*o, f, &ctx),
                            _ => false,
                        })
                        .count();
                    if count > *n as usize {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }

    fn block_declaration_ok(
        &self,
        g: &Game,
        _options: &[(ObjectId, Vec<ObjectId>)],
        decl: &[(ObjectId, ObjectId)],
    ) -> bool {
        for (s, c, r, _) in g.all_restrictions() {
            let Restriction::MaxBlockersOf { who, n } = &r else {
                continue;
            };
            let ctx = Ctx::new(s, c);
            let mut blockers: Vec<ObjectId> = decl.iter().map(|(b, _)| *b).collect();
            blockers.sort();
            blockers.dedup();
            for p in g.player_ids() {
                if !g.player_filter_matches(who, p, &ctx) {
                    continue;
                }
                let k = blockers
                    .iter()
                    .filter(|b| g.obj(**b).controller == p)
                    .count();
                if k > *n as usize {
                    return false;
                }
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&CombatLimits) }
