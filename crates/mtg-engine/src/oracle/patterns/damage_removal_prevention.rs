//! Prevention by source and "would die, exile it instead" statics (CR 614, 615):
//!
//! Static abilities:
//! - "Prevent all [combat] damage that would be dealt to ~ [by creatures]."
//! - "Prevent all [combat] damage that would be dealt by enchanted creature."
//! - "Prevent all [combat] damage that would be dealt to and dealt by enchanted creature."
//! - "If a creature an opponent controls would die, exile it instead.", "If ~ would die,
//!   exile it instead.", "If a creature dealt damage by ~ this turn would die, exile it
//!   instead."
//!
//! One-shot effects (until end of turn):
//! - "Prevent all [combat] damage that would be dealt by target creature this turn."
//! - "Prevent all [combat] damage target creature would deal this turn."
//! - "Prevent all combat damage that would be dealt to and dealt by that creature this
//!   turn." (Maze of Ith)
//! - "Prevent all damage that would be dealt by creatures this turn.", "Prevent all
//!   combat damage that would be dealt to players this turn."

use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn damage_def(
    source: Filter,
    to_players: Option<PlayerFilter>,
    to_objects: Option<Filter>,
    combat_only: bool,
) -> ReplacementDef {
    ReplacementDef {
        event: ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only,
        },
        action: ReplacementAction::Prevent,
        self_replacement: false,
        optional: false,
    }
}

/// "prevent all [combat] damage that would be dealt " → (combat only, rest).
fn prevent_all(l: &str) -> Option<(bool, &str)> {
    let r = l.strip_prefix("prevent all ")?;
    let (combat, r) = match r.strip_prefix("combat ") {
        Some(x) => (true, x),
        None => (false, r),
    };
    Some((combat, r.strip_prefix("damage ")?))
}

/// What the prevention applies to in terms of the damage event.
enum Direction {
    To,
    By,
    ToAndBy,
}

fn direction(r: &str) -> Option<(Direction, &str)> {
    let r = r.strip_prefix("that would be dealt ")?;
    if let Some(x) = r.strip_prefix("to and dealt by ") {
        return Some((Direction::ToAndBy, x));
    }
    if let Some(x) = r.strip_prefix("to ") {
        return Some((Direction::To, x));
    }
    if let Some(x) = r.strip_prefix("by ") {
        return Some((Direction::By, x));
    }
    None
}

/// A damage recipient or source in a prevention clause: objects matching a filter, or
/// players.
#[derive(Clone)]
enum Party {
    Objects(Filter),
    Players(PlayerFilter),
}

/// "~", "enchanted creature", "you", "players", or a plural object phrase ("creatures",
/// "creature tokens you control"). Returns the party and the rest of the text.
fn party(r: &str) -> Option<(Party, &str)> {
    let r = r.trim_start();
    if let Some(x) = r.strip_prefix("~") {
        return Some((Party::Objects(Filter::Source), x));
    }
    if let Some(x) = r
        .strip_prefix("enchanted creature")
        .or_else(|| r.strip_prefix("equipped creature"))
    {
        return Some((Party::Objects(Filter::AttachedToSource), x));
    }
    for (p, pf) in [("you", PlayerFilter::You), ("players", PlayerFilter::Any)] {
        if let Some(x) = r.strip_prefix(p) {
            if x.is_empty() || x.starts_with(' ') {
                return Some((Party::Players(pf), x));
            }
        }
    }
    let (f, plural, x) = parse_object_phrase(r)?;
    if !plural {
        return None;
    }
    Some((Party::Objects(f), x))
}

fn defs_for(dir: &Direction, obj: Party, by: Filter, combat: bool) -> Option<Vec<ReplacementDef>> {
    let to = || match &obj {
        Party::Objects(f) => damage_def(by.clone(), None, Some(f.clone()), combat),
        Party::Players(p) => damage_def(by.clone(), Some(p.clone()), None, combat),
    };
    let from = |f: &Filter| {
        damage_def(
            f.clone(),
            Some(PlayerFilter::Any),
            Some(Filter::Any),
            combat,
        )
    };
    Some(match (dir, &obj) {
        (Direction::To, _) => vec![to()],
        (Direction::By, Party::Objects(f)) => vec![from(f)],
        (Direction::ToAndBy, Party::Objects(f)) => vec![to(), from(f)],
        _ => return None,
    })
}

fn statics(defs: Vec<ReplacementDef>, text: &str) -> Vec<Ability> {
    defs.into_iter()
        .map(|d| {
            AbilityDef::new(
                AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(d))),
                text,
            )
        })
        .collect()
}

/// Static prevention: "prevent all damage that would be dealt to ~", "... dealt by
/// enchanted creature", "... dealt to ~ by creatures".
fn s_prevent(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (combat, r) = prevent_all(l)?;
    let (dir, r) = direction(r)?;
    let (obj, r) = party(r)?;
    let r = r.trim_start();
    let by = if r.is_empty() {
        Filter::Any
    } else {
        // "... dealt to ~ by creatures"
        let x = r.strip_prefix("by ")?;
        if !matches!(dir, Direction::To) {
            return None;
        }
        let (f, plural, rest) = parse_object_phrase(x)?;
        if !plural || !rest.trim().is_empty() {
            return None;
        }
        f
    };
    Some(statics(defs_for(&dir, obj, by, combat)?, text))
}

