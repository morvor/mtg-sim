//! Targets of basic removal and damage effects that the core phrase parser doesn't read:
//!
//! - alternatives with different head nouns: "target enchantment, tapped artifact, or
//!   tapped creature", "target Spirit, creature with disturb, or enchantment", "target
//!   artifact, enchantment, or tapped creature an opponent controls" (a controller after
//!   the list describes every alternative);
//! - several instances of "target" (CR 115.1, 601.2c): "destroy target artifact, target
//!   creature, target enchantment, and target land", "destroy up to one target artifact,
//!   up to one target creature, and up to one target land", "untap up to two target
//!   creatures and up to two target lands", "tap ~ and up to one target creature an
//!   opponent controls";
//! - "controlled by different players" (a requirement on the targets together);
//! - damage divided into parts: "~ deals 2 damage to any target, 2 damage to another
//!   target, and 3 damage to a third target", "deals 3 damage to target player who
//!   attacked this turn and 3 damage to you", "deals 4 damage to each of up to one target
//!   creature, up to one target player, and/or up to one target planeswalker".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

/// Whether a filter is a controller/owner requirement that, after the last item of a
/// list, describes the whole list ("artifact, enchantment, or tapped creature an opponent
/// controls").
fn is_control(f: &Filter) -> bool {
    matches!(
        f,
        Filter::ControlledBy(_)
            | Filter::OwnedBy(_)
            | Filter::ControlledByPlayer(_)
            | Filter::OwnedByPlayer(_)
    )
}

/// "artifact, enchantment, or tapped creature [an opponent controls]": a list of object
/// descriptions with their own head nouns, joined by "or" or "and/or". Returns the filter,
/// whether the nouns are plural, and the rest.
pub(crate) fn object_alternatives(s: &str) -> Option<(Filter, bool, &str)> {
    let mut items = Vec::new();
    let mut plural_any = false;
    let mut rest = s;
    loop {
        let (f, plural, r) = parse_object_phrase(rest)?;
        let consumed = rest[..rest.len() - r.len()].trim_end();
        let took_sep = [",", " or", " and/or"].iter().any(|x| consumed.ends_with(x));
        items.push(f);
        plural_any |= plural;
        rest = r;
        let t = rest.trim_start();
        let next = [", or ", ", and/or ", ", "]
            .iter()
            .find_map(|sep| rest.strip_prefix(sep))
            .or_else(|| strip(t, "or "))
            .or_else(|| strip(t, "and/or "))
            .or_else(|| took_sep.then_some(t));
        match next {
            Some(n) if !starts_new_target(n) && parse_object_phrase(n).is_some() => rest = n,
            _ => break,
        }
    }
    if items.len() < 2 {
        return None;
    }
    // A controller after the last item describes them all.
    let mut shared = Vec::new();
    if let Some(Filter::And(v)) = items.last_mut() {
        let (ctl, own): (Vec<Filter>, Vec<Filter>) = v.drain(..).partition(is_control);
        *v = own;
        shared = ctl;
    }
    let items: Vec<Filter> = items
        .into_iter()
        .map(|f| match f {
            Filter::And(v) => Filter::and(v),
            f => f,
        })
        .collect();
    let mut parts = vec![Filter::Or(items)];
    parts.extend(shared);
    Some((Filter::and(parts), plural_any, rest))
}

/// Whether a list item is another target phrase ("up to one target player"), not an
/// alternative description.
fn starts_new_target(s: &str) -> bool {
    let w = split_word(s).0;
    matches!(w, "up" | "target" | "another" | "any" | "a" | "an" | "each" | "all" | "you")
}

