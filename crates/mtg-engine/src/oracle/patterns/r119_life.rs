//! Oracle patterns for life totals (CR 119): setting a life total ("Target player's life
//! total becomes 20", CR 119.5), exchanging life totals (CR 119.7, 119.8), and "Your life
//! total can't change".

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number, parse_player};
use crate::oracle::CompileContext;

/// A player phrase used as the subject of a sentence, adding a target slot for "target
/// player"/"target opponent".
fn subject(s: &str, b: &mut Builder) -> Option<PlayerRef> {
    let phrase = format!("{} ", s.trim());
    let (r, spec, rest) = parse_player(&phrase)?;
    if !rest.trim().is_empty() {
        return None;
    }
    Some(match spec {
        Some(spec) => {
            let text = spec.text.clone();
            let slot = b.add_target(spec, &text);
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        None => r,
    })
}

/// "[player]'s life total becomes N", "your life total becomes N".
fn set_life(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, n) = if let Some(n) = l.strip_prefix("your life total becomes ") {
        (PlayerRef::You, n)
    } else if let Some(n) = l.strip_prefix("each player's life total becomes ") {
        (PlayerRef::EachPlayer, n)
    } else {
        let (p, n) = l.split_once("'s life total becomes ")?;
        (subject(p, b)?, n)
    };
    let (n, rest) = parse_number(n)?;
    if !end(rest).is_empty() {
        return None;
    }
    Some(Effect::SetLife { who, n })
}

/// "two target players exchange life totals", "exchange life totals with target player".
fn exchange_life(l: &str, b: &mut Builder) -> Option<Effect> {
    if l == "two target players exchange life totals" {
        let a = b.add_target(
            TargetSpec::player(PlayerFilter::Any, "target player"),
            "target player",
        );
        let mut second = TargetSpec::player(PlayerFilter::Any, "target player");
        second.distinct_from = vec![a];
        let c = b.add_target(second, "another target player");
        return Some(Effect::ExchangeLifeTotals {
            a: PlayerRef::Target(a),
            b: PlayerRef::Target(c),
        });
    }
    let r = l.strip_prefix("exchange life totals with ")?;
    let other = subject(r, b)?;
    Some(Effect::ExchangeLifeTotals {
        a: PlayerRef::You,
        b: other,
    })
}

/// "Your life total can't change": you can neither gain nor lose life.
fn life_cant_change(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let who = match l {
        "your life total can't change" => PlayerFilter::You,
        "players' life totals can't change" => PlayerFilter::Any,
        _ => return None,
    };
    Some(
        [
            Restriction::CantGainLife(who.clone()),
            Restriction::CantLoseLife(who),
        ]
        .into_iter()
        .map(|r| {
            AbilityDef::new(
                AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
                text,
            )
        })
        .collect(),
    )
}

inventory::submit! { EffectPattern { name: "life total becomes", priority: 0, parse: set_life } }
inventory::submit! { EffectPattern { name: "exchange life totals", priority: 0, parse: exchange_life } }
inventory::submit! { StaticPattern { name: "life total can't change", priority: 0, parse: life_cant_change } }
