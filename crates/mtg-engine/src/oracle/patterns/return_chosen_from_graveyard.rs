//! Untargeted returns from a graveyard, the cards chosen as the instruction is carried out
//! (CR 608.2c; no target is chosen as the spell is cast, CR 115.10):
//!
//! * "Then return a creature card from your graveyard to the battlefield." (Summon Undead)
//! * "Then you may return a land card from your graveyard to the battlefield tapped."
//!   (Deeproot Wayfinder); "... to the battlefield with a finality counter on it."
//!   (Charnel Serenade)
//! * "Then return up to two creature cards from your graveyard to your hand." (Another
//!   Chance)
//! * "Then return up to one creature card and up to one land card from your graveyard to
//!   your hand." (Druidic Ritual)
//!
//! "A [kind] card" must be chosen if there is one; "up to N" may be fewer.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};

/// "a creature card" → (1, false, "creature card"); "up to two creature cards" →
/// (2, true, "creature cards"); "two land cards" → (2, false, "land cards").
fn quantity(s: &str) -> Option<(Value, bool, &str)> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        return Some((Value::c(1), false, r));
    }
    let (up_to, r) = match s.strip_prefix("up to ") {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (n, r) = parse_number(r)?;
    if !matches!(n, Value::Const(_)) {
        return None;
    }
    Some((n, up_to, r.trim()))
}

/// "the battlefield", "the battlefield tapped" (Deeproot Wayfinder), "the battlefield
/// with a finality counter on it" (Charnel Serenade): the cards enter with those counters
/// (CR 122.6).
fn battlefield(dest: &str) -> Option<Destination> {
    let mut d = Destination::zone(ZoneKind::Battlefield);
    let mut r = dest.strip_prefix("the battlefield")?;
    if let Some(x) = r.strip_prefix(" tapped") {
        d.tapped = true;
        r = x;
    }
    if let Some(x) = r.strip_prefix(" with ") {
        d.with_counters = super::levels_classes_sagas_transformed::with_counters_on_it(x)?;
        r = "";
    }
    r.is_empty().then_some(d)
}

fn return_chosen_from_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("return ")?;
    if r.contains("target") {
        return None;
    }
    let (objs, dest) = r.split_once(" from your graveyard to ")?;
    let to = match dest {
        "your hand" => Destination::zone(ZoneKind::Hand),
        _ => battlefield(dest)?,
    };
    let mut moves = Vec::new();
    for part in objs.split(" and ") {
        let (count, up_to, desc) = quantity(part)?;
        if !(desc.ends_with("card") || desc.ends_with("cards")) {
            return None;
        }
        let kind = super::card_flow_search::card_filter(desc, b)?;
        if kind.zone().is_some() {
            return None;
        }
        moves.push(Effect::Move {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    kind,
                    Filter::InZone(ZoneKind::Graveyard),
                    Filter::OwnedBy(PlayerRel::You),
                ]),
                count,
                up_to,
                store: None,
            },
            to: to.clone(),
        });
    }
    // "It" / "them" afterwards: the returned cards (the new objects, CR 400.7), known only
    // when they were returned by a single choice.
    b.it = if moves.len() == 1 {
        Sel::Var(vars::IT)
    } else {
        Sel::None
    };
    Some(Effect::seq(moves))
}

inventory::submit! { EffectPattern { name: "return a [kind] card from your graveyard (chosen, untargeted)", priority: 100, parse: return_chosen_from_graveyard } }
