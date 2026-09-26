//! Keyword actions (CR 701) implemented in code.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

pub fn perform(
    g: &mut Game,
    action: KeywordAction,
    who: &PlayerRef,
    what: &Sel,
    n: &Value,
    ctx: &mut Ctx,
) {
    let players = g.eval_players(who, ctx);
    let objs = g.resolve_objects(what, ctx);
    let k = g.eval_value(n, ctx).max(0) as u32;
    match action {
        // CR 701.16a: create a Clue token.
        KeywordAction::Investigate => {
            for p in players {
                for _ in 0..k.max(1) {
                    if let Some(spec) = crate::tokens::predefined("Clue") {
                        let chars = crate::tokens::token_characteristics(&spec);
                        g.create_tokens(
                            p,
                            crate::replacement::TokenCreate {
                                chars,
                                card: None,
                                tapped: false,
                                attacking: None,
                                copy_of: None,
                                copy_exceptions: vec![],
                            },
                            1,
                            ctx.source,
                        );
                    }
                }
            }
        }
        // CR 701.34a: proliferate.
        KeywordAction::Proliferate => {
            for p in players {
                crate::keyword_actions_impl::proliferate(g, p, ctx);
            }
        }
        // CR 701.4a: behold a [quality].
        KeywordAction::Behold => {
            let quality = crate::behold::quality(what);
            for p in players {
                crate::behold::behold(g, p, &quality, k.max(1), ctx);
            }
        }
        other => crate::keyword_actions_impl::perform(g, other, &players, &objs, k, ctx),
    }
}

/// CR 701.59a: collect evidence N — exile cards with total mana value N or greater from
/// your graveyard. Returns false if impossible.
pub fn collect_evidence(g: &mut Game, p: PlayerId, n: u32, src: Option<ObjectId>) -> bool {
    let gy = g.player(p).graveyard.clone();
    let total: u32 = gy.iter().map(|c| g.mana_value_of(*c)).sum();
    if total < n {
        return false;
    }
    // Choose cards greedily by highest mana value unless the player chooses.
    let mut sorted = gy.clone();
    sorted.sort_by_key(|c| std::cmp::Reverse(g.mana_value_of(*c)));
    let mut chosen = Vec::new();
    let mut sum = 0;
    for c in sorted {
        if sum >= n {
            break;
        }
        sum += g.mana_value_of(c);
        chosen.push(c);
    }
    for c in chosen {
        g.exile_object(c, src);
    }
    true
}

/// CR 701.61a: forage — exile three cards from your graveyard or sacrifice a Food.
pub fn forage(g: &mut Game, p: PlayerId, src: Option<ObjectId>) -> bool {
    let foods: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Food"))
        .map(|o| o.id)
        .collect();
    let gy = g.player(p).graveyard.clone();
    let can_exile = gy.len() >= 3;
    if !can_exile && foods.is_empty() {
        return false;
    }
    let use_food = if can_exile && !foods.is_empty() {
        g.ask_option(
            p,
            src,
            "Forage",
            vec![
                "Exile three cards from your graveyard".into(),
                "Sacrifice a Food".into(),
            ],
        ) == 1
    } else {
        !can_exile
    };
    if use_food {
        let f = g.ask_objects(p, src, "Choose a Food to sacrifice", foods, 1, 1);
        if let Some(f) = f.first() {
            g.sacrifice(*f, p);
        }
    } else {
        let pick = g.ask_objects(p, src, "Choose three cards to exile", gy, 3, 3);
        for c in pick {
            g.exile_object(c, src);
        }
    }
    true
}
