//! Instructions to tap, sacrifice, or exile objects the player chooses as the effect
//! happens (CR 608.2d), not targets:
//!
//! - "tap any number of untapped creatures you control", "sacrifice any number of lands",
//!   "exile any number of other nonland permanents you own and control", "you may
//!   sacrifice up to three Zombies", "you may sacrifice one or more Treasures";
//! - optional actions that are costs paid on resolution (CR 118.12): "you may tap two
//!   untapped creatures you control. If you do, ...", "any opponent may tap an untapped
//!   creature they control", "you may exile a Human you control and an artifact you
//!   control", "you may sacrifice another creature or an artifact";
//! - what later sentences say about the objects: "for each creature tapped this way",
//!   "that much damage", "draw that many cards", "where X is the number of creatures tapped
//!   this way", "with mana value less than or equal to the number of creatures sacrificed
//!   this way", "each of those Vampires".

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::*;

/// The objects chosen.
pub(crate) const CHOSEN: Var = vars::USER + 3310;
/// The number of objects the instruction actually affected.
pub(crate) const COUNT: Var = vars::USER + 3311;
/// The objects it affected.
pub(crate) const DONE: Var = vars::USER + 3312;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verb {
    Tap,
    Untap,
    Sacrifice,
    Exile,
}

impl Verb {
    fn word(self) -> &'static str {
        match self {
            Verb::Tap => "tap",
            Verb::Untap => "untap",
            Verb::Sacrifice => "sacrifice",
            Verb::Exile => "exile",
        }
    }
    fn participle(self) -> &'static str {
        match self {
            Verb::Tap => "tapped",
            Verb::Untap => "untapped",
            Verb::Sacrifice => "sacrificed",
            Verb::Exile => "exiled",
        }
    }
    /// The effect on the chosen objects, and the variable holding the ones it affected.
    fn effect(self, what: Sel) -> (Effect, Var) {
        match self {
            Verb::Tap => (Effect::Tap { what }, vars::TAPPED),
            Verb::Untap => (Effect::Untap { what: what.clone() }, CHOSEN),
            Verb::Sacrifice => (Effect::SacrificeObjects { what }, vars::IT),
            Verb::Exile => (
                Effect::Exile {
                    what,
                    face_down: false,
                    link: false,
                },
                vars::IT,
            ),
        }
    }
}

/// The objects a player can choose for the verb: their own for sacrificing (CR 701.21a).
fn own(verb: Verb, f: Filter) -> Filter {
    match verb {
        Verb::Sacrifice => Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]),
        _ => f,
    }
}

