//! Triggers on players being attacked (CR 508.3b, 508.3e): "whenever you attack a
//! player", "whenever a player attacks you", "whenever an opponent attacks you with two or
//! more creatures", "whenever enchanted player is attacked", "whenever you attack a player
//! with one or more creatures with power 4 or greater".
//!
//! Referents: "that player" is the attacked player when the attacker is named by the
//! subject ("whenever you attack a player, goad target creature that player controls") and
//! the attacking (active) player when the attacked player is ("whenever a player attacks
//! you, … that player"). "That many" is the number of creatures attacking that player.

use crate::ability::*;
use crate::oracle::patterns::triggers::{player_subject, verb};
use crate::oracle::patterns::TriggerPattern;
use crate::oracle::phrases::{end, parse_number, parse_object_phrase};

inventory::submit! {
    TriggerPattern { name: "[player] attacks [player] / [player] is attacked", priority: 100, parse: player_attacked }
}

type Parsed = (TriggerCond, Sel, PlayerRef);

fn enchanted_player() -> PlayerFilter {
    PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo))))
}

fn player_attacked(r: &str) -> Option<Parsed> {
    let r = r.trim();
    // "enchanted player is attacked"
    if let Some(rest) = r.strip_prefix("enchanted player is attacked") {
        let cond = TriggerCond::PlayerAttacked {
            attacker: PlayerRel::Any,
            defender: enchanted_player(),
            with: Filter::creature(),
            min: 1,
        };
        return Some((
            while_clause(cond, rest)?,
            Sel::None,
            PlayerRef::TriggerPlayer,
        ));
    }
    let (attacker, rest) = player_subject(r)?;
    let rest = verb(rest, "attack")?;
    let (defender, rest, it_player) = [
        ("a player", PlayerFilter::Any),
        ("an opponent", PlayerFilter::Opponent),
        ("one of your opponents", PlayerFilter::Opponent),
        ("you", PlayerFilter::You),
        ("enchanted player", enchanted_player()),
    ]
    .into_iter()
    .find_map(|(p, f)| {
        let x = rest.strip_prefix(p)?;
        (x.is_empty() || x.starts_with(' ')).then_some((f, x, p))
    })?;
    // "That player": the one the subject doesn't name.
    let it_player = match (attacker, it_player) {
        (PlayerRel::You, _) => PlayerRef::TriggerPlayer,
        (_, "you") | (_, "enchanted player") => PlayerRef::ActivePlayer,
        _ => PlayerRef::Iterated,
    };
    if matches!(attacker, PlayerRel::You) && matches!(defender, PlayerFilter::You) {
        return None;
    }
    // "with one or more creatures", "with two or more creatures with flying"
    let (with, min, rest) = match rest.trim_start().strip_prefix("with ") {
        Some(w) => {
            let (min, w) = if let Some(x) = w.strip_prefix("one or more ") {
                (1, x)
            } else {
                let (n, x) = parse_number(w)?;
                let Value::Const(n) = n else {
                    return None;
                };
                (n.max(1) as u32, x.trim_start().strip_prefix("or more ")?)
            };
            let (f, plural, tail) = parse_object_phrase(w)?;
            if !plural {
                return None;
            }
            (Filter::and(vec![Filter::creature(), f]), min, tail)
        }
        None => (Filter::creature(), 1, rest),
    };
    let cond = TriggerCond::PlayerAttacked {
        attacker,
        defender,
        with,
        min,
    };
    Some((while_clause(cond, rest)?, Sel::None, it_player))
}

/// An optional "while you're the monarch" on the trigger event.
fn while_clause(cond: TriggerCond, rest: &str) -> Option<TriggerCond> {
    match end(rest) {
        "" => Some(cond),
        "while you're the monarch" => Some(TriggerCond::Where {
            trigger: Box::new(cond),
            cond: Condition::IsMonarch,
        }),
        _ => None,
    }
}
