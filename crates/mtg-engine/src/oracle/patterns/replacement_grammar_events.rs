//! Replacement grammar for player events (CR 614.1a, 614.11): "[player] would draw a
//! card [while ...]", "[player] would lose life [during your turn]", "[player] would gain
//! life", "[player] would mill one or more cards", "you would scry a number of cards",
//! "you would proliferate", "a creature you control would explore / connive"; scopes (a
//! static ability, "until end of turn", "this turn"); and replacements "[instructions]
//! instead", "instead [instructions]", "you may [instead] ...", "skip that draw instead",
//! "they lose twice that much life instead", "that many plus N instead".
//!
//! The replacing instructions can say "that many" (the event's amount), "that player" /
//! "they" (the player the event is about) and "that creature" (the object it's about), and
//! can continue in later sentences ("Exile the top two cards of your library instead. You
//! may play those cards this turn.").

use super::replacement_grammar::{attempt, duration_prefix, instructions};
use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// "you", "a player", "an opponent", "each player" as the subject of an event.
fn player_subject(s: &str) -> Option<PlayerFilter> {
    Some(match s.trim() {
        "you" => PlayerFilter::You,
        "a player" | "each player" | "any player" => PlayerFilter::Any,
        "an opponent" | "each opponent" | "one of your opponents" => PlayerFilter::Opponent,
        _ => return None,
    })
}

/// A player event pattern: the event, a condition on the game ("while ..."), and what
/// "it" refers to.
#[derive(Clone, Debug)]
struct PlayerEvent {
    event: ReplacementEvent,
    cond: Option<Condition>,
    this_turn: bool,
    it: Sel,
}

/// "[player] would draw a card [this turn] [while ...]", "[player] would lose life
/// [during your turn]", "[player] would gain life".
fn player_event(s: &str, ctx: &CompileContext) -> Option<PlayerEvent> {
    let s = s.trim();
    let (s, this_turn) = match s.strip_suffix(" this turn") {
        Some(x) => (x, true),
        None => (s, false),
    };
    // "while [condition]" / "during your turn"
    let (s, cond) = if let Some(i) = s.find(" while ") {
        let c = crate::oracle::statics::parse_condition(&s[i + " while ".len()..], ctx)?;
        (&s[..i], Some(c))
    } else if let Some(x) = s.strip_suffix(" during your turn") {
        (x, Some(Condition::YourTurn))
    } else {
        (s, None)
    };
    let (subj, verb) = s.split_once(" would ")?;
    // "a creature you control would explore / connive"
    if let Some(kind) = match verb {
        "explore" => Some(ReplaceableAction::Explore),
        "connive" => Some(ReplaceableAction::Connive),
        _ => None,
    } {
        let r = subj.strip_prefix("a ").or_else(|| subj.strip_prefix("an "))?;
        let (f, plural, rest) = parse_object_phrase(r)?;
        if plural || !rest.trim().is_empty() {
            return None;
        }
        return Some(PlayerEvent {
            event: ReplacementEvent::Action {
                kind,
                who: PlayerFilter::Any,
                objects: Some(f),
            },
            cond,
            this_turn,
            it: Sel::TriggerObject,
        });
    }
    let who = player_subject(subj)?;
    let event = match verb {
        "draw a card" => ReplacementEvent::Draw(who),
        // An instruction to draw cards, whatever their number (CR 121.2a, 616.1g).
        "draw one or more cards" => ReplacementEvent::DrawCards { who, min: 1 },
        "lose life" => ReplacementEvent::LoseLife(who),
        "gain life" => ReplacementEvent::GainLife(who),
        "proliferate" => ReplacementEvent::Action {
            kind: ReplaceableAction::Proliferate,
            who,
            objects: None,
        },
        "learn" => ReplacementEvent::Action {
            kind: ReplaceableAction::Learn,
            who,
            objects: None,
        },
        "scry a number of cards" | "scry one or more cards" => ReplacementEvent::Action {
            kind: ReplaceableAction::Scry,
            who,
            objects: None,
        },
        "mill one or more cards" | "mill a number of cards" => ReplacementEvent::Action {
            kind: ReplaceableAction::Mill,
            who,
            objects: None,
        },
        _ => return None,
    };
    Some(PlayerEvent {
        event,
        cond,
        this_turn,
        it: Sel::TriggerObject,
    })
}

