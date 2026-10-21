//! Durations, grammar IV (cards3-triggers-durations): "for as long as" written before the
//! instructions it applies to (CR 611.2b).
//!
//! "For as long as ~ remains tapped, another target permanent loses all abilities and
//! can't attack or block." (Immovable Rod), "For as long as ~ remains on the battlefield,
//! gain control of that permanent, it loses all abilities, and it can't attack or block."
//! (Opportunistic Dragon): each instruction of the list is read with the duration after
//! it, as the core grammar reads "[instruction] for as long as [condition]"; a second
//! predicate of the same subject ("... and can't attack or block") is about it.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// Verbs that start a predicate whose subject is the previous clause's.
const PREDICATE_STARTS: [&str; 14] = [
    "can't ", "cannot ", "doesn't ", "don't ", "gets ", "get ", "gains ", "gain ", "loses ",
    "lose ", "has ", "have ", "is ", "becomes ",
];

/// The instructions of a list: "A, B, and C", "A and C" (a predicate without a subject
/// becomes "it [predicate]").
fn clauses(s: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    for chunk in s.split(", ") {
        let chunk = chunk.strip_prefix("and ").unwrap_or(chunk);
        // "loses all abilities and can't attack or block".
        let mut rest = chunk;
        let mut pieces = Vec::new();
        while let Some((a, b)) = rest.split_once(" and ") {
            if PREDICATE_STARTS.iter().any(|p| b.starts_with(p)) {
                pieces.push(a.to_string());
                rest = b;
                pieces.push(format!("it {rest}"));
                rest = "";
                break;
            }
            break;
        }
        if pieces.is_empty() {
            parts.push(chunk.to_string());
        } else {
            if !rest.is_empty() {
                pieces.push(rest.to_string());
            }
            parts.extend(pieces);
        }
    }
    parts
}

fn for_as_long_as_first(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for as long as ")?;
    let (cond, rest) = r.split_once(", ")?;
    let suffix = format!(" for as long as {cond}");
    // Only durations the core grammar reads.
    let (d, _) = crate::oracle::effects::duration_suffix(&format!("x{suffix}"));
    if matches!(d, Duration::Permanent) {
        return None;
    }
    let parts = clauses(rest);
    if parts.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for p in parts {
        // A clause that isn't an instruction with its own duration (a trigger "at the
        // beginning of ...") isn't read here.
        if p.starts_with("at ") || p.starts_with("when") {
            return None;
        }
        out.push(parse_sentence(&format!("{p}{suffix}"), b)?);
    }
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "durations 4: for as long as [condition], [instructions]", priority: 100, parse: for_as_long_as_first } }

/// "that player can't attack you or permanents you control during their next turn"
/// (Willie Lumpkin, Postman; The Second Doctor's "each opponent who does"): a rule for
/// that player's next turn — creatures they control (the active player's, then) can't
/// attack you or planeswalkers you control, whenever they came under their control
/// (CR 611.2c, 508.1c; see `next_turn.rs` and the interpretation
/// `next-turn-effects-start-with-the-turn`).
fn cant_attack_you_during_their_next_turn(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let rest = l
        .strip_prefix("that player can't attack you")
        .or_else(|| l.strip_prefix("they can't attack you"))?;
    let planeswalkers = match rest {
        " or permanents you control during their next turn"
        | " or planeswalkers you control during their next turn" => true,
        " during their next turn" => false,
        _ => return None,
    };
    let who = b.it_player.clone();
    if matches!(who, PlayerRef::You) {
        return None;
    }
    Some(Effect::DuringNext {
        who,
        part: NextTurnPart::Turn,
        effect: Box::new(Effect::AddRestriction {
            restriction: Restriction::CantAttackPlayer {
                attackers: Filter::and(vec![
                    Filter::creature(),
                    Filter::ControllerMatches(Box::new(PlayerFilter::Active)),
                ]),
                defender: PlayerFilter::You,
                planeswalkers,
                battles: false,
            },
            duration: Duration::EndOfTurn,
        }),
    })
}

inventory::submit! { EffectPattern { name: "durations 4: that player can't attack you during their next turn", priority: 100, parse: cant_attack_you_during_their_next_turn } }

