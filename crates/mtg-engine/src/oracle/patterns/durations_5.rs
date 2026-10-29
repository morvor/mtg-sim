//! Durations, grammar V (cards4-triggers-durations): durations ending at a step of a later
//! turn and "for as long as" conditions about the objects the instruction names (CR 611.2a,
//! 611.2b).
//!
//! - "Target land becomes a Swamp until its controller's next untap step." (Orcish Farmer)
//! - "~'s base power and toughness become equal to the power and toughness of target
//!   creature other than ~ until the end of your next upkeep." (Halfdane)
//! - "~ becomes a copy of that creature for as long as that creature remains tapped."
//!   (Zygon Infiltrator)
//! - "Gain control of target creature with power less than or equal to ~'s power for as
//!   long as ~ remains tapped and that creature's power remains less than or equal to ~'s
//!   power." (Old Man of the Sea)
//!
//! Each instruction is read as the core grammar reads it with "until end of turn", and the
//! duration of the one continuous effect it creates is replaced.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

/// Replaces the `EndOfTurn` duration of the one continuous effect `e` creates with `d`.
/// `allow` says which effects may take the new duration. Returns how many were replaced.
fn swap_duration(e: &mut Effect, d: &Duration, allow: fn(&Effect) -> bool) -> usize {
    match e {
        Effect::Seq(v) => v.iter_mut().map(|x| swap_duration(x, d, allow)).sum(),
        // "you may have that creature become a copy of ..." (Blade of Shared Souls).
        Effect::May { effect, .. } => swap_duration(effect, d, allow),
        _ if !allow(e) => 0,
        Effect::Modify { duration, .. }
        | Effect::BecomeCopy { duration, .. }
        | Effect::BecomeCopyExcept { duration, .. }
        | Effect::GainControl { duration, .. }
            if matches!(duration, Duration::EndOfTurn) =>
        {
            *duration = d.clone();
            1
        }
        _ => 0,
    }
}

/// Effects that apply to particular objects, fixed as they begin (CR 611.2c).
fn object_effect(e: &Effect) -> bool {
    let what = match e {
        Effect::Modify { what, .. }
        | Effect::BecomeCopy { what, .. }
        | Effect::BecomeCopyExcept { what, .. }
        | Effect::GainControl { what, .. } => what,
        _ => return false,
    };
    !matches!(what, Sel::None | Sel::All(_))
}

/// `head` read with "until end of turn", with the duration replaced by `d`.
fn retimed(head: &str, d: Duration, b: &mut Builder, allow: fn(&Effect) -> bool) -> Option<Effect> {
    let saved = (b.targets.len(), b.it.clone());
    let mut e = parse_sentence(&format!("{head} until end of turn"), b)?;
    if swap_duration(&mut e, &d, allow) != 1 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(e)
}

/// "[instruction] until its controller's next untap step", "[instruction] until the end of
/// your next upkeep".
fn until_later_step(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let steps = [
        ("untap step", TriggerStep::Untap),
        ("upkeep", TriggerStep::Upkeep),
        ("draw step", TriggerStep::Draw),
        ("end step", TriggerStep::End),
    ];
    for (name, step) in steps {
        if let Some(head) = l.strip_suffix(&format!(" until its controller's next {name}") as &str)
        {
            return retimed(
                head,
                Duration::UntilItsControllersNextStep(step),
                b,
                object_effect,
            );
        }
        if let Some(head) = l.strip_suffix(&format!(" until the end of your next {name}") as &str) {
            return retimed(head, Duration::UntilEndOfYourNextStep(step), b, object_effect);
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "durations 5: until its controller's next step, until the end of your next step", priority: 90, parse: until_later_step } }

/// One condition of "for as long as [condition] and [condition]", with `it` the object the
/// instruction named ("that creature").
fn remains(c: &str, it: &Sel) -> Option<Condition> {
    let named = |s: &str| -> Option<Sel> {
        match s {
            "~" => Some(Sel::This),
            "that creature" | "that permanent" | "it" if !matches!(it, Sel::None | Sel::This) => {
                Some(it.clone())
            }
            _ => None,
        }
    };
    // "~ remains attached to it" (Assimilation Aegis).
    if let Some((s, o)) = c.split_once(" remains attached to ") {
        return Some(Condition::SelMatches(
            named(s)?,
            Filter::AttachedTo(Box::new(named(o)?)),
        ));
    }
    if let Some(s) = c.strip_suffix(" remains tapped") {
        return Some(match named(s)? {
            Sel::This => match crate::untap_choice::remains_tapped(false) {
                Duration::WhileCondition(c) => c,
                _ => return None,
            },
            s => Condition::SelMatches(s, Filter::Tapped),
        });
    }
    // "that creature's power remains less than or equal to ~'s power".
    let (subject, r) = c.split_once("'s power remains ")?;
    let who = named(subject)?;
    let (cmp, other) = [
        ("less than or equal to ", Cmp::Le),
        ("greater than or equal to ", Cmp::Ge),
        ("less than ", Cmp::Lt),
        ("greater than ", Cmp::Gt),
    ]
    .into_iter()
    .find_map(|(p, cmp)| r.strip_prefix(p).map(|o| (cmp, o)))?;
    let other = other.strip_suffix("'s power")?;
    let other = named(other)?;
    Some(Condition::Compare(
        Value::PowerOf(Box::new(who)),
        cmp,
        Value::PowerOf(Box::new(other)),
    ))
}

/// "[instruction] for as long as that creature remains tapped", "... for as long as ~
/// remains tapped and that creature's power remains less than or equal to ~'s power": a
/// condition about the objects the instruction names, fixed as the effect begins
/// (`duration_refs`).
fn for_as_long_as_it_remains(l: &str, b: &mut Builder) -> Option<Effect> {
    // "For as long as ~ remains attached to it, that creature becomes a copy of ..."
    // (Assimilation Aegis): the duration written first.
    let moved;
    let l = match end(l).strip_prefix("for as long as ") {
        Some(r) => {
            let (cond, rest) = r.split_once(", ")?;
            moved = format!("{rest} for as long as {cond}");
            moved.as_str()
        }
        None => l,
    };
    let (head, cond) = end(l).rsplit_once(" for as long as ")?;
    if cond.contains(", ") {
        return None;
    }
    // Durations the core grammar reads alone go there.
    let parts: Vec<&str> = cond.split(" and ").collect();
    let names_it = parts.iter().any(|p| {
        p.starts_with("that ") || p.starts_with("it ") || p.ends_with(" attached to it")
    });
    if !names_it {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let mut e = parse_sentence(&format!("{head} until end of turn"), b)?;
    // "that creature": what an earlier sentence named ("Tap another target creature ...
    // ~ becomes a copy of that creature ..."), else what this one names.
    let it = match &saved.1 {
        Sel::None | Sel::This => b.it.clone(),
        s => s.clone(),
    };
    let conds: Option<Vec<Condition>> = parts.iter().map(|p| remains(p, &it)).collect();
    let d = match conds {
        Some(mut v) if v.len() == 1 => Duration::WhileCondition(v.remove(0)),
        Some(v) => Duration::WhileCondition(Condition::And(v)),
        None => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        }
    };
    if swap_duration(&mut e, &d, object_effect) != 1 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "durations 5: for as long as that creature remains", priority: 90, parse: for_as_long_as_it_remains } }

