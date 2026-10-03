//! "If [condition], [sentence that continues the previous one]": a sentence that only
//! makes sense after the previous one (it refers to what that one did: "return that card
//! to the battlefield ... at the beginning of the next end step" after an exile, "you may
//! play the exiled cards until ..." after exiling cards) happens only if the condition
//! holds: "Exile target nontoken creature. If the gift wasn't promised, return that card
//! to the battlefield under its owner's control with a +1/+1 counter on it at the
//! beginning of the next end step." (Parting Gust). The continuation is understood by the
//! follow-up patterns; what it adds after the previous effect is wrapped in the
//! condition. Targets it adds are chosen only if an optional cost the condition names was
//! paid (CR 601.2c).
//!
//! Also "[instruction] if [condition]" with the condition last ("Put a +1/+1 counter on
//! the creature you control if the gift was promised.", Longstalk Brawl), read as "If
//! [condition], [instruction]", for conditions that don't refer back to anything.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;

/// The instructions of an effect, in order.
fn parts(e: &Effect) -> Vec<Effect> {
    match e {
        Effect::Seq(v) => v.clone(),
        Effect::Noop => vec![],
        other => vec![other.clone()],
    }
}

fn same(a: &Effect, b: &Effect) -> bool {
    serde_json::to_string(a).ok() == serde_json::to_string(b).ok()
}

fn if_condition_continuation(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    // "If ..., [effect] instead" replaces the previous effect (other patterns).
    if x.ends_with(" instead") || x.starts_with("instead ") || matches!(prev, Effect::Noop) {
        return false;
    }
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx)
        .or_else(|| super::conditions_referents::parse_condition_with(c, b))
    else {
        return false;
    };
    let before = parts(prev);
    let first_new_target = b.targets.len();
    let mut trial = prev.clone();
    if !crate::oracle_ext::apply_followup_ext(x, &mut trial, b) {
        return false;
    }
    let after = parts(&trial);
    if after.len() <= before.len() || !before.iter().zip(&after).all(|(p, q)| same(p, q)) {
        return false;
    }
    targets_only_if_paid(&cond, b, first_new_target);
    let added = Effect::seq(after[before.len()..].to_vec());
    let mut v = before;
    v.push(Effect::If {
        cond,
        then: Box::new(added),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if [condition], [continuation of the previous sentence]", priority: 200, apply: if_condition_continuation } }

/// Whether a condition refers to nothing named earlier (no pronouns).
fn pronoun_free(c: &str) -> bool {
    !c.split(' ').any(|w| {
        matches!(
            w,
            "it" | "its" | "it's" | "that" | "they" | "their" | "them" | "those" | "he" | "she"
        )
    })
}

/// Marks targets from `first_new` on as chosen only if the optional cost `cond` names was
/// (or wasn't) paid (CR 601.2c).
fn targets_only_if_paid(cond: &Condition, b: &mut Builder, first_new: usize) {
    // "If this spell's additional cost was paid, destroy target ...": also announced
    // before targets (CR 601.2b).
    let paid = |c: &Condition| match c {
        Condition::CostPaid(_) => true,
        Condition::Custom(n) => n == crate::player_control::ADDITIONAL_COST_PAID,
        _ => false,
    };
    let cast_time = match cond {
        Condition::Not(inner) => paid(inner),
        c => paid(c),
    };
    if cast_time {
        for spec in &mut b.targets[first_new..] {
            spec.condition = Some(cond.clone());
        }
    }
}

/// "that player has 10 or less life": the life total of the player named earlier.
fn that_player_life(c: &str, b: &Builder) -> Option<Condition> {
    use crate::oracle::patterns::oracle_hardening_referents::is_no_player_referent;
    let r = c.strip_prefix("that player has ")?.strip_suffix(" life")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let cmp = match r.trim() {
        "or less" => Cmp::Le,
        "or more" => Cmp::Ge,
        _ => return None,
    };
    if is_no_player_referent(&b.it_player) || matches!(b.it_player, PlayerRef::You) {
        return None;
    }
    Some(Condition::PlayerMatches(
        b.it_player.clone(),
        PlayerFilter::Life(cmp, Box::new(n)),
    ))
}

/// "no other creature has greater power" (Getaway Glamer): no creature other than the
/// object named earlier has greater power than it.
fn no_other_greater_power(c: &str, b: &Builder) -> Option<Condition> {
    if c != "no other creature has greater power" {
        return None;
    }
    let it = b.it.clone();
    if !matches!(it, Sel::Target(_)) {
        return None;
    }
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![
            Filter::creature(),
            Filter::Not(Box::new(Filter::In(Box::new(it.clone())))),
            Filter::Power(Cmp::Gt, Box::new(Value::PowerOf(Box::new(it)))),
        ])),
        Cmp::Eq,
        Value::c(0),
    ))
}