/// "[player] plays with their hand revealed [for a duration]" (Stromgald Spy, Sen
/// Triplets): a player effect of the resolving spell or ability (CR 701.20; the static
/// form is `a701_reveal.rs`).
fn plays_with_hand_revealed(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "you may have defending player play with their hand revealed ..." ("you may" is
    // read by the core grammar).
    if let Some(r) = l.strip_prefix("have ") {
        return plays_with_hand_revealed(r, b);
    }
    let (d, l) = crate::oracle::effects::duration_suffix(l);
    let l = l.trim();
    let subj = l
        .strip_suffix(" plays with their hand revealed")
        .or_else(|| l.strip_suffix(" play with their hand revealed"))?;
    let who = match subj {
        "that player" | "they" => b.it_player.clone(),
        "defending player" => PlayerRef::DefendingPlayer,
        _ => {
            let phrase = format!("{subj} ");
            let (r, spec, after) = crate::oracle::phrases::parse_player(&phrase)?;
            if !after.trim().is_empty() {
                return None;
            }
            match spec {
                Some(spec) => PlayerRef::Target(b.add_target(spec, subj)),
                None => r,
            }
        }
    };
    if matches!(d, Duration::Permanent) {
        return None;
    }
    Some(Effect::AddPlayerEffect {
        who,
        effect: PlayerModification::Custom(crate::reveal::PLAYS_WITH_HAND_REVEALED.into()),
        duration: d,
    })
}

inventory::submit! { EffectPattern { name: "durations 4: [player] plays with their hand revealed [duration]", priority: 100, parse: plays_with_hand_revealed } }

/// The replacement effect stopping counters from being put on objects or players
/// (CR 614.1, 122.6; the static form is `r122_cant_have_counters_put.rs`).
fn counters_prevented(on_objects: Option<Filter>, on_players: Option<PlayerFilter>, d: Duration) -> Effect {
    Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::PutCounters {
                on_objects,
                on_players,
                kind: None,
            },
            action: ReplacementAction::Prevent,
            self_replacement: false,
            optional: false,
        },
        duration: d,
        uses: None,
    }
}

/// "It can't have counters put on it for as long as ~ remains on the battlefield.", "That
/// player can't get counters for as long as ~ remains on the battlefield." (Suncleanser).
fn cant_have_counters_for_duration(l: &str, b: &mut Builder) -> Option<Effect> {
    let (d, l) = crate::oracle::effects::duration_suffix(end(l));
    if matches!(d, Duration::Permanent) {
        return None;
    }
    let l = l.trim();
    if let Some(subj) = l.strip_suffix(" can't get counters") {
        let who = match subj {
            "that player" | "they" => b.it_player.clone(),
            _ => return None,
        };
        if matches!(who, PlayerRef::You) {
            return None;
        }
        return Some(counters_prevented(
            None,
            Some(PlayerFilter::Ref(Box::new(who))),
            d,
        ));
    }
    let subj = l
        .strip_suffix(" can't have counters put on it")
        .or_else(|| l.strip_suffix(" can't have counters put on them"))?;
    let sel = match subj {
        "it" | "that creature" | "that permanent" => b.it.clone(),
        _ => return None,
    };
    if matches!(sel, Sel::None | Sel::This) {
        return None;
    }
    Some(counters_prevented(Some(Filter::In(Box::new(sel))), None, d))
}

inventory::submit! { EffectPattern { name: "durations 4: it can't have counters put on it [duration]", priority: 100, parse: cant_have_counters_for_duration } }

/// "gain control of target creature that player controls for as long as they're the
/// monarch" (Garland, Royal Kidnapper): a duration about that player (fixed as the effect
/// begins, see `control_change.rs`), which ends for good once they stop being the monarch
/// (CR 611.2b).
fn for_as_long_as_theyre_the_monarch(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" for as long as they're the monarch")?;
    let who = b.it_player.clone();
    if matches!(who, PlayerRef::You) {
        return None;
    }
    let mut e = parse_sentence(r, b)?;
    let Effect::GainControl { duration, .. } = &mut e else {
        return None;
    };
    if !matches!(duration, Duration::Permanent) {
        return None;
    }
    *duration = Duration::WhileCondition(Condition::PlayerMatches(who, PlayerFilter::Monarch));
    Some(e)
}

inventory::submit! { EffectPattern { name: "durations 4: for as long as they're the monarch", priority: 100, parse: for_as_long_as_theyre_the_monarch } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_instruction_lists() {
        assert_eq!(
            clauses("another target permanent loses all abilities and can't attack or block"),
            vec![
                "another target permanent loses all abilities".to_string(),
                "it can't attack or block".to_string()
            ]
        );
        assert_eq!(
            clauses("gain control of that permanent, it loses all abilities, and it can't attack or block"),
            vec![
                "gain control of that permanent".to_string(),
                "it loses all abilities".to_string(),
                "it can't attack or block".to_string()
            ]
        );
    }
}
