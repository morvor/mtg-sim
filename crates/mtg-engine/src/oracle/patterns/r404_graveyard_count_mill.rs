//! "[Trigger], enchanted player mills X cards, where X is the number of cards put into
//! their graveyard from anywhere this turn." (Fraying Sanity). See
//! `kw/cards_put_into_graveyard_this_turn.rs` for the count.

use super::AbilityPattern;
use crate::ability::*;
use crate::kw::cards_put_into_graveyard_this_turn::CARDS_PUT_INTO_ENCHANTED_PLAYERS_GRAVEYARD;
use crate::oracle::CompileContext;

const TAIL: &str = ", enchanted player mills X cards, where X is the number of cards put into their graveyard from anywhere this turn.";

fn mill_cards_put_into_graveyard(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let trigger = t.strip_suffix(TAIL)?;
    let mut ability =
        crate::oracle::triggers::parse_triggered(&format!("{trigger}, enchanted player mills a card."), ctx)?;
    let a = std::sync::Arc::make_mut(&mut ability);
    let AbilityKind::Triggered(tr) = &mut a.kind else {
        return None;
    };
    let Effect::Mill { who, .. } = &tr.body.effect else {
        return None;
    };
    tr.body.effect = Effect::Mill {
        who: who.clone(),
        n: Value::Custom(CARDS_PUT_INTO_ENCHANTED_PLAYERS_GRAVEYARD.into()),
    };
    a.text = t.into();
    Some(vec![ability])
}

inventory::submit! { AbilityPattern { name: "enchanted player mills X, X = cards put into their graveyard this turn", priority: 60, parse: mill_cards_put_into_graveyard } }
