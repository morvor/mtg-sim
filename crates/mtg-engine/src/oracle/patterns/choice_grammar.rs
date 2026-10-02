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
    // "As ~ enters, choose a player." ... "the chosen player" (CR 607.2d): the player
    // chosen as the permanent entered.
    if let Some(r) = s.strip_prefix("the chosen player") {
        let raw = crate::oracle::raw_text().to_lowercase();
        let chose = raw.lines().any(|line| {
            line.starts_with("as ")
                && line.contains(" enters, choose ")
                && (line.contains("choose a player") || line.contains("choose an opponent"))
        });
        if chose && (r.is_empty() || r.starts_with([' ', '\'', ',', '.'])) {
            return Some((PlayerRef::ChosenOpponent, r.to_string()));
        }
    }
    // "Whenever a player attacks enchanted player ..., that attacking player ...".
    if let Some(r) = s.strip_prefix("that attacking player") {
        let attacking = crate::oracle::raw_text().to_lowercase().contains("whenever a player attacks");
        if b.in_trigger
            && attacking
            && !super::oracle_hardening_referents::is_no_player_referent(&b.it_player)
            && (r.is_empty() || r.starts_with(' '))
        {
            return Some((b.it_player.clone(), r.to_string()));
        }
        return None;
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
    let who = chosen_player(b)?;
    // "If they lost life this turn, ...", "If they control fewer lands than you, create a
    // number of ... tokens equal to the difference."
    if let Some(r) = end(l).strip_prefix("if they ") {
        let (c, rest) = r.split_once(", ")?;
        let (cond, difference) = chosen_player_condition(c, &who)?;
        let rest = they_as_that_player(rest).unwrap_or_else(|| rest.to_string());
        let then = match (rest.contains(" equal to the difference"), difference) {
            (true, Some(diff)) => {
                let t = rest
                    .replacen("a number of ", "x ", 1)
                    .replacen(" equal to the difference", "", 1);
                let e = crate::oracle::effects::parse_sentence(&t, b)?;
                super::r107_numbers::substitute_x(&e, &diff)?
            }
            (true, None) => return None,
            _ => crate::oracle::effects::parse_sentence(&rest, b)?,
        };
        return Some(Effect::If {
            cond,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        });
    }
    let t = they_as_that_player(l)?;
    crate::oracle::effects::parse_sentence(&t, b)
}

/// A condition about the chosen player ("they lost life this turn", "they control fewer
/// lands than you", "they have more life than you"), and for a comparison of numbers,
/// "the difference" between them.
fn chosen_player_condition(c: &str, who: &PlayerRef) -> Option<(Condition, Option<Value>)> {
    if c == "lost life this turn" {
        return Some((
            Condition::Compare(Value::LifeLostThisTurn(who.clone()), Cmp::Ge, Value::c(1)),
            None,
        ));
    }
    if let Some(x) = c.strip_prefix("have more life than you") {
        if !x.is_empty() {
            return None;
        }
        let (a, y) = (Value::LifeTotal(who.clone()), Value::LifeTotal(PlayerRef::You));
        return Some((
            Condition::Compare(a.clone(), Cmp::Gt, y.clone()),
            Some(Value::Diff(Box::new(a), Box::new(y))),
        ));
    }
    let (fewer, r) = if let Some(r) = c.strip_prefix("control fewer ") {
        (true, r)
    } else if let Some(r) = c.strip_prefix("control more ") {
        (false, r)
    } else {
        return None;
    };
    let noun = r.strip_suffix(" than you")?;
    let (f, true, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    let theirs = Value::Count(Filter::and(vec![
        f.clone(),
        Filter::ControlledByPlayer(Box::new(who.clone())),
    ]));
    let yours = Value::Count(Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]));
    Some(if fewer {
        (
            Condition::Compare(theirs.clone(), Cmp::Lt, yours.clone()),
            Some(Value::Diff(Box::new(yours), Box::new(theirs))),
        )
    } else {
        (
            Condition::Compare(theirs.clone(), Cmp::Gt, yours.clone()),
            Some(Value::Diff(Box::new(theirs), Box::new(yours))),
        )
    })
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
    let named = b.named.len();
    let mut acc = Effect::Noop;
    for p in parts.iter().copied().chain(std::iter::once(last)) {
        // Each part is an instruction of its own ("you draw a card"), not a list item, or
        // one about what the earlier ones did ("you choose a nonland card from it").
        if let Some(e) = crate::oracle::effects::parse_simple(p, b) {
            acc = Effect::seq(vec![acc, e]);
            continue;
        }
        if !matches!(acc, Effect::Noop) && crate::oracle_ext::apply_followup_ext(p, &mut acc, b) {
            continue;
        }
        b.targets.truncate(saved.0);
        b.named.truncate(named);
        (b.it, b.it_player, b.group) = (saved.1, saved.2, saved.3);
        return None;
    }
    Some(acc)
}

