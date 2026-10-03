//! Replacement grammar for zone changes (CR 614.1a, 614.6, 614.1c): event patterns
//! "[object] would die", "[object] would be put into [a | your | an opponent's | enchanted
//! player's] graveyard [from anywhere | from the battlefield]", "[object] would leave the
//! battlefield", "[object] would be destroyed", "[object] would enter [and it wasn't
//! cast]"; scopes (a static ability, "this turn", "until end of turn"); and replacements
//! "exile it [with N counters on it] instead", "return it to its owner's hand instead",
//! "put it on top of its owner's library instead", "[reveal ~ and] shuffle it into its
//! owner's library instead", "exile it instead of putting it anywhere else", "you may
//! ... instead", and "instead [instructions]" (with "it" / "that card" the object).

use super::replacement_grammar::{attempt, duration_prefix, instructions, word, OneShot};
use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::oracle_hardening_referents::is_no_referent;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// A parsed zone-change event pattern.
#[derive(Clone, Debug)]
struct ZoneEvent {
    event: ReplacementEvent,
    this_turn: bool,
    /// What "it" refers to in the replacement.
    it: Sel,
}

/// The object of a zone-change event: "~", "it", "enchanted creature", "a nontoken
/// creature an opponent controls", "a card", "another creature", "a creature dealt damage
/// by enchanted creature this turn".
fn subject(s: &str, b: &mut OneShot) -> Option<(Filter, Sel)> {
    let s = s.trim();
    match s {
        "~" | "this creature" | "this permanent" => return Some((Filter::Source, Sel::This)),
        "enchanted creature" | "equipped creature" | "enchanted permanent" => {
            return Some((Filter::AttachedToSource, Sel::TriggerObject))
        }
        _ => {}
    }
    if let Some(bb) = b.as_deref_mut() {
        if matches!(s, "it" | "that creature" | "that permanent") && !is_no_referent(&bb.it) {
            return Some((Filter::In(Box::new(bb.it.clone())), Sel::TriggerObject));
        }
    }
    // "a creature dealt damage by enchanted creature this turn", "a creature dealt damage
    // this turn by a source you controlled"
    for (p, by) in [
        (
            " dealt damage by enchanted creature this turn",
            Filter::DealtDamageThisTurnBy(Box::new(Sel::AttachedTo)),
        ),
        (
            " dealt damage by ~ this turn",
            Filter::DealtDamageThisTurnBy(Box::new(Sel::This)),
        ),
    ] {
        if let Some(x) = s.strip_suffix(p) {
            let (f, _) = subject(x, b)?;
            return Some((Filter::and(vec![f, by]), Sel::TriggerObject));
        }
    }
    let (other, r) = if let Some(r) = word(s, "another") {
        (true, r.trim_start())
    } else {
        let r = word(s, "a").or_else(|| word(s, "an"))?;
        (false, r.trim_start())
    };
    // "a creature you control that's enchanted"
    let (r, enchanted) = match r.strip_suffix(" that's enchanted") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let (f, plural, rest) = parse_object_phrase(r)?;
    if plural || !rest.trim().is_empty() {
        return None;
    }
    let mut parts = vec![f];
    if other {
        parts.push(Filter::Other);
    }
    if enchanted {
        parts.push(Filter::Enchanted);
    }
    Some((Filter::and(parts), Sel::TriggerObject))
}

/// "a graveyard", "your graveyard", "an opponent's graveyard", "its owner's graveyard": the filter on whose graveyard (the card's owner,
/// CR 400.3).
fn graveyard(s: &str) -> Option<(Option<Filter>, &str)> {
    for (p, f) in [
        ("a graveyard", None),
        ("its owner's graveyard", None),
        ("your graveyard", Some(Filter::OwnedBy(PlayerRel::You))),
        (
            "an opponent's graveyard",
            Some(Filter::OwnedBy(PlayerRel::Opponent)),
        ),
    ] {
        if let Some(r) = word(s, p) {
            return Some((f, r));
        }
    }
    None
}

