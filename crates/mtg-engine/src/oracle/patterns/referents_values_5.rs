//! Referent and value grammar V (cards4-referents-statics): results of earlier steps,
//! instead alternatives and values.
//!
//! - "Search your library for a basic land card, reveal it, put it into your hand, then
//!   shuffle. You may put that card onto the battlefield instead of putting it into your
//!   hand if a creature died this turn." (Caravan Vigil): if the condition holds, the
//!   searcher may put the found card onto the battlefield instead, chosen as it's put
//!   ([`SearchDest::or`], CR 608.2c).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, parse_sentence, Builder};
use crate::oracle::phrases::end;

/// [`Builder::named`] key for the players of a "one or more [objects] deal damage to
/// [players]" trigger's batch (`PlayerRef::TriggerPlayers`, CR 603.2c).
pub const BATCH_PLAYERS: &str = "\u{1}batch players";

fn named(b: &Builder, key: &str) -> Option<Sel> {
    b.named
        .iter()
        .rev()
        .find(|(n, _)| n == key)
        .map(|(_, s)| s.clone())
}

/// Whether `r` ends a phrase.
fn phrase_end(r: &str) -> bool {
    r.is_empty() || r.starts_with([' ', ',', '.'])
}

/// The value grammar's atoms of this file:
/// - "the number of opponents dealt damage this way" (Hordewing Skaab): the players of
///   the triggering batch of damage events, each once.
pub fn atom(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    for p in [
        "the number of opponents dealt damage this way",
        "the number of players dealt damage this way",
        "the number of opponents dealt combat damage this way",
        "the number of players dealt combat damage this way",
    ] {
        if let Some(r) = s.strip_prefix(p).filter(|r| phrase_end(r)) {
            let players = named(b, BATCH_PLAYERS)?;
            return Some((Value::CountSel(Box::new(players)), r.to_string()));
        }
    }
    // "Whenever ~ deals combat damage to a creature, if that creature was dealt excess
    // damage this turn, you get X {E}, where X is that excess damage." (Overclocked
    // Electromancer): the excess damage the creature was dealt this turn (CR 120.10).
    if let Some(r) = s.strip_prefix("that excess damage").filter(|r| phrase_end(r)) {
        let raw = crate::oracle::raw_text().to_lowercase();
        if b.in_trigger
            && matches!(b.it, Sel::TriggerObject)
            && raw.contains(", if that creature was dealt excess damage this turn, ")
        {
            return Some((
                Value::EventsThisTurn(
                    Box::new(TriggerCond::DealtExcessDamage {
                        filter: Filter::In(Box::new(Sel::TriggerObject)),
                        noncombat_only: false,
                    }),
                    Tally::Amount,
                ),
                r.to_string(),
            ));
        }
    }
    None
}

/// Replaces each JSON value equal to `from` by `to`; the number replaced.
fn replace_json(v: &mut serde_json::Value, from: &serde_json::Value, to: &serde_json::Value) -> usize {
    if v == from {
        *v = to.clone();
        return 1;
    }
    match v {
        serde_json::Value::Array(a) => a.iter_mut().map(|x| replace_json(x, from, to)).sum(),
        serde_json::Value::Object(m) => m.values_mut().map(|x| replace_json(x, from, to)).sum(),
        _ => 0,
    }
}

/// "exile the top card of each of those opponents' libraries" (Breeches, Brazen
/// Plunderer): the players of the triggering batch, read as "each opponent" is.
fn each_of_those_opponents(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains("each of those opponents") || l.contains("each opponent") {
        return None;
    }
    named(b, BATCH_PLAYERS)?;
    let plain = l
        .replace("each of those opponents' libraries", "each opponent's library")
        .replace("each of those opponents' hands", "each opponent's hand")
        .replace("each of those opponents' graveyards", "each opponent's graveyard")
        .replace("each of those opponents", "each opponent");
    let e = parse_sentence(&plain, b)?;
    let mut json = serde_json::to_value(&e).ok()?;
    let from = serde_json::to_value(PlayerRef::EachOpponent).ok()?;
    let to = serde_json::to_value(PlayerRef::TriggerPlayers).ok()?;
    if replace_json(&mut json, &from, &to) == 0 {
        return None;
    }
    // Only "each opponent" as a player reference: a filter's opponents
    // (`PlayerFilter::Opponent`) would stay all opponents.
    if serde_json::to_string(&json).ok()?.contains("Opponent") {
        return None;
    }
    serde_json::from_value(json).ok()
}

inventory::submit! { EffectPattern { name: "referents 5: each of those opponents (the players of the batch)", priority: 100, parse: each_of_those_opponents } }

