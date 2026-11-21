//! Noted mana (CR 607.2e; `noted_mana.rs`): "Note the type of mana spent to pay this
//! activation cost." / "Add one mana of ~'s last noted type." (Jeweled Amulet) and "Put a
//! charge counter on ~ and note the type and amount of mana spent to pay this activation
//! cost." / "Add ~'s last noted type and amount of mana." (Ice Cauldron).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

const NOTE_TYPE: &str = "note the type of mana spent to pay this activation cost";
const NOTE_AMOUNT: &str = "note the type and amount of mana spent to pay this activation cost";

fn note_kind(l: &str) -> Option<bool> {
    match l {
        NOTE_TYPE => Some(false),
        NOTE_AMOUNT => Some(true),
        _ => None,
    }
}

/// "note the type [and amount] of mana spent to pay this activation cost", alone or after
/// another instruction ("put a charge counter on ~ and note ...").
fn note_mana_spent(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if let Some(amount) = note_kind(l) {
        return Some(Effect::NoteManaSpent { amount });
    }
    let (first, rest) = l.split_once(" and note ")?;
    let amount = note_kind(&format!("note {rest}"))?;
    let first = parse_effect_text(first, b)?;
    Some(Effect::Seq(vec![first, Effect::NoteManaSpent { amount }]))
}

inventory::submit! { EffectPattern { name: "noted mana: note the type [and amount] of mana spent to pay this activation cost", priority: 100, parse: note_mana_spent } }

/// "add one mana of ~'s last noted type", "add ~'s last noted type and amount of mana".
fn add_noted_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    let amount = match end(l) {
        "add one mana of ~'s last noted type" => false,
        "add ~'s last noted type and amount of mana" => true,
        _ => return None,
    };
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::Noted { amount },
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "noted mana: add the last noted mana", priority: 100, parse: add_noted_mana } }
