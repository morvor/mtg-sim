//! Alternative costs a permanent offers for the spells its controller casts (CR 118.9,
//! 601.2b): "You may pay {W}{U}{B}{R}{G} rather than pay the mana cost for spells you
//! cast." (Fist of Suns, Jodah, Leyline of Mutation), "You may pay {0} rather than pay the
//! mana cost for Zombie creature spells you cast." (Rooftop Storm), "You may pay eight {E}
//! rather than pay the mana cost for permanent spells you cast." (Nissa, Worldsoul
//! Speaker), "You may collect evidence 10 rather than pay the mana cost for spells you
//! cast." (Conspiracy Unraveler, CR 701.59), "Once each turn, you may pay {0} rather than
//! pay the mana cost for a spell you
//! cast from exile." (Warped Space; also "a creature spell ... from exile", "a colorless
//! spell ... from your hand", "a spell you cast that you don't own with mana value 3 or
//! less", "a spell you cast with mana value X or less, where X is the number of time
//! counters on ~"), "Once during each of your turns, you may cast an enchantment spell by
//! paying life equal to its mana value rather than paying its mana cost." (Demon of Fate's
//! Design; "a spell from your hand", Access Maze).
//!
//! "You may cast creature spells with mana value 3 or less by paying {E} rather than paying
//! their mana costs. If you cast a spell this way, you may cast it as though it had flash."
//! (Primal Prayers) is `CostChange::AlternativeCostWithFlash` (CR 601.3c).
//!
//! Each is a `CostChange::AlternativeCost` for `CostTarget::Spells`, offered as a way of
//! casting those spells by `kw/offered_costs.rs`; a once-each-turn one has the condition
//! `once_unused(ALT_COST_SLOT)` ("during each of your turns": and it's your turn).

use super::StaticPattern;
use crate::ability::*;
use crate::kw::offered_costs::ALT_COST_SLOT;
use crate::kw::once_each_turn_cast::once_unused;
use crate::mana::ManaCost;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "pay {W}{U}{B}{R}{G}", "pay {0}", "pay eight {E}", "pay life equal to its mana value"
/// (the mana value of the spell, see `permissions::spell_relative_cost`), or "collect
/// evidence 10" (CR 701.59).
fn offered_action(s: &str) -> Option<Cost> {
    if let Some(n) = s.strip_prefix("collect evidence ") {
        let n: u32 = n.trim().parse().ok()?;
        return Some(Cost::free().with(CostPart::CollectEvidence(n)));
    }
    offered_cost(s.strip_prefix("pay ")?)
}

/// "{W}{U}{B}{R}{G}", "{0}", "eight {E}", "life equal to its mana value".
fn offered_cost(s: &str) -> Option<Cost> {
    let s = s.trim();
    if matches!(
        s,
        "life equal to its mana value" | "life equal to that spell's mana value"
    ) {
        return Some(Cost::free().with(CostPart::PayLife(Value::ManaValueOf(Box::new(Sel::This)))));
    }
    if s.starts_with('{') && !s.contains("{e}") {
        let m = ManaCost::parse(&s.to_uppercase())?;
        // A value defined by the spell ("{X}, where X is that spell's mana value") isn't a
        // cost to choose X for.
        if m.has_x() || format!("{m}").to_lowercase() != s {
            return None;
        }
        return Some(Cost::mana(m));
    }
    // "eight {E}", "{E}{E}".
    let n = match parse_number(s) {
        Some((Value::Const(n), rest)) if end(rest) == "{e}" => n,
        _ if !s.is_empty() && s.split("{e}").all(str::is_empty) => (s.len() / 3) as i32,
        _ => return None,
    };
    Some(Cost::free().with(CostPart::PayEnergy(Value::c(n))))
}

/// "from exile": a card in that zone, or a spell cast from it (CR 601.2a).
fn from_zone(k: ZoneKind) -> Filter {
    Filter::Or(vec![Filter::InZone(k), Filter::CastFrom(k)])
}

/// The spells described, from "spells you cast", "Zombie creature spells you cast", "a
/// creature spell you cast from exile", "a spell you cast that you don't own with mana value
/// 3 or less", ... — `singular` for "a [kind] spell".
fn spells_cast(s: &str, singular: bool, ctx: &CompileContext) -> Option<Filter> {
    let (head, mut quals) = s
        .split_once(" that you cast")
        .or_else(|| s.split_once(" you cast"))?;
    let head = if singular {
        head.strip_prefix("a ")
            .or_else(|| head.strip_prefix("an "))?
    } else {
        head
    };
    let mut parts = Vec::new();
    match (head, singular) {
        ("spell", true) | ("spells", false) => {}
        _ => {
            let (f, plural, tail) = parse_object_phrase(head)?;
            // "Zombie creature spells": a type word before "spell(s)".
            let plural = match end(tail) {
                "" => plural,
                "spells" => true,
                "spell" => false,
                _ => return None,
            };
            if plural == singular {
                return None;
            }
            match f {
                Filter::And(v) => parts.extend(v),
                f => parts.push(f),
            }
        }
    }
    loop {
        quals = quals.trim_start();
        if quals.is_empty() {
            break;
        }
        if let Some(r) = quals.strip_prefix("from exile") {
            parts.push(from_zone(ZoneKind::Exile));
            quals = r;
        } else if let Some(r) = quals.strip_prefix("from your hand") {
            parts.push(from_zone(ZoneKind::Hand));
            quals = r;
        } else if let Some(r) = quals.strip_prefix("that you don't own") {
            parts.push(Filter::not(Filter::OwnedBy(PlayerRel::You)));
            quals = r;
        } else if let Some(r) = quals.strip_prefix("with mana value x or less, where x is ") {
            let mut b = Builder::new(ctx);
            let (v, rest) = crate::oracle::statics::parse_value_phrase(r, &mut b)?;
            if !end(&rest).is_empty() || !b.targets.is_empty() {
                return None;
            }
            parts.push(Filter::ManaValue(Cmp::Le, Box::new(v)));
            break;
        } else if let Some(r) = quals.strip_prefix("with mana value ") {
            let (n, rest) = parse_number(r)?;
            let rest = rest
                .trim_start()
                .strip_prefix("or less")
                .filter(|_| matches!(n, Value::Const(_)))?;
            parts.push(Filter::ManaValue(Cmp::Le, Box::new(n)));
            quals = rest;
        } else {
            return None;
        }
    }
    Some(match parts.len() {
        0 => Filter::Any,
        1 => parts.remove(0),
        _ => Filter::And(parts),
    })
}

