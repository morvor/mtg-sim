//! CR 700.3: separating objects into piles.
//!
//! [`PileAction::Separate`] has a player separate objects into two piles, bound to
//! [`PILE_A`] and [`PILE_B`]; [`PileAction::Choose`] has a player choose one of them, bound
//! to [`CHOSEN`] (the other to [`OTHER`]) for the instructions that follow ("Put that pile
//! into your hand and the other into your graveyard").
//!
//! * Each object is put into exactly one pile (CR 700.3a) and remains an individual
//!   object; a pile isn't an object (CR 700.3b).
//! * Objects grouped into piles don't leave their zone, and cards in a graveyard keep
//!   their order (CR 700.3c).
//! * A pile can be empty (CR 700.3d).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// The first pile.
pub const PILE_A: Var = vars::USER + 700;
/// The second pile.
pub const PILE_B: Var = vars::USER + 701;
/// The pile that was chosen.
pub const CHOSEN: Var = vars::USER + 702;
/// The pile that wasn't chosen.
pub const OTHER: Var = vars::USER + 703;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PileAction {
    /// `separator` separates the selected objects into two piles.
    Separate { what: Sel, separator: PlayerRef },
    /// `chooser` chooses one of the two piles.
    Choose { chooser: PlayerRef },
}

/// The one player a pile instruction refers to. "An opponent" (several players) means the
/// opponent of the controller's choice.
fn one_player(g: &mut Game, who: &PlayerRef, ctx: &Ctx) -> PlayerId {
    let cands = g.eval_players(who, ctx);
    match cands.len() {
        0 => ctx.controller,
        1 => cands[0],
        _ => g
            .ask_entities(
                ctx.controller,
                ctx.source,
                "Choose a player",
                cands.iter().map(|p| Entity::Player(*p)).collect(),
                1,
                1,
            )
            .first()
            .and_then(|e| e.player())
            .unwrap_or(cands[0]),
    }
}

fn describe(g: &Game, pile: &[ObjectId]) -> String {
    if pile.is_empty() {
        return "(empty)".into();
    }
    pile.iter()
        .map(|o| g.obj(*o).chars.name.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn pile(ctx: &Ctx, var: Var) -> Vec<ObjectId> {
    ctx.vars
        .get(&var)
        .map(|v| v.iter().filter_map(|e| e.object()).collect())
        .unwrap_or_default()
}

fn set_pile(ctx: &mut Ctx, var: Var, objs: &[ObjectId]) {
    ctx.set_var(var, objs.iter().map(|o| Entity::Object(*o)).collect());
}

pub fn perform(g: &mut Game, action: &PileAction, ctx: &mut Ctx) {
    match action {
        PileAction::Separate { what, separator } => {
            let objs: Vec<ObjectId> = g
                .resolve_objects(what, ctx)
                .into_iter()
                .filter(|o| g.is_live(*o))
                .collect();
            let p = one_player(g, separator, ctx);
            let n = objs.len() as u32;
            // The separator chooses the first pile; everything else is the second
            // (CR 700.3a); either may be empty (CR 700.3d).
            let a = g.ask_objects(
                p,
                ctx.source,
                "Separate into two piles: choose the objects in the first pile",
                objs.clone(),
                0,
                n,
            );
            let b: Vec<ObjectId> = objs.iter().copied().filter(|o| !a.contains(o)).collect();
            // Piles list their objects in the order they're in (CR 700.3c).
            let a: Vec<ObjectId> = objs.iter().copied().filter(|o| a.contains(o)).collect();
            g.log(|g| {
                format!(
                    "{p} separates cards into piles: [{}] and [{}]",
                    describe(g, &a),
                    describe(g, &b)
                )
            });
            set_pile(ctx, PILE_A, &a);
            set_pile(ctx, PILE_B, &b);
        }
        PileAction::Choose { chooser } => {
            let a = pile(ctx, PILE_A);
            let b = pile(ctx, PILE_B);
            let p = one_player(g, chooser, ctx);
            let i = g.ask_option(
                p,
                ctx.source,
                "Choose a pile",
                vec![
                    format!("Pile 1: {}", describe(g, &a)),
                    format!("Pile 2: {}", describe(g, &b)),
                ],
            );
            let (chosen, other) = if i == 1 { (b, a) } else { (a, b) };
            g.log(|g| format!("{p} chooses the pile [{}]", describe(g, &chosen)));
            set_pile(ctx, CHOSEN, &chosen);
            set_pile(ctx, OTHER, &other);
            ctx.set_var(
                vars::IT,
                chosen.iter().map(|o| Entity::Object(*o)).collect(),
            );
        }
    }
}