/// The last instruction of an effect, mutably, past the bookkeeping that records its
/// referents ([`Effect::Store`]).
fn last_mut(e: &mut Effect) -> &mut Effect {
    let i = match &*e {
        Effect::Seq(v) => v.iter().rposition(|x| !matches!(x, Effect::Store { .. })),
        _ => None,
    };
    match (e, i) {
        (Effect::Seq(v), Some(i)) => last_mut(&mut v[i]),
        (other, _) => other,
    }
}

/// "You may put that card onto the battlefield instead of putting it into your hand if
/// [condition]." and "If [condition], you may put that card onto the battlefield instead
/// of putting it into your hand." after a search that puts one card into its searcher's
/// hand.
fn search_destination_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    const ALT: &str = "you may put that card onto the battlefield instead of putting it into your hand";
    let l = end(l);
    let cond_s = if let Some(r) = l.strip_prefix(ALT) {
        match r.strip_prefix(" if ") {
            Some(c) => c,
            None => return false,
        }
    } else if let Some(r) = l.strip_prefix("if ") {
        match r.strip_suffix(&format!(", {ALT}")) {
            Some(c) => c,
            None => return false,
        }
    } else {
        return false;
    };
    let Some(cond) = crate::oracle::statics::parse_condition(cond_s, b.ctx) else {
        return false;
    };
    let last = last_mut(prev);
    // The plain one-card search, as the general search it is.
    let plain_search = matches!(&*last, Effect::Search {
        who: PlayerRef::You,
        whose: PlayerRef::You,
        count,
        to,
        ..
    } if count.as_const() == Some(1) && to.zone == ZoneKind::Hand);
    if let (
        true,
        Effect::Search {
            filter,
            count,
            to,
            reveal,
            shuffle,
            ..
        },
    ) = (plain_search, &*last)
    {
        let filter = match filter {
            Filter::And(v) => Filter::and(
                v.iter()
                    .filter(|f| !matches!(f, Filter::InZone(ZoneKind::Library)))
                    .cloned()
                    .collect(),
            ),
            f => f.clone(),
        };
        *last = Effect::SearchCards(Box::new(SearchSpec {
            who: PlayerRef::You,
            whose: PlayerRef::You,
            zones: vec![ZoneKind::Library],
            zones_optional: false,
            parts: vec![SearchPart {
                filter,
                count: count.clone(),
                up_to: false,
                all: false,
            }],
            distinct_names: false,
            optional: false,
            reveal: *reveal,
            dests: vec![SearchDest {
                count: None,
                to: to.clone(),
                or: vec![],
            }],
            shuffle: if *shuffle {
                SearchShuffle::After
            } else {
                SearchShuffle::No
            },
        }));
    }
    let Effect::SearchCards(spec) = last else {
        return false;
    };
    // One card, found by you, put into your hand.
    let one = spec.parts.len() == 1
        && spec.parts[0].count.as_const() == Some(1)
        && matches!(spec.who, PlayerRef::You);
    let [dest] = spec.dests.as_slice() else {
        return false;
    };
    if !one || dest.to.zone != ZoneKind::Hand || !dest.or.is_empty() {
        return false;
    }
    let mut alt = spec.clone();
    alt.dests[0].or.push(Destination::battlefield());
    let plain = std::mem::replace(last, Effect::Noop);
    *last = Effect::If {
        cond,
        then: Box::new(Effect::SearchCards(alt)),
        otherwise: Box::new(plain),
    };
    true
}

inventory::submit! { FollowupPattern { name: "referents 5: you may put that card onto the battlefield instead of putting it into your hand if [condition]", priority: 100, apply: search_destination_instead } }

/// "Whenever one or more creature tokens you control deal combat damage to a player,
/// exile target instant or sorcery card ... from their graveyard." (Soundwave, Sonic
/// Spy): "their" is the player dealt damage. The objects dealing it are yours, whose
/// graveyard the text would call "your graveyard".
fn from_their_graveyard_damaged_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !b.in_trigger
        || !matches!(b.it_player, PlayerRef::TriggerPlayer)
        || !l.contains(" from their graveyard")
        || l.contains("that player")
        || l.contains("target player")
        || l.contains("target opponent")
    {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    if !(raw.contains(" you control deal combat damage to a player,")
        || raw.contains(" you control deal damage to a player,"))
    {
        return None;
    }
    parse_sentence(
        &l.replace(" from their graveyard", " from that player's graveyard"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "referents 5: from their graveyard (the player your objects dealt damage to)", priority: 100, parse: from_their_graveyard_damaged_player } }

/// "Whenever a creature deals combat damage to a player, if that player controls one or
/// more lands with contested counters on them, that creature's controller gains control
/// of one of those lands of their choice and untaps it." (Turf War): "one of those lands"
/// is one of the objects the intervening "if" clause names (CR 603.4), chosen as the
/// instruction is performed by the player performing it.
fn one_of_those_from_condition(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, rest) = l.split_once(" one of those ")?;
    let (noun, after) = rest.split_once(' ').unwrap_or((rest, ""));
    let after = after.strip_prefix("of their choice").unwrap_or(after).trim_start();
    let raw = crate::oracle::raw_text().to_lowercase();
    // The condition's phrase: "if that player controls one or more [nouns] [qualifier],".
    let key = format!("if that player controls one or more {noun} ");
    let i = raw.find(&key)?;
    let qualifier = raw[i + key.len()..].split(',').next()?;
    let singular = noun.strip_suffix('s')?;
    // "with contested counters on them" -> "with a contested counter on it".
    let q = match qualifier
        .strip_prefix("with ")
        .and_then(|q| q.strip_suffix(" counters on them"))
    {
        Some(kind) if !kind.contains(' ') => format!("with a {kind} counter on it"),
        _ => return None,
    };
    let text = format!("{head} a {singular} {q} that player controls {after}");
    parse_sentence(text.trim_end(), b)
}

