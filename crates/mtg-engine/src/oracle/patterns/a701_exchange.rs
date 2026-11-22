//! Exchanges (CR 701.12):
//!
//! * "Exchange control of two target creatures." / "... of target A and target B.";
//! * "Exchange your life total with ~'s toughness." (CR 701.12g);
//! * "exchange its power and the power of target creature it's blocking until end of
//!   combat" (CR 701.12g);
//! * "Exchange your graveyard and library." (CR 701.12d, 701.12f);
//! * "Choose two target creatures. For as long as ~ remains on the battlefield, exchange
//!   the text boxes of those creatures." (CR 701.12h).

use super::EffectPattern;
use crate::ability::*;
use crate::exchange::ExchangeSpec;
use crate::oracle::effects::{duration_suffix, object_ref, player_ref, Builder};
use crate::oracle::phrases::*;

/// "exchange control of two target creatures", "exchange control of target artifact and
/// target creature".
fn exchange_control(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exchange control of ")?;
    if r.starts_with("two target ") {
        let (spec, tail) = parse_target(r)?;
        if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
            return None;
        }
        let text = r[..r.len() - tail.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        return Some(Effect::ExchangeControl {
            a: Sel::Target(slot),
            b: Sel::Target(slot),
        });
    }
    let (first, second) = r.split_once(" and ")?;
    let (a, t1) = object_ref(first, b)?;
    let (bb, t2) = object_ref(second, b)?;
    if !end(&t1).is_empty() || !end(&t2).is_empty() {
        return None;
    }
    Some(Effect::ExchangeControl { a, b: bb })
}

inventory::submit! { EffectPattern { name: "a701 exchange control", priority: 100, parse: exchange_control } }

fn exchange(spec: ExchangeSpec) -> Effect {
    Effect::Exchange(Box::new(spec))
}

/// "[object]'s power" / "[object]'s toughness" → (object text, power?).
fn stat_of(s: &str) -> Option<(&str, bool)> {
    if let Some(o) = s.strip_suffix("'s power") {
        return Some((o, true));
    }
    s.strip_suffix("'s toughness").map(|o| (o, false))
}

/// "exchange your life total with ~'s toughness", "exchange target opponent's life total
/// with ~'s toughness".
fn exchange_life_and_stat(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exchange ")?;
    let (who, r) = r.split_once(" life total with ")?;
    let who = if who == "your" {
        "you"
    } else {
        who.strip_suffix("'s")?
    };
    let (player, tail) = player_ref(who, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let (obj, power) = stat_of(r)?;
    let (what, tail) = object_ref(obj, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(exchange(ExchangeSpec::LifeAndStat {
        player,
        what,
        power,
    }))
}

inventory::submit! { EffectPattern { name: "a701 exchange life total and toughness", priority: 100, parse: exchange_life_and_stat } }

/// "exchange its power and the power of target creature it's blocking until end of
/// combat", "exchange ~'s power and target creature's power until end of turn".
fn exchange_stats(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exchange ")?;
    let (duration, r) = duration_suffix(r);
    let (first, second) = r.split_once(" and ")?;
    let (a_text, power) = match first {
        "its power" => ("it", true),
        "its toughness" => ("it", false),
        _ => stat_of(first)?,
    };
    let b_text = match second
        .strip_prefix("the power of ")
        .or_else(|| second.strip_prefix("the toughness of "))
    {
        Some(o) => o,
        None => {
            let (o, p2) = stat_of(second)?;
            if p2 != power {
                return None;
            }
            o
        }
    };
    let (a, t1) = object_ref(a_text, b)?;
    // "target creature it's blocking": a creature the source is blocking.
    let (bsel, t2) = match b_text.strip_suffix(" it's blocking") {
        Some(o) => {
            let (spec, tail) = parse_target(o)?;
            let TargetKind::Object(f) = spec.what.clone() else {
                return None;
            };
            let mut spec = spec;
            spec.what = TargetKind::Object(Filter::and(vec![f, Filter::BlockedBySource]));
            let slot = b.add_target(spec, o);
            (Sel::Target(slot), tail.to_string())
        }
        None => object_ref(b_text, b)?,
    };
    if !end(&t1).is_empty() || !end(&t2).is_empty() {
        return None;
    }
    Some(exchange(ExchangeSpec::Stats {
        a,
        b: bsel,
        power,
        duration,
    }))
}

inventory::submit! { EffectPattern { name: "a701 exchange powers", priority: 100, parse: exchange_stats } }

fn zone_word(w: &str) -> Option<ZoneKind> {
    Some(match w {
        "hand" => ZoneKind::Hand,
        "library" => ZoneKind::Library,
        "graveyard" => ZoneKind::Graveyard,
        _ => return None,
    })
}

/// "exchange your graveyard and library", "exchange your hand and library".
fn exchange_zones(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exchange ")?;
    let (who, r) = if let Some(r) = r.strip_prefix("your ") {
        (PlayerRef::You, r)
    } else {
        let (p, r) = player_ref(r, b)?;
        let r = r.trim_start().strip_prefix("'s ")?.to_string();
        return exchange_zone_words(p, &r);
    };
    exchange_zone_words(who, r)
}

fn exchange_zone_words(player: PlayerRef, r: &str) -> Option<Effect> {
    let (a, bb) = r.split_once(" and ")?;
    let (a, bb) = (zone_word(a)?, zone_word(bb)?);
    Some(exchange(ExchangeSpec::Zones { player, a, b: bb }))
}

inventory::submit! { EffectPattern { name: "a701 exchange zones", priority: 100, parse: exchange_zones } }

/// "choose two target creatures": the targets that "those creatures" refers to.
fn choose_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    if !r.contains("target") {
        return None;
    }
    let (spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
        return None;
    }
    if matches!(spec.max, Value::Const(1)) {
        return None;
    }
    let text = r[..r.len() - tail.len()].trim().to_string();
    b.add_target(spec, &text);
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "a701 choose targets", priority: 100, parse: choose_targets } }

/// "for as long as ~ remains on the battlefield, exchange the text boxes of those
/// creatures" (CR 701.12h, 612).
fn exchange_text_boxes(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (duration, r) = match l.strip_prefix("for as long as ~ remains on the battlefield, ") {
        Some(r) => (Duration::WhileSourceOnBattlefield, r),
        None => (Duration::Permanent, l),
    };
    let objs = r.strip_prefix("exchange the text boxes of ")?;
    let what = match objs {
        "those creatures" | "them" => b.it.clone(),
        _ => {
            let (s, tail) = object_ref(objs, b)?;
            if !end(&tail).is_empty() {
                return None;
            }
            s
        }
    };
    Some(Effect::Modify {
        what,
        mods: vec![Modification::ExchangeText],
        duration,
    })
}

inventory::submit! { EffectPattern { name: "a701 exchange text boxes", priority: 100, parse: exchange_text_boxes } }
