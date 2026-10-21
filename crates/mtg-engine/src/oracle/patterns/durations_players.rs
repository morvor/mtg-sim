//! Durations bound to another player's next turn or combat phase, as grammar
//! ([`Effect::DuringNext`], `next_turn.rs`):
//!
//! * "During target opponent's next turn, creatures that player controls attack ~ if
//!   able." / "During target player's next turn, each creature that player controls
//!   attacks if able." — the player phrase is the target "that player" refers to;
//! * "that creature can't attack during its controller's next turn", "Up to one target
//!   creature an opponent controls attacks ~ during its controller's next turn if able." —
//!   the controller of the object the instruction is about, as the effect is created;
//! * "Each opponent can't cast instant or sorcery spells during that player's next
//!   turn." — each of those players, during their own next turn;
//! * "target creature an opponent controls attacks during its controller's next combat
//!   phase if able" — that player's next combat phase, until it ends;
//! * "Target opponent skips all combat phases of their next turn." (CR 614.10).
//!
//! The instruction is read as the same instruction lasting "this turn". Only effects that
//! modify the rules (restrictions, requirements and skips) are accepted: they're created
//! as that turn begins (see `next_turn.rs`), which only they don't notice.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::{end, parse_player};

/// Whether `e` only creates rule-modifying effects lasting "this turn", and makes them
/// last until end of combat instead with `combat`.
fn rules_only(e: &mut Effect, combat: bool) -> bool {
    let fix = |d: &mut Duration| {
        if !matches!(d, Duration::EndOfTurn | Duration::ThisTurn) {
            return false;
        }
        if combat {
            *d = Duration::EndOfCombat;
        }
        true
    };
    match e {
        Effect::AddRestriction { duration, .. } => fix(duration),
        Effect::AddReplacement {
            duration,
            def,
            uses: None,
        } => matches!(def.event, ReplacementEvent::SkipStep { .. }) && fix(duration),
        Effect::Seq(v) => !v.is_empty() && v.iter_mut().all(|x| rules_only(x, combat)),
        _ => false,
    }
}

/// Inserts "this turn" before a trailing "if able", or at the end.
fn this_turn(s: &str) -> String {
    match s.strip_suffix(" if able") {
        Some(r) => format!("{r} this turn if able"),
        None => format!("{s} this turn"),
    }
}

/// Parses `inner` (with "this turn" added) as the waiting effect.
fn waiting(inner: &str, b: &mut Builder, combat: bool) -> Option<Effect> {
    let mut e = parse_sentence(&this_turn(inner), b)?;
    rules_only(&mut e, combat).then_some(e)
}

fn during(who: PlayerRef, part: NextTurnPart, effect: Effect) -> Effect {
    Effect::DuringNext {
        who,
        part,
        effect: Box::new(effect),
    }
}

/// "During target opponent's next turn, [instruction about that player]".
fn during_target_players_next_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("during ")?;
    let (player, rest) = r.split_once("'s next turn, ")?;
    if !matches!(player, "target opponent" | "target player") {
        return None;
    }
    // "that player" is the target: the instruction names it in its place.
    let i = rest.find("that player")?;
    let inner = format!("{}{player}{}", &rest[..i], &rest[i + "that player".len()..]);
    let phrase = format!("{player} ");
    let (_, spec, after) = parse_player(&phrase)?;
    if !after.is_empty() {
        return None;
    }
    spec?;
    let n0 = b.targets.len();
    let e = waiting(&inner, b, false)?;
    // The player phrase is the instruction's first new target.
    let slot = b
        .targets
        .iter()
        .enumerate()
        .skip(n0)
        .find(|(_, t)| matches!(t.what, TargetKind::Player(_)))
        .map(|(i, _)| i as u8)?;
    Some(during(PlayerRef::Target(slot), NextTurnPart::Turn, e))
}

inventory::submit! { EffectPattern { name: "durations players: during target opponent's next turn", priority: 120, parse: during_target_players_next_turn } }

/// "[object instruction] during its controller's next turn [if able]", "... during its
/// controller's next combat phase if able".
fn during_its_controllers_next(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (part, phrase) = [
        (NextTurnPart::Turn, " during its controller's next turn"),
        (
            NextTurnPart::CombatPhase,
            " during its controller's next combat phase",
        ),
    ]
    .into_iter()
    .find(|(_, p)| l.contains(p))?;
    let i = l.find(phrase)?;
    let tail = &l[i + phrase.len()..];
    if !(tail.is_empty() || tail == " if able") {
        return None;
    }
    let inner = format!("{}{tail}", &l[..i]);
    let e = waiting(&inner, b, part == NextTurnPart::CombatPhase)?;
    // "its": the object the instruction is about.
    let obj = restricted_object(&e)?;
    Some(during(
        PlayerRef::ControllerOf(Box::new(obj)),
        part,
        e,
    ))
}

inventory::submit! { EffectPattern { name: "durations players: during its controller's next turn", priority: 120, parse: during_its_controllers_next } }