/// "[verb] any number of [objects]", "[verb] up to three [objects]", "[verb] one or more
/// [objects]": the player chooses the objects as the instruction is performed, then the
/// instruction is performed on them. Records them and how many it affected for later
/// sentences.
fn verb_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (verb, r) = [Verb::Tap, Verb::Untap, Verb::Sacrifice, Verb::Exile]
        .into_iter()
        .find_map(|v| l.strip_prefix(v.word()).and_then(|r| r.strip_prefix(' ')).map(|r| (v, r)))?;
    // "any number of", "one or more", "up to N", or exactly N ("exile a land you
    // control"; exactly N only for exiling, the others' exact forms are core verbs).
    let (count, up_to, r) = if let Some(r) = r
        .strip_prefix("any number of ")
        .or_else(|| r.strip_prefix("one or more "))
    {
        (None, true, r)
    } else if let Some(r) = r.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        (Some(n), true, r)
    } else if verb == Verb::Exile {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        (Some(n), false, r)
    } else {
        return None;
    };
    if r.starts_with("target") {
        return None;
    }
    // "other nonland permanents you own and control".
    let owned = r.contains(" you own and control");
    let r_owned = r.replacen(" you own and control", " you control", 1);
    let r = if owned { r_owned.as_str() } else { r };
    let (f, plural, tail) = parse_object_phrase(r)?;
    let f = if owned {
        Filter::and(vec![f, Filter::OwnedBy(PlayerRel::You)])
    } else {
        f
    };
    if plural != (up_to || count.as_ref().is_some_and(|n| n.as_const() != Some(1))) {
        return None;
    }
    // "exile any number of other nontoken creatures you control until it leaves the
    // battlefield" (CR 610.3).
    let tail = end(tail);
    let until = match tail {
        "" => None,
        "until it leaves the battlefield" | "until ~ leaves the battlefield"
            if verb == Verb::Exile && matches!(b.it, Sel::This) =>
        {
            Some(UntilEvent::SourceLeavesBattlefield)
        }
        _ => return None,
    };
    let f = own(verb, f);
    let count = count.unwrap_or_else(|| Value::Count(f.clone()));
    let choose = Effect::Store {
        var: CHOSEN,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count,
            up_to,
            store: None,
        },
    };
    let (act, done) = match until {
        Some(until) => (
            Effect::ExileUntil {
                what: Sel::Var(CHOSEN),
                until,
            },
            vars::IT,
        ),
        None => verb.effect(Sel::Var(CHOSEN)),
    };
    // "Return those cards to the battlefield ...", "each of those Vampires": the objects
    // the instruction affected (exiled cards are new objects, CR 400.7).
    b.it = Sel::Var(DONE);
    Some(Effect::seq(vec![
        choose,
        act,
        Effect::Store {
            var: DONE,
            sel: Sel::Var(done),
        },
        Effect::StoreValue {
            var: COUNT,
            value: Value::CountSel(Box::new(Sel::Var(DONE))),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "basic effects: [verb] any number of [objects]", priority: 45, parse: verb_chosen } }

/// "you may tap two untapped creatures you control", "any opponent may tap an untapped
/// creature they control", "you may exile a Human you control and an artifact you
/// control": an optional action that is a cost paid as the ability resolves (CR 118.12),
/// so "if you do" means it was paid in full.
fn may_pay_action(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("you may ") {
        (PlayerRef::You, r)
    } else if let Some(r) = l.strip_prefix("any opponent may ") {
        (PlayerRef::EachOpponent, r)
    } else {
        return None;
    };
    if !["tap ", "sacrifice ", "exile "]
        .iter()
        .any(|v| r.starts_with(v))
        || r.contains("target")
        || r.contains("any number of")
        || r.contains("one or more")
        || r.contains(" up to ")
    {
        return None;
    }
    // "they control" for an opponent's own permanents.
    let text = if matches!(who, PlayerRef::EachOpponent) {
        r.replace(" they control", " you control")
    } else {
        r.to_string()
    };
    let (cost, _) = crate::oracle::costs::parse_cost(&text)?;
    if cost.mana.is_some()
        || cost.parts.is_empty()
        || !cost.parts.iter().all(|p| {
            // Exiling from the battlefield only: an exiled card from a hand or graveyard is
            // what later instructions are about ("create a token that's a copy of the
            // exiled card"), which a cost doesn't record.
            matches!(
                p,
                CostPart::TapUntapped { .. }
                    | CostPart::Sacrifice { .. }
                    | CostPart::Exile {
                        zone: ZoneKind::Battlefield,
                        ..
                    }
            )
        })
    {
        return None;
    }
    let _ = b;
    Some(Effect::PayOptional {
        who,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "basic effects: may [tap/sacrifice/exile] as a cost", priority: 45, parse: may_pay_action } }

// ---------------------------------------------------------------------------
// Later sentences about the chosen objects
// ---------------------------------------------------------------------------

/// The verb and filter of the chosen-objects instruction at the end of `e` (looking into
/// sequences and "you may").
fn chosen_instruction(e: &Effect) -> Option<(Verb, &Filter)> {
    match e {
        Effect::Seq(v) => {
            let last_is_count = matches!(v.last(), Some(Effect::StoreValue { var, .. }) if *var == COUNT);
            if last_is_count {
                let f = v.iter().find_map(|x| match x {
                    Effect::Store {
                        var,
                        sel: Sel::Choose { filter, .. },
                    } if *var == CHOSEN => Some(filter),
                    _ => None,
                })?;
                let verb = v.iter().find_map(|x| match x {
                    Effect::Tap { .. } => Some(Verb::Tap),
                    Effect::Untap { .. } => Some(Verb::Untap),
                    Effect::SacrificeObjects { .. } => Some(Verb::Sacrifice),
                    Effect::Exile { .. } => Some(Verb::Exile),
                    _ => None,
                })?;
                return Some((verb, f));
            }
            v.last().and_then(chosen_instruction)
        }
        Effect::May { effect, .. } => chosen_instruction(effect),
        _ => None,
    }
}

/// Whether the counted noun ("creature", "Human", "permanent") describes every object the
/// instruction could choose.
fn noun_covers(noun: &Filter, chosen: &Filter) -> bool {
    let parts: Vec<&Filter> = match chosen {
        Filter::And(v) => v.iter().collect(),
        f => vec![f],
    };
    match noun {
        Filter::Permanent => true,
        Filter::And(v) => v.iter().all(|n| noun_covers(n, chosen)),
        n => parts
            .iter()
            .any(|p| format!("{p:?}") == format!("{n:?}") || matches!(p, Filter::Or(alts) if alts.iter().all(|a| format!("{a:?}") == format!("{n:?}")))),
    }
}

/// Rewrites a mention of the number of affected objects in `s` as "x": "that much",
/// "that many", "the number of [noun] [verb]ed this way". Returns the text and whether
/// anything was rewritten.
fn rewrite_count(s: &str, verb: Verb, chosen: &Filter) -> Option<(String, bool)> {
    let mut out = s.to_string();
    let mut did = false;
    for p in ["that much", "that many"] {
        if out.contains(p) {
            out = out.replace(p, "x");
            did = true;
        }
    }
    let way = format!(" {} this way", verb.participle());
    // ", where x is the number of creatures tapped this way"
    if let Some(i) = out.find(", where x is the number of ") {
        let rest = &out[i + ", where x is the number of ".len()..];
        let j = rest.find(&way)?;
        let (noun, _, tail) = parse_object_phrase(&rest[..j])?;
        if !tail.trim().is_empty() || !noun_covers(&noun, chosen) {
            return None;
        }
        let after = rest[j + way.len()..].to_string();
        out = format!("{}{}", &out[..i], after);
        did = true;
    }
    // "with mana value less than or equal to the number of creatures sacrificed this way"
    for stat in ["mana value", "power", "toughness"] {
        let pat = format!("with {stat} less than or equal to the number of ");
        if let Some(i) = out.find(&pat) {
            let rest = &out[i + pat.len()..];
            let j = rest.find(&way)?;
            let (noun, _, tail) = parse_object_phrase(&rest[..j])?;
            if !tail.trim().is_empty() || !noun_covers(&noun, chosen) {
                return None;
            }
            let after = rest[j + way.len()..].to_string();
            out = format!("{}with {stat} x or less{}", &out[..i], after);
            did = true;
        }
    }
    Some((out, did))
}

/// A sentence after a chosen-objects instruction that mentions how many objects it
/// affected: "~ deals that much damage to any target", "If you do, draw that many cards.",
/// "When you do, ... where X is the number of creatures tapped this way.", "You gain 2
/// life for each creature tapped this way."
fn f_about_chosen(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some((verb, chosen)) = chosen_instruction(prev) else {
        return false;
    };
    let chosen = chosen.clone();
    let mut l = end(&s.to_lowercase()).to_string();
    // "When you sacrifice one or more artifacts this way, ...": when you do (CR 603.12).
    if let Some(r) = l.strip_prefix(&format!("when you {} one or more ", verb.word())) {
        let way = format!(" this way, ");
        let Some(i) = r.find(&way) else {
            return false;
        };
        let Some((nf, _, tail)) = parse_object_phrase(&r[..i]) else {
            return false;
        };
        if !tail.trim().is_empty() || !noun_covers(&nf, &chosen) {
            return false;
        }
        l = format!("when you do, {}", &r[i + way.len()..]);
    }
    // An "x" the sentence doesn't define as the number of objects would be ambiguous.
    if !l.contains(", where x is the number of ")
        && l.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x")
    {
        return false;
    }
    let count = Value::Var(COUNT);
    if verb == Verb::Exile {
        if let Some(e) = return_exiled_later(&l) {
            append(prev, e);
            return true;
        }
    }
    // "[effect] for each [noun] [verb]ed this way"
    let way = format!(" {} this way", verb.participle());
    if let Some((clause, thing)) = l.rsplit_once(" for each ") {
        let Some(noun) = thing.strip_suffix(way.as_str()) else {
            return false;
        };
        let Some((nf, _, tail)) = parse_object_phrase(noun) else {
            return false;
        };
        if !tail.trim().is_empty() || !noun_covers(&nf, &chosen) {
            return false;
        }
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let Some(e) = parse_sentence(clause, b) else {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            return false;
        };
        let Some(e) = super::damage_removal_foreach::multiply(e, count) else {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            return false;
        };
        append(prev, e);
        return true;
    }
    let Some((text, mut did)) = rewrite_count(&l, verb, &chosen) else {
        return false;
    };
    // "each of those Vampires": the objects the instruction affected.
    let text = match those_objects(&text, &chosen) {
        Some(t) => {
            did = true;
            t
        }
        None => text,
    };
    if !did {
        return false;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let prev_targets = b.targets.len();
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    let Some(e) = parse_sentence(&text, b) else {
        restore(b);
        return false;
    };
    let Some(e) = crate::oracle::patterns::r107_numbers::substitute_x(&e, &count) else {
        restore(b);
        return false;
    };
    // Targets the sentence added ("up to that many target creatures").
    for i in prev_targets..b.targets.len() {
        match crate::oracle::patterns::r107_numbers::substitute_x_in_target(&b.targets[i], &count)
        {
            Some(t) => b.targets[i] = t,
            None => {
                restore(b);
                return false;
            }
        }
    }
    append(prev, e);
    true
}

/// "put a +1/+1 counter on each of those Vampires" → "... on them", when the noun
/// describes what the instruction could choose.
fn those_objects(s: &str, chosen: &Filter) -> Option<String> {
    let (i, p) = ["each of those ", "those "]
        .iter()
        .find_map(|p| s.find(p).map(|i| (i, *p)))?;
    let rest = &s[i + p.len()..];
    let (w, after) = split_word(rest);
    let (nf, plural, tail) = parse_object_phrase(w)?;
    if !plural || !tail.trim().is_empty() || !noun_covers(&nf, chosen) {
        return None;
    }
    Some(format!("{}them {}", &s[..i], after).trim_end().to_string())
}

/// The exiled objects returned later.
const RETURNED: Var = vars::USER + 3313;

/// "Return those cards to the battlefield [under their owner's control] at the beginning
/// of the next end step / your next upkeep": a delayed triggered ability that returns the
/// cards the instruction exiled (CR 603.7, 603.7c).
fn return_exiled_later(l: &str) -> Option<Effect> {
    let r = l
        .strip_prefix("return those cards ")
        .or_else(|| l.strip_prefix("return them "))?;
    let (trigger, dest) = super::triggers_delayed::split_delay(r)?;
    let to = super::damage_removal::battlefield_destination(dest, Sel::Var(RETURNED))?;
    Some(Effect::seq(vec![
        Effect::Store {
            var: RETURNED,
            sel: Sel::Var(DONE),
        },
        Effect::DelayedTrigger {
            trigger,
            body: Box::new(Body::effect(Effect::Move {
                what: Sel::Var(RETURNED),
                to,
            })),
            once: true,
        },
    ]))
}

fn append(prev: &mut Effect, e: Effect) {
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
}

inventory::submit! { FollowupPattern { name: "basic effects: about the chosen objects", priority: 45, apply: f_about_chosen } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::costs::parse_cost;

    #[test]
    fn optional_action_costs() {
        for s in [
            "tap two untapped creatures you control",
            "tap an untapped artifact you control",
            "tap another untapped citizen you control",
            "tap two other untapped creatures you control",
            "tap two untapped creatures and/or treasures you control",
            "tap five untapped artifacts and/or creatures you control",
            "tap an untapped non-human creature you control",
            "exile a human you control and an artifact you control",
            "exile a nontoken creature you control",
            "sacrifice another creature or an artifact",
        ] {
            println!("{s} => {:?}", parse_cost(s));
        }
    }
}

/// "sacrifice another creature or an artifact" (after "you may"): one of the objects the
/// cost grammar describes (alternatives the core object phrase doesn't read).
fn sacrifice_described(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("sacrifice ")?;
    if r.contains("target") || r.starts_with('~') {
        return None;
    }
    let (cost, _) = crate::oracle::costs::parse_cost(l)?;
    match cost.parts.as_slice() {
        [CostPart::Sacrifice { filter, count }] if cost.mana.is_none() => Some(Effect::Sacrifice {
            who: PlayerRef::You,
            filter: filter.clone(),
            count: count.clone(),
        }),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "basic effects: sacrifice [described object]", priority: 46, parse: sacrifice_described } }

/// "sacrifice each other creature you control", "sacrifice all Dragons you control",
/// "sacrifice half the non-Demon permanents you control, rounded up", "enchanted
/// permanent's controller sacrifices it", "target artifact creature's controller
/// sacrifices it" (CR 701.21a: a player sacrifices only their own permanents).
fn sacrifice_group(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(subject) = l.strip_suffix("'s controller sacrifices it") {
        let (what, tail) = crate::oracle::effects::object_ref(subject, b)?;
        if !tail.trim().is_empty() || matches!(what, Sel::All(_) | Sel::Players(_)) {
            return None;
        }
        // "That player may search ...": the controller.
        b.it_player = PlayerRef::ControllerOf(Box::new(what.clone()));
        return Some(Effect::SacrificeObjects { what });
    }
    let r = l.strip_prefix("sacrifice ")?;
    if let Some(x) = r.strip_prefix("half the ") {
        let x = x.strip_suffix(", rounded up")?;
        let (f, plural, tail) = parse_object_phrase(x)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        let f = own(Verb::Sacrifice, f);
        return Some(Effect::Sacrifice {
            who: PlayerRef::You,
            filter: f.clone(),
            count: Value::Div(Box::new(Value::Count(f)), 2, true),
        });
    }
    let x = r.strip_prefix("each ").or_else(|| r.strip_prefix("all "))?;
    let (f, _, tail) = parse_object_phrase(x)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::SacrificeObjects {
        what: Sel::All(own(Verb::Sacrifice, f)),
    })
}

inventory::submit! { EffectPattern { name: "basic effects: sacrifice a group", priority: 60, parse: sacrifice_group } }

/// "you untap all lands you control" (Sword of Feast and Famine), "you sacrifice a land"
/// (Redcap Melee): the controller performs the instruction.
fn you_untap(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you ")?;
    if !(r.starts_with("untap ") || r.starts_with("tap ") || r.starts_with("sacrifice ")) {
        return None;
    }
    crate::oracle::effects::parse_clause(r, b)
}

inventory::submit! { EffectPattern { name: "basic effects: you untap/tap ...", priority: 60, parse: you_untap } }

/// "If that player does, they lose 2 life." after an instruction a player might not be
/// able to follow ("target opponent sacrifices a green or white creature of their
/// choice"): whether they did (CR 118.12, 608.2c).
fn if_that_player_does(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = ["if that player does, ", "if the player does, ", "if they do, "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    else {
        return false;
    };
    // An instruction whose doing is recorded, performed by another player.
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        e => Some(e),
    };
    let recorded = matches!(last, Some(Effect::Sacrifice { who, .. }) if !matches!(who, PlayerRef::You));
    if !recorded || super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
        return false;
    }
    // "they" is that player.
    let r = match r.strip_prefix("they ") {
        Some(x) => format!("that player {x}"),
        None => r.to_string(),
    };
    let Some(e) = crate::oracle::effects::parse_clause(&r, b) else {
        return false;
    };
    append(
        prev,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    );
    true
}

inventory::submit! { FollowupPattern { name: "basic effects: if that player does", priority: 60, apply: if_that_player_does } }
