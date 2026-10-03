//! Replacement grammar for permanents entering the battlefield (CR 614.1c–d, 614.12):
//!
//! - "~ enters under the control of an opponent of your choice." (the player who would
//!   control it chooses as it enters, CR 614.12a)
//! - "This turn, each creature you control enters with an additional +1/+1 counter on
//!   it.", "Until your next turn, ..." (one-shot effects)
//! - Follow-ups modifying how the permanents an instruction puts onto the battlefield
//!   enter (CR 614.1c with a self-replacement effect of that instruction, CR 614.15):
//!   "It enters tapped and attacking.", "If a Hero enters this way, it enters with two
//!   additional +1/+1 counters on it.", "If that card has mana value 3 or less, it enters
//!   with three additional +1/+1 counters on it.", "It enters with two additional +1/+1
//!   counters on it if it's a creature.", "If there are two or more instant and/or sorcery
//!   cards in your graveyard, that creature enters with two additional +1/+1 counters on
//!   it.", "If a land enters this way, it enters tapped."

use super::replacement_grammar::duration_prefix;
use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::CounterKind;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

/// "~ enters under the control of an opponent of your choice."
fn s_enters_under_control(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let who = match end(l.trim()) {
        "~ enters under the control of an opponent of your choice" => {
            PlayerRef::Each(PlayerFilter::Opponent)
        }
        "~ enters under the control of a player of your choice" => {
            PlayerRef::Each(PlayerFilter::Any)
        }
        _ => return None,
    };
    Some(vec![static_ability(
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::EnterUnderControl(who),
            self_replacement: false,
            optional: false,
        }),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "etb replacement grammar: enters under the control of an opponent of your choice", priority: 150, parse: s_enters_under_control } }

/// "N [additional] [kind] counter(s) on it/them" → (kind, N).
fn counters_on(s: &str) -> Option<(CounterKind, Value)> {
    let (n, r) = parse_number(s)?;
    let r = r.trim_start();
    let r = r.strip_prefix("additional ").unwrap_or(r);
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start();
    let r = r
        .strip_prefix("counters")
        .or_else(|| r.strip_prefix("counter"))?;
    if !matches!(r.trim(), "on it" | "on them" | "on each of them") {
        return None;
    }
    Some((kind, n))
}

/// "[Until your next turn, | This turn, ] each creature you control enters with an
/// additional +1/+1 counter on it."
fn p_each_enters_with(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, r) = duration_prefix(l);
    let dur = dur?;
    let r = r.strip_prefix("each ")?;
    let (subj, rest) = r.split_once(" enters with ")?;
    let (f, plural, tail) = parse_object_phrase(subj)?;
    if plural || !tail.trim().is_empty() {
        return None;
    }
    let (kind, n) = counters_on(rest)?;
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(f),
            action: ReplacementAction::EnterWithCounters(kind, n),
            self_replacement: false,
            optional: false,
        },
        duration: dur,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "etb replacement grammar: this turn, each creature you control enters with counters", priority: 150, parse: p_each_enters_with } }

/// How an entering permanent is modified.
#[derive(Clone, Debug)]
enum Entry {
    Counters(CounterKind, Value),
    Tapped,
    TappedAndAttacking,
}

/// "enters with N counters on it", "enters tapped", "enters tapped and attacking".
fn entry(s: &str) -> Option<Entry> {
    let s = s.trim();
    match s {
        "tapped" => return Some(Entry::Tapped),
        "tapped and attacking" => return Some(Entry::TappedAndAttacking),
        _ => {}
    }
    let r = s.strip_prefix("with ")?;
    let (k, n) = counters_on(r)?;
    Some(Entry::Counters(k, n))
}

/// The last instruction of `e` that puts objects onto the battlefield.
fn last_entry_effect(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => v.iter_mut().rev().find_map(last_entry_effect),
        Effect::May { effect, .. } => last_entry_effect(effect),
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield => Some(e),
        Effect::Dig { take_to, .. } if take_to.zone == ZoneKind::Battlefield => Some(e),
        Effect::CreateToken { .. } => Some(e),
        _ => None,
    }
}

