//! Choice grammar VI (cards5-choices-players): objects chosen by other players.
//!
//! - "You choose target creature an opponent controls, and that opponent chooses target
//!   creature." (Mogg Assassin): two targets, the second chosen by the controller of the
//!   first (CR 601.2c, 115.1); "the creature you chose" and "the creature your opponent
//!   chose" are the two targets afterward.
//! - "The owner of each creature card revealed this way with the lowest mana value puts
//!   it onto the battlefield." (Stronghold Gambit, after "Each player chooses a card in
//!   their hand. Then each player reveals their chosen card."): the revealed cards of
//!   that kind with the least mana value among them (tied or not) enter under their
//!   owners' control (CR 110.2a); none if no such card was revealed.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::*;

/// "you choose target [object], and that opponent chooses target [object]".
fn you_and_that_opponent_choose_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("you choose target ")
        .or_else(|| l.strip_prefix("choose target "))?;
    let (first, second) = r.split_once(", and that opponent chooses target ")?;
    // The first target is another player's object: "that opponent" is its controller.
    if !first.ends_with(" an opponent controls") {
        return None;
    }
    let t1 = format!("target {first}");
    let (s1, tail) = parse_target(&t1)?;
    if !end(tail).is_empty() || !matches!(s1.what, TargetKind::Object(_)) {
        return None;
    }
    let t2 = format!("target {second}");
    let (mut s2, tail) = parse_target(&t2)?;
    if !end(tail).is_empty() || !matches!(s2.what, TargetKind::Object(_)) {
        return None;
    }
    let first_slot = b.add_target(s1, &t1);
    s2.chosen_by = Some(PlayerRef::ControllerOf(Box::new(Sel::Target(first_slot))));
    let second_slot = b.add_target(s2, &t2);
    let noun = |t: &str| t.split_whitespace().nth(1).unwrap_or("permanent").to_string();
    let (n1, n2) = (noun(&t1), noun(&t2));
    b.named
        .push((format!("the {n1} you chose"), Sel::Target(first_slot)));
    b.named.push((
        format!("the {n2} your opponent chose"),
        Sel::Target(second_slot),
    ));
    b.named.push((
        format!("the {n2} that opponent chose"),
        Sel::Target(second_slot),
    ));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "you choose target [object], and that opponent chooses target [object]", priority: 90, parse: you_and_that_opponent_choose_targets } }

/// "the owner of each [card] revealed this way [with the lowest/greatest mana value] puts
/// it onto the battlefield", after cards were chosen and revealed.
fn owners_put_revealed(l: &str, b: &mut Builder) -> Option<Effect> {
    use super::filters_relational::{stat_word, tested};
    let l = end(l);
    let r = l
        .strip_prefix("the owner of each ")?
        .strip_suffix(" puts it onto the battlefield")?;
    let (noun, quality) = r.split_once(" revealed this way")?;
    let revealed = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == "the chosen card")
        .map(|(_, s)| s.clone())?;
    let (f, _, tail) = parse_object_phrase(noun)?;
    if !end(tail).is_empty() {
        return None;
    }
    let group = Sel::Matching(Box::new(revealed), f);
    let what = match quality.trim() {
        "" => group,
        q => {
            let q = q.strip_prefix("with the ")?;
            let (greatest, q) = if let Some(x) = q.strip_prefix("lowest ") {
                (false, x)
            } else {
                (true, q.strip_prefix("greatest ")?)
            };
            let (stat, rest) = stat_word(q)?;
            if !rest.trim().is_empty() {
                return None;
            }
            let ext = Value::Extreme(Box::new(tested(stat)), Box::new(group.clone()), greatest);
            let cmp = if greatest { Cmp::Ge } else { Cmp::Le };
            Sel::Matching(
                Box::new(group),
                Filter::ValueCmp(Box::new(tested(stat)), cmp, Box::new(ext)),
            )
        }
    };
    // Each card under its own owner's control.
    let mut to = Destination::battlefield();
    to.controller = Some(PlayerRef::OwnerOf(Box::new(what.clone())));
    Some(Effect::Move { what, to })
}