fn zone_event(s: &str, b: &mut OneShot) -> Option<ZoneEvent> {
    let s = s.trim();
    let (this_turn, s) = match s.strip_suffix(" this turn") {
        Some(x) => (true, x),
        None => (false, s),
    };
    if let Some(subj) = s.strip_suffix(" would die") {
        let (f, it) = subject(subj, b)?;
        return Some(ZoneEvent {
            event: ReplacementEvent::Dies(f),
            this_turn,
            it,
        });
    }
    if let Some(subj) = s.strip_suffix(" would leave the battlefield") {
        let (f, it) = subject(subj, b)?;
        return Some(ZoneEvent {
            event: ReplacementEvent::ZoneChange {
                filter: f,
                from: Some(ZoneKind::Battlefield),
                to: None,
            },
            this_turn,
            it,
        });
    }
    if let Some(subj) = s.strip_suffix(" would be destroyed") {
        let (f, it) = subject(subj, b)?;
        return Some(ZoneEvent {
            event: ReplacementEvent::Destroy(f),
            this_turn,
            it,
        });
    }
    let (subj, r) = s.split_once(" would be put into ")?;
    let (owner, r) = graveyard(r)?;
    let from = match r.trim() {
        "" | "from anywhere" => None,
        "from the battlefield" => Some(ZoneKind::Battlefield),
        _ => return None,
    };
    let (f, it) = subject(subj, b)?;
    let filter = match owner {
        Some(o) => Filter::and(vec![f, o]),
        None => f,
    };
    Some(ZoneEvent {
        event: ReplacementEvent::ZoneChange {
            filter,
            from,
            to: Some(ZoneKind::Graveyard),
        },
        this_turn,
        it,
    })
}

/// "exile it", "exile that card with an ice counter on it", "return it to its owner's
/// hand", "put it on top of its owner's library", "[reveal ~ and] shuffle it into its
/// owner's library": a move of the object somewhere else.
fn move_instead(s: &str) -> Option<Destination> {
    let s = s.trim();
    let s = s
        .strip_prefix("reveal ~ and ")
        .or_else(|| s.strip_prefix("reveal it and "))
        .unwrap_or(s);
    for it in ["it", "that card", "~", "him", "her"] {
        if s == format!("exile {it}") {
            return Some(Destination::zone(ZoneKind::Exile));
        }
        if let Some(c) = s
            .strip_prefix(&format!("exile {it} with "))
            .and_then(|c| c.strip_suffix(" on it"))
        {
            let (n, rest) = parse_number(c)?;
            let (kind, rest) = crate::oracle::costs::counter_kind(rest)?;
            if !matches!(rest.trim(), "counter" | "counters") {
                return None;
            }
            let mut d = Destination::zone(ZoneKind::Exile);
            d.with_counters = vec![(kind, n)];
            return Some(d);
        }
        if s == format!("return {it} to its owner's hand") {
            return Some(Destination::zone(ZoneKind::Hand));
        }
        if s == format!("put {it} on top of its owner's library") {
            return Some(Destination::library_top());
        }
        if s == format!("put {it} on the bottom of its owner's library") {
            return Some(Destination::library_bottom());
        }
        if s == format!("shuffle {it} into its owner's library") {
            let mut d = Destination::zone(ZoneKind::Library);
            d.position = LibraryPosition::Shuffled;
            return Some(d);
        }
    }
    None
}

