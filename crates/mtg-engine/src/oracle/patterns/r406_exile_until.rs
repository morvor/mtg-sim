//! Exiling cards from the top of a library until a card with some quality is exiled:
//! "Exile cards from the top of your library until you exile a nonland card.", "Target
//! opponent exiles cards from the top of their library until they exile a nonland card."
//! Every card is exiled (CR 406); "that card" is the last one (`vars::IT`). Followed by
//! what may be done with it: "You may cast that card without paying its mana cost.",
//! "Until end of turn, you may cast that card without paying its mana cost." (Stolen
//! Goods, Nicol Bolas, God-Pharaoh): a permission its controller may use this turn.

use super::card_flow_search::card_filter;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn exile_until(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, desc) = if let Some(r) =
        l.strip_prefix("exile cards from the top of your library until you exile ")
    {
        (PlayerRef::You, r)
    } else if let Some(r) = l.strip_prefix(
        "target opponent exiles cards from the top of their library until they exile ",
    ) {
        let slot = b.add_target(
            TargetSpec::player(PlayerFilter::Opponent, "target opponent"),
            "target opponent",
        );
        b.it_player = PlayerRef::Target(slot);
        (PlayerRef::Target(slot), r)
    } else {
        return None;
    };
    let desc = desc.strip_prefix("a ").or_else(|| desc.strip_prefix("an "))?;
    // "with lesser mana value" in a "whenever you cast a [kind of] spell" trigger (Jodah,
    // the Unifier): less than that spell's mana value (as it last existed, with its X).
    let (desc, lesser) = match desc.strip_suffix(" with lesser mana value") {
        Some(d) if b.in_trigger && matches!(b.it, Sel::TriggerSpell) => (
            d,
            Some(Filter::ManaValue(
                Cmp::Lt,
                Box::new(Value::ManaValueOf(Box::new(Sel::TriggerSpell))),
            )),
        ),
        Some(_) => return None,
        None => (desc, None),
    };
    let mut filter = card_filter(desc, b)?;
    if let Some(l) = lesser {
        filter = Filter::and(vec![filter, l]);
    }
    b.it = Sel::Var(vars::IT);
    b.named.push((
        THE_REST.into(),
        Sel::All(Filter::and(vec![
            Filter::In(Box::new(Sel::Var(vars::REVEALED))),
            Filter::InZone(ZoneKind::Exile),
        ])),
    ));
    let exile = Destination::zone(ZoneKind::Exile);
    Some(Effect::RevealUntil {
        who,
        filter: Filter::And(vec![Filter::Card, filter]),
        found_to: exile.clone(),
        rest_to: exile,
    })
}

inventory::submit! { EffectPattern { name: "r406 exile cards until you exile a [card]", priority: 90, parse: exile_until } }

/// "Until end of turn, you may cast that card without paying its mana cost." after
/// exiling it: its controller may cast it this turn (CR 601.2, 118.9).
fn until_end_of_turn_cast_it_free(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l != "until end of turn, you may cast that card without paying its mana cost" {
        return None;
    }
    if !matches!(b.it, Sel::Var(vars::IT)) {
        return None;
    }
    Some(Effect::GrantPlayPermission {
        who: PlayerRef::You,
        what: Sel::Var(vars::IT),
        duration: Duration::EndOfTurn,
        free: true,
    })
}

inventory::submit! { EffectPattern { name: "r406 until end of turn, you may cast that card for free", priority: 80, parse: until_end_of_turn_cast_it_free } }

/// Whether the effect ends by exiling the top cards of your library.
fn exiles_your_top_cards(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(PlayerRef::You, _),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(exiles_your_top_cards),
        _ => false,
    }
}

/// "Until end of turn, you may cast noncreature spells from among those cards without
/// paying their mana costs." after exiling the top cards of your library (Narset,
/// Enlightened Master): the exiled cards of that kind may be cast this turn for free.
fn until_end_of_turn_cast_those_free(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l
        .strip_prefix("until end of turn, you may cast ")
        .and_then(|r| {
            r.strip_suffix(" from among those cards without paying their mana costs")
                .or_else(|| r.strip_suffix(" from among them without paying their mana costs"))
        })
    else {
        return false;
    };
    if !exiles_your_top_cards(prev) {
        return false;
    }
    // Cards that can be cast: a permission to cast spells doesn't let lands be played.
    let exiled = Filter::and(vec![
        Filter::In(Box::new(Sel::Var(vars::IT))),
        Filter::not(Filter::Type(crate::types::CardType::Land)),
    ]);
    let what = if r == "spells" {
        exiled
    } else {
        let Some(kind) = r.strip_suffix(" spells") else {
            return false;
        };
        let Some(f) = card_filter(&format!("{kind} cards"), b) else {
            return false;
        };
        Filter::and(vec![exiled, f])
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::All(what),
            duration: Duration::EndOfTurn,
            free: true,
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "r406 until end of turn, you may cast spells from among those cards for free", priority: 80, apply: until_end_of_turn_cast_those_free } }

/// What "the rest" means after exiling cards until one is exiled: the other exiled cards
/// still in exile (the one exiled last stays there if it isn't cast).
const THE_REST: &str = "the rest";

/// "Put the rest on the bottom of your library in a random order." after exiling cards
/// until one is exiled (and what's done with that card, Jodah, the Unifier).
fn put_the_rest_on_the_bottom(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l.trim()) != "put the rest on the bottom of your library in a random order" {
        return None;
    }
    let (_, rest) = b.named.iter().rev().find(|(n, _)| n == THE_REST)?;
    let mut to = Destination::zone(ZoneKind::Library);
    to.position = LibraryPosition::BottomRandom;
    Some(Effect::Move {
        what: rest.clone(),
        to,
    })
}

inventory::submit! { EffectPattern { name: "r406 put the rest of the cards exiled until on the bottom in a random order", priority: 80, parse: put_the_rest_on_the_bottom } }