inventory::submit! { EffectPattern { name: "the owner of each [card] revealed this way [with the lowest mana value] puts it onto the battlefield", priority: 90, parse: owners_put_revealed } }

/// "Then each player reveals their chosen card." (Stronghold Gambit): each player reveals
/// the card they chose (read for each player as "reveal your chosen card").
fn reveal_their_chosen_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("reveal ")
        .or_else(|| l.strip_prefix("that player reveals "))?;
    let word = r
        .strip_prefix("your chosen ")
        .or_else(|| r.strip_prefix("their chosen "))?;
    if word.contains(' ') {
        return None;
    }
    let theirs = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| *n == format!("their chosen {word}"))
        .map(|(_, s)| s.clone())?;
    crate::kw::zones_dig_effects::reveal_effect(&theirs)
}

inventory::submit! { EffectPattern { name: "each player reveals their chosen card", priority: 90, parse: reveal_their_chosen_card } }

/// "Whenever an artifact, creature, or enchantment enters, its controller chooses target
/// permanent another player controls that shares a card type with it. Exchange control of
/// those permanents." (Confusion in the Ranks): the target and the permanent that entered
/// (CR 701.12a).
fn exchange_those_permanents(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "exchange control of those permanents" || !b.in_trigger {
        return None;
    }
    let (slot, t) = b.targets.iter().enumerate().next_back()?;
    let trigger_controller = PlayerRef::ControllerOf(Box::new(Sel::TriggerObject));
    if !matches!(&t.chosen_by, Some(c) if format!("{c:?}") == format!("{trigger_controller:?}"))
        || !matches!(t.what, TargetKind::Object(_))
    {
        return None;
    }
    Some(Effect::ExchangeControl {
        a: Sel::Target(slot as u8),
        b: Sel::TriggerObject,
    })
}

inventory::submit! { EffectPattern { name: "exchange control of those permanents (the target and the triggering permanent)", priority: 90, parse: exchange_those_permanents } }

/// "you may choose new targets for any number of other spells and/or abilities"
/// (Boltbender): the controller chooses any number of the other spells and abilities on
/// the stack (not targeted) and may choose new targets for each (CR 115.7d); a new target
/// must be legal.
fn new_targets_for_any_number(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let what = l
        .strip_prefix("you may choose new targets for any number of other ")
        .or_else(|| l.strip_prefix("choose new targets for any number of other "))?;
    if !matches!(
        what,
        "spells and/or abilities" | "spells and abilities" | "spells or abilities"
    ) {
        return None;
    }
    let _ = b;
    // Every spell and ability on the stack is an object there (CR 113.1, 405.1).
    let filter = Filter::and(vec![
        Filter::InZone(ZoneKind::Stack),
        Filter::Custom(crate::kw::choices_5_filters::NOT_RESOLVING.into()),
    ]);
    Some(Effect::ChangeTargets {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: filter.clone(),
            count: Value::CountSel(Box::new(Sel::All(filter))),
            up_to: true,
            store: None,
        },
        who: PlayerRef::You,
        how: TargetChange::ChooseNew,
        to: None,
    })
}

inventory::submit! { EffectPattern { name: "choose new targets for any number of other spells and/or abilities", priority: 90, parse: new_targets_for_any_number } }