inventory::submit! { EffectPattern { name: "choice grammar: A, B, and C (a series of instructions)", priority: 990, parse: serial_instructions } }

/// "~ attacks that player this combat if able" after choosing a player ("choose an
/// opponent at random"): a requirement that the creature attack that player (CR 508.1d),
/// locked onto the player chosen.
fn attacks_that_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, rest) = l.split_once(" attacks that player ")?;
    let duration = match rest {
        "this combat if able" => Duration::EndOfCombat,
        "this turn if able" => Duration::EndOfTurn,
        _ => return None,
    };
    let attackers = match subject {
        "~" => Filter::Source,
        "it" if matches!(b.it, Sel::This) => Filter::Source,
        _ => return None,
    };
    let (who, _) = crate::oracle::effects::player_ref("that player", b)?;
    if !matches!(who, PlayerRef::Var(_) | PlayerRef::Target(_)) {
        return None;
    }
    Some(Effect::AddRestriction {
        restriction: Restriction::MustAttackPlayer {
            attackers,
            defender: PlayerFilter::Ref(Box::new(who)),
        },
        duration,
    })
}

inventory::submit! { EffectPattern { name: "choice grammar: ~ attacks that player if able", priority: 85, parse: attacks_that_player } }

/// The internal form of "[objects] that player controls" / "[objects] they control" for a
/// player the text named earlier (see [`that_player_controls`]), followed by the JSON of
/// the player reference in hex digits.
const CONTROLLED_BY_REF: &str = "controlled by player@";

/// "creatures that player controls can't block this turn", "other creatures they control
/// can't block this turn" after the text named a player ("target opponent chooses a
/// creature they control"): objects that player controls. The phrase is rewritten to an
/// internal form the object phrase parser reads as that player (see
/// [`controlled_by_ref`]). Only where "that player" has an antecedent other than the
/// trigger's player (handled by `triggers_effects::that_player_controls`).
fn that_player_controls(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !(l.contains(" that player controls") || l.contains(" they control"))
        || l.contains(CONTROLLED_BY_REF)
    {
        return None;
    }
    let who = b.it_player.clone();
    if super::oracle_hardening_referents::is_no_player_referent(&who)
        || matches!(who, PlayerRef::TriggerPlayer | PlayerRef::You)
    {
        return None;
    }
    let json = serde_json::to_string(&who).ok()?;
    let hex: String = json.bytes().map(|x| format!("{x:02x}")).collect();
    let form = format!(" {CONTROLLED_BY_REF}{hex}");
    let t = l
        .replace(" that player controls", &form)
        .replace(" they control", &form);
    crate::oracle::effects::parse_sentence(&t, b)
}

inventory::submit! { EffectPattern { name: "choice grammar: [objects] that player controls", priority: 992, parse: that_player_controls } }

/// Reads the internal form written by [`that_player_controls`].
fn controlled_by_ref<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix(CONTROLLED_BY_REF)?;
    // Hex digits of the JSON (sentences are lowercased).
    let k = r
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(r.len());
    let bytes: Option<Vec<u8>> = (0..k / 2)
        .map(|i| u8::from_str_radix(&r[2 * i..2 * i + 2], 16).ok())
        .collect();
    let json = String::from_utf8(bytes?).ok()?;
    let who: PlayerRef = serde_json::from_str(&json).ok()?;
    Some((Filter::ControlledByPlayer(Box::new(who)), &r[k..]))
}

inventory::submit! { super::FilterSuffixPattern { name: "choice grammar: controlled by a referenced player", priority: 10, parse: controlled_by_ref } }

/// "You gain control of those creatures.", "you put that card on the bottom of your
/// library": an instruction with "you" as its subject is the imperative instruction
/// (the controller of the spell or ability performs it).
fn you_imperative(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you ")?;
    let verb = r.split(' ').next()?;
    if !matches!(
        verb,
        "gain" | "put" | "return" | "exile" | "destroy" | "tap" | "untap" | "create" | "sacrifice"
    ) || r.starts_with("gain ") && !r.starts_with("gain control of ")
    {
        return None;
    }
    crate::oracle::effects::parse_clause(r, b)
}

inventory::submit! { EffectPattern { name: "choice grammar: you [verb] (imperative)", priority: 996, parse: you_imperative } }