/// The replacement for a zone change: (action, optional).
fn zone_action(s: &str, ev: &ZoneEvent, ctx: &CompileContext) -> Option<(ReplacementAction, bool)> {
    let s = end(s.trim());
    if let Some(r) = s.strip_prefix("you may ") {
        let (a, _) = zone_action(r, ev, ctx)?;
        return Some((a, true));
    }
    // "exile it instead of putting it anywhere else" (leaving the battlefield).
    if let Some(r) = s.strip_suffix(" instead of putting it anywhere else") {
        if !matches!(
            ev.event,
            ReplacementEvent::ZoneChange { to: None, .. }
        ) {
            return None;
        }
        return Some((ReplacementAction::MoveInstead(move_instead(r)?), false));
    }
    let inner = s
        .strip_suffix(" instead")
        .or_else(|| s.strip_prefix("instead "))?;
    if let Some(d) = move_instead(inner) {
        return Some((ReplacementAction::MoveInstead(d), false));
    }
    // "instead exile it and put a hit counter on it"
    let e = instructions(inner, ctx, ev.it.clone())?;
    Some((ReplacementAction::Instead(Box::new(e)), false))
}

/// Splits "[event], [action]" at each comma.
fn splits(s: &str) -> Vec<(&str, &str)> {
    s.match_indices(", ")
        .map(|(i, _)| (&s[..i], &s[i + 2..]))
        .collect()
}

fn s_if_zone(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let r = l.strip_prefix("if ")?;
    for (ev, act) in splits(r) {
        let mut none: OneShot = None;
        let Some(zev) = zone_event(ev, &mut none) else {
            continue;
        };
        if zev.this_turn {
            return None;
        }
        let Some((action, optional)) = zone_action(act, &zev, ctx) else {
            continue;
        };
        let def = ReplacementDef {
            event: zev.event,
            action,
            self_replacement: false,
            optional,
        };
        return Some(vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
            text,
        )]);
    }
    None
}

inventory::submit! { StaticPattern { name: "replacement grammar: if [zone change], [replacement]", priority: 150, parse: s_if_zone } }

fn p_if_zone(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, r) = duration_prefix(l);
    let r = r.strip_prefix("if ")?;
    let ctx = b.ctx;
    for (ev, act) in splits(r) {
        let got = attempt(b, |b| {
            let mut ob: OneShot = Some(b);
            let zev = zone_event(ev, &mut ob)?;
            let duration = match (&dur, zev.this_turn) {
                (Some(d), false) => d.clone(),
                (None, true) => Duration::EndOfTurn,
                _ => return None,
            };
            let (action, optional) = zone_action(act, &zev, ctx)?;
            Some(Effect::AddReplacement {
                def: ReplacementDef {
                    event: zev.event,
                    action,
                    self_replacement: false,
                    optional,
                },
                duration,
                uses: None,
            })
        });
        if got.is_some() {
            return got;
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "replacement grammar: [this turn] if [zone change], [replacement]", priority: 150, parse: p_if_zone } }

/// "If [a nontoken creature | ~] would enter and it wasn't cast, exile it instead."
/// (Containment Priest, Hallowed Moonlight): a permanent that enters other than by
/// resolving as a spell (CR 601.1, 608.3) — one that isn't coming from the stack.
fn not_cast_entry(l: &str, b: &mut OneShot) -> Option<(ReplacementDef, bool)> {
    let r = end(l.trim()).strip_prefix("if ")?;
    let (subj, act) = r.split_once(" would enter and it wasn't cast, ")?;
    let (this_turn, act) = (false, act);
    let (f, _) = subject(subj, b)?;
    let d = move_instead(act.strip_suffix(" instead")?)?;
    Some((
        ReplacementDef {
            // Checked against the object where it is (not as it would exist on the
            // battlefield): a spell resolving is on the stack.
            event: ReplacementEvent::Where {
                event: Box::new(ReplacementEvent::EntersBattlefield(f)),
                cond: Condition::Not(Box::new(Condition::SelMatches(
                    Sel::TriggerObject,
                    Filter::InZone(ZoneKind::Stack),
                ))),
            },
            action: ReplacementAction::MoveInstead(d),
            self_replacement: false,
            optional: false,
        },
        this_turn,
    ))
}