/// "target [alternatives]", "up to one target ...", "up to two target ... and/or ...",
/// "another target ...", with "controlled by different players".
fn target_alternatives(s: &str) -> Option<(TargetSpec, &str)> {
    let s = s.trim_start();
    let (min, max, r) = if let Some(r) = strip(s, "up to ") {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        (0, n, r)
    } else if let Some(r) = strip(s, "any number of ") {
        (0, Value::Const(99), r)
    } else {
        (1, Value::Const(1), s)
    };
    let (other, r) = if let Some(x) = strip(r, "target ") {
        (false, x)
    } else if let Some(x) = strip(r, "other target ").or_else(|| strip(r, "another target ")) {
        (true, x)
    } else {
        return None;
    };
    let (mut f, _, mut rest) = match object_alternatives(r) {
        Some(x) => x,
        None => {
            // A single description with a requirement on the group.
            let (f, plural, rest) = parse_object_phrase(r)?;
            if !strip(rest, "controlled by different players").is_some() {
                return None;
            }
            (f, plural, rest)
        }
    };
    let mut together = None;
    if let Some(x) = strip(rest, "controlled by different players") {
        together = Some(TargetGroup::DifferentControllers);
        rest = x;
    }
    if other {
        f = Filter::and(vec![f, Filter::Other]);
    }
    let mut spec = TargetSpec::one(
        TargetKind::Object(f),
        s[..s.len() - rest.len()].trim().to_string(),
    );
    spec.min = Value::Const(min);
    spec.max = max;
    spec.together = together;
    Some((spec, rest))
}

/// "that Wall ..." → "that creature ...": a trigger's creature named by its subtype.
pub(crate) fn that_subtype_as_creature(s: &str) -> Option<String> {
    let r = s.strip_prefix("that ")?;
    let (w, rest) = split_word(r);
    let sub = crate::oracle::phrases::subtype_word(w)?;
    if !crate::types::is_creature_type(&sub) {
        return None;
    }
    Some(format!("that creature {rest}").trim_end().to_string())
}

/// One target phrase: the core's, or alternatives.
fn one_target(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim_start();
    // "up to five target permanents that player controls": the player an earlier part
    // named.
    if let Some(i) = s.find(" that player controls") {
        let head = &s[..i];
        let rest = &s[i + " that player controls".len()..];
        if !super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
            if let Some((mut spec, tail)) = parse_target(head) {
                if end(tail).is_empty() {
                    if let TargetKind::Object(f) = &mut spec.what {
                        *f = Filter::and(vec![
                            f.clone(),
                            Filter::ControlledByPlayer(Box::new(b.it_player.clone())),
                        ]);
                        let text = s[..s.len() - rest.len()].to_string();
                        let slot = b.add_target(spec, &text);
                        return Some((Sel::Target(slot), rest.to_string()));
                    }
                }
            }
        }
    }
    if let Some((spec, rest)) = target_alternatives(s) {
        let text = spec.text.clone();
        let slot = b.add_target(spec, &text);
        return Some((Sel::Target(slot), rest.to_string()));
    }
    // "all nonland permanents that player controls" (the player an earlier part named).
    if let Some(r) = s.strip_prefix("all ").or_else(|| s.strip_prefix("each ")) {
        if let Some(i) = r.find(" that player controls") {
            if !super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
                if let Some((f, _, tail)) = parse_object_phrase(&r[..i]) {
                    if end(tail).is_empty() {
                        let f = Filter::and(vec![
                            f,
                            Filter::ControlledByPlayer(Box::new(b.it_player.clone())),
                        ]);
                        return Some((Sel::All(f), r[i + " that player controls".len()..].to_string()));
                    }
                }
            }
        }
    }
    // "target instant or sorcery spell that targets you".
    if let Some((spec, rest)) = super::basic_effects_counter::stack_target(s) {
        let text = spec.text.clone();
        let slot = b.add_target(spec, &text);
        return Some((Sel::Target(slot), rest.to_string()));
    }
    let (sel, rest) = object_ref(s, b)?;
    Some((sel, rest))
}

/// "target A, target B, and target C", "up to one target A, up to one target B, and/or up
/// to one target C", "~ and up to one target creature": several objects, each named on
/// its own. Returns the selections and the rest.
fn object_list(s: &str, b: &mut Builder) -> Option<(Vec<Sel>, String)> {
    let mut sels = Vec::new();
    let mut rest = s.to_string();
    loop {
        let (sel, r) = one_target(&rest, b)?;
        // The object phrase parser may have taken the separator after the phrase.
        let consumed = rest[..rest.len() - r.len()].trim_end().to_string();
        let took_sep = [",", " and", " and/or"].iter().any(|x| consumed.ends_with(x));
        sels.push(sel);
        rest = r;
        let t = rest.trim_start().to_string();
        let next = [", and/or ", ", and ", ", "]
            .iter()
            .find_map(|sep| rest.strip_prefix(sep).map(str::to_string))
            .or_else(|| strip(&t, "and/or ").map(str::to_string))
            .or_else(|| strip(&t, "and ").map(str::to_string))
            .or_else(|| (took_sep && starts_new_target(&t)).then(|| t.clone()));
        match next {
            Some(n) => rest = n,
            None => break,
        }
    }
    Some((sels, rest))
}

