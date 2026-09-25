//! Oracle patterns for abilities that function while the game starts (CR 103): "You are the
//! starting player." (Power Play, CR 103.1c), companion conditions (CR 103.2b,
//! 702.139a), and "Any time you could mulligan and this card is in your hand, you may ..."
//! (CR 103.5b).

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;
use crate::start::DeckCondition;

inventory::submit! { AbilityPattern { name: "you are the starting player", priority: 0, parse: starting_player } }
inventory::submit! { AbilityPattern { name: "companion condition", priority: 0, parse: companion } }
inventory::submit! { AbilityPattern { name: "any time you could mulligan", priority: 0, parse: could_mulligan } }
inventory::submit! { EffectPattern { name: "exile your hand, then draw that many cards", priority: 100, parse: exile_hand_draw_that_many } }

/// "exile all [the] cards from your hand, then draw that many cards" (Serum Powder).
fn exile_hand_draw_that_many(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("exile all the cards from your hand")
        .or_else(|| l.strip_prefix("exile all cards from your hand"))
        .or_else(|| l.strip_prefix("exile your hand"))?;
    if r != ", then draw that many cards" {
        return None;
    }
    let hand = Filter::and(vec![
        Filter::Card,
        Filter::InZone(ZoneKind::Hand),
        Filter::OwnedBy(PlayerRel::You),
    ]);
    Some(Effect::seq(vec![
        Effect::Exile {
            what: Sel::All(hand),
            face_down: false,
            link: false,
        },
        Effect::Draw {
            who: PlayerRef::You,
            n: Value::CountSel(Box::new(Sel::Var(vars::IT))),
        },
    ]))
}

fn static_in(effect: StaticEffect, zone: FunctionZone, text: &str) -> Vec<Ability> {
    let mut s = StaticAbility::new(effect);
    s.zone = zone;
    vec![AbilityDef::new(AbilityKind::Static(s), text)]
}

/// "You are the starting player. If multiple players would be the starting player, one of
/// those players is chosen at random." (CR 103.1c) — functions from the command zone.
fn starting_player(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let l = t.to_lowercase();
    let r = l.strip_prefix("you are the starting player.")?;
    let r = r.trim();
    if !r.is_empty()
        && r != "if multiple players would be the starting player, one of those players is chosen at random."
    {
        return None;
    }
    Some(static_in(
        StaticEffect::Custom(crate::start::YOU_ARE_STARTING_PLAYER.into()),
        FunctionZone::Command,
        t,
    ))
}

/// "Companion — Each [kind] card in your starting deck has [property]." / "... is a
/// [kind] card." / "Your starting deck contains only cards with [property][ and land
/// cards]." (CR 702.139a). The ability functions outside the game.
fn companion(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let l = t.to_lowercase();
    // The keyword may already have been stripped like an ability word; only companion
    // conditions talk about "your starting deck".
    let cond = l
        .strip_prefix("companion — ")
        .or_else(|| l.strip_prefix("companion - "))
        .unwrap_or(&l);
    if !cond.contains("your starting deck") {
        return None;
    }
    // Drop the reminder text.
    let cond = match cond.split_once(" (") {
        Some((c, _)) => c,
        None => cond,
    };
    let cond = end(cond);
    let dc = if let Some(r) = cond.strip_prefix("each ") {
        let (head, prop) = r.split_once(" in your starting deck ")?;
        let (each, plural, tail) = parse_object_phrase(head)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        let must_text = if let Some(p) = prop.strip_prefix("has ") {
            format!("{head} with {p}")
        } else if let Some(p) = prop
            .strip_prefix("is an ")
            .or_else(|| prop.strip_prefix("is a "))
        {
            p.to_string()
        } else {
            return None;
        };
        let (must, _, tail) = parse_object_phrase(&must_text)?;
        if !end(tail).is_empty() {
            return None;
        }
        DeckCondition::Each { each, must }
    } else if let Some(r) = cond.strip_prefix("your starting deck contains only cards with ") {
        let (prop, lands) = match r.strip_suffix(" and land cards") {
            Some(p) => (p, true),
            None => (r, false),
        };
        let must_text = format!("cards with {prop}");
        let (must, _, tail) = parse_object_phrase(&must_text)?;
        if !end(tail).is_empty() {
            return None;
        }
        let each = if lands {
            Filter::Not(Box::new(Filter::Type(crate::types::CardType::Land)))
        } else {
            Filter::Any
        };
        DeckCondition::Each { each, must }
    } else {
        return None;
    };
    Some(static_in(
        StaticEffect::Companion(dc),
        FunctionZone::Anywhere,
        t,
    ))
}

/// "Any time you could mulligan and this card is in your hand, you may [effect]."
/// (CR 103.5b), with its reminder text.
fn could_mulligan(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let l = t.to_lowercase();
    let mut rest = None;
    for this in ["this card", "~"] {
        let prefix = format!("any time you could mulligan and {this} is in your hand, you may ");
        if let Some(r) = l.strip_prefix(&prefix) {
            rest = Some(r.to_string());
        }
    }
    let r = rest?;
    let r = match r.split_once(" (") {
        Some((e, _)) => e.to_string(),
        None => r,
    };
    let mut b = Builder::new(ctx);
    let effect = parse_effect_text(&r, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    Some(static_in(
        StaticEffect::AnyTimeCouldMulligan(Box::new(effect)),
        FunctionZone::Hand,
        t,
    ))
}
