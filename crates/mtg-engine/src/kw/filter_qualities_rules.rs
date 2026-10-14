//! Object qualities, values and conditions of the filter and condition grammar
//! (`oracle/patterns/filter_qualities.rs`, `conditions_compare.rs`) implemented in code.
//! No keyword: only the registry-wide `custom_*` hooks.

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::oracle::patterns::conditions_compare::PLAYERS_LOST_GAME;
use crate::types::*;

pub struct FilterQualities;

impl KeywordRules for FilterQualities {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_value(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<i64> {
        match name {
            // CR 800.4: players who have lost the game (or left it).
            PLAYERS_LOST_GAME => Some(
                g.player_ids()
                    .into_iter()
                    .filter(|p| {
                        let pl = g.player(*p);
                        pl.has_lost || pl.left_game
                    })
                    .count() as i64,
            ),
            _ => None,
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        use crate::oracle::patterns::filter_qualities as q;
        let _ = ctx;
        let c = &g.obj(id).chars;
        match name {
            // CR 201.2a, 708.2a: a face-down permanent has no name.
            q::NO_NAME => Some(!c.has_a_name()),
            // An activated ability (mana abilities included) with {T} in its cost.
            q::TAP_ABILITY => Some(c.abilities.iter().any(|a| {
                matches!(&a.kind, crate::ability::AbilityKind::Activated(x) if x.cost.has_tap())
            })),
            // CR 205.2a.
            q::TWO_CARD_TYPES => Some(c.card_types.count() >= 2),
            q::MOST_COMMON_COLOR => {
                let mut counts = [0usize; 5];
                for p in g.permanents() {
                    let colors = p.chars.colors;
                    for (i, col) in Color::ALL.iter().enumerate() {
                        if colors.contains(*col) {
                            counts[i] += 1;
                        }
                    }
                }
                let most = counts.iter().copied().max().unwrap_or(0);
                Some(
                    most > 0
                        && Color::ALL
                            .iter()
                            .enumerate()
                            .any(|(i, col)| counts[i] == most && c.colors.contains(*col)),
                )
            }
            _ => None,
        }
    }
}

inventory::submit! { KeywordRegistration(&FilterQualities) }
