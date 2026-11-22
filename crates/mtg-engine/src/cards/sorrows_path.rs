//! Sorrow's Path: "{T}: Choose two target blocking creatures controlled by the same
//! opponent. If each of those creatures could block all creatures that the other is
//! blocking, remove both of them from combat. Each one then blocks all creatures the other
//! was blocking." The targets are the compiled ones; the swap is hand-written (CR 506.4,
//! 509.1). Whether each could block the other's attackers takes evasion, protection,
//! blocking restrictions, how many creatures it can block and being tapped into account,
//! not costs to block; a creature whose attackers have all left combat is "able" to block
//! all of them, and after the swap it blocks nothing (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, Sel};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const TEXT: &str = "{T}: Choose two target blocking creatures controlled by the same opponent. If each of those creatures could block all creatures that the other is blocking, remove both of them from combat. Each one then blocks all creatures the other was blocking.";
const SWAP: &str = "card:Sorrow's Path:the two blockers swap what they block";

inventory::submit! { ManualAbility {
    card: "Sorrow's Path",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{T}: Choose two target blocking creatures controlled by the same opponent. Tap them.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(SWAP.into())))
            .collect()
    },
    reason: "swaps what two blockers block: only this card",
} }

/// The creatures still attacking that `blocker` is blocking.
fn blocked_by(g: &Game, blocker: ObjectId) -> Vec<ObjectId> {
    g.combat
        .as_ref()
        .map(|c| c.blocking(blocker))
        .unwrap_or_default()
        .into_iter()
        .filter(|a| g.is_attacking(*a))
        .collect()
}

/// Whether `blocker` could block all of `attackers` (CR 509.1b, 509.1a).
fn could_block_all(g: &Game, blocker: ObjectId, attackers: &[ObjectId]) -> bool {
    if attackers.is_empty() {
        return true;
    }
    g.max_blocks(blocker)
        .is_none_or(|n| n as usize >= attackers.len())
        && attackers.iter().all(|a| g.can_block(blocker, *a))
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != SWAP {
            return false;
        }
        if g.dirty {
            g.recompute();
        }
        let both: Vec<ObjectId> = g
            .resolve_sel(&Sel::Target(0), ctx)
            .into_iter()
            .filter_map(|e| e.object())
            .filter(|o| {
                g.is_live(*o) && g.is_blocking(*o)
            })
            .collect();
        let [x, y] = both[..] else {
            return true;
        };
        let (xs, ys) = (blocked_by(g, x), blocked_by(g, y));
        if !could_block_all(g, x, &ys) || !could_block_all(g, y, &xs) {
            return true;
        }
        crate::combat::remove_from_combat(g, x);
        crate::combat::remove_from_combat(g, y);
        for a in ys {
            crate::combat::block_by_effect(g, x, a);
        }
        for a in xs {
            crate::combat::block_by_effect(g, y, a);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