/// "destroy/exile/tap/untap [target alternatives or a list of targets]".
fn verb_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (verb, r) = ["destroy ", "exile ", "tap ", "untap "]
        .iter()
        .find_map(|v| l.strip_prefix(v).map(|r| (*v, r)))?;
    let (mut sels, tail) = object_list(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    // A list of several, or one target described by alternatives (which the core
    // patterns didn't read).
    if sels.len() < 2 && !matches!(sels.first(), Some(Sel::Target(_))) {
        return None;
    }
    let what = if sels.len() == 1 {
        sels.pop().expect("one")
    } else {
        Sel::Union(sels)
    };
    if sels_len(&what) > 1 {
        // "They": every object named.
        b.it = what.clone();
    }
    Some(match verb {
        "destroy " => Effect::Destroy {
            what,
            no_regen: false,
        },
        "exile " => Effect::Exile {
            what,
            face_down: false,
            link: false,
        },
        "tap " => Effect::Tap { what },
        _ => Effect::Untap { what },
    })
}

fn sels_len(s: &Sel) -> usize {
    match s {
        Sel::Union(v) => v.len(),
        _ => 1,
    }
}

inventory::submit! { EffectPattern { name: "basic effects: [verb] several targets", priority: 150, parse: verb_targets } }

/// A player-or-object target the core phrase parser doesn't read: "target player or
/// battle", "target opponent or battle", "another target battle or opponent".
fn player_or_battle(s: &str) -> Option<(TargetSpec, &str)> {
    let battle = Filter::Type(crate::types::CardType::Battle);
    for (p, other, players) in [
        ("target player or battle", false, PlayerFilter::Any),
        ("target battle or player", false, PlayerFilter::Any),
        ("target opponent or battle", false, PlayerFilter::Opponent),
        ("target battle or opponent", false, PlayerFilter::Opponent),
        ("another target battle or opponent", true, PlayerFilter::Opponent),
        ("another target opponent or battle", true, PlayerFilter::Opponent),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if !(r.is_empty() || r.starts_with(' ') || r.starts_with(',')) {
                continue;
            }
            let f = if other {
                Filter::and(vec![battle.clone(), Filter::Other])
            } else {
                battle.clone()
            };
            return Some((TargetSpec::one(TargetKind::ObjectOrPlayer(f, players), p), r));
        }
    }
    None
}

