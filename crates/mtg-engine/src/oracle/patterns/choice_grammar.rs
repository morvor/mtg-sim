//! Choices as instructions whose results later sentences refer to (CR 608.2d: choices made
//! as a spell or ability resolves; CR 115.1, 601.2c: targets chosen as it's put on the
//! stack).
//!
//! - "Choose target opponent." / "Choose target player." / "Choose two target players.":
//!   the targets are chosen as the spell or ability is put on the stack; "that player",
//!   "they" and "the chosen player" refer to them in the sentences that follow.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "choose target opponent", "choose target player", "choose two target players",
/// "choose target player and another target player": a sentence that only chooses player
/// targets. "That player" / "the chosen player" refer to the (first) target afterward;
/// with two targets, "the first player" and "the second player" do.
fn choose_target_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (spec, tail) = parse_target(r)?;
    let TargetKind::Player(_) = &spec.what else {
        return None;
    };
    let n = match (&spec.min, &spec.max) {
        (Value::Const(1), Value::Const(1)) => 1,
        (Value::Const(2), Value::Const(2)) => 2,
        _ => return None,
    };
    let tail = end(tail);
    let text = r[..r.len() - tail.len()].trim().to_string();
    // "choose target player and another target player"
    let second = match tail {
        "" => None,
        _ if n == 1 => {
            let r2 = tail.strip_prefix("and ")?;
            let (spec2, t2) = parse_target(r2)?;
            if !end(t2).is_empty() || !matches!(spec2.what, TargetKind::Player(_)) {
                return None;
            }
            if !matches!((&spec2.min, &spec2.max), (Value::Const(1), Value::Const(1))) {
                return None;
            }
            Some((spec2, r2.trim().to_string()))
        }
        _ => return None,
    };
    let slot = b.add_target(spec, &text);
    b.it_player = PlayerRef::Target(slot);
    match (n, second) {
        (1, None) => {
            b.named.push((
                "the chosen player".into(),
                Sel::Players(PlayerRef::Target(slot)),
            ));
        }
        (1, Some((spec2, text2))) => {
            let slot2 = b.add_target(spec2, &text2);
            b.it_player = PlayerRef::Target(slot);
            b.named.push((
                "the first player".into(),
                Sel::Players(PlayerRef::Target(slot)),
            ));
            b.named.push((
                "the second player".into(),
                Sel::Players(PlayerRef::Target(slot2)),
            ));
        }
        _ => {
            b.named
                .push(("they".into(), Sel::Players(PlayerRef::Target(slot))));
        }
    }
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choice grammar: choose target player(s)", priority: 85, parse: choose_target_player } }

/// Player phrases this item adds to the core's (`effects::player_ref` calls this first):
///
/// - players the text chose earlier ("the chosen player", "the first player");
/// - "that source's controller" in a trigger about damage a source dealt (the source is
///   the trigger's other object: "Whenever a source deals damage to ~, that source's
///   controller ...");
/// - "enchanted creature's controller", "equipped creature's controller";
/// - "each player who controls the most [objects]" (with ties, each of the tied players).
///
/// Returns the players and the rest of the text.
pub fn player_phrase(s: &str, b: &mut Builder) -> Option<(PlayerRef, String)> {
    let named = b
        .named
        .iter()
        .filter_map(|(p, sel)| {
            let Sel::Players(who) = sel else {
                return None;
            };
            let rest = s.strip_prefix(p.as_str())?;
            (rest.is_empty() || rest.starts_with([' ', '\'', ',', '.']))
                .then(|| (p.len(), who.clone(), rest.to_string()))
        })
        .max_by_key(|(len, _, _)| *len)
        .map(|(_, who, rest)| (who, rest));
    if named.is_some() {
        return named;
    }
    if let Some(r) = s.strip_prefix("that source's controller") {
        if b.in_trigger && (r.is_empty() || r.starts_with(' ')) {
            return Some((
                PlayerRef::ControllerOf(Box::new(Sel::TriggerOtherObject)),
                r.to_string(),
            ));
        }
        return None;
    }
    for p in [
        "enchanted creature's controller",
        "enchanted permanent's controller",
        "enchanted artifact's controller",
        "enchanted land's controller",
        "equipped creature's controller",
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with(' ') {
                // "..., then that player loses 1 life."
                let who = PlayerRef::ControllerOf(Box::new(Sel::AttachedTo));
                b.it_player = who.clone();
                return Some((who, r.to_string()));
            }
        }
    }
    if let Some(r) = s.strip_prefix("each player who controls the most ") {
        let (f, true, rest) = parse_object_phrase(r)? else {
            return None;
        };
        let rest = rest.trim_start();
        let rest = rest
            .strip_prefix("or is tied for the most ")
            .unwrap_or(rest);
        let mine = Filter::and(vec![f.clone(), Filter::ControlledBy(PlayerRel::Iterated)]);
        return Some((
            PlayerRef::Each(PlayerFilter::Controls(
                Box::new(f),
                Cmp::Ge,
                Box::new(Value::OverPlayers(
                    AggOp::Max,
                    PlayerFilter::Any,
                    Box::new(Value::Count(mine)),
                )),
            )),
            format!(" {rest}"),
        ));
    }
    None
}

