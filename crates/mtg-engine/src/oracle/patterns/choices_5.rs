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