/// The name of the token the last instruction creates ("Ragavan").
fn token_name(e: &mut Effect) -> Option<String> {
    match last_entry_effect(e)? {
        Effect::CreateToken { spec, .. } if !spec.name.is_empty() => {
            Some(spec.name.to_lowercase())
        }
        _ => None,
    }
}

/// Applies an unconditional entry modification to an instruction.
fn modify_entry(e: &mut Effect, entry: &Entry) -> bool {
    match (e, entry) {
        (Effect::Move { to, .. }, Entry::Counters(k, n)) => {
            to.with_counters.push((k.clone(), n.clone()));
            true
        }
        (Effect::Move { to, .. }, Entry::Tapped) => {
            to.tapped = true;
            true
        }
        (Effect::Move { to, .. }, Entry::TappedAndAttacking) => {
            to.tapped = true;
            to.attacking = true;
            true
        }
        (Effect::Dig { take_to, .. }, Entry::Counters(k, n)) => {
            take_to.with_counters.push((k.clone(), n.clone()));
            true
        }
        (Effect::CreateToken { tapped, .. }, Entry::Tapped) => {
            *tapped = true;
            true
        }
        (
            Effect::CreateToken {
                tapped, attacking, ..
            },
            Entry::TappedAndAttacking,
        ) => {
            *tapped = true;
            *attacking = true;
            true
        }
        _ => false,
    }
}

/// The self-replacement effect of an instruction for the permanents matching `f` it puts
/// onto the battlefield (CR 614.15).
fn self_replacement(f: Filter, entry: &Entry, e: Effect) -> Option<Effect> {
    let action = match entry {
        Entry::Counters(k, n) => ReplacementAction::EnterWithCounters(k.clone(), n.clone()),
        Entry::Tapped => ReplacementAction::EnterTapped,
        Entry::TappedAndAttacking => return None,
    };
    Some(Effect::SelfReplace {
        replacement: ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(f),
            action,
            self_replacement: true,
            optional: false,
        },
        effect: Box::new(e),
    })
}

/// A condition on the entering permanent: "a Hero enters this way", "a creature enters
/// this way", "that card has mana value 3 or less", "it's a creature".
fn entering_filter(c: &str) -> Option<Filter> {
    let c = c.trim();
    if let Some(x) = c.strip_suffix(" enters this way") {
        let r = x.strip_prefix("a ").or_else(|| x.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(r)?;
        return (!plural && tail.trim().is_empty()).then_some(f);
    }
    for p in ["it's a ", "it's an ", "that card is a ", "that card is an "] {
        if let Some(r) = c.strip_prefix(p) {
            let r = r.strip_suffix(" card").unwrap_or(r);
            let (f, plural, tail) = parse_object_phrase(r)?;
            return (!plural && tail.trim().is_empty()).then_some(f);
        }
    }
    for p in ["that card has ", "it has "] {
        if let Some(r) = c.strip_prefix(p) {
            let phrase = format!("permanent with {r}");
            let (f, plural, tail) = parse_object_phrase(&phrase)?;
            return (!plural && tail.trim().is_empty()).then_some(f);
        }
    }
    None
}

/// The subject of the entry: "it", "that creature", "the token", "that card".
fn entering_subject(s: &str) -> Option<&str> {
    for p in [
        "it enters ",
        "that creature enters ",
        "the token enters ",
        "that card enters ",
        "that permanent enters ",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some(r);
        }
    }
    None
}