/// "[Once each turn, |Once during each of your turns, ]you may pay [cost] rather than pay
/// the mana cost for [spells you cast]" and "Once during each of your turns, you may cast
/// [a spell] by paying [cost] rather than paying its mana cost".
fn offered_alternative_cost(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (once, r) = if let Some(r) = l.strip_prefix("once each turn, ") {
        (Some(false), r)
    } else if let Some(r) = l.strip_prefix("once during each of your turns, ") {
        (Some(true), r)
    } else {
        (None, l)
    };
    // "If you cast a spell this way, you may cast it as though it had flash." (CR 601.3c)
    let (r, flash) = match r
        .strip_suffix(". if you cast a spell this way, you may cast it as though it had flash")
    {
        Some(x) => (x, true),
        None => (r, false),
    };
    // "{X}, where X is that spell's mana value" (Kentaro): that much generic mana.
    let (r, x_is_mana_value) = match r.strip_suffix(", where x is that spell's mana value") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let (cost, spells) = if let Some((cost, spells)) = r
        .strip_prefix("you may ")
        .and_then(|r| r.split_once(" rather than pay the mana cost for "))
    {
        let cost = if x_is_mana_value {
            if cost != "pay {x}" {
                return None;
            }
            Cost::free().with(CostPart::Repeated {
                cost: Box::new(Cost::mana(ManaCost::generic(1))),
                times: Value::ManaValueOf(Box::new(Sel::This)),
            })
        } else {
            offered_action(cost)?
        };
        (cost, spells_cast(spells, once.is_some(), ctx)?)
    } else if x_is_mana_value {
        return None;
    } else {
        // "you may cast a spell from your hand by paying ..." doesn't let its controller
        // cast spells from anywhere else (that would be a permission too).
        let r = r.strip_prefix("you may cast ")?;
        let (spell, cost) = r.split_once(" by paying ")?;
        // One spell once each turn, or spells ("... rather than paying their mana costs").
        let (cost, singular) = match cost.strip_suffix(" rather than paying its mana cost") {
            Some(c) => (c, true),
            None => (cost.strip_suffix(" rather than paying their mana costs")?, false),
        };
        if singular != once.is_some() {
            return None;
        }
        let spell = match spell.strip_suffix(" from your hand") {
            Some(s) => format!("{s} you cast from your hand"),
            None => format!("{spell} you cast"),
        };
        (offered_cost(cost)?, spells_cast(&spell, singular, ctx)?)
    };
    let change = if flash {
        CostChange::AlternativeCostWithFlash(cost)
    } else {
        CostChange::AlternativeCost(cost)
    };
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Spells(spells),
        who: PlayerRel::You,
        change,
    }));
    s.condition = match once {
        Some(false) => Some(once_unused(ALT_COST_SLOT)),
        Some(true) => Some(Condition::And(vec![
            Condition::YourTurn,
            once_unused(ALT_COST_SLOT),
        ])),
        None => None,
    };
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "r118.9 you may pay [cost] rather than pay the mana cost for [spells] you cast", priority: 100, parse: offered_alternative_cost } }

/// "Rather than pay the mana cost for a spell, its controller may discard a card that
/// shares a color with that spell." (Dream Halls): an alternative cost every player may pay
/// for each spell they cast; the card discarded is judged against the spell (`Sel::This`
/// as the cost is paid; the card being cast isn't in the hand then).
fn discard_sharing_a_color(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l)
        != "rather than pay the mana cost for a spell, its controller may discard a card that shares a color with that spell"
    {
        return None;
    }
    let cost = Cost::free().with(CostPart::Discard {
        filter: Filter::SharesColor(Box::new(Sel::This)),
        count: Value::c(1),
        random: false,
    });
    let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Spells(Filter::Any),
        who: PlayerRel::Any,
        change: CostChange::AlternativeCost(cost),
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "r118.9 rather than pay the mana cost for a spell, its controller may discard a card that shares a color with it", priority: 100, parse: discard_sharing_a_color } }