/// "~'s base power and toughness become equal to the power and toughness of target
/// creature other than ~ until end of turn" (Halfdane): the values of that creature as the
/// effect begins (CR 608.2h), in layer 7b (CR 613.4b).
fn base_pt_equal_to_other(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, r) = l.split_once("'s base power and toughness become equal to the power and toughness of ")?;
    let r = r.strip_suffix(" until end of turn")?;
    let what = match subject {
        "~" => Sel::This,
        _ => return None,
    };
    let (of, tail) = crate::oracle::effects::object_ref(r, b)?;
    if !end(&tail).is_empty() || matches!(of, Sel::None | Sel::This | Sel::All(_)) {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::SetPT(
            Some(Value::PowerOf(Box::new(of.clone()))),
            Some(Value::ToughnessOf(Box::new(of))),
        )],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "durations 5: ~'s base power and toughness become equal to another's", priority: 100, parse: base_pt_equal_to_other } }

/// "For as long as ~ remains on the battlefield, the chosen permanents become Treasure
/// artifacts with \"{T}, Sacrifice ~: Add one mana of any color\" and lose all other
/// abilities." (Kitesail Larcenist): the instruction read with a leading "until end of
/// turn" (whose list of predicates may hold quoted abilities), for as long as the source
/// remains on the battlefield (CR 611.2b).
fn for_as_long_as_source_remains_first(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (d, rest) = [
        (
            "for as long as ~ remains on the battlefield, ",
            Duration::WhileSourceOnBattlefield,
        ),
        ("for as long as you control ~, ", Duration::WhileYouControlSource),
    ]
    .into_iter()
    .find_map(|(p, d)| l.strip_prefix(p).map(|r| (d, r)))?;
    if !rest.contains('"') {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let mut e = parse_sentence(&format!("until end of turn, {rest}"), b)?;
    if swap_duration(&mut e, &d, object_effect) != 1 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "durations 5: for as long as ~ remains on the battlefield, [instruction with quoted abilities]", priority: 90, parse: for_as_long_as_source_remains_first } }

/// Verbs that start a second predicate of a player subject ("can't cast spells ... and
/// plays with their hand revealed").
const PLAYER_PREDICATES: [&str; 6] = ["plays ", "can't ", "skips ", "loses ", "gains ", "gets "];