/// "You choose the name of a nonland card revealed this way." after "each opponent
/// reveals their hand" (Alhammarret, High Arbiter): a name among those cards.
fn name_of_revealed_card(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(noun) = l
        .strip_prefix("you choose the name of a ")
        .or_else(|| l.strip_prefix("choose the name of a "))
        .and_then(|r| r.strip_suffix(" revealed this way"))
    else {
        return false;
    };
    // The hands the previous instruction revealed: each opponent's.
    let revealed_hands = |e: &Effect| match e {
        Effect::ForEachPlayer { who, effect } => {
            matches!(who, PlayerRef::EachOpponent)
                && matches!(effect.as_ref(), Effect::AsPlayer { effect, .. }
                    if matches!(effect.as_ref(), Effect::RevealHand { who: PlayerRef::You }))
        }
        _ => false,
    };
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        e => Some(e),
    };
    if !last.is_some_and(revealed_hands) {
        return false;
    }
    let Some((f, false, tail)) = parse_object_phrase(noun) else {
        return false;
    };
    if !end(tail).is_empty() || f.zone().is_some() {
        return false;
    }
    let cards = Sel::All(Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Hand),
        Filter::OwnedBy(PlayerRel::Opponent),
    ]));
    let Some(choose) = crate::kw::choices_5_filters::name_among(&cards) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, choose]);
    true
}

inventory::submit! { super::FollowupPattern { name: "you choose the name of a [card] revealed this way (each opponent's hand)", priority: 60, apply: name_of_revealed_card } }

/// "As ~ enters, each opponent reveals their hand. You choose the name of a nonland card
/// revealed this way." (Alhammarret, High Arbiter): the reveal and the choice happen as
/// the permanent enters (CR 614.1c, 614.12a).
fn as_enters_name_of_revealed(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let lower = block.to_lowercase();
    let body = end(&lower).strip_prefix("as ~ enters, ")?;
    if !body.contains(" revealed this way") {
        return None;
    }
    let mut b = Builder::new(ctx);
    let e = crate::oracle::effects::parse_effect_text(body, &mut b)?;
    let names = serde_json::to_string(&e).ok()?;
    if !b.targets.is_empty() || !names.contains(crate::kw::choices_5_filters::NAME_AMONG) {
        return None;
    }
    let effect = StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(e)),
        self_replacement: false,
        optional: false,
    });
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        block,
    )])
}

inventory::submit! { super::AbilityPattern { name: "as ~ enters, [reveal]; you choose the name of a card revealed this way", priority: 40, parse: as_enters_name_of_revealed } }

/// "[cards] you discarded this way" when you are the one who discarded (Queen Kayla
/// bin-Kroog: "Discard all the cards in your hand, then draw that many cards. You may
/// choose an artifact or creature card with mana value 1 you discarded this way, ..."):
/// the cards discarded this way.
fn you_discarded_this_way(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" you discarded this way") {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    // Only your own discard: an imperative "discard ..." instruction, and no other
    // player's.
    let yours = raw.starts_with("discard ")
        || [": discard ", ". discard ", ", discard ", "you discard "]
            .iter()
            .any(|p| raw.contains(p));
    if !yours || raw.contains(" discards ") {
        return None;
    }
    let e = parse_sentence(&l.replace(" you discarded this way", " discarded this way"), b)?;
    let mut j = serde_json::to_value(&e).ok()?;
    // "You may choose ..., then do the same for ...": each choice is optional on its own
    // (Queen Kayla bin-Kroog's ruling), within the "may" around them all.
    if raw.contains(&format!("you may {l}")) {
        let mut n = 0;
        optional_choices(&mut j, &mut n);
        if n == 0 {
            return None;
        }
    }
    // A card discarded this way may have gone to exile instead (a replacement effect):
    // it's found wherever it is.
    anywhere_discarded(&mut j);
    serde_json::from_value(j).ok()
}

/// Makes each single choice of objects (`Sel::Choose`) in `v` "up to one".
fn optional_choices(v: &mut serde_json::Value, n: &mut usize) {
    match v {
        serde_json::Value::Object(m) => {
            if let Some(c) = m.get_mut("Choose") {
                if c.get("chooser").is_some() && c.get("count") == Some(&serde_json::json!({"Const": 1})) {
                    c["up_to"] = serde_json::Value::Bool(true);
                    *n += 1;
                }
            }
            for x in m.values_mut() {
                optional_choices(x, n);
            }
        }
        serde_json::Value::Array(a) => a.iter_mut().for_each(|x| optional_choices(x, n)),
        _ => {}
    }
}