inventory::submit! { StaticPattern { name: "damage_removal: static prevent all damage to/by", priority: 60, parse: s_prevent } }

/// "If [objects] would die, exile it instead." (CR 614.1a, 700.4)
fn s_die_exile(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("if ")?;
    let who = r.strip_suffix(" would die, exile it instead")?;
    let filter = if who == "~" {
        Filter::Source
    } else if let Some(x) = who.strip_prefix("a creature dealt damage by ~ this turn") {
        if !x.is_empty() {
            return None;
        }
        Filter::and(vec![
            Filter::creature(),
            Filter::DealtDamageThisTurnBy(Box::new(Sel::This)),
        ])
    } else {
        let x = who.strip_prefix("a ").or_else(|| who.strip_prefix("an "))?;
        let (f, plural, rest) = parse_object_phrase(x)?;
        if plural || !rest.trim().is_empty() {
            return None;
        }
        f
    };
    Some(statics(
        vec![ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter,
                from: Some(ZoneKind::Battlefield),
                to: Some(ZoneKind::Graveyard),
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        }],
        text,
    ))
}

inventory::submit! { StaticPattern { name: "damage_removal: static would die, exile instead", priority: 60, parse: s_die_exile } }

/// One-shot prevention by or to and by an object: "prevent all combat damage that would
/// be dealt by target creature this turn", "prevent all damage target creature would
/// deal this turn", "prevent all combat damage that would be dealt to and dealt by that
/// creature this turn", "prevent all damage that would be dealt by creatures this turn".
fn p_prevent_by(l: &str, b: &mut Builder) -> Option<Effect> {
    let (combat, r) = prevent_all(l)?;
    let (dir, obj, tail) = if let Some((dir, x)) = direction(r) {
        // Plural groups ("creatures", "players") aren't locked in: the prevention applies
        // to whatever matches when the damage would be dealt.
        if let Some((p, tail)) = party(x).filter(|(p, _)| {
            matches!(p, Party::Players(PlayerFilter::Any))
                || matches!(p, Party::Objects(f) if !matches!(f, Filter::Source | Filter::AttachedToSource))
        }) {
            (dir, p, tail.to_string())
        } else {
            if matches!(dir, Direction::To) {
                // "Dealt to [a specific object or player]" is the core prevention
                // pattern.
                return None;
            }
            let (what, tail) = object_ref(x, b)?;
            (dir, Party::Objects(locked(what)?), tail)
        }
    } else {
        // "[object] would deal this turn"
        let (what, tail) = object_ref(r, b)?;
        let tail = tail.trim_start().strip_prefix("would deal")?.to_string();
        (Direction::By, Party::Objects(locked(what)?), tail)
    };
    if tail.trim() != "this turn" {
        return None;
    }
    let effects = defs_for(&dir, obj, Filter::Any, combat)?
        .into_iter()
        .map(|def| Effect::AddReplacement {
            def,
            duration: Duration::EndOfTurn,
            uses: None,
        })
        .collect();
    Some(Effect::seq(effects))
}

/// A specific object (target, pronoun), locked in as the effect is created (CR 609.7b
/// via `prevention::lock_def`).
fn locked(what: Sel) -> Option<Filter> {
    match what {
        Sel::Target(_) | Sel::This | Sel::TriggerObject | Sel::Var(_) => {
            Some(Filter::In(Box::new(what)))
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "damage_removal: prevent damage dealt by", priority: 55, parse: p_prevent_by } }

/// Statics: "Damage can't be prevented.", "Damage that would be dealt by ~ can't be
/// prevented." (CR 615.12)
fn s_cant_prevent(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = match l {
        "damage can't be prevented" => Restriction::DamageCantBePrevented,
        "damage that would be dealt by ~ can't be prevented" => {
            Restriction::SourceDamageCantBePrevented(Filter::Source)
        }
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "damage_removal: static damage can't be prevented", priority: 60, parse: s_cant_prevent } }

/// "~ deals 4 damage to target creature. The damage can't be prevented." in an instant or
/// sorcery: the spell's damage can't be prevented. The restriction is created first and
/// names the spell, which deals no other damage.
fn f_the_damage_cant_be_prevented(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(
        l,
        "the damage can't be prevented" | "this damage can't be prevented"
    ) || !b.ctx.is_spell()
    {
        return false;
    }
    let from_this = |e: &Effect| {
        matches!(
            e,
            Effect::DealDamage {
                source: Sel::This,
                ..
            }
        )
    };
    let ok = match &*prev {
        Effect::Seq(v) => !v.is_empty() && v.iter().all(from_this),
        e => from_this(e),
    };
    if !ok {
        return false;
    }
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        Effect::AddRestriction {
            restriction: Restriction::SourceDamageCantBePrevented(Filter::In(Box::new(
                Sel::This,
            ))),
            duration: Duration::EndOfTurn,
        },
        old,
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "damage_removal: the damage can't be prevented", priority: 50, apply: f_the_damage_cant_be_prevented } }
