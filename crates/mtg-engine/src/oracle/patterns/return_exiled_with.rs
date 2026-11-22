//! "Return all/each [creature] card(s) exiled with ~ to the battlefield under [your control /
//! their owners' control / its owner's control]" (Cold Storage, Synod Sanctum, Endless
//! Sands, Helvault): the cards the permanent's linked abilities exiled (CR 607.2a) that are
//! still in exile return to the battlefield.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

fn return_exiled_with(l: &str, _b: &mut Builder) -> Option<Effect> {
    const V: Var = vars::USER + 81;
    let r = end(l.trim()).strip_prefix("return ")?;
    let r = r
        .strip_prefix("all ")
        .or_else(|| r.strip_prefix("each "))?;
    let (creature, r) = match r.strip_prefix("creature ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let r = r
        .strip_prefix("cards ")
        .or_else(|| r.strip_prefix("card "))?;
    let r = r
        .strip_prefix("exiled with ~ ")
        .or_else(|| r.strip_prefix("exiled with it "))?;
    let whose = r.strip_prefix("to the battlefield under ")?;
    let mut to = Destination::battlefield();
    match whose {
        "your control" => to = to.under_your_control(),
        "their owners' control" | "its owner's control" | "their owner's control" => {
            to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(V))));
        }
        _ => return None,
    }
    let mut filter = vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ];
    if creature {
        filter.push(Filter::Type(CardType::Creature));
    }
    Some(Effect::ForEach {
        sel: Sel::All(Filter::And(filter)),
        var: V,
        effect: Box::new(Effect::Move {
            what: Sel::Var(V),
            to,
        }),
    })
}

inventory::submit! { EffectPattern { name: "return cards exiled with ~ to the battlefield", priority: 0, parse: return_exiled_with } }
