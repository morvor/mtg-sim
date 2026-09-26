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
    // Keyword actions implemented in the `kwa/` registry.
    if crate::kwa::perform(g, action, who, what, n, ctx) {
        return;
    }
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
/// your graveyard. Returns false if impossible. See `kwa/evidence_forage.rs`.
pub fn collect_evidence(g: &mut Game, p: PlayerId, n: u32, src: Option<ObjectId>) -> bool {
    crate::kwa::evidence_forage::collect_evidence(g, p, n, src)
}

/// CR 701.61a: forage — exile three cards from your graveyard or sacrifice a Food.
/// Returns false if impossible. See `kwa/evidence_forage.rs`.
pub fn forage(g: &mut Game, p: PlayerId, src: Option<ObjectId>) -> bool {
    crate::kwa::evidence_forage::forage(g, p, src)
}
