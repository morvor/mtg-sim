//! Replacement grammar VI (cards5-mana-replacements).
//!
//! - "If an opponent who controls at least as many lands as you do would put a land onto
//!   the battlefield, that player instead puts that land onto the battlefield then
//!   sacrifices a land of their choice." (Land Equilibrium): a replacement effect on a
//!   player's putting a land onto the battlefield, however it happens: playing it
//!   (CR 305.1), an effect putting it there or creating it (CR 305.4, 701.7a), under any
//!   player's control (the player who puts it is the one the event records,
//!   `MoveEv::by`). The land still enters and then that player sacrifices a land, which
//!   may be that same land (rulings). With the land's own "as it enters" replacement, its
//!   controller orders them (CR 616.1).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};
use crate::oracle::CompileContext;
use crate::types::CardType;

/// "who controls at least as many lands as you do", "who controls more lands than you",
/// "who controls fewer lands than you do", "who controls three or more lands".
fn who_controls(s: &str) -> Option<PlayerFilter> {
    let x = s.strip_prefix("who controls ")?;
    let you = |y: &str| -> Option<(Filter, String)> {
        let y = y
            .strip_suffix(" than you do")
            .or_else(|| y.strip_suffix(" than you"))
            .or_else(|| y.strip_suffix(" as you do"))
            .or_else(|| y.strip_suffix(" as you"))?;
        let (f, plural, tail) = parse_object_phrase(y)?;
        (plural && end(tail).is_empty()).then(|| (f, y.to_string()))
    };
    for (p, cmp) in [
        ("at least as many ", Cmp::Ge),
        ("more ", Cmp::Gt),
        ("fewer ", Cmp::Lt),
    ] {
        let Some(y) = x.strip_prefix(p) else { continue };
        // "as many ... as" goes with "at least", "than" with "more"/"fewer".
        let comparative = cmp != Cmp::Ge;
        if comparative != (y.ends_with(" than you") || y.ends_with(" than you do")) {
            return None;
        }
        let (f, _) = you(y)?;
        let yours = Value::Count(f.clone().you_control());
        return Some(PlayerFilter::Controls(Box::new(f), cmp, Box::new(yours)));
    }
    let (n, y) = parse_number(x)?;
    let y = y.trim_start().strip_prefix("or more ")?;
    let (f, plural, tail) = parse_object_phrase(y)?;
    if !plural || !end(tail).is_empty() || !matches!(n, Value::Const(_)) {
        return None;
    }
    Some(PlayerFilter::Controls(Box::new(f), Cmp::Ge, Box::new(n)))
}

/// "an opponent [who ...]", "a player [who ...]": the player who would put the land.
fn putting_player(s: &str) -> Option<PlayerFilter> {
    let (base, rest) = if let Some(r) = s.strip_prefix("an opponent") {
        (PlayerFilter::Opponent, r)
    } else if let Some(r) = s.strip_prefix("a player") {
        (PlayerFilter::Any, r)
    } else {
        return None;
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return Some(base);
    }
    let q = who_controls(rest)?;
    Some(match base {
        PlayerFilter::Any => q,
        b => PlayerFilter::And(vec![b, q]),
    })
}

