//! Card-flow grammar V (round 4): object phrases over several zones and their qualifiers.
//!
//! * "basic land cards which have the same name as the chosen land" (Canoptek Wraith): a
//!   "which have" relative clause reads as "with".
//! * "Exile target nonland card from a graveyard, all other cards from graveyards with the
//!   same name as that card, and all permanents with that name." (Moratorium Stone).
//! * "The owner of target spell, nonland permanent, or card in a graveyard puts it ..."
//!   (Endless Detour).
//! * Graveyard order (CR 404.2): "the top three black cards of your graveyard" (Spinning
//!   Darkness's alternative cost), "Target opponent chooses one of the top two cards of
//!   your graveyard. Exile that card and put the other one into your hand." (Phyrexian
//!   Grimoire); the filter is `kw/graveyard_top_n.rs`.

use super::{CostPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// "which have [quality]" / "which has [quality]": the same qualifier as "with [quality]";
/// the clause is read with "with" (by the effect parser, which knows what "the chosen
/// land" names).
fn which_have(l: &str, b: &mut Builder) -> Option<Effect> {
    if !l.contains(" which have ") && !l.contains(" which has ") {
        return None;
    }
    let l = l.replace(" which have ", " with ").replace(" which has ", " with ");
    parse_clause(&l, b)
}

inventory::submit! { EffectPattern { name: "card flow 5: [objects] which have [quality]", priority: 90, parse: which_have } }

/// The objects the first `Exile` of an instruction exiles.
fn exiled_sel(e: &mut Effect) -> Option<&mut Sel> {
    match e {
        Effect::Exile { what, .. } => Some(what),
        Effect::Seq(v) => v.first_mut().and_then(exiled_sel),
        _ => None,
    }
}

/// "Exile target nonland card from a graveyard, all other cards from graveyards with the
/// same name as that card, and all permanents with that name." (Moratorium Stone): a list
/// of three or more things exiled at once, the later ones read with the first item (a
/// target) as "that card"; "with that name" is the target's name (CR 201.2).
fn exile_list_after_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile ")?;
    let (front, last) = r.rsplit_once(", and ")?;
    let mut items: Vec<&str> = front.split(", ").collect();
    items.push(last);
    if items.len() < 3 || !items[0].starts_with("target ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
    };
    let Some(mut e) = parse_clause(&format!("exile {} and {}", items[0], items[1]), b) else {
        restore(b);
        return None;
    };
    if b.targets.len() != saved.0 + 1 || exiled_sel(&mut e).is_none() {
        restore(b);
        return None;
    }
    let target = Sel::Target(saved.0 as u8);
    let mut extra = Vec::new();
    for item in &items[2..] {
        let item = item.replace(" with that name", " with the same name as that card");
        b.it = target.clone();
        let n = b.targets.len();
        let Some(mut e2) = parse_clause(&format!("exile {item}"), b) else {
            restore(b);
            return None;
        };
        if b.targets.len() != n {
            restore(b);
            return None;
        }
        match &mut e2 {
            Effect::Exile { what, .. } => extra.push(what.clone()),
            _ => {
                restore(b);
                return None;
            }
        }
    }
    let w = exiled_sel(&mut e)?;
    let mut all = match std::mem::replace(w, Sel::This) {
        Sel::Union(v) => v,
        s => vec![s],
    };
    all.extend(extra);
    *w = Sel::Union(all);
    b.it = target;
    Some(e)
}

inventory::submit! { EffectPattern { name: "card flow 5: exile target X, Y, and Z", priority: 90, parse: exile_list_after_target } }

/// "The owner of target spell, nonland permanent, or card in a graveyard puts it on their
/// choice of the top or bottom of their library." (Endless Detour): a target of several
/// kinds in several zones (a spell, a permanent or a card in a graveyard, CR 115.1), then
/// "its owner ..." about it.
fn owner_of_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the owner of ")?;
    let (obj, rest) = r.split_once(" puts ")?;
    if !obj.starts_with("target ") || !obj.contains(", ") {
        return None;
    }
    let spec = target_list(obj)?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let slot = b.add_target(spec, obj);
    b.it = Sel::Target(slot);
    let e = parse_clause(&format!("its owner puts {rest}"), b);
    if e.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    e
}

