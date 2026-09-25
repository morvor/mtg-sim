//! Cards from outside the game (CR 108.3b, 400.11b): "reveal a sorcery card you own from
//! outside the game and put it into your hand" (wishes), "put a card you own from outside
//! the game into your hand". Counting card types among cards in graveyards (Tarmogoyf):
//! tokens aren't cards (CR 108.2b).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{EffectPattern, StaticPattern};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "reveal a [filter] card you own from outside the game and put it into your hand",
/// "put a card you own from outside the game into your hand",
/// "choose a [filter] card you own from outside the game and put it into your hand".
fn wish(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (verb, r) = l.split_once(' ')?;
    if !matches!(verb, "reveal" | "put" | "choose") {
        return None;
    }
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (filter, plural, rest) = parse_object_phrase(r)?;
    if plural {
        return None;
    }
    let rest = rest.trim().strip_prefix("from outside the game")?.trim();
    let ok = match verb {
        "put" => rest == "into your hand",
        _ => rest == "and put it into your hand" || rest == "and put that card into your hand",
    };
    if !ok {
        return None;
    }
    // The card must be one the player owns outside the game (their sideboard, CR 108.3b).
    let filter = Filter::and(vec![
        filter,
        Filter::Card,
        Filter::InZone(ZoneKind::Outside),
        Filter::OwnedBy(PlayerRel::You),
    ]);
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "r108 wish from outside the game", priority: 100, parse: wish } }

/// "the number of card types among cards in all graveyards" / "... in your graveyard".
/// Only cards count: tokens in a graveyard aren't cards (CR 108.2b).
fn card_types_among(s: &str) -> Option<(Value, &str)> {
    let r = s.strip_prefix("the number of card types among cards in ")?;
    for (zone, owner) in [
        ("all graveyards", None),
        ("your graveyard", Some(PlayerRel::You)),
    ] {
        if let Some(rest) = r.strip_prefix(zone) {
            let mut parts = vec![Filter::Card, Filter::InZone(ZoneKind::Graveyard)];
            if let Some(o) = owner {
                parts.push(Filter::OwnedBy(o));
            }
            return Some((Value::CardTypesAmong(Filter::and(parts)), rest));
        }
    }
    None
}

/// Tarmogoyf: "~'s power is equal to the number of card types among cards in all
/// graveyards and its toughness is equal to that number plus 1." A characteristic-defining
/// ability (CR 604.3) that counts cards (CR 108.2).
fn card_types_cda(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("~'s power is equal to ")?;
    let (v, rest) = card_types_among(r)?;
    let rest = end(rest);
    let t = if rest.is_empty() {
        None
    } else {
        let plus = rest.strip_prefix("and its toughness is equal to that number plus ")?;
        let n: i32 = plus.trim().parse().ok()?;
        Some(Value::Sum(vec![v.clone(), Value::c(n)]))
    };
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::CdaPT(Some(v), t)],
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "r108 card types among cards cda", priority: 100, parse: card_types_cda } }
