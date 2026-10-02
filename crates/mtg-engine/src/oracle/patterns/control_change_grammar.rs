//! Control-change grammar (CR 108.4, 613.1b, 701.12), read compositionally.
//!
//! ```text
//! control  := [subject] ("gain" | "gains") "control of" objects [duration]
//!           | "until end of turn," instruction "and" instruction
//! subject  := "" | "you" | player ("target opponent", "that player", "its controller")
//!           | object "'s controller" ("that creature's controller", "that source's
//!             controller")
//! objects  := referent | target phrase | "all" objects ["that were attached to it"]
//! duration := "until end of turn" | "for as long as it has a shield counter on it" | ...
//! regain   := "each player gains control of each/all" objects "they own [that you
//!             control]"
//! exchange := "exchange control of two [other] target" objects "controlled by different
//!             players"
//! ```
//!
//! A control-changing effect of a resolving spell or ability lasts for its duration, or
//! until the game ends if there's none (CR 611.2a); it applies in layer 2 (CR 613.1b).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, object_ref, parse_clause, player_ref, Builder};
use crate::oracle::patterns::oracle_hardening_referents::is_no_referent;
use crate::oracle::phrases::*;

/// The player who gains control.
fn subject(s: &str, b: &mut Builder) -> Option<PlayerRef> {
    let s = s.trim();
    if s.is_empty() || s == "you" {
        return Some(PlayerRef::You);
    }
    // "that creature's controller": the controller of what the text refers to. ("That
    // source's controller" needs the trigger's source, which "it" doesn't name.)
    if let Some(r) = s.strip_suffix("'s controller") {
        let (sel, rest) = object_ref(r, b)?;
        if !rest.trim().is_empty() || is_no_referent(&sel) || matches!(sel, Sel::None) {
            return None;
        }
        return Some(PlayerRef::ControllerOf(Box::new(sel)));
    }
    let (p, rest) = player_ref(s, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    Some(p)
}

/// The objects whose control changes: "all Equipment that were attached to it" (as they
/// were attached to it as it last existed, CR 608.2h), or any object phrase.
fn objects(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("all ") {
        let r = r.replace(" that were attached to ", " attached to ");
        if let Some((f, rest)) = super::attach_control_grammar::attached_objects(&r, b) {
            return Some((Sel::All(f), rest));
        }
    }
    let (sel, rest) = object_ref(s, b)?;
    if matches!(sel, Sel::Players(_) | Sel::None) {
        return None;
    }
    if let Sel::All(f) = &sel {
        if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
            return None;
        }
    }
    Some((sel, rest))
}

/// "[subject] gain(s) control of [objects] [duration]".
fn p_control(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subj, obj) = if let Some(r) = l.strip_prefix("gain control of ") {
        ("", r)
    } else {
        l.split_once(" gains control of ")
            .or_else(|| l.split_once(" gain control of "))?
    };
    // "Each player gains control of ...": see [`p_each_player_regains`].
    if subj.starts_with("each ") || subj.contains(" and ") || subj.contains(" may") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let who = subject(subj, b)?;
        let (duration, obj) = duration_suffix(obj);
        let (what, rest) = objects(obj, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        Some(Effect::GainControl {
            what,
            who,
            duration,
        })
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "control grammar: [player] gains control of [objects] [duration]", priority: 110, parse: p_control } }

/// "Each player gains control of each land they own that you control" (Herald of
/// Leshrac), "... of all nontoken permanents they own": each player gains control of
/// their own permanents of that kind (CR 108.4).
fn p_each_player_regains(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player gains control of ")?;
    let r = r.strip_prefix("each ").or_else(|| r.strip_prefix("all "))?;
    let (r, from_you) = match r.strip_suffix(" they own that you control") {
        Some(r) => (r, true),
        None => (r.strip_suffix(" they own")?, false),
    };
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    let mut parts = vec![
        f,
        Filter::InZone(ZoneKind::Battlefield),
        Filter::OwnedBy(PlayerRel::Iterated),
        Filter::not(Filter::ControlledBy(PlayerRel::Iterated)),
    ];
    if from_you {
        parts.push(Filter::ControlledBy(PlayerRel::You));
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::GainControl {
            what: Sel::All(Filter::and(parts)),
            who: PlayerRef::Iterated,
            duration: Duration::Permanent,
        }),
    })
}

inventory::submit! { EffectPattern { name: "control grammar: each player gains control of each [permanent] they own", priority: 110, parse: p_each_player_regains } }

