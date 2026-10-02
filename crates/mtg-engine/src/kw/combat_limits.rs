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