fn trailing_if(l: &str, b: &mut Builder) -> Option<Effect> {
    let (x, c) = crate::oracle::phrases::end(l).rsplit_once(" if ")?;
    // An instruction with a condition of its own ("A if [c1], and B if [c2]", Invert the
    // Skies) isn't governed as a whole by the last condition.
    if x.is_empty() || x.starts_with("if ") || x.contains(" if ") || c.contains(',') {
        return None;
    }
    let first_new = b.targets.len();
    // "Destroy target creature if no other creature has greater power.": the condition is
    // about the target the instruction names.
    if c == "no other creature has greater power" {
        let e = crate::oracle::effects::parse_clause(x, b)?;
        let Some(cond) = no_other_greater_power(c, b) else {
            b.targets.truncate(first_new);
            return None;
        };
        return Some(Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    // "You may cast the exiled card without paying its mana cost if it's an instant spell
    // with mana value 2 or less", "you may cast it if it's a creature spell": the card,
    // which isn't a spell yet, is checked for the qualities the spell would have (a spell
    // is a card on the stack, CR 112.1).
    let as_card;
    let c = if x.contains("cast ")
        && c.starts_with("it's ")
        && (c.ends_with(" spell") || c.contains(" spell with "))
    {
        // Not of a card exiled face down (it has no characteristics there).
        if super::dig_grammar::exiled_face_down(b) {
            return None;
        }
        as_card = c.replacen(" spell", " card", 1);
        as_card.as_str()
    } else {
        c
    };
    let original = c;
    // "You may put the exiled card onto the battlefield if it's a creature card" (The
    // Creation of Avacyn): "it" is the card exiled with the source that the instruction
    // names (CR 607.2a), not the source.
    let exiled_card = if matches!(b.it, Sel::This) && !pronoun_free(c) {
        instruction_names_exiled_card(x, b)
    } else {
        None
    };
    // Where "it" is the ability's source ("Whenever ~ attacks, you win the game if there
    // are twenty or more counters on it"), so is the condition's "it".
    let about_source;
    let c = if matches!(b.it, Sel::This) && !pronoun_free(c) && exiled_card.is_none() {
        about_source = c
            .split(' ')
            .map(|w| if w == "it" { "~" } else { w })
            .collect::<Vec<_>>()
            .join(" ");
        about_source.as_str()
    } else {
        c
    };
    let simple = match that_player_life(c, b) {
        Some(c) => Some(c),
        None if pronoun_free(c) => {
            let parse = |c: &str| crate::oracle::statics::parse_condition(c, b.ctx);
            // "if there are twenty or more counters on ~ or you have twenty or more cards
            // in hand": either of two conditions.
            parse(c).or_else(|| {
                c.match_indices(" or ").find_map(|(i, _)| {
                    Some(Condition::Or(vec![parse(&c[..i])?, parse(&c[i + 4..])?]))
                })
            })
        }
        None => None,
    };
    let cond = match simple {
        Some(c) => c,
        // "Destroy target creature if it's white.", "draw a card if that player has more
        // cards in hand than each other player": the condition is about what the
        // instruction (or an earlier one) names, so the instruction is read first.
        None => {
            if super::conditions_referents::ambiguous_it(original, b) {
                return None;
            }
            let it_before = b.it.clone();
            let e = crate::oracle::effects::parse_clause(x, b)?;
            // The condition is checked before the instruction happens: "it" is what it
            // was before, or a target the instruction named, not the objects it produced
            // ("Put target creature card ... onto the battlefield ... if its mana value is
            // ...").
            let it_after = b.it.clone();
            if let Some(sel) = exiled_card {
                b.it = sel;
            } else if matches!(it_after, Sel::Var(_)) {
                let new_object_target = (first_new..b.targets.len())
                    .rev()
                    .find(|i| !matches!(b.targets[*i].what, TargetKind::Player(_)));
                b.it = match new_object_target {
                    Some(i) => Sel::Target(i as u8),
                    None => it_before,
                };
            }
            let cond = super::conditions_referents::parse_condition_with(original, b)
                .map(card_in_new_zone);
            b.it = it_after;
            let Some(cond) = cond else {
                b.targets.truncate(first_new);
                return None;
            };
            return Some(Effect::If {
                cond,
                then: Box::new(e),
                otherwise: Box::new(Effect::Noop),
            });
        }
    };
    let e = crate::oracle::effects::parse_clause(x, b)?;
    targets_only_if_paid(&cond, b, first_new);
    Some(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "[instruction] if [condition]", priority: 250, parse: trailing_if } }

/// "Whenever a face-down creature you control dies, return it to the battlefield ... if
/// it's a permanent card" (Yarus, Roar of the Old Gods): a condition on the triggering
/// object as a card is about the card in its new zone (CR 400.7), not about the permanent
/// as it last existed (a face-down permanent's last known information is a 2/2 creature
/// with no name, CR 708.2a).
fn card_in_new_zone(c: Condition) -> Condition {
    fn names_card(f: &Filter) -> bool {
        match f {
            Filter::Card | Filter::PermanentCard => true,
            Filter::And(v) => v.iter().any(names_card),
            _ => false,
        }
    }
    match c {
        Condition::SelMatches(Sel::TriggerLki, f) if names_card(&f) => {
            Condition::SelMatches(Sel::TriggerObject, f)
        }
        c => c,
    }
}

/// The cards exiled with the source that the instruction `x` (read on a copy of the
/// builder) moves ("put the exiled card onto the battlefield"), which a following "it"
/// then refers to.
fn instruction_names_exiled_card(x: &str, b: &Builder) -> Option<Sel> {
    let mut probe = crate::oracle::effects::Builder {
        targets: b.targets.clone(),
        it: b.it.clone(),
        it_player: b.it_player.clone(),
        in_trigger: b.in_trigger,
        sentences: b.sentences,
        chosen_creature: b.chosen_creature.clone(),
        group: b.group.clone(),
        named: b.named.clone(),
        its_is_it: b.its_is_it,
        ctx: b.ctx,
    };
    let e = crate::oracle::effects::parse_clause(x, &mut probe)?;
    if probe.targets.len() != b.targets.len() {
        return None;
    }
    fn linked(f: &Filter) -> bool {
        match f {
            Filter::In(s) => matches!(**s, Sel::Linked),
            Filter::And(v) => v.iter().any(linked),
            _ => false,
        }
    }
    // The instruction (perhaps optional) moves the linked exiled cards.
    let e = match e {
        Effect::May { effect, .. } => *effect,
        e => e,
    };
    match e {
        Effect::Move {
            what: what @ Sel::All(_),
            ..
        } if matches!(&what, Sel::All(f) if linked(f)) => Some(what),
        _ => None,
    }
}

/// "that creature isn't legendary", "it's legendary": whether the object named earlier is
/// legendary.
fn legendary_condition(c: &str, b: &mut Builder) -> Option<Condition> {
    let saved = b.targets.len();
    let (sel, rest) = crate::oracle::effects::object_ref(c, b)?;
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    let legendary = Filter::Supertype(crate::types::Supertype::Legendary);
    match rest.trim() {
        "isn't legendary" | "is not legendary" => {
            Some(Condition::SelMatches(sel, Filter::Not(Box::new(legendary))))
        }
        "is legendary" => Some(Condition::SelMatches(sel, legendary)),
        _ => None,
    }
}

/// "If the gift was promised and that creature isn't legendary, create a token that's a
/// copy of that creature, except it's 1/1." (Coiling Rebirth): conditions joined by "and",
/// some about an object named earlier.
fn if_and_object_condition(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = crate::oracle::phrases::end(l).strip_prefix("if ")?;
    let (c, x) = r.split_once(", ")?;
    let (c1, c2) = c.split_once(" and ")?;
    let first = crate::oracle::statics::parse_condition(c1, b.ctx)?;
    let second = legendary_condition(c2, b)?;
    let first_new = b.targets.len();
    let e = crate::oracle::effects::parse_clause(x, b)?;
    targets_only_if_paid(&first, b, first_new);
    Some(Effect::If {
        cond: Condition::And(vec![first, second]),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if [condition] and [object] isn't legendary, [instruction]", priority: 250, parse: if_and_object_condition } }

/// Groups a trailing run of two or more conditional instructions about the same object
/// ("If it was a creature card, create a 2/2 black Rogue creature token. If it was a land
/// card, create a Treasure token.") into one effect, for an "Otherwise, ..." that follows
/// them all.
pub(crate) fn group_condition_run(effects: &mut Vec<Effect>) {
    let subject_of = |e: &Effect| match e {
        Effect::If { cond, otherwise, .. } if matches!(**otherwise, Effect::Noop) => {
            super::conditions_this_way::cond_subject(cond).map(|s| format!("{s:?}"))
        }
        _ => None,
    };
    let Some(subject) = effects.last().and_then(subject_of) else {
        return;
    };
    let run = effects
        .iter()
        .rev()
        .take_while(|e| subject_of(e).as_ref() == Some(&subject))
        .count();
    if run >= 2 {
        let tail = effects.split_off(effects.len() - run);
        effects.push(Effect::Seq(tail));
    }
}

/// "Otherwise, [instruction]." after a conditional instruction ("You lose life equal to
/// that card's mana value if ~ isn't saddled. Otherwise, each opponent loses that much
/// life.", Caustic Bronco): what happens if the condition doesn't hold.
fn otherwise(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = crate::oracle::phrases::end(l).strip_prefix("otherwise, ") else {
        return false;
    };
    // "You may put that card onto the battlefield if it's a permanent card ... Otherwise,
    // ...": the condition's alternative.
    super::conditions_this_way::normalize_may_if(prev);
    // "If it was a creature card, ... If it was a land card, ... Otherwise, ...": the
    // alternative to every one of a run of conditions about the same object.
    let earlier: Vec<Condition> = match &*prev {
        Effect::Seq(v) if v.len() >= 2 => {
            let subject_of = |e: &Effect| match e {
                Effect::If {
                    cond, otherwise, ..
                } if matches!(**otherwise, Effect::Noop) => {
                    super::conditions_this_way::cond_subject(cond)
                        .map(|s| (format!("{s:?}"), cond.clone()))
                }
                _ => None,
            };
            match subject_of(&v[v.len() - 1]) {
                Some((subject, _)) => v[..v.len() - 1]
                    .iter()
                    .rev()
                    .map_while(|e| subject_of(e).filter(|(s, _)| *s == subject))
                    .map(|(_, c)| c)
                    .collect(),
                None => vec![],
            }
        }
        _ => vec![],
    };
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(Effect::If {
        cond,
        then,
        otherwise,
    }) = last
    else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    // "Otherwise, put it into your hand.": "it" is what the condition is about, not what
    // the instruction it governed (which didn't happen) produced.
    let subject = super::conditions_this_way::cond_subject(cond);
    // "that much life": the amount of life the instruction the condition governs would
    // have gained or lost.
    let much = r.contains("that much life");
    let amount = match &**then {
        Effect::LoseLife { n, .. } | Effect::GainLife { n, .. } => Some(n.clone()),
        _ => None,
    };
    if much && amount.is_none() {
        return false;
    }
    let text = r.replace("that much life", "1 life");
    let first_new = b.targets.len();
    let saved_it = b.it.clone();
    if let Some(sel) = &subject {
        b.it = sel.clone();
    }
    // "Otherwise, you may put it into your graveyard.": an optional instruction.
    let parsed = match text.strip_prefix("you may ") {
        Some(_) => crate::oracle::effects::parse_sentence(&text, b),
        None => crate::oracle::effects::parse_clause(&text, b),
    };
    b.it = saved_it;
    let Some(mut e) = parsed else {
        b.targets.truncate(first_new);
        return false;
    };
    if much {
        match &mut e {
            Effect::LoseLife { n, .. } | Effect::GainLife { n, .. } => {
                *n = amount.unwrap_or(Value::c(0))
            }
            _ => {
                b.targets.truncate(first_new);
                return false;
            }
        }
    }
    // "Otherwise, they put it into their graveyard and ~ deals 2 damage to them": an
    // instruction that starts by putting the card the condition is about somewhere, joined
    // with more, is about that card; with no card (an empty library) none of it happens.
    if let (Some(sel @ Sel::Var(_)), Effect::Seq(v)) = (&subject, &e) {
        if v.len() >= 2 && matches!(&v[0], Effect::Move { what, .. } if format!("{what:?}") == format!("{sel:?}"))
        {
            e = Effect::If {
                cond: Condition::SelMatches(sel.clone(), Filter::Any),
                then: Box::new(e),
                otherwise: Box::new(Effect::Noop),
            };
        }
    }
    if !earlier.is_empty() {
        e = Effect::If {
            cond: Condition::Not(Box::new(Condition::Or(earlier))),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        };
    }
    **otherwise = e;
    true
}

inventory::submit! { FollowupPattern { name: "otherwise, [instruction]", priority: 200, apply: otherwise } }

/// "[instruction] instead if [condition]" ("That card gains flashback {0} until end of turn
/// instead if ~ is saddled.", Archmage's Newt): the same as "If [condition], [instruction]
/// instead."
fn instead_if(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some((x, c)) = crate::oracle::phrases::end(l).rsplit_once(" instead if ") else {
        return false;
    };
    if x.is_empty() || x.starts_with("if ") || c.contains(',') {
        return false;
    }
    crate::oracle_ext::apply_followup_ext(&format!("if {c}, {x} instead"), prev, b)
}

inventory::submit! { FollowupPattern { name: "[instruction] instead if [condition]", priority: 200, apply: instead_if } }