/// "target spell, nonland permanent, or card in a graveyard": one target of any of the
/// kinds, each read as a target phrase of its own (the kinds don't share qualifiers).
fn target_list(obj: &str) -> Option<TargetSpec> {
    let r = obj.strip_prefix("target ")?;
    let (front, last) = r.rsplit_once(", or ")?;
    let mut items: Vec<&str> = front.split(", ").collect();
    items.push(last);
    if items.len() < 3 {
        return None;
    }
    let mut kinds = Vec::new();
    for item in items {
        let text = format!("target {item}");
        let (spec, tail) = crate::oracle::phrases::parse_target(&text)?;
        if !tail.trim().is_empty() || spec.max.as_const() != Some(1) {
            return None;
        }
        kinds.push(match spec.what {
            TargetKind::Object(f) => f,
            TargetKind::Spell(Filter::Spell) => Filter::Spell,
            TargetKind::Spell(f) => Filter::and(vec![Filter::Spell, f]),
            _ => return None,
        });
    }
    let (mut spec, _) = crate::oracle::phrases::parse_target("target permanent")?;
    spec.what = TargetKind::Object(Filter::Or(kinds));
    spec.text = obj.to_string();
    Some(spec)
}

inventory::submit! { EffectPattern { name: "card flow 5: the owner of target X, Y, or Z puts it ...", priority: 90, parse: owner_of_target } }

/// "the top N [quality] cards of your graveyard" (CR 404.2), N of two or more: the count,
/// the cards' filter and the rest.
fn top_cards_of_your_graveyard(s: &str) -> Option<(u32, Filter, &str)> {
    let r = s.strip_prefix("the top ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let n = u32::try_from(n.as_const()?).ok().filter(|n| *n >= 2)?;
    let r = r.trim_start();
    let (word, r) = match r.strip_prefix("cards of ") {
        Some(r) => ("", r),
        None => {
            let (w, r) = r.split_once(' ')?;
            (w, r.strip_prefix("cards of ")?)
        }
    };
    let rest = r.strip_prefix("your graveyard")?;
    let f = crate::kw::graveyard_top_n::top_cards_of_graveyard(n, word, PlayerRel::You)?;
    Some((n, f, rest))
}

/// "Exile the top three black cards of your graveyard" as a cost (Spinning Darkness).
fn exile_top_cards_cost(p: &str) -> Option<CostPart> {
    let r = crate::oracle::phrases::strip(p, "exile")?;
    let (n, filter, rest) = top_cards_of_your_graveyard(r.trim_start())?;
    if !end(rest).is_empty() {
        return None;
    }
    Some(CostPart::Exile {
        filter,
        zone: ZoneKind::Graveyard,
        count: Value::c(n as i32),
    })
}

inventory::submit! { CostPattern { name: "card flow 5: exile the top N [quality] cards of your graveyard", priority: 50, parse: exile_top_cards_cost } }

/// The top cards of a graveyard a player chooses among, and the chosen one.
pub const TOP_CARDS: Var = vars::USER + 52000;
pub const TOP_CHOSEN: Var = vars::USER + 52001;

/// "Target opponent chooses one of the top two cards of your graveyard." (Phyrexian
/// Grimoire): the cards are fixed as the instruction begins; "that card" is the chosen
/// one, "the other one" the rest of them (as the next sentence says what happens to each).
fn chooses_one_of_top_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, spec, r) = crate::oracle::phrases::parse_player(l)?;
    let r = r.strip_prefix("chooses one of ")?;
    let (n, filter, rest) = top_cards_of_your_graveyard(r)?;
    if !rest.is_empty() || n != 2 {
        return None;
    }
    let who = match spec {
        Some(spec) => {
            if !matches!(who, PlayerRef::Target(0)) {
                return None;
            }
            let text = spec.text.clone();
            PlayerRef::Target(b.add_target(spec, &text))
        }
        None => who,
    };
    b.it = Sel::Var(TOP_CHOSEN);
    b.it_player = who.clone();
    b.named.push(("that card".into(), Sel::Var(TOP_CHOSEN)));
    b.named.push((
        "the other one".into(),
        Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(TOP_CARDS))),
            Filter::not(Filter::In(Box::new(Sel::Var(TOP_CHOSEN)))),
        ])),
    ));
    Some(Effect::seq(vec![
        Effect::Store {
            var: TOP_CARDS,
            sel: Sel::All(filter),
        },
        Effect::Store {
            var: TOP_CHOSEN,
            sel: Sel::Choose {
                chooser: who,
                filter: Filter::In(Box::new(Sel::Var(TOP_CARDS))),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
    ]))
}

inventory::submit! { EffectPattern { name: "card flow 5: [player] chooses one of the top two cards of your graveyard", priority: 90, parse: chooses_one_of_top_cards } }

/// "The controller of target artifact sacrifices it, ..." (Shape Anew): read as "target
/// artifact's controller sacrifices it, ...".
fn the_controller_of_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the controller of ")?;
    if !r.starts_with("target ") {
        return None;
    }
    let (_, rest) = crate::oracle::phrases::parse_target(r)?;
    let phrase = &r[..r.len() - rest.len()];
    if rest.is_empty() {
        return None;
    }
    parse_clause(&format!("{}'s controller {rest}", phrase.trim_end()), b)
}