fn s_not_cast_entry(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut none: OneShot = None;
    let (def, _) = not_cast_entry(l, &mut none)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacement grammar: would enter and it wasn't cast", priority: 150, parse: s_not_cast_entry } }

fn p_not_cast_entry(l: &str, b: &mut Builder) -> Option<Effect> {
    let (dur, r) = duration_prefix(end(l.trim()));
    let dur = dur?;
    attempt(b, |b| {
        let mut ob: OneShot = Some(b);
        let (def, _) = not_cast_entry(r, &mut ob)?;
        Some(Effect::AddReplacement {
            def,
            duration: dur,
            uses: None,
        })
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: until end of turn, would enter and it wasn't cast", priority: 150, parse: p_not_cast_entry } }

/// "If ~ would be put into a graveyard from anywhere, [replacement]." on an instant or
/// sorcery (Nexus of Fate): the ability functions from every zone (CR 113.6b), including
/// the stack as the spell resolves.
fn a_spell_from_anywhere(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() {
        return None;
    }
    let l = block.trim().to_lowercase();
    let l = end(&l);
    let r = l.strip_prefix("if ~ would be put into a graveyard from anywhere, ")?;
    let zev = zone_event("~ would be put into a graveyard from anywhere", &mut None)?;
    let (action, optional) = zone_action(r, &zev, ctx)?;
    let mut st = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: zev.event,
        action,
        self_replacement: false,
        optional,
    }));
    st.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), block.trim())])
}

inventory::submit! { super::AbilityPattern { name: "replacement grammar: spell put into a graveyard from anywhere", priority: 150, parse: a_spell_from_anywhere } }

/// "If a spell or ability an opponent controls causes you to discard [~ | a card], [put it
/// onto the battlefield [with N counters on it] instead of putting it into your graveyard
/// | you may reveal that card and put it on top of your library instead of putting it
/// anywhere else]." (CR 701.9, 614.1a, 614.6)
fn s_discard_caused_by(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l.trim()).strip_prefix("if a spell or ability an opponent controls causes you to discard ")?;
    let (what, act) = r.split_once(", ")?;
    let filter = match what {
        "~" => Filter::Source,
        "a card" => Filter::Any,
        _ => return None,
    };
    let (optional, act) = match act.strip_prefix("you may ") {
        Some(x) => (true, x),
        None => (false, act),
    };
    let dest = if let Some(x) = act
        .strip_suffix(" instead of putting it into your graveyard")
        .and_then(|x| x.strip_prefix("put it onto the battlefield"))
    {
        let mut d = Destination::battlefield();
        let x = x.trim();
        if let Some(c) = x.strip_prefix("with ").and_then(|c| c.strip_suffix(" on it")) {
            let (n, rest) = parse_number(c)?;
            let (kind, rest) = crate::oracle::costs::counter_kind(rest)?;
            if !matches!(rest.trim(), "counter" | "counters") {
                return None;
            }
            d.with_counters = vec![(kind, n)];
        } else if !x.is_empty() {
            return None;
        }
        d
    } else {
        match act {
            "reveal that card and put it on top of your library instead of putting it anywhere else" => {
                Destination::library_top()
            }
            _ => return None,
        }
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility {
            condition: None,
            effect: StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::DiscardCausedBy {
                    who: PlayerFilter::You,
                    filter,
                    by: PlayerRel::Opponent,
                },
                action: ReplacementAction::MoveInstead(dest),
                self_replacement: false,
                optional,
            }),
            // The card's own ability functions in the hand it's discarded from (CR
            // 113.6); a permanent's (Nephalia Academy) on the battlefield.
            zone: if what == "~" {
                FunctionZone::Hand
            } else {
                FunctionZone::Battlefield
            },
            is_cda: false,
        }),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacement grammar: if an opponent's spell or ability causes you to discard", priority: 150, parse: s_discard_caused_by } }