/// "destroy target nonartifact creature that player controls of their choice" (The
/// Abyss), "that creature's controller may have target creature of their choice get
/// -3/-3" (Death Match), "that attacking player may tap or untap target permanent of their
/// choice" (Curse of Inertia), "destroy target creature of your choice": the target is
/// chosen by the player "their" refers to (the sentence's subject, or else "that
/// player"), as the spell or ability is put on the stack (CR 601.2c).
fn target_of_their_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (k, theirs) = match (l.find(" of their choice"), l.find(" of your choice")) {
        (Some(k), _) => (k, true),
        (None, Some(k)) => (k, false),
        _ => return None,
    };
    // Right after a target phrase.
    let before = &l[..k];
    let t = before.rfind("target ")?;
    if before[t..].contains(", ") || before[t..].contains(" and ") {
        return None;
    }
    let phrase = if theirs { " of their choice" } else { " of your choice" };
    let text = format!("{before}{}", &l[k + phrase.len()..]);
    if text.contains(" of their choice") || text.contains(" of your choice") {
        return None;
    }
    // Who "their" is: a player subject ("that creature's controller may have ...").
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let chooser = if theirs {
        let subject = ["may have ", "may tap ", "may "]
            .iter()
            .find_map(|m| l.find(&format!(" {m}")).map(|i| &l[..i]));
        let who = match subject {
            Some(s) => {
                let (who, rest) = crate::oracle::effects::player_ref(s, b)?;
                if !end(&rest).is_empty() {
                    b.targets.truncate(saved.0);
                    (b.it, b.it_player) = (saved.1, saved.2);
                    return None;
                }
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
                who
            }
            None => b.it_player.clone(),
        };
        if super::oracle_hardening_referents::is_no_player_referent(&who)
            || matches!(who, PlayerRef::You)
        {
            return None;
        }
        Some(who)
    } else {
        None
    };
    let first = b.targets.len();
    let e = crate::oracle::effects::parse_sentence(&text, b)?;
    // The target named right before "of their choice" (the last one the sentence added).
    let Some(slot) = (first..b.targets.len()).rev().find(|i| {
        matches!(b.targets[*i].what, TargetKind::Object(_))
    }) else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    b.targets[slot].chosen_by = chooser;
    Some(e)
}

inventory::submit! { EffectPattern { name: "choice grammar: target ... of their choice", priority: 85, parse: target_of_their_choice } }

/// "that player reveals their hand and you choose a nonland card from it": an instruction
/// joined by "and" to one about what it did (read like the next sentence would be).
fn and_followup(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (a, c) = l.split_once(" and ")?;
    let saved = (
        b.targets.len(),
        b.it.clone(),
        b.it_player.clone(),
        b.group.clone(),
    );
    let named = b.named.len();
    let mut e = crate::oracle::effects::parse_simple(a, b)?;
    if crate::oracle_ext::apply_followup_ext(c, &mut e, b) {
        return Some(e);
    }
    b.targets.truncate(saved.0);
    b.named.truncate(named);
    (b.it, b.it_player, b.group) = (saved.1, saved.2, saved.3);
    None
}

inventory::submit! { EffectPattern { name: "choice grammar: A and [followup of A]", priority: 993, parse: and_followup } }

/// The players an instruction affected as a group ("each opponent", "each player"), as
/// the subject words naming them.
fn affected_players(e: &Effect) -> Option<&'static str> {
    let who = match e {
        Effect::Seq(v) => return v.last().and_then(affected_players),
        Effect::DealDamage {
            to: Sel::Players(r),
            ..
        } => r,
        Effect::LoseLife { who, .. } | Effect::GainLife { who, .. } => who,
        _ => return None,
    };
    match who {
        PlayerRef::EachOpponent => Some("each opponent"),
        PlayerRef::EachPlayer => Some("each player"),
        PlayerRef::EachOtherPlayer => Some("each other player"),
        _ => None,
    }
}

/// "~ deals 2 damage to each opponent. Those players each discard two cards at random.":
/// "those players" are the players the previous instruction affected.
fn those_players_each(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l)
        .strip_prefix("those players each ")
        .or_else(|| end(l).strip_prefix("each of those players "))
    else {
        return false;
    };
    let Some(subject) = affected_players(prev) else {
        return false;
    };
    // Third person singular verb ("discard" -> "discards").
    let (verb, rest) = r.split_once(' ').unwrap_or((r, ""));
    let verb = if l.starts_with("each of those players ") {
        verb.to_string()
    } else if verb.ends_with("sh") || verb.ends_with("ch") || verb.ends_with('s') || verb.ends_with('x') {
        format!("{verb}es")
    } else {
        format!("{verb}s")
    };
    let text = format!("{subject} {verb} {rest}");
    let Some(e) = crate::oracle::effects::parse_sentence(text.trim(), b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "choice grammar: those players each [verb]", priority: 60, apply: those_players_each } }
