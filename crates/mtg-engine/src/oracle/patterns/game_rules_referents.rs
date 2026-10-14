//! Referents in the "instead" sentences of the Courts (and others like them): "each of
//! those players mills ten cards" (the players an earlier instruction's target slot
//! named), "return that permanent card to the battlefield" (the card targeted), and
//! "Draw another card if [condition]." (Imoen, Mystic Trickster).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_simple, Builder};
use crate::oracle::phrases::end;

/// "each of those players mills ten cards": each player of the last target slot that
/// can name several players performs the instruction.
fn each_of_those_players(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each of those players ")?;
    let (verb, rest) = r.split_once(' ').unwrap_or((r, ""));
    let base = verb
        .strip_suffix("es")
        .filter(|v| v.ends_with("ss") || v.ends_with("sh"))
        .or_else(|| verb.strip_suffix('s'))?;
    let slot = b.targets.iter().rposition(|t| {
        matches!(t.what, TargetKind::Player(_)) && !matches!(t.max, Value::Const(1))
    })?;
    let effect = parse_simple(&format!("you {base} {rest}").trim().to_string(), b)?;
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Target(slot as u8),
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(effect),
        }),
    })
}

inventory::submit! { EffectPattern { name: "game rules: each of those players [does something]", priority: 100, parse: each_of_those_players } }

/// "return that permanent card to the battlefield instead": the card the ability
/// targeted ("target permanent card"), which "it" names.
fn that_kind_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find("that permanent card")?;
    if !matches!(b.it, Sel::Target(_)) {
        return None;
    }
    let reworded = format!("{}it{}", &l[..i], &l[i + "that permanent card".len()..]);
    parse_simple(&reworded, b)
}

inventory::submit! { EffectPattern { name: "game rules: that permanent card (the target)", priority: 100, parse: that_kind_card } }

/// "Draw another card if you've completed a dungeon.": a conditional instruction.
fn draw_another_if(l: &str, b: &mut Builder) -> Option<Effect> {
    let c = end(l).strip_prefix("draw another card if ")?;
    let cond = super::conditions_referents::parse_condition_with(c, b)
        .or_else(|| crate::oracle::statics::parse_condition(c, b.ctx))?;
    Some(Effect::If {
        cond,
        then: Box::new(Effect::Draw {
            who: PlayerRef::You,
            n: Value::c(1),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "game rules: draw another card if [condition]", priority: 100, parse: draw_another_if } }

/// "Each opponent who doesn't control a legendary creature or planeswalker loses the
/// game.": each such player performs the instruction (whether they control one is
/// checked for each as the instruction is performed).
fn each_who_controls(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("each opponent who ") {
        (PlayerRef::EachOpponent, r)
    } else {
        (PlayerRef::EachPlayer, l.strip_prefix("each player who ")?)
    };
    // ("each player who controls [object]" is the core grammar's.)
    let (negated, r) = (true, r.strip_prefix("doesn't control ")?);
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, false, rest) = crate::oracle::phrases::parse_object_phrase(r)? else {
        return None;
    };
    let (verb, tail) = rest.trim_start().split_once(' ')?;
    let base = verb
        .strip_suffix("es")
        .filter(|v| v.ends_with("ss") || v.ends_with("sh"))
        .or_else(|| verb.strip_suffix('s'))?;
    let effect = parse_simple(&format!("you {base} {tail}"), b)?;
    let controls = Condition::Exists(Filter::and(vec![
        f,
        Filter::ControlledBy(PlayerRel::You),
        Filter::InZone(ZoneKind::Battlefield),
    ]));
    let cond = if negated {
        Condition::Not(Box::new(controls))
    } else {
        controls
    };
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(Effect::If {
                cond,
                then: Box::new(effect),
                otherwise: Box::new(Effect::Noop),
            }),
        }),
    })
}

inventory::submit! { EffectPattern { name: "game rules: each opponent who (doesn't) control(s) [object] [does something]", priority: 100, parse: each_who_controls } }