inventory::submit! { EffectPattern { name: "card flow 5: the controller of target X [does something]", priority: 990, parse: the_controller_of_target } }

/// A third-person verb ("sacrifices", "reveals", "puts", "shuffles").
fn third_person_verb(w: &str) -> bool {
    w.len() > 3 && w.ends_with('s') && !matches!(w, "its" | "this" | "his" | "us")
}

/// "[player] sacrifices it, then reveals cards ..." (Shape Anew): the player does both;
/// the second instruction is read as "that player reveals cards ...".
fn player_then_verb(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first, second) = l.split_once(", then ")?;
    let verb = second.split(' ').next()?;
    if !third_person_verb(verb) || first.starts_with("that player ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
    };
    let Some(e1) = parse_clause(first, b) else {
        restore(b);
        return None;
    };
    // The first instruction's subject is the player the second is about.
    let subject_is_player = !matches!(b.it_player, PlayerRef::You)
        && !super::oracle_hardening_referents::is_no_player_referent(&b.it_player);
    if !subject_is_player {
        restore(b);
        return None;
    }
    let Some(e2) = parse_clause(&format!("that player {second}"), b) else {
        restore(b);
        return None;
    };
    Some(Effect::seq(vec![e1, e2]))
}

inventory::submit! { EffectPattern { name: "card flow 5: [player] does X, then does Y", priority: 990, parse: player_then_verb } }

/// "[player] reveals cards from the top of their library until they reveal [a card],
/// puts that card onto the battlefield, then shuffles" (Chaotic Transformation, in a "for
/// each permanent exiled this way" loop: its controller), "its controller reveals cards
/// ... until an artifact or creature card is revealed and exiles that card" (Indomitable
/// Creativity), "they reveal cards ... until they reveal ..., put that card onto the
/// battlefield, then shuffle" (Reweave, after "If the player does,"): the player performs
/// the whole reveal-until instruction, as "you" would (CR 701.20a, 608.2c).
fn player_reveals_until(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (who, rest) = if let Some(r) =
        l.strip_prefix("they reveal cards from the top of their library until ")
    {
        (b.it_player.clone(), r)
    } else {
        let (subj, r) = l.split_once(" reveals cards from the top of their library until ")?;
        if subj == "you" || subj.starts_with("each ") || subj.contains(" and ") {
            return None;
        }
        let (who, tail) = crate::oracle::effects::player_ref(subj, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        (who, r)
    };
    if matches!(who, PlayerRef::You)
        || super::oracle_hardening_referents::is_no_player_referent(&who)
    {
        return None;
    }
    // "they reveal [a card]", "[a card] is revealed": what's looked for.
    let rest = if let Some(r) = rest.strip_prefix("they reveal ") {
        r.to_string()
    } else {
        let (desc, tail) = rest.split_once(" is revealed")?;
        format!("{desc}{tail}")
    };
    // Only a whole instruction chain ("..., puts that card onto the battlefield, then
    // shuffles", "... and exiles that card"): a bare reveal is the player subject
    // grammar's, which the next sentences ("That player puts that card ...") continue.
    if ![", puts ", " and puts ", " and exiles ", ", put ", ", then "]
        .iter()
        .any(|p| rest.contains(p))
    {
        return None;
    }
    let mut mine = format!("reveal cards from the top of your library until you reveal {rest}");
    for (third, base) in [
        (", puts ", ", put "),
        (" and puts ", " and put "),
        (", then puts ", ", then put "),
        (", then shuffles", ", then shuffle"),
        (" and exiles ", " and exile "),
        (" their library", " your library"),
    ] {
        mine = mine.replace(third, base);
    }
    if mine.contains(" they ") || mine.contains(" their ") {
        return None;
    }
    let saved = (b.targets.len(), b.it_player.clone());
    let Some(e) = parse_clause(&mine, b) else {
        b.targets.truncate(saved.0);
        b.it_player = saved.1;
        return None;
    };
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "card flow 5: [player] reveals cards until [a card], puts that card ...", priority: 20, parse: player_reveals_until } }

/// The cards a "for each" loop's players exiled, for a later group step.
pub const LOOP_EXILED: Var = vars::USER + 52002;

/// Whether an effect exiles something (its last instruction, looking into a player's
/// performing it).
fn ends_exiling(e: &Effect) -> bool {
    match e {
        Effect::Exile { .. } => true,
        Effect::AsPlayer { effect, .. } => ends_exiling(effect),
        Effect::Seq(v) => v.last().is_some_and(ends_exiling),
        _ => false,
    }
}