/// The single object a restriction or requirement is about ("that creature", "target
/// creature").
fn restricted_object(e: &Effect) -> Option<Sel> {
    let Effect::AddRestriction { restriction, .. } = e else {
        return None;
    };
    let f = match restriction {
        Restriction::CantAttack(f)
        | Restriction::CantBlock(f)
        | Restriction::CantAttackOrBlock(f)
        | Restriction::MustAttack(f) => f,
        Restriction::MustAttackPlayer { attackers, .. }
        | Restriction::MustAttackObject { attackers, .. } => attackers,
        _ => return None,
    };
    match f {
        Filter::In(s) if matches!(**s, Sel::Target(_) | Sel::TriggerObject | Sel::Var(_)) => {
            Some((**s).clone())
        }
        _ => None,
    }
}

/// "Each opponent can't cast instant or sorcery spells during that player's next turn."
fn each_player_during_their_next_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let rest = l.strip_suffix(" during that player's next turn")?;
    let (who, r) = [
        ("each opponent ", PlayerRef::EachOpponent),
        ("each player ", PlayerRef::EachPlayer),
    ]
    .into_iter()
    .find_map(|(p, w)| rest.strip_prefix(p).map(|r| (w, r)))?;
    let saved = b.it_player.clone();
    b.it_player = PlayerRef::Iterated;
    let e = waiting(&format!("that player {r}"), b, false);
    b.it_player = saved;
    Some(during(who, NextTurnPart::Turn, e?))
}

inventory::submit! { EffectPattern { name: "durations players: each opponent ... during that player's next turn", priority: 120, parse: each_player_during_their_next_turn } }

/// "Target opponent skips all combat phases of their next turn." (CR 614.10: each of
/// that turn's combat phases is replaced with nothing).
fn skips_all_combat_phases_of_next_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let player = l.strip_suffix(" skips all combat phases of their next turn")?;
    let phrase = format!("{player} ");
    let (_, spec, after) = parse_player(&phrase)?;
    if !after.is_empty() {
        return None;
    }
    let slot = b.add_target(spec?, player);
    let skip = Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::SkipStep {
                step: StepKind::Combat,
                // Only that player's turn: it lasts for that turn.
                whose: PlayerRel::Any,
            },
            action: ReplacementAction::Prevent,
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: None,
    };
    Some(during(PlayerRef::Target(slot), NextTurnPart::Turn, skip))
}

inventory::submit! { EffectPattern { name: "durations players: skips all combat phases of their next turn", priority: 120, parse: skips_all_combat_phases_of_next_turn } }

/// "Target player skips their next combat phase this turn." (Moment of Silence): only
/// the active player has combat phases this turn, so it does something only if that's
/// the player, and only to a combat phase that begins before the turn ends (CR 614.10).
fn skips_next_combat_phase_this_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let player = l.strip_suffix(" skips their next combat phase this turn")?;
    let phrase = format!("{player} ");
    let (_, spec, after) = parse_player(&phrase)?;
    if !after.is_empty() {
        return None;
    }
    let slot = b.add_target(spec?, player);
    let skip = Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::SkipStep {
                step: StepKind::Combat,
                whose: PlayerRel::Any,
            },
            action: ReplacementAction::Prevent,
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: Some(1),
    };
    Some(Effect::If {
        cond: Condition::PlayerMatches(PlayerRef::Target(slot), PlayerFilter::Active),
        then: Box::new(skip),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "durations players: skips their next combat phase this turn", priority: 120, parse: skips_next_combat_phase_this_turn } }

#[cfg(test)]
mod tests {
    use crate::card::Layout;
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    fn compiles(text: &str, types: &str) -> bool {
        let tl = TypeLine::parse(types);
        let ctx = CompileContext {
            card_name: "Probe",
            full_name: "Probe",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: Some("1"),
            toughness: Some("1"),
        };
        compile(text, &ctx).unsupported.is_empty()
    }

    #[test]
    fn next_turn_durations() {
        for t in [
            "Each opponent can't cast instant or sorcery spells during that player's next turn.",
            "During target player's next turn, creatures that player controls attack you if able.",
            "During target opponent's next turn, each creature that player controls attacks if able.",
            "Target opponent skips all combat phases of their next turn.",
            "Target player skips all combat phases of their next turn.",
        ] {
            assert!(compiles(t, "Sorcery"), "{t}");
        }
        for t in [
            "Whenever ~ blocks a creature, that creature can't attack during its controller's next turn.",
            "Whenever a land you control enters, target creature an opponent controls attacks during its controller's next combat phase if able.",
        ] {
            assert!(compiles(t, "Creature — Wall"), "{t}");
        }
        // Not rule-modifying, or not about a player's next turn.
        for t in [
            "During target player's next turn, that player draws a card.",
            "During target player's next turn, creatures you control attack if able.",
            "Target creature gets +2/+2 during its controller's next turn.",
            "Target opponent skips all combat phases of your next turn.",
        ] {
            assert!(!compiles(t, "Sorcery"), "{t}");
        }
    }
}