/// Drops the graveyard from filters of cards discarded this way.
fn anywhere_discarded(v: &mut serde_json::Value) {
    let discarded = serde_json::json!({"In": {"Var": crate::discard_rules::DISCARDED}});
    let graveyard = serde_json::json!({"InZone": "Graveyard"});
    match v {
        serde_json::Value::Object(m) => {
            if let Some(serde_json::Value::Array(parts)) = m.get_mut("And") {
                if parts.contains(&discarded) {
                    parts.retain(|p| *p != graveyard);
                }
            }
            for x in m.values_mut() {
                anywhere_discarded(x);
            }
        }
        serde_json::Value::Array(a) => a.iter_mut().for_each(anywhere_discarded),
        _ => {}
    }
}

inventory::submit! { EffectPattern { name: "[cards] you discarded this way (your own discard)", priority: 200, parse: you_discarded_this_way } }

/// "for each land target player controls in excess of the number you control, choose a
/// land that player controls" (Equipoise): as many of that player's lands as they control
/// beyond the number you control (none if they don't control more), chosen by you; "the
/// chosen permanents" afterward. With "Repeat this process for artifacts and creatures",
/// each pass counts again (Equipoise's ruling).
fn choose_in_excess(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (clause, then) = match l.split_once(", then ") {
        Some((c, t)) => (c, Some(t)),
        None => (l, None),
    };
    let r = clause.strip_prefix("for each ")?;
    let (noun, r) = r.split_once(" target player controls in excess of the number you control, choose a ")?;
    let noun2 = r
        .strip_suffix(" that player controls")
        .or_else(|| r.strip_suffix(" they control"))?;
    if noun != noun2 || noun.contains(' ') {
        return None;
    }
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let first = b.targets.len();
    let slot = b.add_target(
        TargetSpec::player(PlayerFilter::Any, "target player"),
        "target player",
    );
    let theirs = Filter::and(vec![
        f.clone(),
        Filter::Permanent,
        Filter::ControlledByPlayer(Box::new(PlayerRef::Target(slot))),
    ]);
    let yours = Filter::and(vec![f, Filter::Permanent, Filter::ControlledBy(PlayerRel::You)]);
    let excess = Value::Max(
        Box::new(Value::c(0)),
        Box::new(Value::Diff(
            Box::new(Value::Count(theirs.clone())),
            Box::new(Value::Count(yours)),
        )),
    );
    let chosen = super::choice_grammar_objects::CHOSEN;
    let choose = Effect::Store {
        var: chosen,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: theirs,
            count: excess,
            up_to: false,
            store: None,
        },
    };
    for name in [
        "the chosen permanents".to_string(),
        format!("the chosen {noun}s"),
        format!("those {noun}s"),
    ] {
        b.named.push((name, Sel::Var(chosen)));
    }
    let Some(then) = then else {
        return Some(choose);
    };
    let Some(rest) = crate::oracle::effects::parse_clause(then, b) else {
        b.targets.truncate(first);
        return None;
    };
    Some(Effect::seq(vec![choose, rest]))
}

inventory::submit! { EffectPattern { name: "for each [permanent] target player controls in excess of the number you control, choose a [permanent] that player controls", priority: 90, parse: choose_in_excess } }