/// A damage recipient phrase in a list: "any target", "another target", "a third target",
/// "another target creature", "a third target creature", "you", "target player who
/// attacked this turn", "target player or battle", "up to one target creature". Returns
/// the recipient, the rest, and whether the core damage pattern wouldn't read it alone.
fn damage_recipient(s: &str, b: &mut Builder) -> Option<(Sel, String, bool)> {
    let s = s.trim_start();
    if let Some((spec, r)) = player_or_battle(s) {
        let text = spec.text.clone();
        let slot = b.add_target(spec, &text);
        return Some((Sel::Target(slot), r.to_string(), true));
    }
    for (p, rep) in [
        ("another target creature", "another target creature"),
        ("a third target creature", "another target creature"),
        ("another target", "any other target"),
        ("a third target", "any other target"),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with(' ') || r.starts_with(',') {
                if rep == "any other target" {
                    let mut spec = TargetSpec::any_target();
                    spec.text = rep.to_string();
                    let slot = b.add_target(spec, rep);
                    return Some((Sel::Target(slot), r.to_string(), false));
                }
                let (sel, rest) = one_target(&format!("{rep}{r}"), b)?;
                return Some((sel, rest, false));
            }
        }
    }
    if let Some(r) = s.strip_prefix("you") {
        if r.is_empty() || r.starts_with(' ') || r.starts_with(',') {
            return Some((Sel::Players(PlayerRef::You), r.to_string(), false));
        }
    }
    // Players: "each other opponent" (other than the one the ability is about), "each
    // opponent", "that player", ...
    let players: [(&str, Option<PlayerRef>); 6] = [
        (
            "each other opponent",
            (!super::oracle_hardening_referents::is_no_player_referent(&b.it_player)).then(|| {
                PlayerRef::Each(PlayerFilter::And(vec![
                    PlayerFilter::Opponent,
                    PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(b.it_player.clone())))),
                ]))
            }),
        ),
        ("each opponent", Some(PlayerRef::EachOpponent)),
        ("each player", Some(PlayerRef::EachPlayer)),
        ("each other player", Some(PlayerRef::EachOtherPlayer)),
        ("defending player", Some(PlayerRef::DefendingPlayer)),
        (
            "that player",
            (!super::oracle_hardening_referents::is_no_player_referent(&b.it_player))
                .then(|| b.it_player.clone()),
        ),
    ];
    for (p, who) in players {
        if let Some(r) = s.strip_prefix(p) {
            if r.is_empty() || r.starts_with(' ') || r.starts_with(',') {
                return Some((Sel::Players(who?), r.to_string(), true));
            }
        }
    }
    // "each creature without flying that player controls": the player an earlier part of
    // the text named.
    if let Some(r) = s.strip_prefix("each ") {
        if let Some(i) = r.find(" that player controls") {
            if !super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
                let (f, plural, tail) = parse_object_phrase(&r[..i])?;
                if plural || !end(tail).is_empty() {
                    return None;
                }
                let f = Filter::and(vec![
                    f,
                    Filter::ControlledByPlayer(Box::new(b.it_player.clone())),
                ]);
                let rest = &r[i + " that player controls".len()..];
                return Some((Sel::All(f), rest.to_string(), true));
            }
        }
    }
    // "the creature's controller", "that land's controller", "the Wall's controller": the
    // object the text is about.
    if let Some(r) = s.strip_prefix("the ").or_else(|| s.strip_prefix("that ")) {
        let (w, rest) = split_word(r);
        if let Some(noun) = w.strip_suffix("'s") {
            if parse_object_phrase(noun).is_some() {
                if let Some(rest) = rest.strip_prefix("controller") {
                    let it = super::pronoun_groups::singular_it(b);
                    if super::oracle_hardening_referents::is_no_referent(&it)
                        || matches!(it, Sel::This)
                    {
                        return None;
                    }
                    return Some((
                        Sel::Players(PlayerRef::ControllerOf(Box::new(it))),
                        rest.to_string(),
                        true,
                    ));
                }
            }
        }
    }
    // "enchanted artifact's controller", "target spell's controller".
    if let Some(i) = s.find("'s controller") {
        let head = &s[..i];
        let rest = &s[i + "'s controller".len()..];
        let who = if head.starts_with("enchanted ") || head.starts_with("equipped ") {
            let (sel, tail) = object_ref(head, b)?;
            if !tail.trim().is_empty() {
                return None;
            }
            sel
        } else if head.starts_with("target ") {
            let (spec, tail) = parse_target(head)
                .or_else(|| super::basic_effects_counter::stack_target(head))?;
            if !end(tail).is_empty() {
                return None;
            }
            let slot = b.add_target(spec, head);
            Sel::Target(slot)
        } else {
            return None;
        };
        return Some((
            Sel::Players(PlayerRef::ControllerOf(Box::new(who))),
            rest.to_string(),
            true,
        ));
    }
    // "any target that isn't a Dinosaur", "any target that was dealt damage this turn"
    if let Some(r) = s.strip_prefix("any target that ") {
        let any = Filter::Or(vec![
            Filter::creature(),
            Filter::Type(crate::types::CardType::Planeswalker),
            Filter::Type(crate::types::CardType::Battle),
        ]);
        let (objects, players, rest) = if let Some(x) = r.strip_prefix("was dealt damage this turn") {
            (Filter::DealtDamageThisTurn, PlayerFilter::DealtDamageThisTurn, x)
        } else {
            let x = r.strip_prefix("isn't a ").or_else(|| r.strip_prefix("isn't an "))?;
            let (f, plural, x) = parse_object_phrase(x)?;
            if plural {
                return None;
            }
            // A player is never an object.
            (Filter::not(f), PlayerFilter::Any, x)
        };
        let text = s[..s.len() - rest.len()].trim().to_string();
        let spec = TargetSpec::one(
            TargetKind::ObjectOrPlayer(Filter::and(vec![any, objects]), players),
            text.clone(),
        );
        let slot = b.add_target(spec, &text);
        return Some((Sel::Target(slot), rest.to_string(), true));
    }
    // "target player who attacked this turn"
    if let Some(r) = s.strip_prefix("target player who attacked this turn") {
        let spec = TargetSpec::player(
            PlayerFilter::AttackedThisTurn,
            "target player who attacked this turn",
        );
        let slot = b.add_target(spec, "target player who attacked this turn");
        return Some((Sel::Target(slot), r.to_string(), true));
    }
    let (sel, rest) = one_target(s, b)?;
    Some((sel, rest, false))
}