fn f_enters_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let ctx = b.ctx;
    let l = end(l.trim());
    // "If [condition], it enters with ..."
    let (cond, rest) = match l.strip_prefix("if ").and_then(|r| r.split_once(", ")) {
        Some((c, r)) => (Some(c), r),
        None => (None, l),
    };
    // "it enters with ... if it's a creature"
    let (rest, cond) = match (cond, rest.rsplit_once(" if ")) {
        (None, Some((a, c))) => (a, Some(c)),
        (c, _) => (rest, c),
    };
    let named = token_name(prev).map(|n| format!("{n} enters "));
    let Some(r) = entering_subject(rest)
        .or_else(|| named.as_deref().and_then(|n| rest.strip_prefix(n)))
    else {
        return false;
    };
    // "It enters tapped and attacking and gains indestructible until end of turn": the
    // rest is an effect on the entered permanent.
    let (r, more) = match r.split_once(" and gains ") {
        Some((a, m)) if cond.is_none() => (a, Some(format!("it gains {m}"))),
        _ => (r, None),
    };
    let Some(entry) = entry(r) else {
        return false;
    };
    let extra = match &more {
        Some(m) => match crate::oracle::effects::parse_clause(m, b) {
            Some(e) => Some(e),
            None => return false,
        },
        None => None,
    };
    if let Some(x) = extra {
        if last_entry_effect(prev).is_none() {
            return false;
        }
        let ok = modify_entry(last_entry_effect(prev).expect("checked"), &entry);
        if ok {
            let old = std::mem::replace(prev, Effect::Noop);
            *prev = Effect::seq(vec![old, x]);
        }
        return ok;
    }
    let Some(target) = last_entry_effect(prev) else {
        return false;
    };
    match cond {
        None => modify_entry(target, &entry),
        Some(c) => {
            if let Some(f) = entering_filter(c) {
                let original = target.clone();
                let Some(e) = self_replacement(f, &entry, original) else {
                    return false;
                };
                *target = e;
                return true;
            }
            // A condition of the spell or ability ("if {G} was spent to cast ~", "if
            // there are two or more instant and/or sorcery cards in your graveyard").
            let Some(cond) = crate::oracle::statics::parse_condition(c, ctx) else {
                return false;
            };
            let original = target.clone();
            let mut modified = original.clone();
            if !modify_entry(&mut modified, &entry) {
                return false;
            }
            *target = Effect::If {
                cond,
                then: Box::new(modified),
                otherwise: Box::new(original),
            };
            true
        }
    }
}

inventory::submit! { FollowupPattern { name: "etb replacement grammar: it enters this way with counters / tapped and attacking", priority: 150, apply: f_enters_this_way } }


/// "Lands you control enter untapped.", "Gates you control enter untapped.", "Other
/// permanents enter untapped." (with "As long as ~ is untapped, " from the core "as long
/// as" prefix): the controller of an entering permanent that an "enters tapped" effect
/// would make enter tapped chooses the order of the two (CR 616.1); one an instruction
/// puts onto the battlefield tapped still enters tapped.
fn s_enter_untapped(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (subj, action) = match l.strip_suffix(" enter untapped") {
        Some(x) => (x, ReplacementAction::EnterUntapped),
        // "Other permanents enter tapped." (the other half of Archelos)
        None => (
            l.strip_suffix(" enter tapped")
                .filter(|x| x.starts_with("other "))?,
            ReplacementAction::EnterTapped,
        ),
    };
    let (f, plural, rest) = parse_object_phrase(subj)?;
    if !plural || !rest.trim().is_empty() {
        return None;
    }
    Some(vec![static_ability(
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(f),
            action,
            self_replacement: false,
            optional: false,
        }),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "etb replacement grammar: permanents enter untapped", priority: 150, parse: s_enter_untapped } }

/// "Creatures played by your opponents enter tapped." (Uphill Battle): creature spells
/// they cast (CR 601.1) — permanents that come from the stack under their control.
fn s_played_enter_tapped(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l.trim()).strip_suffix(" enter tapped")?;
    let (subj, who) = r.split_once(" played by ")?;
    let rel = match who {
        "your opponents" => PlayerRel::Opponent,
        "you" => PlayerRel::You,
        _ => return None,
    };
    let (f, plural, rest) = parse_object_phrase(subj)?;
    if !plural || !rest.trim().is_empty() {
        return None;
    }
    Some(vec![static_ability(
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Where {
                event: Box::new(ReplacementEvent::EntersBattlefield(Filter::and(vec![
                    f,
                    Filter::ControlledBy(rel),
                ]))),
                cond: Condition::SelMatches(Sel::TriggerObject, Filter::InZone(ZoneKind::Stack)),
            },
            action: ReplacementAction::EnterTapped,
            self_replacement: false,
            optional: false,
        }),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "etb replacement grammar: creatures played by your opponents enter tapped", priority: 150, parse: s_played_enter_tapped } }