/// The player the text chose with "choose target player/opponent" (see
/// [`choose_target_player`]), while "that player" still means them.
fn chosen_player(b: &Builder) -> Option<PlayerRef> {
    let (_, sel) = b.named.iter().find(|(p, _)| p == "the chosen player")?;
    let Sel::Players(who @ PlayerRef::Target(n)) = sel else {
        return None;
    };
    matches!(b.it_player, PlayerRef::Target(m) if m == *n).then(|| who.clone())
}

/// Third-person singular of a verb ("reveal" → "reveals"), or the verb unchanged if it's a
/// modal or in the past tense.
pub fn third_person_verb(v: &str) -> String {
    match v {
        "have" => "has".into(),
        "do" => "does".into(),
        "don't" => "doesn't".into(),
        "are" => "is".into(),
        "were" => "was".into(),
        "may" | "can't" | "can" | "cannot" | "lost" | "had" | "did" | "didn't" | "cast"
        | "drew" | "put" | "dealt" | "chose" | "controlled" | "gained" | "attacked"
        | "sacrificed" | "discarded" | "won" | "would" | "each" | "both" | "all" => v.into(),
        v if v.ends_with("ed") => v.into(),
        v if v.ends_with('s') || v.ends_with("sh") || v.ends_with("ch") || v.ends_with('x') => {
            format!("{v}es")
        }
        v if v.ends_with('y') && !v.ends_with("ay") && !v.ends_with("ey") => {
            format!("{}ies", &v[..v.len() - 1])
        }
        v => format!("{v}s"),
    }
}

/// The sentence with each "they" (as a subject) replaced by "that player", its verb made
/// singular ("they control" → "that player controls"). `None` if there's no "they".
pub fn they_as_that_player(s: &str) -> Option<String> {
    let words: Vec<&str> = s.split(' ').collect();
    if !words.contains(&"they") {
        return None;
    }
    let mut out: Vec<String> = Vec::with_capacity(words.len() + 2);
    let mut i = 0;
    while i < words.len() {
        if words[i] == "they" {
            out.push("that player".into());
            if let Some(v) = words.get(i + 1) {
                let (core, punct) = match v.find([',', '.']) {
                    Some(k) => (&v[..k], &v[k..]),
                    None => (*v, ""),
                };
                out.push(format!("{}{punct}", third_person_verb(core)));
                i += 2;
                continue;
            }
        } else {
            out.push(words[i].to_string());
        }
        i += 1;
    }
    Some(out.join(" "))
}

/// After "choose target player/opponent", "they" in a sentence is that player: "If they
/// control fewer lands than you, ...", "~ deals 3 damage to that player and 1 damage to
/// each creature they control."
fn they_the_chosen_player(l: &str, b: &mut Builder) -> Option<Effect> {
    chosen_player(b)?;
    let t = they_as_that_player(l)?;
    crate::oracle::effects::parse_sentence(&t, b)
}

inventory::submit! { EffectPattern { name: "choice grammar: they (the chosen player)", priority: 995, parse: they_the_chosen_player } }

/// "tap ~, you gain 3 life, and you draw a card": a series of three or more instructions
/// separated by commas, the last after ", and" (each one performed in order).
fn serial_instructions(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, last) = l.rsplit_once(", and ")?;
    let parts: Vec<&str> = head.split(", ").collect();
    if parts.len() < 2 || parts.iter().any(|p| p.is_empty()) {
        return None;
    }
    let saved = (
        b.targets.len(),
        b.it.clone(),
        b.it_player.clone(),
        b.group.clone(),
    );
    let mut effects = Vec::new();
    for p in parts.iter().copied().chain(std::iter::once(last)) {
        // Each part is an instruction of its own ("you draw a card"), not a list item.
        match crate::oracle::effects::parse_simple(p, b) {
            Some(e) => effects.push(e),
            None => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player, b.group) = (saved.1, saved.2, saved.3);
                return None;
            }
        }
    }
    Some(Effect::seq(effects))
}

inventory::submit! { EffectPattern { name: "choice grammar: A, B, and C (a series of instructions)", priority: 990, parse: serial_instructions } }