/// The subject of a damage sentence: "~ deals", "it deals", "he deals", or "[object]
/// deals" (one object). Returns the source and the rest after "deals ".
fn damage_subject(l: &str, b: &mut Builder) -> Option<(Sel, String)> {
    if let Some(r) = l.strip_prefix("~ deals ") {
        return Some((Sel::This, r.to_string()));
    }
    for p in ["it deals ", "he deals ", "she deals "] {
        if let Some(r) = l.strip_prefix(p) {
            let it = super::pronoun_groups::singular_it(b);
            if super::oracle_hardening_referents::is_no_referent(&it) {
                return None;
            }
            return Some((it, r.to_string()));
        }
    }
    // "Gideon deals 1 damage to that player" (Gideon the Oathless): a legendary card
    // called by the first word of its name.
    if b.ctx
        .type_line
        .supertypes
        .contains(crate::types::Supertype::Legendary)
    {
        if let Some((first, _)) = b.ctx.card_name.split_once(' ') {
            let first = first.trim_end_matches(',').to_lowercase();
            if let Some(r) = l
                .strip_prefix(first.as_str())
                .and_then(|r| r.strip_prefix(" deals "))
            {
                return Some((Sel::This, r.to_string()));
            }
        }
    }
    // "Target creature an opponent controls deals damage equal to its power to ...".
    if super::pronoun_groups::plural_pronoun(l).is_some() || l.starts_with("each ") {
        return None;
    }
    let (sel, rest) = object_ref(l, b)?;
    if matches!(sel, Sel::All(_) | Sel::Union(_) | Sel::Players(_)) {
        return None;
    }
    let r = rest.trim_start().strip_prefix("deals ")?;
    Some((sel, r.to_string()))
}

/// The amount of one part: "3 damage to", "that much damage to" (the triggering event's
/// amount), "damage equal to [value] to" ("its" being the source). Returns the amount and
/// the recipient text.
fn damage_amount(s: &str, source: &Sel, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(r) = s.strip_prefix("that much damage to ") {
        if !b.in_trigger {
            return None;
        }
        return Some((Value::EventAmount, r.to_string()));
    }
    // "damage equal to its loyalty" (a planeswalker source).
    if let Some(r) = s.strip_prefix("damage equal to its loyalty to ") {
        return Some((Value::LoyaltyOf(Box::new(source.clone())), r.to_string()));
    }
    if let Some(r) = s.strip_prefix("damage equal to ") {
        // "its power": the source's; "the creature's power", "that Wall's mana value": the
        // object the text is about.
        let (subject_its, r) = match referent_possessive(r) {
            Some(x) => (false, x),
            None => (r.starts_with("its ") || r.starts_with("their "), r.to_string()),
        };
        let saved = subject_its.then(|| std::mem::replace(&mut b.it, source.clone()));
        let parsed = crate::oracle::statics::parse_value_phrase(&r, b);
        if let Some(it) = saved {
            b.it = it;
        }
        let (v, rest) = parsed?;
        let rest = rest.trim_start().strip_prefix("to ")?;
        return Some((v, rest.to_string()));
    }
    let (n, r) = number_expr(s)?;
    let r = strip(r, "damage to ")?;
    Some((n, r.to_string()))
}

/// "the creature's power ...", "that wall's mana value ..." → "its power ...": a
/// possessive naming the object the text is about by its noun.
fn referent_possessive(s: &str) -> Option<String> {
    let r = s.strip_prefix("the ").or_else(|| s.strip_prefix("that "))?;
    let (w, rest) = split_word(r);
    let noun = w.strip_suffix("'s")?;
    parse_object_phrase(noun)?;
    Some(format!("its {rest}"))
}