/// "Then if one or more of the chosen permanents are still on the battlefield, you search
/// your library for up to that many land cards, ..." (Disorienting Choice): the chosen
/// objects still on the battlefield (the same objects, CR 400.7), counted as the
/// instruction begins; with none, nothing happens (no search, no shuffle).
fn if_chosen_still_on_battlefield(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let rest = l.strip_prefix("if one or more of the chosen permanents are still on the battlefield, ")?;
    if !rest.contains("that many") {
        return None;
    }
    let chosen = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == "the chosen permanents")
        .map(|(_, s)| s.clone())?;
    let count = Value::CountSel(Box::new(Sel::Matching(
        Box::new(chosen),
        Filter::Custom(crate::kw::choices_5_filters::STILL_ON_BATTLEFIELD.into()),
    )));
    let then = parse_sentence(&rest.replacen("that many", "x", 1), b)?;
    if !serde_json::to_string(&then).is_ok_and(|j| j.contains("\"X\"")) {
        return None;
    }
    Some(Effect::If {
        cond: Condition::Compare(count.clone(), Cmp::Ge, Value::c(1)),
        then: Box::new(Effect::seq(vec![Effect::SetX { value: count }, then])),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if one or more of the chosen permanents are still on the battlefield, [do something that many times]", priority: 90, parse: if_chosen_still_on_battlefield } }

/// The chosen piles of all the players a "for each defending player, separate ... and
/// that player chooses one" was performed for.
const CHOSEN_PILES: Var = vars::USER + 9541;

/// "for each defending player, separate all creatures that player controls into two piles
/// and that player chooses one" (Stand or Fall): you separate each defending player's
/// creatures (CR 700.3), and that player chooses one of the two piles; "the chosen piles"
/// are all those chosen.
fn for_each_defending_player_piles(l: &str, b: &mut Builder) -> Option<Effect> {
    use crate::piles::PileAction;
    let l = end(l);
    let what = l
        .strip_prefix("for each defending player, separate all ")?
        .strip_suffix(" that player controls into two piles and that player chooses one")?;
    let (f, true, tail) = parse_object_phrase(what)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let each = Effect::seq(vec![
        Effect::Piles(Box::new(PileAction::Separate {
            what: Sel::All(Filter::and(vec![
                f,
                Filter::ControlledBy(PlayerRel::Iterated),
            ])),
            separator: PlayerRef::You,
            count: 0,
            face_down: false,
            own: false,
        })),
        Effect::Piles(Box::new(PileAction::Choose {
            chooser: PlayerRef::Iterated,
            picked_by: None,
            of: None,
            at_random: false,
        })),
        Effect::Store {
            var: CHOSEN_PILES,
            sel: Sel::Union(vec![
                Sel::Var(CHOSEN_PILES),
                Sel::Var(crate::piles::CHOSEN),
            ]),
        },
    ]);
    b.named
        .push(("the chosen piles".into(), Sel::Var(CHOSEN_PILES)));
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN_PILES,
            sel: Sel::Union(vec![]),
        },
        Effect::ForEachPlayer {
            who: PlayerRef::Each(PlayerFilter::History(Box::new(Condition::Custom(
                crate::kw::choices_5_filters::IS_DEFENDING_PLAYER.into(),
            )))),
            effect: Box::new(each),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "for each defending player, separate all [objects] that player controls into two piles and that player chooses one", priority: 90, parse: for_each_defending_player_piles } }

/// "Only creatures in the chosen piles can block this turn." (Stand or Fall): every other
/// creature can't block this turn; which creatures are in the piles is fixed as the
/// effect begins (a creature that enters later can't block either).
fn only_chosen_piles_can_block(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let noun = l
        .strip_prefix("only ")?
        .strip_suffix(" in the chosen piles can block this turn")?;
    let piles = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == "the chosen piles")
        .map(|(_, s)| s.clone())?;
    let (f, true, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::AddRestriction {
        restriction: Restriction::CantBlock(Filter::and(vec![
            f,
            Filter::not(Filter::In(Box::new(piles))),
        ])),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "only [objects] in the chosen piles can block this turn", priority: 90, parse: only_chosen_piles_can_block } }

/// "distribute X +1/+1 counters among it and any number of commanders" (Stumpsquall
/// Hydra): the source and any number of the other objects, chosen as the ability resolves
/// (not targeted); the source only while it's still on the battlefield (its ruling).
fn distribute_among_it_and_others(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, tail) = [" among it and any number of ", " among ~ and any number of "]
        .iter()
        .find_map(|p| l.split_once(p))?;
    if !head.starts_with("distribute ") {
        return None;
    }
    let e = parse_sentence(&format!("{head} among any number of {tail}"), b)?;
    let Effect::DistributeCounters {
        n,
        kind,
        among: Sel::All(f),
        min_objects,
        max_objects,
        up_to,
    } = e
    else {
        return None;
    };
    Some(Effect::DistributeCounters {
        n,
        kind,
        among: Sel::All(Filter::Or(vec![Filter::Source, f])),
        min_objects,
        max_objects,
        up_to,
    })
}