/// "This turn, that player can't cast spells or activate abilities and plays with their
/// hand revealed." (Sen Triplets): each predicate of the player is its own instruction
/// lasting this turn.
fn this_turn_player_predicates(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("this turn, ")?;
    let (subject, preds) = ["that player ", "target player ", "target opponent "]
        .into_iter()
        .find_map(|s| r.strip_prefix(s).map(|p| (s, p)))?;
    let (a, c) = preds
        .match_indices(" and ")
        .map(|(i, _)| (&preds[..i], &preds[i + 5..]))
        .find(|(_, c)| PLAYER_PREDICATES.iter().any(|v| c.starts_with(v)))?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let first = parse_sentence(&format!("{subject}{a} this turn"), b);
    // The second predicate is about the same player.
    let second = first
        .as_ref()
        .and_then(|_| parse_sentence(&format!("that player {c} this turn"), b));
    match (first, second) {
        (Some(x), Some(y)) => Some(Effect::seq(vec![x, y])),
        _ => {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "durations 5: this turn, [player] [predicate] and [predicate]", priority: 90, parse: this_turn_player_predicates } }

/// "You may play lands and cast spells from that player's hand this turn." (Sen
/// Triplets): a permission to play the cards in that player's hand (the player as the
/// effect begins), as lands or spells (CR 305.1, 601.2a).
fn play_from_that_players_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("you may ")?;
    let (lands, spells, r) = if let Some(r) = r.strip_prefix("play lands and cast spells ") {
        (true, true, r)
    } else if let Some(r) = r.strip_prefix("cast spells ") {
        (false, true, r)
    } else {
        return None;
    };
    if r != "from that player's hand this turn"
        || !matches!(b.it_player, PlayerRef::Target(_) | PlayerRef::TriggerPlayer)
    {
        return None;
    }
    let terms = PlayTerms {
        any_owner: true,
        ..Default::default()
    };
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::PlayPermission(PlayPermission {
            who: PlayerRel::You,
            zone: ZoneKind::Hand,
            top_only: false,
            what: Filter::OwnedByPlayer(Box::new(b.it_player.clone())),
            lands,
            spells,
            cost: None,
            flash: false,
            terms,
        }),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "durations 5: you may play lands and cast spells from that player's hand this turn", priority: 100, parse: play_from_that_players_hand } }

/// What "the chosen creatures" names (a choice made earlier in the ability).
fn chosen_creatures(b: &Builder) -> Option<Sel> {
    b.named
        .iter()
        .rev()
        .find(|(p, _)| p == "the chosen creatures")
        .map(|(_, s)| s.clone())
}

/// "During that player's next turn, the chosen creatures attack if able, and other
/// creatures can't attack." (Oracle en-Vec): a requirement and a restriction that apply
/// during that player's next turn (`next_turn.rs`), about the creatures chosen as the
/// ability resolved; "other creatures" are all creatures but those, whenever they came
/// under that player's control (CR 611.2c, 508.1c-d).
fn during_next_turn_chosen_attack(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("during that player's next turn, ")?;
    if r != "the chosen creatures attack if able, and other creatures can't attack"
        || !matches!(b.it_player, PlayerRef::Target(_) | PlayerRef::TriggerPlayer)
    {
        return None;
    }
    let chosen = chosen_creatures(b)?;
    let them = Filter::In(Box::new(chosen));
    Some(Effect::DuringNext {
        who: b.it_player.clone(),
        part: NextTurnPart::Turn,
        effect: Box::new(Effect::seq(vec![
            Effect::AddRestriction {
                restriction: Restriction::MustAttack(them.clone()),
                duration: Duration::EndOfTurn,
            },
            Effect::AddRestriction {
                restriction: Restriction::CantAttack(Filter::and(vec![
                    Filter::creature(),
                    Filter::not(them),
                ])),
                duration: Duration::EndOfTurn,
            },
        ])),
    })
}

inventory::submit! { EffectPattern { name: "durations 5: during that player's next turn, the chosen creatures attack if able, and other creatures can't attack", priority: 100, parse: during_next_turn_chosen_attack } }

/// "At the beginning of that turn's end step, destroy each of the chosen creatures that
/// didn't attack this turn." after an instruction waiting for that player's next turn
/// (Oracle en-Vec): a delayed ability created as that turn begins, for its end step
/// (CR 603.7a), about the creatures chosen as the ability resolved.
fn at_that_turns_end_step(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let lower = l.to_lowercase();
    let Some(r) = end(&lower).strip_prefix("at the beginning of that turn's end step, ") else {
        return false;
    };
    if r != "destroy each of the chosen creatures that didn't attack this turn" {
        return false;
    }
    fn waiting(e: &mut Effect) -> Option<&mut Effect> {
        match e {
            Effect::Seq(v) => v.last_mut().and_then(waiting),
            Effect::DuringNext {
                part: NextTurnPart::Turn,
                effect,
                ..
            } => Some(&mut **effect),
            _ => None,
        }
    }
    let Some(chosen) = chosen_creatures(b) else {
        return false;
    };
    let Some(effect) = waiting(prev) else {
        return false;
    };
    let destroy = Effect::Destroy {
        what: Sel::All(Filter::and(vec![
            Filter::In(Box::new(chosen)),
            Filter::not(Filter::AttackedThisTurn),
        ])),
        no_regen: false,
    };
    let old = std::mem::take(effect);
    *effect = Effect::seq(vec![
        old,
        Effect::AtNext {
            step: TriggerStep::End,
            effect: Box::new(destroy),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "durations 5: at the beginning of that turn's end step, destroy each of the chosen creatures that didn't attack", priority: 40, apply: at_that_turns_end_step } }