/// "2 or the greatest power among Dinosaurs you control, whichever is greater".
fn whichever_greater(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let (a, r) = parse_number(s)?;
    let r = strip(r, "or ")?;
    let (v, tail) = crate::oracle::statics::parse_value_phrase(r, b)?;
    let tail = tail.trim_start().strip_prefix(", whichever is greater")?;
    Some((Value::Max(Box::new(a), Box::new(v)), tail.to_string()))
}

/// "3", "x plus 2", "five times x".
pub(crate) fn number_expr(s: &str) -> Option<(Value, &str)> {
    let (n, r) = parse_number(s)?;
    if let Some(x) = strip(r, "plus ") {
        let (m, r2) = parse_number(x)?;
        return Some((Value::Sum(vec![n, m]), r2));
    }
    if let Some(x) = strip(r, "times ") {
        let (m, r2) = parse_number(x)?;
        return Some((Value::Mul(Box::new(n), Box::new(m)), r2));
    }
    Some((n, r))
}

/// "~ deals 2 damage to any target, 2 damage to another target, and 3 damage to a third
/// target", "it deals 3 damage to another target battle or opponent and 2 damage to up to
/// one target creature", "~ deals 4 damage to target player who attacked this turn and 4
/// damage to you", "it deals that much damage to each other opponent", "target creature an
/// opponent controls deals damage equal to its power to another target creature that
/// player controls": one source, one or more amounts each to its own recipient, all dealt
/// at the same time (CR 120.2).
fn damage_parts(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (source, r) = damage_subject(l, b)?;
    let mut parts = Vec::new();
    let mut rest = r;
    loop {
        // "damage to target spell's controller equal to that spell's mana value"
        if let Some(r) = rest.strip_prefix("damage to ") {
            let (to, r, _) = damage_recipient(r, b)?;
            let r = r.trim_start().strip_prefix("equal to ")?;
            let (n, r) = match whichever_greater(r, b) {
                Some(x) => x,
                None => crate::oracle::statics::parse_value_phrase(r, b)?,
            };
            parts.push((n, to));
            rest = r;
            break;
        }
        let (n, r) = damage_amount(&rest, &source, b)?;
        let (to, r, _) = damage_recipient(&r, b)?;
        parts.push((n, to));
        rest = r;
        let next = [", and ", ", ", " and "]
            .iter()
            .find_map(|sep| rest.strip_prefix(sep).map(str::to_string));
        match next {
            Some(n) if parse_number(&n).is_some() => rest = n,
            _ => break,
        }
    }
    if !end(&rest).is_empty() {
        return None;
    }
    Some(Effect::seq(
        parts
            .into_iter()
            .map(|(amount, to)| Effect::DealDamage {
                source: source.clone(),
                amount,
                to,
            })
            .collect(),
    ))
}

inventory::submit! { EffectPattern { name: "basic effects: damage in parts", priority: 150, parse: damage_parts } }

/// "~ deals 4 damage to each of up to one target creature, up to one target player, and/or
/// up to one target planeswalker": one amount to each object or player named.
fn damage_to_each_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (source, r) = damage_subject(l, b)?;
    let (amount, r) = number_expr(&r)?;
    let r = strip(r, "damage to each of ")?;
    // "each of up to X targets": that many "any target"s (CR 115.4).
    if let Some(x) = r.strip_prefix("up to ") {
        let (n, x) = parse_number(x)?;
        if end(x) == "targets" {
            let mut spec = TargetSpec::any_target();
            spec.min = Value::Const(0);
            spec.max = n;
            spec.text = "up to x targets".to_string();
            let slot = b.add_target(spec, "targets");
            return Some(Effect::DealDamage {
                source,
                amount,
                to: Sel::Target(slot),
            });
        }
    }
    let (sels, tail) = object_list(r, b)?;
    if sels.len() < 2 || !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::DealDamage {
        source,
        amount,
        to: Sel::Union(sels),
    })
}

inventory::submit! { EffectPattern { name: "basic effects: damage to each of several targets", priority: 150, parse: damage_to_each_of } }