inventory::submit! { EffectPattern { name: "distribute counters among it and any number of [objects]", priority: 90, parse: distribute_among_it_and_others } }

/// "If that permanent or card has a time counter on it, ..." (Jhoira's Timebug, after
/// "Choose target permanent you control or suspended card you own"): the one target,
/// whichever it is.
fn that_permanent_or_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains("that permanent or card") {
        return None;
    }
    let [t] = b.targets.as_slice() else {
        return None;
    };
    if t.max.as_const() != Some(1) || !(t.text.contains("permanent") && t.text.contains(" or ") && t.text.contains("card")) {
        return None;
    }
    parse_sentence(&l.replace("that permanent or card", "it"), b)
}

inventory::submit! { EffectPattern { name: "that permanent or card (the target that's one or the other)", priority: 200, parse: that_permanent_or_card } }

/// "Choose up to seven face-up exiled cards you own." (Mirror of Fate): cards in exile,
/// chosen as the ability resolves (not targeted); face-down ones can't be (CR 406.3);
/// "the chosen cards" afterward.
fn choose_exiled_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose ")?;
    let (up_to, r) = match r.strip_prefix("up to ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let (n, r) = parse_number(r)?;
    let r = r.trim_start();
    let (face_up, r) = match r.strip_prefix("face-up ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let r = r.strip_prefix("exiled ")?;
    let noun = r.strip_suffix(" you own")?;
    let (f, _, tail) = parse_object_phrase(noun)?;
    if !end(tail).is_empty() || f.zone().is_some() {
        return None;
    }
    let mut parts = vec![
        f,
        Filter::Card,
        Filter::InZone(ZoneKind::Exile),
        Filter::OwnedBy(PlayerRel::You),
    ];
    if face_up {
        parts.push(Filter::not(Filter::FaceDown));
    }
    let chosen = super::choice_grammar_objects::CHOSEN;
    for name in ["the chosen cards", "the chosen card", "those cards"] {
        b.named.push((name.into(), Sel::Var(chosen)));
    }
    Some(Effect::Store {
        var: chosen,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(parts),
            count: n,
            up_to,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "choose [up to N] [face-up] exiled cards you own", priority: 90, parse: choose_exiled_cards } }

/// "You may cast the other cards without paying their mana costs." (Emergent Ultimatum,
/// after "An opponent chooses one of those cards. Shuffle that card into your library."):
/// the cards the opponent didn't choose.
fn cast_the_other_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (optional, r) = match l.strip_prefix("you may cast ") {
        Some(r) => (true, r),
        None => (false, l.strip_prefix("cast ")?),
    };
    let what = r.strip_suffix(" without paying their mana costs")?;
    let key = match what {
        "the other cards" | "the other card" => "the other",
        "the rest" => "the rest",
        _ => return None,
    };
    let sel = b
        .named
        .iter()
        .rev()
        .find(|(n, _)| n == key)
        .map(|(_, s)| s.clone())?;
    Some(Effect::CastCard {
        who: PlayerRef::You,
        what: sel,
        free: true,
        optional,
    })
}

inventory::submit! { EffectPattern { name: "you may cast the other cards without paying their mana costs", priority: 90, parse: cast_the_other_cards } }
