//! "Exile target card that's an instant or sorcery from your graveyard. For each card
//! exiled this way, copy it, and you may cast the copy without paying its mana cost."
//! (Mizzix's Mastery; overloaded, "each card"): each card the previous instruction exiled
//! is copied in exile (CR 707.12) and the copy may be cast while the spell resolves, one
//! at a time in the order the player chooses; a copy that isn't cast ceases to exist
//! (CR 707.12a, 704.5e).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// The variable bound to each exiled card in turn.
const EACH_EXILED: Var = vars::USER + 2612;

/// Whether the effect ends by exiling cards face up.
fn ends_with_exiling(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            face_down: false, ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_exiling),
        _ => false,
    }
}

fn copy_each_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("for each card exiled this way, copy it, and ") else {
        return false;
    };
    let (optional, free) = match r {
        "you may cast the copy without paying its mana cost" => (true, true),
        "you may cast the copy" => (true, false),
        _ => return false,
    };
    if !ends_with_exiling(prev) {
        return false;
    }
    let each = Effect::ForEach {
        sel: Sel::Var(vars::IT),
        var: EACH_EXILED,
        effect: Box::new(Effect::seq(vec![
            Effect::CopyCard {
                what: Sel::Var(EACH_EXILED),
                named: None,
            },
            Effect::CastCard {
                who: PlayerRef::You,
                what: Sel::Var(vars::CREATED),
                free,
                optional,
            },
        ])),
    };
    // Several copies are cast one at a time, in the order the player chooses.
    let order = Effect::Custom(crate::copy_rules::ORDER_AFFECTED.into());
    *prev = Effect::seq(vec![std::mem::take(prev), order, each]);
    true
}

inventory::submit! { FollowupPattern { name: "for each card exiled this way, copy it, and you may cast the copy", priority: 90, apply: copy_each_exiled } }

/// "exile target card that's an instant or sorcery from your graveyard": "card that's
/// [types]" is "[types] card".
fn exile_target_card_thats(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile target card that's ")?;
    let r = r.strip_prefix("an ").or_else(|| r.strip_prefix("a "))?;
    let (types, rest) = r.split_once(" from ")?;
    if !types
        .split(' ')
        .all(|w| matches!(w, "or" | "instant" | "sorcery" | "creature" | "artifact" | "enchantment" | "land" | "planeswalker"))
    {
        return None;
    }
    let e = parse_clause(&format!("exile target {types} card from {rest}"), b)?;
    ends_with_exiling(&e).then_some(e)
}

inventory::submit! { EffectPattern { name: "exile target card that's [types] from [zone]", priority: 90, parse: exile_target_card_thats } }
