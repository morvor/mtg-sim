//! Effect clauses that refer back to a triggered ability's event: "sacrifice it" (the
//! source, for self triggers), "you gain that much life", "that player loses that much
//! life" (CR 603.2, 608.2h — "that much" is the amount from the trigger event).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::EffectPattern;
use crate::oracle::phrases::*;

inventory::submit! {
    EffectPattern { name: "sacrifice it (self trigger)", priority: 100, parse: sacrifice_it }
}
inventory::submit! {
    EffectPattern { name: "gain/lose that much life", priority: 100, parse: that_much_life }
}

inventory::submit! {
    EffectPattern { name: "deals that much damage", priority: 100, parse: that_much_damage }
}

/// "it deals that much damage to any target", "~ deals that much damage to each
/// opponent", "~ deals that much damage to that player".
fn that_much_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger {
        return None;
    }
    let l = end(l);
    let (source, rest) = if let Some(r) = l.strip_prefix("~ deals that much damage to ") {
        (Sel::This, r)
    } else if let Some(r) = l.strip_prefix("it deals that much damage to ") {
        (b.it.clone(), r)
    } else {
        return None;
    };
    if matches!(source, Sel::None) {
        return None;
    }
    let to = match rest {
        "each opponent" => Sel::Players(PlayerRef::EachOpponent),
        "each player" => Sel::Players(PlayerRef::EachPlayer),
        "you" => Sel::Players(PlayerRef::You),
        "that player" => Sel::Players(b.it_player.clone()),
        _ => {
            let (spec, tail) = parse_any_target(rest)?;
            if !end(tail).is_empty() {
                return None;
            }
            let text = rest.to_string();
            Sel::Target(b.add_target(spec, &text))
        }
    };
    Some(Effect::DealDamage {
        source,
        amount: Value::EventAmount,
        to,
    })
}

inventory::submit! {
    EffectPattern { name: "you draw N cards", priority: 100, parse: you_draw }
}
inventory::submit! {
    EffectPattern { name: "remove N counters from ~/it", priority: 100, parse: remove_counters }
}

/// "you draw a card" (with an explicit subject, as in "you draw a card and you lose 1
/// life").
fn you_draw(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you draw ")?;
    let (n, tail) = parse_card_count(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Draw {
        who: PlayerRef::You,
        n,
    })
}

/// "remove a -1/-1 counter from ~", "remove a +1/+1 counter from it".
fn remove_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("remove ")?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = strip(r, "counters")
        .or_else(|| strip(r, "counter"))?
        .strip_prefix("from ")?;
    let what = match r {
        "~" => Sel::This,
        "it" if matches!(b.it, Sel::This | Sel::TriggerObject) => b.it.clone(),
        _ => return None,
    };
    Some(Effect::RemoveCounters {
        what,
        kind: Some(kind),
        n,
    })
}

/// "sacrifice it" where "it" is the ability's source.
fn sacrifice_it(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "sacrifice it" || !b.in_trigger || !matches!(b.it, Sel::This) {
        return None;
    }
    Some(Effect::SacrificeObjects { what: Sel::This })
}

/// "you gain that much life", "that player loses that much life", "each opponent loses
/// that much life", "target opponent loses that much life".
fn that_much_life(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger {
        return None;
    }
    let l = end(l);
    let (who, rest) = if let Some(r) = l.strip_prefix("that player ") {
        (b.it_player.clone(), r.to_string())
    } else if let Some(r) = l.strip_prefix("gain ").or_else(|| l.strip_prefix("lose ")) {
        // Imperative: "gain that much life".
        let verb = &l[..l.len() - r.len()];
        (PlayerRef::You, format!("{}s {r}", verb.trim()))
    } else {
        let with_space = format!("{l} ");
        let (who, spec, rest) = parse_player(&with_space)?;
        if matches!(who, PlayerRef::TriggerPlayer) {
            return None;
        }
        let who = match spec {
            Some(spec) => {
                let slot = b.add_target(spec, "target player");
                b.it_player = PlayerRef::Target(slot);
                PlayerRef::Target(slot)
            }
            None => who,
        };
        (who, rest.trim().to_string())
    };
    let rest = rest.trim();
    let gain = if let Some(r) = rest
        .strip_prefix("gains ")
        .or_else(|| rest.strip_prefix("gain "))
    {
        (r == "that much life").then_some(true)?
    } else if let Some(r) = rest
        .strip_prefix("loses ")
        .or_else(|| rest.strip_prefix("lose "))
    {
        (r == "that much life").then_some(false)?
    } else {
        return None;
    };
    let n = Value::EventAmount;
    Some(if gain {
        Effect::GainLife { who, n }
    } else {
        Effect::LoseLife { who, n }
    })
}