/// "Counter target spell. If that spell targets a commander you control, instead counter
/// that spell, scry 2, then draw a card." (Bane's Contingency): the "instead" sentence
/// replaces the countering when the countered spell has such a target (CR 115.10).
fn if_that_spell_targets_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if that spell targets ") else {
        return false;
    };
    let Some((obj, rest)) = r.split_once(", instead ") else {
        return false;
    };
    let Effect::CounterSpell { what } = &*prev else {
        return false;
    };
    let spell = what.clone();
    let Some(obj) = obj.strip_prefix("a ").or_else(|| obj.strip_prefix("an ")) else {
        return false;
    };
    let Some((f, false, tail)) = crate::oracle::phrases::parse_object_phrase(obj) else {
        return false;
    };
    if !end(tail).is_empty() {
        return false;
    }
    let saved_it = std::mem::replace(&mut b.it, spell.clone());
    // "counter that spell, scry 2, then draw a card": a list of instructions.
    let rest = rest.replace("that spell", "it").replace(", then ", ", ");
    let parts: Option<Vec<Effect>> = rest
        .split(", ")
        .map(|p| crate::oracle::effects::parse_clause(p, b))
        .collect();
    b.it = saved_it;
    let Some(parts) = parts else {
        return false;
    };
    let new = Effect::seq(parts);
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::If {
        cond: Condition::SelMatches(spell, Filter::Targets(Box::new(f))),
        then: Box::new(new),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "game rules: if that spell targets [object], instead [effect]", priority: 100, apply: if_that_spell_targets_instead } }

/// "they shuffle another creature they own into their library" (This Is How It Ends):
/// the player chooses one of their permanents and shuffles it into their library.
fn player_shuffles_own_permanent(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" shuffles ")?;
    let (subject, r) = (&l[..i], &l[i + " shuffles ".len()..]);
    let r = r.strip_suffix(" into their library")?;
    let r = r
        .strip_prefix("another ")
        .or_else(|| r.strip_prefix("a "))
        .or_else(|| r.strip_prefix("an "))?;
    let (obj, owner) = if let Some(o) = r.strip_suffix(" they own") {
        (o, Filter::OwnedBy(PlayerRel::You))
    } else {
        (r.strip_suffix(" they control")?, Filter::ControlledBy(PlayerRel::You))
    };
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(obj)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let (who, rest) = crate::oracle::effects::player_ref(subject, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    let f = if f.zone().is_none() {
        Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])
    } else {
        f
    };
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(Effect::ShuffleIntoLibrary {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![f, owner]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            library: PlayerRef::You,
        }),
    })
}

inventory::submit! { EffectPattern { name: "game rules: [player] shuffles a [permanent] they own into their library", priority: 100, parse: player_shuffles_own_permanent } }

/// "Put a point counter on ~. Then if it has ten or more point counters on it, remove
/// them all and that player loses the game." (Strixhaven Stadium): "it" is what the
/// counter was put on, and "them" are those counters.
fn if_counters_remove_them_all(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    fn last_counters(e: &Effect) -> Option<(Sel, crate::types::CounterKind)> {
        match e {
            Effect::AddCounters { what, kind, .. } => Some((what.clone(), kind.clone())),
            Effect::Seq(v) => v.last().and_then(last_counters),
            _ => None,
        }
    }
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let Some(r) = l.strip_prefix("if it has ") else {
        return false;
    };
    let Some((it, put_kind)) = last_counters(prev) else {
        return false;
    };
    let Some((n, r)) = crate::oracle::phrases::parse_number(r) else {
        return false;
    };
    if n.as_const().is_none() {
        return false;
    }
    let Some((kind, r)) = r
        .trim_start()
        .strip_prefix("or more ")
        .and_then(|r| r.split_once(" counters on it, "))
    else {
        return false;
    };
    if kind != put_kind.as_str() {
        return false;
    }
    let Some(rest) = r.strip_prefix("remove them all") else {
        return false;
    };
    let count = Value::CountersOn(Box::new(it.clone()), Some(kind.into()));
    let mut then = vec![Effect::RemoveCounters {
        what: it,
        kind: Some(kind.into()),
        n: count.clone(),
    }];
    if let Some(r) = rest.strip_prefix(" and ") {
        let Some(e) = crate::oracle::effects::parse_clause(r, b) else {
            return false;
        };
        then.push(e);
    } else if !rest.is_empty() {
        return false;
    }
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::Compare(count, Cmp::Ge, n),
            then: Box::new(Effect::seq(then)),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "game rules: then if it has N or more [kind] counters on it, remove them all and [instruction]", priority: 100, apply: if_counters_remove_them_all } }
