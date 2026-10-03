//! "Search its controller's graveyard, hand, and library for all cards with the same name
//! as that creature and exile them." (Eradicate, Splinter, Counterbore, ...): the
//! `Effect::Custom` instruction that finds those cards (parsed in
//! `oracle/patterns/search_zones_same_name.rs`).
//!
//! The name is the targeted object's as it last existed (CR 201.2, 608.2h). In the
//! graveyard, a public zone, every card with that name is found; in the hand and the
//! library, hidden zones, the searching player may leave some or all of them (CR 701.23b),
//! automated agents finding all of them by default. The library search is a search of
//! that library (CR 701.23f: a replaced or restricted search, "searches" triggers). The
//! cards found are stored in a variable for the instruction that exiles them.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{Filter, PlayerRef, Sel, Var};
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::{Entity, ObjectId};

const PREFIX: &str = "search graveyard hand library same name:";
/// "for any number of cards with the same name": the searcher may leave cards in the
/// graveyard too.
const PREFIX_ANY: &str = "search graveyard hand library any same name:";

/// The instruction finding the cards named like the target in `slot`, stored in `var`.
pub fn search_effect(slot: u8, var: Var) -> crate::ability::Effect {
    crate::ability::Effect::Custom(format!("{PREFIX}{slot}:{var}").into())
}

/// Like [`search_effect`], for "any number of cards with the same name": the searching
/// player chooses which of the cards in the graveyard to find as well.
pub fn search_any_effect(slot: u8, var: Var) -> crate::ability::Effect {
    crate::ability::Effect::Custom(format!("{PREFIX_ANY}{slot}:{var}").into())
}

/// The target slot and variable of a [`search_effect`] or [`search_any_effect`]
/// instruction.
pub fn parse_search(name: &str) -> Option<(u8, Var)> {
    let (slot, var) = name
        .strip_prefix(PREFIX)
        .or_else(|| name.strip_prefix(PREFIX_ANY))?
        .split_once(':')?;
    Some((slot.parse().ok()?, var.parse().ok()?))
}

pub struct SearchSameName;

impl KeywordRules for SearchSameName {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((slot, var)) = parse_search(name) else {
            return false;
        };
        let any_number = name.starts_with(PREFIX_ANY);
        let searcher = ctx.controller;
        let target = Sel::Target(slot);
        let Some(owner) = g.eval_player(&PlayerRef::ControllerOf(Box::new(target.clone())), ctx)
        else {
            ctx.set_var(var, vec![]);
            return true;
        };
        let filter = Filter::and(vec![Filter::Card, Filter::SameNameAs(Box::new(target))]);
        let named = |g: &Game, zone: &[ObjectId], ctx: &Ctx| -> Vec<ObjectId> {
            zone.iter()
                .copied()
                .filter(|c| g.matches(*c, &filter, ctx))
                .collect()
        };
        // The graveyard: every such card ("all cards"), or those the searcher chooses
        // ("any number of cards").
        let in_graveyard = named(g, &g.player(owner).graveyard.clone(), ctx);
        let mut found = if any_number {
            vec![]
        } else {
            in_graveyard.clone()
        };
        // The hand (and the graveyard for "any number"): the searcher chooses which to
        // find (all by default).
        let in_hand = named(g, &g.player(owner).hand.clone(), ctx);
        for cands in [
            if any_number { in_graveyard } else { vec![] },
            in_hand,
        ] {
            if cands.is_empty() {
                continue;
            }
            let ans = g.ask(
                searcher,
                Decision::ChooseEntities {
                    source: ctx.source,
                    prompt: "Search: choose cards".into(),
                    candidates: cands.iter().map(|c| Entity::Object(*c)).collect(),
                    min: 0,
                    max: cands.len() as u32,
                },
            );
            let chosen: Option<Vec<ObjectId>> = match ans {
                Answer::Entities(v) => {
                    let objs: Vec<ObjectId> = v.iter().filter_map(|e| e.object()).collect();
                    let mut uniq = objs.clone();
                    uniq.sort();
                    uniq.dedup();
                    (objs.len() == v.len()
                        && uniq.len() == objs.len()
                        && objs.iter().all(|o| cands.contains(o)))
                    .then_some(objs)
                }
                _ => None,
            };
            found.extend(match chosen {
                Some(v) => v,
                // Default (or invalid) answers: automated agents prefer finding cards.
                None if g.search_finds_by_default => cands,
                None => vec![],
            });
        }
        // The library.
        found.extend(crate::library::search(
            g,
            searcher,
            owner,
            &filter,
            u32::MAX,
            ctx,
        ));
        ctx.set_var(var, found.into_iter().map(Entity::Object).collect());
        true
    }
}

inventory::submit! { KeywordRegistration(&SearchSameName) }