/// "You may cast ~ from your graveyard. If you do, it enters with a finality counter on
/// it." (Hundred-Battle Veteran): the permission, and an "as enters" replacement effect
/// for the permanent the spell cast that way becomes (CR 614.1c, 601.2).
fn a_cast_from_zone_enters_with(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let (first, second) = t.split_once(". ")?;
    let lower = first.to_lowercase();
    let zone = match lower.as_str() {
        "you may cast ~ from your graveyard" => ZoneKind::Graveyard,
        "you may cast ~ from exile" => ZoneKind::Exile,
        _ => return None,
    };
    let s2 = second.to_lowercase();
    let r = end(&s2).strip_prefix("if you do, it enters with ")?;
    let (kind, n) = counters_on(r)?;
    let mut out = crate::oracle::statics::parse_static(&format!("{first}."), ctx)?;
    out.push(static_ability(
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::If {
                cond: Condition::CastFrom(zone),
                then: Box::new(Effect::EnterWithCounters { kind, n }),
                otherwise: Box::new(Effect::Noop),
            })),
            self_replacement: false,
            optional: false,
        }),
        t,
    ));
    Some(out)
}

inventory::submit! { super::AbilityPattern { name: "etb replacement grammar: you may cast ~ from your graveyard. if you do, it enters with counters", priority: 150, parse: a_cast_from_zone_enters_with } }

/// "You may have ~ enter tapped. [If you do, ...]" (Mariposa Military Base): an optional
/// "as enters" replacement (CR 614.1c, 614.12a).
fn a_may_enter_tapped(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let l = end(&lower);
    let rest = l.strip_prefix("you may have ~ enter tapped")?;
    let then = match rest.strip_prefix(". if you do, ") {
        Some(r) => {
            let mut b = Builder::new(ctx);
            let e = crate::oracle::effects::parse_effect_text(&format!("{r}."), &mut b)?;
            if !b.targets.is_empty() {
                return None;
            }
            Effect::seq(vec![Effect::EnterTapped, e])
        }
        None if rest.is_empty() => Effect::EnterTapped,
        None => return None,
    };
    Some(vec![static_ability(
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::AsEnters(Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(then),
            })),
            self_replacement: false,
            optional: false,
        }),
        t,
    )])
}

inventory::submit! { super::AbilityPattern { name: "etb replacement grammar: you may have ~ enter tapped", priority: 150, parse: a_may_enter_tapped } }

/// "[Until your next turn, ] [instruction], and each creature you control enters with an
/// additional +1/+1 counter on it." (Arlinn, the Pack's Hope): both for the duration.
fn p_and_each_enters_with(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, r) = duration_prefix(l);
    dur.as_ref()?;
    let prefix = &l[..l.len() - r.len()];
    let (a, each) = r.split_once(", and each ")?;
    if !each.contains(" enters with ") {
        return None;
    }
    let saved = b.targets.len();
    let e2 = p_each_enters_with(&format!("{prefix}each {each}"), b)?;
    let Some(e1) = crate::oracle::effects::parse_clause(&format!("{prefix}{a}"), b) else {
        b.targets.truncate(saved);
        return None;
    };
    Some(Effect::seq(vec![e1, e2]))
}

inventory::submit! { EffectPattern { name: "etb replacement grammar: [instruction], and each creature you control enters with counters", priority: 150, parse: p_and_each_enters_with } }

/// "Nontoken creatures you control enter as a copy of enchanted creature." (Infinite
/// Reflection): CR 707.9, 614.1c.
fn s_others_enter_as_copy(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (subj, of) = end(l.trim()).split_once(" enter as a copy of ")?;
    let (f, plural, rest) = parse_object_phrase(subj)?;
    if !plural || !rest.trim().is_empty() {
        return None;
    }
    let what = match of {
        "enchanted creature" | "equipped creature" => Filter::AttachedToSource,
        _ => return None,
    };
    Some(vec![static_ability(
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(f),
            action: ReplacementAction::EnterAsCopy {
                filter: what,
                optional: false,
            },
            self_replacement: false,
            optional: false,
        }),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "etb replacement grammar: creatures enter as a copy of enchanted creature", priority: 150, parse: s_others_enter_as_copy } }