inventory::submit! { EffectPattern { name: "referents 5: one of those [objects] (named by the intervening if)", priority: 100, parse: one_of_those_from_condition } }

/// "there are eight or more different names among unlocked doors of Rooms you control"
/// (Central Elevator // Promising Stairs, CR 709.5).
fn different_door_names(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("there are ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let r = r
        .trim_start()
        .strip_prefix("or more different names among unlocked doors of rooms you control")?;
    if !r.trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::Custom(crate::kw::room_door_names::DIFFERENT_DOOR_NAMES.into()),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! { super::ConditionPattern { name: "referents 5: there are N or more different names among unlocked doors", priority: 100, parse: different_door_names } }

/// "{T}, Return a land you control to its owner's hand: Draw a card. Then discard a card
/// unless that land had a nonbasic land type." (Wonderscape Sage): the land the cost
/// returned, as it last existed on the battlefield (`zones::COST_RETURNED`, CR 608.2h),
/// had one of the land types other than the basic ones (CR 205.3i).
fn returned_land_had_nonbasic_type(c: &str) -> Option<Condition> {
    if end(c) != "that land had a nonbasic land type" {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("return a land you control to its owner's hand:") {
        return None;
    }
    let lists = crate::types::subtype_lists();
    let types: Vec<Filter> = lists
        .land
        .iter()
        .filter(|t| !lists.basic_land.contains(&t.as_str()))
        .map(|t| Filter::Subtype(crate::types::Subtype::from(t.as_str())))
        .collect();
    Some(Condition::SelMatches(
        Sel::Var(crate::zones::COST_RETURNED),
        Filter::Or(types),
    ))
}

inventory::submit! { super::ConditionPattern { name: "referents 5: that land (returned by the cost) had a nonbasic land type", priority: 100, parse: returned_land_had_nonbasic_type } }

/// "When ~ dies, if it had one or more counters on it, create a 0/0 ... token, then put
/// ~'s counters on that token." (Ambitious Augmenter): the counters ~ had as it last
/// existed on the battlefield (CR 608.2h), each kind and number.
fn put_sources_counters_on(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put ~'s counters on ")?;
    let (to, rest) = crate::oracle::effects::object_ref(r, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    let from = if raw.contains("when ~ dies,") || raw.contains("when this creature dies,") {
        if !b.in_trigger {
            return None;
        }
        Sel::TriggerLki
    } else {
        Sel::This
    };
    Some(Effect::PutCountersOf {
        from,
        to,
        kind: None,
    })
}

inventory::submit! { EffectPattern { name: "referents 5: put ~'s counters on [object]", priority: 100, parse: put_sources_counters_on } }

/// "Otherwise, you may turn a permanent you control face up." (Zimone, Mystery
/// Unraveler): only a face-down permanent can be turned face up (CR 702.37e, 708.7), so
/// the choice is among those.
fn turn_a_permanent_face_up(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains("turn a permanent you control face up") {
        return None;
    }
    parse_sentence(
        &l.replace(
            "turn a permanent you control face up",
            "turn a face-down permanent you control face up",
        ),
        b,
    )
}

inventory::submit! { EffectPattern { name: "referents 5: turn a permanent you control face up", priority: 100, parse: turn_a_permanent_face_up } }

/// "Whenever you cast your second spell each turn, copy it, then exile the spell you cast
/// with four time counters on it." (Taigam, Master Opportunist): the spell the trigger is
/// about.
fn the_spell_you_cast(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains("the spell you cast") || !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    if raw.matches("cast").count() < 2 || !raw.contains("whenever you cast ") {
        return None;
    }
    parse_sentence(&l.replace("the spell you cast", "that spell"), b)
}

inventory::submit! { EffectPattern { name: "referents 5: the spell you cast (the triggering spell)", priority: 100, parse: the_spell_you_cast } }

/// The card "a card you put into your graveyard this way" chose.
const DREAD_CARD: Var = vars::USER + 51_101;

/// "Whenever you manifest dread, put a card you put into your graveyard this way into your
/// hand." (Paranormal Analyst): one of the cards that manifest dread put into your
/// graveyard (CR 701.62a), still there.
fn card_put_into_graveyard_by_dread(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    const P: &str = "a card you put into your graveyard this way";
    if !l.contains(P) || !b.in_trigger {
        return None;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("whenever you manifest dread,") {
        return None;
    }
    let choose = Effect::Store {
        var: DREAD_CARD,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::InZone(ZoneKind::Graveyard),
                Filter::Custom(crate::kwa::manifest::PUT_IN_GRAVEYARD_BY_THAT_DREAD.into()),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    };
    let saved = b.it.clone();
    b.it = Sel::Var(DREAD_CARD);
    let e = parse_sentence(&l.replace(P, "it"), b);
    if matches!(b.it, Sel::Var(DREAD_CARD)) {
        b.it = saved;
    }
    Some(Effect::seq(vec![choose, e?]))
}

inventory::submit! { EffectPattern { name: "referents 5: a card you put into your graveyard this way (manifest dread)", priority: 100, parse: card_put_into_graveyard_by_dread } }

/// The objects the counter instructions of the previous sentence put counters on.
const GOT_COUNTERS: Var = vars::USER + 51_102;

/// "At the beginning of your upkeep, for each color, put a +1/+1 counter on a Dragon you
/// control of that color. If you put +1/+1 counters on five Dragons this way, you win the
/// game." (Call the Spirit Dragons): the different objects the previous sentence's counter
/// instructions put counters of that kind on (one Dragon chosen for two colors counts
/// once; one that got no counter doesn't count).
fn if_you_put_counters_on_n(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you put ") else {
        return false;
    };
    let Some((kind, r)) = r.split_once(" counters on ") else {
        return false;
    };
    let Some((what, then)) = r.split_once(" this way, ") else {
        return false;
    };
    let Some((n, noun)) = crate::oracle::phrases::parse_number(what) else {
        return false;
    };
    let Some((f, true, tail)) = crate::oracle::phrases::parse_object_phrase(noun.trim()) else {
        return false;
    };
    if !end(tail).is_empty() {
        return false;
    }
    // The previous sentence: counter instructions (one, or one for each of a list).
    let steps: Vec<Effect> = match &*prev {
        Effect::Seq(v) => v.clone(),
        e => vec![e.clone()],
    };
    let counter_kind = |e: &Effect| match e {
        Effect::AddCounters { kind: k, .. } => Some(k.to_string()),
        _ => None,
    };
    if steps.is_empty() || steps.iter().any(|e| counter_kind(e).as_deref() != Some(kind)) {
        return false;
    }
    let Some(e) = parse_clause(then, b) else {
        return false;
    };
    let mut out = vec![Effect::Store {
        var: GOT_COUNTERS,
        sel: Sel::None,
    }];
    for s in steps {
        out.push(Effect::Store {
            var: vars::IT,
            sel: Sel::None,
        });
        out.push(s);
        out.push(Effect::Store {
            var: GOT_COUNTERS,
            sel: Sel::Union(vec![Sel::Var(GOT_COUNTERS), Sel::Var(vars::IT)]),
        });
    }
    out.push(Effect::If {
        cond: Condition::Compare(
            Value::Count(Filter::and(vec![
                Filter::In(Box::new(Sel::Var(GOT_COUNTERS))),
                f,
            ])),
            Cmp::Ge,
            n,
        ),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    });
    *prev = Effect::Seq(out);
    true
}

inventory::submit! { FollowupPattern { name: "referents 5: if you put [kind] counters on N [objects] this way", priority: 100, apply: if_you_put_counters_on_n } }

/// "Reveal up to five nonland cards from your hand. For each of those cards that has the
/// same mana value as another card revealed this way, create a Treasure token." (Vault 21:
/// House Gambit): the instruction once for each revealed card sharing its mana value with
/// another revealed card (CR 202.3).
fn for_each_sharing_mana_value(l: &str, b: &mut Builder) -> Option<Effect> {
    let rest = end(l).strip_prefix(
        "for each of those cards that has the same mana value as another card revealed this way, ",
    )?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("reveal up to") && !raw.contains("reveal ") {
        return None;
    }
    let e = parse_clause(rest, b)?;
    Some(Effect::Repeat {
        times: Value::CountSel(Box::new(Sel::Matching(
            Box::new(Sel::Var(vars::IT)),
            Filter::Custom(crate::kw::referents_5::SAME_MANA_VALUE_AS_ANOTHER_OF_IT.into()),
        ))),
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "referents 5: for each of those cards that has the same mana value as another card revealed this way", priority: 100, parse: for_each_sharing_mana_value } }
