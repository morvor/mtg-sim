//! Kinship (an ability word, CR 207.2c): "At the beginning of your upkeep, you may look
//! at the top card of your library. If it shares a creature type with ~, you may reveal
//! it. If you do, [effect]." The sentence "If it shares a creature type with ~, you may
//! reveal it." continues the "look at the top card of your library" instruction (see
//! `kw/kinship.rs`); "If you do" then checks whether the card was revealed.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Whether the effect is "[you may] look at the top card of your library".
fn looks_at_top_card(e: &Effect) -> bool {
    match e {
        Effect::May { effect, .. } => looks_at_top_card(effect),
        Effect::Dig {
            who: PlayerRef::You,
            n: Value::Const(1),
            reveal: false,
            take: Value::Const(0),
            ..
        } => true,
        _ => false,
    }
}

fn reveal_if_shares_creature_type(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if end(l) != "if it shares a creature type with ~, you may reveal it"
        || !looks_at_top_card(prev)
    {
        return false;
    }
    *prev = Effect::seq(vec![
        prev.clone(),
        Effect::Custom(crate::kw::kinship::REVEAL_TOP_IF_SHARES_TYPE.into()),
    ]);
    // "that card": the revealed card.
    b.it = Sel::Var(vars::IT);
    true
}

inventory::submit! { FollowupPattern { name: "kinship: if it shares a creature type with ~, you may reveal it", priority: 60, apply: reveal_if_shares_creature_type } }