/// The replacement for a player event, given the rest of the text (which can continue
/// in later sentences).
fn player_action(s: &str, ev: &PlayerEvent, ctx: &CompileContext) -> Option<(ReplacementAction, bool)> {
    let s = s.trim();
    let s = s.strip_suffix('.').unwrap_or(s);
    if let Some(r) = s.strip_prefix("you may ") {
        let (a, _) = player_action(r, ev, ctx)?;
        return Some((a, true));
    }
    if let ReplacementEvent::DrawCards { .. } = ev.event {
        for p in ["you draw that many cards plus ", "they draw that many cards plus "] {
            if let Some(x) = s.strip_prefix(p).and_then(|x| x.strip_suffix(" instead")) {
                let (n, rest) = parse_number(x)?;
                if !rest.trim().is_empty() {
                    return None;
                }
                return Some((ReplacementAction::Add(n), false));
            }
        }
        return None;
    }
    let draw = matches!(ev.event, ReplacementEvent::Draw(_));
    if draw && s == "skip that draw instead" {
        return Some((ReplacementAction::Instead(Box::new(Effect::Noop)), false));
    }
    let life = matches!(
        ev.event,
        ReplacementEvent::LoseLife(_) | ReplacementEvent::GainLife(_)
    );
    if life {
        for (p, k) in [
            ("they lose twice that much life instead", 2),
            ("that player loses twice that much life instead", 2),
            ("you lose twice that much life instead", 2),
            ("they gain twice that much life instead", 2),
            ("that player gains twice that much life instead", 2),
            ("you gain twice that much life instead", 2),
        ] {
            if s == p {
                return Some((ReplacementAction::Multiply(k), false));
            }
        }
        if matches!(
            s,
            "that player gains no life instead" | "they gain no life instead"
        ) {
            return Some((ReplacementAction::Prevent, false));
        }
    }
    // "they mill that many cards plus four instead", "scry that many cards plus one
    // instead", "they mill twice that many cards instead": the amount changes (the
    // affected player orders several of them, CR 616.1).
    if let ReplacementEvent::Action {
        kind: ReplaceableAction::Mill | ReplaceableAction::Scry,
        ..
    } = ev.event
    {
        let r = s
            .strip_prefix("they ")
            .or_else(|| s.strip_prefix("you "))
            .or_else(|| s.strip_prefix("that player "))
            .unwrap_or(s);
        let verb_rest = ["mill ", "mills ", "scry ", "scries "]
            .iter()
            .find_map(|p| r.strip_prefix(p));
        // "they mill twice that many cards instead" (Bruvac the Grandiloquent).
        match verb_rest {
            Some("twice that many cards instead") => {
                return Some((ReplacementAction::Multiply(2), false))
            }
            Some("three times that many cards instead") => {
                return Some((ReplacementAction::Multiply(3), false))
            }
            _ => {}
        }
        if let Some(x) = verb_rest
            .and_then(|x| x.strip_prefix("that many cards plus "))
            .and_then(|x| x.strip_suffix(" instead"))
        {
            let (n, rest) = parse_number(x)?;
            if !rest.trim().is_empty() {
                return None;
            }
            return Some((ReplacementAction::Add(n), false));
        }
    }
    // "[instructions] instead[. more instructions]" / "instead [instructions]".
    let (first, rest) = match s.split_once(". ") {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };
    let inner = first
        .strip_prefix("instead ")
        .or_else(|| first.strip_suffix(" instead"))?;
    let text = match rest {
        Some(r) => format!("{inner}. {r}"),
        None => inner.to_string(),
    };
    let e = instructions(&text, ctx, ev.it.clone())?;
    Some((ReplacementAction::Instead(Box::new(e)), false))
}

fn s_if_player(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = l.trim();
    // "The first time you would draw a card each turn, ...": the first draw event this
    // turn, even one another replacement effect replaced (see
    // `kw::first_draw_each_turn`).
    if let Some(r) = l.strip_prefix("the first time you would draw a card each turn, ") {
        let mut pev = player_event("you would draw a card", ctx)?;
        let (action, optional) = player_action(r, &pev, ctx)?;
        pev.event = ReplacementEvent::Where {
            event: Box::new(pev.event),
            cond: Condition::Custom(crate::kw::first_draw_each_turn::FIRST_DRAW.into()),
        };
        let st = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: pev.event,
            action,
            self_replacement: false,
            optional,
        }));
        return Some(vec![AbilityDef::new(AbilityKind::Static(st), text)]);
    }
    let r = l.strip_prefix("if ")?;
    let (ev, act) = r.split_once(", ")?;
    let pev = player_event(ev, ctx)?;
    if pev.this_turn {
        return None;
    }
    let (action, optional) = player_action(act, &pev, ctx)?;
    let mut st = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: pev.event,
        action,
        self_replacement: false,
        optional,
    }));
    st.condition = pev.cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { StaticPattern { name: "replacement grammar: if [player event], [replacement]", priority: 150, parse: s_if_player } }

/// "[Until end of turn, ] if [player event] [this turn], [replacement]" as a one-shot
/// effect (one sentence).
fn p_if_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, r) = duration_prefix(l);
    let r = r.strip_prefix("if ")?;
    let ctx = b.ctx;
    attempt(b, |b| {
        let (ev, act) = r.split_once(", ")?;
        // "If that player or that planeswalker's controller would gain life this turn"
        // (after "target player or planeswalker"): the targeted player, or the controller
        // of the targeted planeswalker, as the effect is created.
        let pev = match ev.strip_prefix("that player or that planeswalker's controller would ")
        {
            Some(verb) => {
                let k = b.targets.len().checked_sub(1)? as u8;
                let who = PlayerFilter::Or(vec![
                    PlayerFilter::Ref(Box::new(PlayerRef::Target(k))),
                    PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(Sel::Target(
                        k,
                    ))))),
                ]);
                let mut pev = player_event(&format!("a player would {verb}"), ctx)?;
                pev.event = match pev.event {
                    ReplacementEvent::GainLife(_) => ReplacementEvent::GainLife(who),
                    ReplacementEvent::LoseLife(_) => ReplacementEvent::LoseLife(who),
                    ReplacementEvent::Draw(_) => ReplacementEvent::Draw(who),
                    _ => return None,
                };
                pev
            }
            None => player_event(ev, ctx)?,
        };
        if pev.cond.is_some() {
            return None;
        }
        let duration = match (&dur, pev.this_turn) {
            (Some(d), false) => d.clone(),
            (None, true) => Duration::EndOfTurn,
            _ => return None,
        };
        if act.contains(". ") {
            return None;
        }
        let (action, optional) = player_action(act, &pev, ctx)?;
        Some(Effect::AddReplacement {
            def: ReplacementDef {
                event: pev.event,
                action,
                self_replacement: false,
                optional,
            },
            duration,
            uses: None,
        })
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: [this turn] if [player event], [replacement]", priority: 150, parse: p_if_player } }