/// "Those players put the exiled cards onto the battlefield, then shuffle." after "For each
/// permanent destroyed this way, its controller reveals cards ... and exiles that card."
/// (Indomitable Creativity): all the cards exiled in the loop enter at the same time, each
/// under the control of the player who exiled it (its owner: from their own library), then
/// each of those players shuffles (CR 101.4).
fn those_players_put_the_exiled_cards(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "those players put the exiled cards onto the battlefield, then shuffle" {
        return false;
    }
    let last = match prev {
        Effect::Seq(v) => match v.last_mut() {
            Some(x) => x,
            None => return false,
        },
        e => e,
    };
    let Effect::ForEach { sel, var, effect } = last else {
        return false;
    };
    if !ends_exiling(effect) {
        return false;
    }
    let (sel, var) = (sel.clone(), *var);
    let body = std::mem::replace(effect.as_mut(), Effect::Noop);
    **effect = Effect::seq(vec![
        body,
        Effect::Store {
            var: LOOP_EXILED,
            sel: Sel::Union(vec![Sel::Var(LOOP_EXILED), Sel::Var(vars::IT)]),
        },
    ]);
    let looped = std::mem::replace(last, Effect::Noop);
    let mut to = Destination::battlefield();
    to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(LOOP_EXILED))));
    *last = Effect::seq(vec![
        Effect::Store {
            var: LOOP_EXILED,
            sel: Sel::Union(vec![]),
        },
        looped,
        Effect::Move {
            what: Sel::All(Filter::and(vec![
                Filter::In(Box::new(Sel::Var(LOOP_EXILED))),
                Filter::InZone(ZoneKind::Exile),
            ])),
            to,
        },
        Effect::ForEach {
            sel,
            var,
            effect: Box::new(Effect::Shuffle {
                who: PlayerRef::ControllerOf(Box::new(Sel::Var(var))),
            }),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "card flow 5: those players put the exiled cards onto the battlefield, then shuffle", priority: 90, apply: those_players_put_the_exiled_cards } }

/// "Choose any number of target players or planeswalkers. For each of them, reveal cards
/// from the top of your library until you reveal a nonland card, ~ deals damage equal to
/// that card's mana value to that player or planeswalker, then you put the revealed cards
/// on the bottom of your library in any order." (Kaboom!): for each target in turn, a
/// reveal-until, damage from the card found, and the revealed cards put on the bottom
/// (CR 701.20a, 608.2c).
fn for_each_target_reveal_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each of them, ")?;
    let (reveal, rest) = r.split_once(", ~ deals damage equal to that card's mana value to that ")?;
    let (_noun, then) = rest.split_once(", then ")?;
    if !reveal.starts_with("reveal cards from the top of your library until you reveal ") {
        return None;
    }
    let slot = b.targets.len().checked_sub(1)?;
    if !matches!(
        b.targets[slot].what,
        TargetKind::ObjectOrPlayer(..) | TargetKind::Player(_) | TargetKind::Object(_)
    ) || b.targets[slot].max.as_const().is_some_and(|m| m < 2)
    {
        return None;
    }
    let then = then.strip_prefix("you ").unwrap_or(then);
    if then != "put the revealed cards on the bottom of your library in any order" {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let e1 = parse_clause(reveal, b);
    // All the cards revealed, the one found (`vars::IT`) included.
    let e3 = Effect::Move {
        what: Sel::All(Filter::and(vec![
            Filter::Or(vec![
                Filter::In(Box::new(Sel::Var(vars::DUG))),
                Filter::In(Box::new(Sel::Var(vars::IT))),
                Filter::In(Box::new(Sel::Var(vars::REVEALED))),
            ]),
            Filter::InZone(ZoneKind::Library),
        ])),
        to: Destination::library_bottom(),
    };
    let Some(e1) = e1 else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    if b.targets.len() != saved.0 || !matches!(e1, Effect::RevealUntil { .. }) {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    let each = super::iteration_grammar::EACH;
    Some(Effect::ForEach {
        sel: Sel::Target(slot as u8),
        var: each,
        // One target at a time, the whole process for each (Kaboom!'s ruling: "those
        // targets will be handled one at a time"): one instruction, not three performed
        // for all of them together (`simultaneous.rs`).
        effect: Box::new(Effect::If {
            cond: Condition::Always,
            then: Box::new(Effect::seq(vec![
                e1,
                Effect::DealDamage {
                    source: Sel::This,
                    amount: Value::ManaValueOf(Box::new(Sel::Var(vars::IT))),
                    to: Sel::Var(each),
                },
                e3,
            ])),
            otherwise: Box::new(Effect::Noop),
        }),
    })
}

inventory::submit! { EffectPattern { name: "card flow 5: for each of them, reveal until, damage equal to that card's mana value, then put the revealed cards", priority: 90, parse: for_each_target_reveal_damage } }