/// "Until end of turn, you gain control of target creature and it gains haste." (Grab
/// the Reins): the duration applies to both instructions.
fn p_leading_until_end_of_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("until end of turn, ")?;
    let (first, second) = r.split_once(" and ")?;
    if first.contains(" until ") || second.contains(" until ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let a = parse_clause(&format!("{first} until end of turn"), b);
    let c = a
        .as_ref()
        .and_then(|_| parse_clause(&format!("{second} until end of turn"), b));
    match (a, c) {
        (Some(a), Some(c)) if has_end_of_turn(&a) && has_end_of_turn(&c) => {
            Some(Effect::seq(vec![a, c]))
        }
        _ => {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            None
        }
    }
}

/// Whether an instruction took the "until end of turn" duration.
fn has_end_of_turn(e: &Effect) -> bool {
    matches!(
        e,
        Effect::GainControl {
            duration: Duration::EndOfTurn,
            ..
        } | Effect::Modify {
            duration: Duration::EndOfTurn,
            ..
        }
    )
}

inventory::submit! { EffectPattern { name: "control grammar: until end of turn, [instruction] and [instruction]", priority: 110, parse: p_leading_until_end_of_turn } }

/// "Gain control of that creature until end of turn, untap it, and it gains haste until
/// end of turn": a list of three instructions, performed in order.
fn p_three_instructions(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, last) = l.rsplit_once(", and ")?;
    let (first, second) = head.split_once(", ")?;
    if second.contains(", ") || !first.contains("gain control of ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let mut out = Vec::new();
    for part in [first, second, last] {
        match parse_clause(part, b) {
            Some(e) => out.push(e),
            None => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
                return None;
            }
        }
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "control grammar: gain control of [object], [instruction], and [instruction]", priority: 110, parse: p_three_instructions } }

/// "Exchange control of two [other] target creatures controlled by different players"
/// (Modify Memory, Kitsune, Dragon's Daughter): the targets must have different
/// controllers (CR 701.12b: an exchange between one player's permanents does nothing).
fn p_exchange_different_players(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exchange control of ")?;
    let r = r.strip_suffix(" controlled by different players")?;
    if !r.starts_with("two ") {
        return None;
    }
    let (mut spec, tail) = parse_target(r)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
        return None;
    }
    if spec.min.as_const() != Some(2) || spec.max.as_const() != Some(2) {
        return None;
    }
    spec.together = Some(TargetGroup::DifferentControllers);
    let slot = b.add_target(spec, &format!("{r} controlled by different players"));
    Some(Effect::ExchangeControl {
        a: Sel::Target(slot),
        b: Sel::Target(slot),
    })
}

inventory::submit! { EffectPattern { name: "control grammar: exchange control of two target creatures controlled by different players", priority: 110, parse: p_exchange_different_players } }

/// Whether an effect ends by changing control.
fn ends_with_control_change(e: &Effect) -> bool {
    match e {
        Effect::GainControl { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_control_change),
        _ => false,
    }
}

/// "Target opponent gains control of another target permanent you control. If they do,
/// you draw a card." / "When they do, you draw two cards ..." (Yes Man, Personal
/// Securitron; a reflexive triggered ability, CR 603.12): only if the player gained
/// control of it (not if it left the battlefield first, or they already controlled it).
fn f_if_they_gain_control(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_control_change(prev) {
        return false;
    }
    let l = end(l);
    let then = if let Some(r) = l.strip_prefix("if they do, ") {
        match parse_clause(r, b) {
            Some(e) => e,
            None => return false,
        }
    } else if let Some(r) = l.strip_prefix("when they do, ") {
        match super::r600_triggers::reflexive_body(r, b) {
            Some(body) => Effect::Reflexive {
                body: Box::new(body),
            },
            None => return false,
        }
    } else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "control grammar: if/when they do (gained control)", priority: 110, apply: f_if_they_gain_control } }

/// "target permanent you own but don't control" (Coveted Falcon).
fn but_dont_control<'a>(t: &'a str, so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("but don't control")?;
    // Only after "you own".
    let owned = match so_far {
        Filter::And(v) => v.iter().any(|f| matches!(f, Filter::OwnedBy(PlayerRel::You))),
        f => matches!(f, Filter::OwnedBy(PlayerRel::You)),
    };
    (owned && (r.is_empty() || r.starts_with([' ', ',', '.'])))
        .then_some((Filter::ControlledBy(PlayerRel::NotYou), r))
}