/// "Each creature deals 1 damage to its controller." (Rakdos Charm): each object deals
/// damage to its own controller, all at the same time (CR 120.2).
fn each_deals_to_its_controller(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each ")?;
    let (f, plural, rest) = parse_object_phrase(r)?;
    if plural {
        return None;
    }
    let r = rest.trim_start().strip_prefix("deals ")?;
    let (n, r) = parse_number(r)?;
    if strip(r, "damage to its controller").is_none_or(|x| !x.is_empty()) {
        return None;
    }
    const EACH: Var = vars::USER + 3330;
    Some(Effect::ForEach {
        sel: Sel::All(f),
        var: EACH,
        effect: Box::new(Effect::DealDamage {
            source: Sel::Var(EACH),
            amount: n,
            to: Sel::Players(PlayerRef::ControllerOf(Box::new(Sel::Var(EACH)))),
        }),
    })
}

inventory::submit! { EffectPattern { name: "basic effects: each [object] deals damage to its controller", priority: 150, parse: each_deals_to_its_controller } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternatives() {
        let (f, _, rest) = object_alternatives("enchantment, tapped artifact, or tapped creature").unwrap();
        assert_eq!(end(rest), "");
        assert!(matches!(f, Filter::Or(ref v) if v.len() == 3));
        let (f, _, rest) =
            object_alternatives("artifact, enchantment, or tapped creature an opponent controls")
                .unwrap();
        assert_eq!(end(rest), "");
        assert!(matches!(f, Filter::And(ref v) if matches!(v[1], Filter::ControlledBy(_))));
        let (_, _, rest) = object_alternatives("spirit, creature with disturb, or enchantment").unwrap();
        assert_eq!(end(rest), "");
        let (_, _, rest) = object_alternatives("creature, vehicle, or nonbasic land").unwrap();
        assert_eq!(end(rest), "");
        assert!(object_alternatives("creature and/or up to one target planeswalker").is_none());
        assert!(object_alternatives("creature").is_none());
    }
}

#[cfg(test)]
mod builder_tests {
    use super::*;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;

    pub(crate) fn with_builder<T>(type_line: &str, f: impl FnOnce(&mut Builder) -> T) -> T {
        let tl = TypeLine::parse(type_line);
        let ctx = CompileContext {
            card_name: "Probe",
            full_name: "Probe",
            type_line: &tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let mut b = Builder::new(&ctx);
        f(&mut b)
    }

    #[test]
    fn target_lists() {
        with_builder("Sorcery", |b| {
            let (sels, rest) = object_list(
                "target artifact, target creature, target enchantment, and target land",
                b,
            )
            .unwrap();
            assert_eq!((sels.len(), rest.as_str()), (4, ""));
        });
        with_builder("Sorcery", |b| {
            let (sels, rest) = object_list(
                "up to one target creature, up to one target player, and/or up to one target planeswalker",
                b,
            )
            .unwrap();
            assert_eq!((sels.len(), rest.as_str()), (3, ""));
            assert!(matches!(b.targets[1].what, TargetKind::Player(_)));
        });
    }
}

/// "~ gains swampwalk until end of turn and deals 1 damage to you", "it gets +3/+3 until
/// end of turn and deals 3 damage to each opponent": the second instruction's subject is
/// the first one's, the source.
fn deals_without_subject(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("deals ")?;
    if !matches!(super::pronoun_groups::singular_it(b), Sel::This) {
        return None;
    }
    damage_parts(&format!("~ deals {r}"), b)
}

inventory::submit! { EffectPattern { name: "basic effects: deals damage (subject of the previous instruction)", priority: 150, parse: deals_without_subject } }

/// "When ~ blocks a creature, destroy both creatures." (Alaborn Zealot): ~ and the creature
/// the trigger is about.
fn both_creatures(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("destroy both creatures")?;
    if !r.is_empty() || !b.in_trigger || b.sentences != 0 {
        return None;
    }
    let (other, _) = object_ref("that creature", b)?;
    if matches!(other, Sel::This) {
        return None;
    }
    Some(Effect::Destroy {
        what: Sel::Union(vec![Sel::This, other]),
        no_regen: false,
    })
}

inventory::submit! { EffectPattern { name: "basic effects: destroy both creatures", priority: 150, parse: both_creatures } }
