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

/// The general form: "search same name:{zones}:{any}:{object}:{var}[:{player}]" — the
/// zones searched (`ghl`: graveyard, hand, and library; `l`: library), `all` or `any`
/// (any number of cards), the object whose name it is (`t3`: the target in slot 3, `v65`:
/// the object in variable 65), the variable for the cards found, and the player whose
/// zones they are (JSON), when the text names one ("Search that player's graveyard, hand,
/// and library"): that player's zones are searched even when no object was chosen (the
/// Lobotomy and Shimian Specter rulings: with no card chosen, you still search the
/// library and that player shuffles). Without a player, the zones are those of the
/// object's controller (as it last existed; for a card, its owner).
const PREFIX_GENERAL: &str = "search same name:";

/// What a general same-name search searches for.
pub struct SameNameSearch {
    pub all_zones: bool,
    pub any_number: bool,
    pub object: Sel,
    pub var: Var,
    /// The player whose zones are searched, if the text names one.
    pub whose: Option<PlayerRef>,
}

impl SameNameSearch {
    /// The player whose zones are searched.
    pub fn player(&self) -> PlayerRef {
        self.whose
            .clone()
            .unwrap_or_else(|| PlayerRef::ControllerOf(Box::new(self.object.clone())))
    }
}

/// The instruction finding the cards named like `object` (a target or an object in a
/// variable) in the zones of `whose` (or of the object's controller); `None` for other
/// objects.
pub fn general_effect(
    all_zones: bool,
    any_number: bool,
    object: &Sel,
    var: Var,
    whose: Option<&PlayerRef>,
) -> Option<crate::ability::Effect> {
    let obj = match object {
        Sel::Target(n) => format!("t{n}"),
        Sel::Var(v) => format!("v{v}"),
        _ => return None,
    };
    let zones = if all_zones { "ghl" } else { "l" };
    let any = if any_number { "any" } else { "all" };
    let whose = match whose {
        Some(p) => format!(":{}", serde_json::to_string(p).ok()?),
        None => String::new(),
    };
    Some(crate::ability::Effect::Custom(
        format!("{PREFIX_GENERAL}{zones}:{any}:{obj}:{var}{whose}").into(),
    ))
}

/// The parts of a [`general_effect`] instruction.
pub fn parse_general(name: &str) -> Option<SameNameSearch> {
    let r = name.strip_prefix(PREFIX_GENERAL)?;
    let mut it = r.splitn(5, ':');
    let all_zones = match it.next()? {
        "ghl" => true,
        "l" => false,
        _ => return None,
    };
    let any_number = match it.next()? {
        "any" => true,
        "all" => false,
        _ => return None,
    };
    let obj = it.next()?;
    let object = if let Some(n) = obj.strip_prefix('t') {
        Sel::Target(n.parse().ok()?)
    } else {
        Sel::Var(obj.strip_prefix('v')?.parse().ok()?)
    };
    let var = it.next()?.parse().ok()?;
    let whose = match it.next() {
        Some(j) => Some(serde_json::from_str(j).ok()?),
        None => None,
    };
    Some(SameNameSearch {
        all_zones,
        any_number,
        object,
        var,
        whose,
    })
}

pub struct SearchSameName;

impl KeywordRules for SearchSameName {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let search = if let Some((slot, var)) = parse_search(name) {
            SameNameSearch {
                all_zones: true,
                any_number: name.starts_with(PREFIX_ANY),
                object: Sel::Target(slot),
                var,
                whose: None,
            }
        } else if let Some(s) = parse_general(name) {
            s
        } else {
            return false;
        };
        run(g, &search, ctx);
        true
    }
}

/// Searches the zones of the controller of `search.object` for cards with its name
/// (CR 701.23, 201.2), storing the cards found in `search.var`.
fn run(g: &mut Game, search: &SameNameSearch, ctx: &mut Ctx) {
    {
        let SameNameSearch {
            all_zones,
            any_number,
            ref object,
            var,
            ..
        } = *search;
        let searcher = ctx.controller;
        let target = object.clone();
        let Some(owner) = g.eval_player(&search.player(), ctx) else {
            ctx.set_var(var, vec![]);
            return;
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
        let (in_graveyard, in_hand) = if all_zones {
            (
                named(g, &g.player(owner).graveyard.clone(), ctx),
                named(g, &g.player(owner).hand.clone(), ctx),
            )
        } else {
            (vec![], vec![])
        };
        let mut found = if any_number {
            vec![]
        } else {
            in_graveyard.clone()
        };
        // The hand (and the graveyard for "any number"): the searcher chooses which to
        // find (all by default).
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
    }
}

inventory::submit! { KeywordRegistration(&SearchSameName) }