inventory::submit! { super::FilterSuffixPattern { name: "control grammar: you own but don't control", priority: 100, parse: but_dont_control } }

/// "Draw a card for each one they gained control of this way." (Coveted Falcon) after a
/// control change: one card for each permanent whose controller changed (not those the
/// player already controlled, or that had left the battlefield).
fn f_draw_for_each_gained(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !ends_with_control_change(prev) {
        return false;
    }
    if !matches!(
        end(l),
        "draw a card for each one they gained control of this way"
            | "draw a card for each permanent they gained control of this way"
    ) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::Draw {
            who: PlayerRef::You,
            n: Value::Prev,
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "control grammar: draw a card for each one they gained control of this way", priority: 110, apply: f_draw_for_each_gained } }

/// First variables for the two sets of permanents swapped by "you and [player] each gain
/// control of all [objects] the other controls".
const SWAPPED_THEIRS: Var = vars::USER + 8320;
const SWAPPED_YOURS: Var = vars::USER + 8321;

/// "You and target opponent each gain control of all creatures the other controls until
/// end of turn." (Twist Allegiance): both sets are determined first, then each player
/// gains control of the other's (CR 611.2c). "Those creatures" afterwards are both sets.
fn p_swap_all(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you and ")?;
    let (other, r) = r.split_once(" each gain control of all ")?;
    let (r, duration) = {
        let (d, r) = duration_suffix(r);
        (r, d)
    };
    let noun = r.strip_suffix(" the other controls")?;
    let (f, plural, tail) = parse_object_phrase(noun)?;
    if !plural || !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let Some((who, rest)) = player_ref(other, b).filter(|(_, rest)| end(rest).is_empty()) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    let _ = rest;
    if matches!(who, PlayerRef::You) {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    let on_battlefield = |c: Filter| Filter::and(vec![f.clone(), Filter::InZone(ZoneKind::Battlefield), c]);
    let theirs = Sel::All(on_battlefield(super::value_grammar::controlled_by(&who)));
    let yours = Sel::All(on_battlefield(Filter::ControlledBy(PlayerRel::You)));
    let both = Sel::Union(vec![Sel::Var(SWAPPED_THEIRS), Sel::Var(SWAPPED_YOURS)]);
    let group = super::pronoun_groups::GROUP;
    let it_before = std::mem::replace(&mut b.it, Sel::Var(group));
    b.group = Some(super::pronoun_groups::GroupRef {
        sel: both.clone(),
        it_before,
    });
    Some(Effect::Seq(vec![
        Effect::Store {
            var: SWAPPED_THEIRS,
            sel: theirs,
        },
        Effect::Store {
            var: SWAPPED_YOURS,
            sel: yours,
        },
        Effect::Store {
            var: group,
            sel: both,
        },
        Effect::GainControl {
            what: Sel::Var(SWAPPED_THEIRS),
            who: PlayerRef::You,
            duration: duration.clone(),
        },
        Effect::GainControl {
            what: Sel::Var(SWAPPED_YOURS),
            who,
            duration,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "control grammar: you and [player] each gain control of all [objects] the other controls", priority: 110, parse: p_swap_all } }

/// "If you control neither creature, ..." after an instruction about two targets (Modify
/// Memory, after the exchange): you control none of the targets.
fn neither_target(c: &str) -> Option<Condition> {
    let noun = end(c).strip_prefix("you control neither ")?;
    let (f, plural, tail) = parse_object_phrase(noun)?;
    if plural || !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    Some(Condition::Not(Box::new(Condition::Exists(Filter::and(vec![
        f,
        Filter::In(Box::new(Sel::AllTargets)),
        Filter::ControlledBy(PlayerRel::You),
    ])))))
}

inventory::submit! { super::ConditionPattern { name: "control grammar: you control neither [target]", priority: 110, parse: neither_target } }

/// "[you may] have two target players exchange life totals" (Axis of Mortality): the
/// players named perform the exchange (CR 701.12).
fn p_have_players_exchange(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("have ")?;
    let (subject, _) = r.split_once(" exchange ")?;
    if !subject.contains("player") && !subject.contains("opponent") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = parse_clause(r, b);
    if !matches!(e, Some(Effect::ExchangeLifeTotals { .. })) {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    }
    e
}

inventory::submit! { EffectPattern { name: "control grammar: have [players] exchange [...]", priority: 110, parse: p_have_players_exchange } }