/// "If [player] would put a land onto the battlefield, [result]."
fn put_land_replacement(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if ")?;
    let (who, rest) = r.split_once(" would put ")?;
    let (what, result) = rest.split_once(" onto the battlefield, ")?;
    let pf = putting_player(who)?;
    let (f, plural, tail) =
        parse_object_phrase(what.strip_prefix("a ").or_else(|| what.strip_prefix("an "))?)?;
    if plural || !end(tail).is_empty() || !matches!(f, Filter::Type(CardType::Land)) {
        return None;
    }
    let action = if let Some(then) = [
        "that player instead puts that land onto the battlefield then ",
        "that player instead puts that land onto the battlefield, then ",
        "that player instead puts it onto the battlefield, then ",
        "they instead put that land onto the battlefield, then ",
    ]
    .iter()
    .find_map(|p| result.strip_prefix(p))
    {
        // "sacrifices a land of their choice": that player is the subject.
        let then = format!("that player {then}");
        let e = super::replacement_grammar::instructions(&then, ctx, Sel::TriggerObject)?;
        ReplacementAction::Also(Box::new(e))
    } else {
        return None;
    };
    // Whoever puts the land there, under anyone's control (rulings): the event's player.
    let event = ReplacementEvent::Where {
        event: Box::new(ReplacementEvent::ZoneChange {
            filter: f,
            from: None,
            to: Some(ZoneKind::Battlefield),
        }),
        cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, pf),
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event,
                action,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacements 6: if [player] would put a land onto the battlefield, [result]", priority: 100, parse: put_land_replacement } }

/// The draw replacement of an ability parsed from `block` (its `Instead` action and its
/// event).
fn draw_replacement(abilities: &mut [Ability]) -> Option<&mut ReplacementDef> {
    let [a] = abilities else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Static(s) = &mut a.kind else {
        return None;
    };
    match &mut s.effect {
        StaticEffect::Replacement(d @ ReplacementDef {
            event: ReplacementEvent::Draw(_),
            action: ReplacementAction::Instead(_),
            ..
        }) => Some(d),
        _ => None,
    }
}

/// "If a player would draw a card, they reveal it instead." (Zur's Weirding): "it" is the
/// card they would draw, the top card of their library (CR 121.1), revealed where it is.
/// Read as "instead they reveal the top card of their library", with the rest of the
/// paragraph as it is.
fn reveal_it_instead(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let (head, rest) = text.split_once(" would draw a card, ")?;
    let (sentence, rest) = match rest.split_once(". ") {
        Some((s, r)) => (s, Some(r)),
        None => (rest.strip_suffix('.')?, None),
    };
    let body = match sentence {
        "they reveal it instead" => "instead they reveal the top card of their library",
        "you reveal it instead" => "instead you reveal the top card of your library",
        _ => return None,
    };
    let rewritten = match rest {
        Some(r) => format!("{head} would draw a card, {body}. {r}"),
        None => format!("{head} would draw a card, {body}."),
    };
    let mut abilities = crate::oracle::parse_ability(&rewritten, ctx)?;
    draw_replacement(&mut abilities)?;
    let a = std::sync::Arc::make_mut(&mut abilities[0]);
    a.text = text.to_string();
    Some(abilities)
}

inventory::submit! { super::AbilityPattern { name: "replacements 6: if a player would draw a card, they reveal it instead", priority: 70, parse: reveal_it_instead } }

/// "If a cycling ability of another nonland card would cause you to draw a card, instead
/// [instructions]." (Unpredictable Cyclone): a draw replacement for the draws a cycling
/// ability causes as it resolves (CR 702.29a), of a card other than the replacement's
/// source and of the given kind, as the cycled card last existed. "The cycled card" in the
/// instructions is that card (`kw/cycling_draw.rs`).
fn cycling_draw_instead(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let r = text.strip_prefix("If a cycling ability")?;
    let (of, rest) = r.split_once(" would cause you to draw a card, ")?;
    let (other, nonland) = match of.trim() {
        "" => (false, false),
        "of another card" => (true, false),
        "of another nonland card" => (true, true),
        "of a nonland card" => (false, true),
        _ => return None,
    };
    let mut abilities = crate::oracle::parse_ability(&format!("If you would draw a card, {rest}"), ctx)?;
    let def = draw_replacement(&mut abilities)?;
    if !matches!(def.event, ReplacementEvent::Draw(PlayerFilter::You)) {
        return None;
    }
    def.event = ReplacementEvent::Where {
        event: Box::new(def.event.clone()),
        cond: Condition::Custom(crate::kw::cycling_draw::caused_by_cycling(other, nonland).into()),
    };
    let a = std::sync::Arc::make_mut(&mut abilities[0]);
    a.text = text.to_string();
    Some(abilities)
}

inventory::submit! { super::AbilityPattern { name: "replacements 6: if a cycling ability would cause you to draw a card, instead ...", priority: 70, parse: cycling_draw_instead } }

/// "that shares a card type with the cycled card" in a cycling draw replacement
/// (Unpredictable Cyclone): the card whose cycling ability is resolving.
fn shares_type_with_cycled<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let rest = r
        .strip_prefix("that shares a card type with the cycled card")
        .or_else(|| r.strip_prefix("sharing a card type with the cycled card"))?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("if a cycling ability") || !raw.contains("would cause you to draw a card") {
        return None;
    }
    Some((
        Filter::Custom(crate::kw::cycling_draw::SHARES_TYPE_WITH_CYCLED.into()),
        rest,
    ))
}

inventory::submit! { super::FilterSuffixPattern { name: "replacements 6: that shares a card type with the cycled card", priority: 100, parse: shares_type_with_cycled } }
