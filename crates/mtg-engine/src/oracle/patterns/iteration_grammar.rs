//! "For each [players or objects], [instruction]": the instruction is performed once for
//! each of them, with the one it's performed for as the referent of "that player",
//! "they", "it", "its controller", "that creature" inside it (CR 608.2c).
//!
//! - Players: "for each opponent, ~ deals 1 damage to that player unless they pay {1}",
//!   "for each player, choose a creature that player controls", "for each opponent who
//!   has less life than you, create a 1/3 ... token" → [`Effect::ForEachPlayer`] with
//!   `PlayerRef::Iterated` as "that player".
//! - Objects: "for each creature destroyed this way, its controller creates a 1/1 white
//!   Spirit creature token", "for each of them, put a +1/+1 counter on it", "for each
//!   creature, its controller sacrifices it unless they pay X life", "for each attacking
//!   creature, its owner puts it on the top or bottom of their library" →
//!   [`Effect::ForEach`] with the object bound to a variable ("it").
//!
//! Each instruction is performed for all of them at the same time, as several players or
//! objects do something at once (CR 101.4, 608.2f; see `simultaneous.rs`).
//!
//! Only instructions that don't target: targets chosen for each player are
//! `per_player_targets`.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_clause, Builder};
use crate::oracle::phrases::*;

/// The object the instruction is being performed for.
pub const EACH: Var = vars::USER + 7700;

/// The comma positions in `r` (outside quotes) where "X, Y" may split.
fn comma_splits(r: &str) -> Vec<(&str, &str)> {
    let mut out = vec![];
    let mut in_quote = false;
    for (i, c) in r.char_indices() {
        if c == '"' {
            in_quote = !in_quote;
        }
        if c == ',' && !in_quote && r[i..].starts_with(", ") {
            out.push((&r[..i], &r[i + 2..]));
        }
    }
    out
}

/// Words that refer to the iterated player inside the instruction.
fn mentions_player_pronoun(y: &str) -> bool {
    y.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .any(|w| matches!(w, "they" | "their" | "them" | "that" | "player" | "opponent"))
}

/// The players a "for each [players]" phrase names: "opponent", "player", "other
/// player", "opponent who [condition]".
fn iterated_players(x: &str, b: &Builder) -> Option<PlayerRef> {
    Some(match x {
        "opponent" => PlayerRef::EachOpponent,
        "player" => PlayerRef::EachPlayer,
        "other player" => PlayerRef::EachOtherPlayer,
        _ => {
            let (base, cond) = if let Some(c) = x.strip_prefix("opponent who ") {
                (PlayerFilter::Opponent, c)
            } else if let Some(c) = x.strip_prefix("player who ") {
                (PlayerFilter::Any, c)
            } else {
                return None;
            };
            let cond = player_condition(cond, b)?;
            PlayerRef::Each(PlayerFilter::And(vec![base, cond]))
        }
    })
}

/// A condition on the iterated player compared with you: "has less life than you", "has
/// more cards in hand than you", "controls more creatures than you".
fn player_condition(c: &str, _b: &Builder) -> Option<PlayerFilter> {
    if let Some(f) = super::choice_grammar_players::compared_with_you(&format!("who {c}")) {
        return Some(f);
    }
    // "Each opponent discards a card. For each opponent who can't, ...": a player who
    // discarded nothing (the discard instruction records what each player discarded).
    if c == "can't" {
        let raw = crate::oracle::raw_text().to_lowercase();
        let after_discard = ["each opponent discards a card. ", "each player discards a card. "]
            .iter()
            .any(|p| {
                ["for each opponent who can't", "for each player who can't", "each opponent who can't"]
                    .iter()
                    .any(|w| raw.contains(&format!("{p}{w}")))
            });
        if !after_discard {
            return None;
        }
        let discarded = PlayerFilter::Ref(Box::new(PlayerRef::OwnerOf(Box::new(Sel::All(
            Filter::In(Box::new(Sel::Var(crate::discard_rules::DISCARDED))),
        )))));
        return Some(PlayerFilter::Not(Box::new(discarded)));
    }
    let c = c.strip_suffix(" do").unwrap_or(c);
    match c {
        "has less life than you" => Some(PlayerFilter::Life(
            Cmp::Lt,
            Box::new(Value::LifeTotal(PlayerRef::You)),
        )),
        "has fewer cards in hand than you" => Some(PlayerFilter::HandSize(
            Cmp::Lt,
            Box::new(Value::HandSize(PlayerRef::You)),
        )),
        _ => None,
    }
}

/// Whether `l` is only the part of a sentence before "unless": "For each land, destroy
/// that land unless any player pays 1 life." — the payment is part of what's done for each
/// one, so the sentence is read as a whole.
fn before_unless(l: &str) -> bool {
    let mut raw = crate::oracle::raw_text().to_lowercase();
    let name = crate::oracle::card_name().to_lowercase();
    if !name.is_empty() {
        raw = raw.replace(&name, "~");
    }
    raw.contains(&format!("{} unless ", end(l)))
}

/// "for each [players], [instruction]".
fn for_each_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    if before_unless(l) {
        return None;
    }
    for (x, y) in comma_splits(r) {
        let Some(who) = iterated_players(x, b) else {
            continue;
        };
        // Targets chosen for each player are another grammar (`per_player_targets`).
        if y.contains("target ") {
            return None;
        }
        // "For each opponent, create a token" is counted (`for_each_opponent`); only an
        // instruction about the player, or for players described by a condition.
        let qualified = x.contains(" who ");
        if !mentions_player_pronoun(y) && !qualified {
            return None;
        }
        if let Some(e) = choose_for_each(who.clone(), y, b) {
            return Some(e);
        }
        if let Some(e) = act_on_one_for_each(who.clone(), y, b) {
            return Some(e);
        }
        let saved = (b.it_player.clone(), b.targets.len(), b.it.clone());
        b.it_player = PlayerRef::Iterated;
        let body = parse_clause(&they_as_that_player(y), b);
        b.it_player = saved.0;
        let Some(body) = body else {
            b.targets.truncate(saved.1);
            b.it = saved.2;
            return None;
        };
        if b.targets.len() != saved.1 || !(qualified || mentions_iterated_player(&body)) {
            b.targets.truncate(saved.1);
            b.it = saved.2;
            return None;
        }
        return Some(Effect::ForEachPlayer {
            who,
            effect: Box::new(body),
        });
    }
    None
}

/// The objects chosen by the latest "for each player, choose ..." (all players' choices).
pub const CHOSEN: Var = vars::USER + 7702;

thread_local! {
    /// The text whose "chosen this way" names [`CHOSEN`]: set as a "for each player,
    /// choose ..." instruction is read, for the qualifier "not chosen this way" in later
    /// sentences of the same text (see [`not_chosen_suffix`]).
    static CHOSEN_TEXT: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// "[you] choose a [object] that player controls" / "... in that player's graveyard" /
/// "up to one [object] that player controls" for each player: the controller chooses one
/// object for each player (not targeted, CR 115.10), in turn order (CR 101.4). Later
/// sentences name all the chosen objects ("those creatures", "the chosen cards",
/// "creatures chosen this way", "all creatures they control not chosen this way").
fn choose_for_each(who: PlayerRef, y: &str, b: &mut Builder) -> Option<Effect> {
    let r = y.strip_prefix("you choose ").or_else(|| y.strip_prefix("choose "))?;
    let (count, up_to, r) = if let Some(r) = r.strip_prefix("up to one ") {
        (1, true, r)
    } else if let Some(r) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) {
        (1, false, r)
    } else {
        return None;
    };
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural {
        return None;
    }
    let tail = tail.trim();
    let (zone, rel) = match tail {
        "that player controls" | "they control" => (ZoneKind::Battlefield, Filter::ControlledBy(PlayerRel::Iterated)),
        "in that player's graveyard" | "in their graveyard" => (ZoneKind::Graveyard, Filter::OwnedBy(PlayerRel::Iterated)),
        _ => return None,
    };
    if f.zone().is_some_and(|z| z != zone) {
        return None;
    }
    let noun = r.split([' ', ',']).find(|w| head_noun(w).is_some())?;
    let noun = if zone == ZoneKind::Graveyard { "card" } else { noun };
    let choose = Effect::Store {
        var: CHOSEN,
        sel: Sel::Union(vec![
            Sel::Var(CHOSEN),
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![f, Filter::InZone(zone), rel]),
                count: Value::c(count),
                up_to,
                store: None,
            },
        ]),
    };
    name_chosen(b, noun);
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN,
            sel: Sel::Union(vec![]),
        },
        Effect::ForEachPlayer {
            who,
            effect: Box::new(choose),
        },
    ]))
}

/// "starting with you, each player chooses a creature", "starting with you, each player
/// may choose an artifact or enchantment you don't control", "starting with the next
/// opponent in turn order, each opponent chooses a creature card in your graveyard that
/// hasn't been chosen", "... chooses a different nonland card from among them": the players
/// choose one at a time in turn order (CR 101.4b), each knowing the earlier choices (not
/// targeted, CR 115.10); later sentences name all the chosen objects.
fn starting_with_choose(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first, r) = if let Some(r) = l.strip_prefix("starting with you, ") {
        (TurnOrderStart::You, r)
    } else {
        (
            TurnOrderStart::NextOpponent,
            l.strip_prefix("starting with the next opponent in turn order, ")?,
        )
    };
    let (who, r) = if let Some(r) = r.strip_prefix("each player ") {
        (PlayerFilter::Any, r)
    } else {
        (PlayerFilter::Opponent, r.strip_prefix("each opponent ")?)
    };
    let (may, r) = match r.strip_prefix("may choose ") {
        Some(r) => (true, r),
        None => (false, r.strip_prefix("chooses ")?),
    };
    let (count, up_to, r) = if let Some(r) = r.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        (n.as_const()?, true, r.trim_start())
    } else if let Some(r) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) {
        (1, may, r)
    } else {
        return None;
    };
    // "a different nonland card", "a creature card ... that hasn't been chosen".
    let (different, r) = match r.strip_prefix("different ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let (r, unchosen) = match r.strip_suffix(" that hasn't been chosen") {
        Some(r) => (r, true),
        None => (r, false),
    };
    let (f, _, tail) = parse_object_phrase(r)?;
    let mut parts = vec![f.clone()];
    match tail.trim() {
        "" => {}
        "they control" => parts.push(Filter::ControlledBy(PlayerRel::Iterated)),
        "from among permanents your opponents control" => {
            parts.push(Filter::ControlledBy(PlayerRel::Opponent))
        }
        "from among them" => {
            let Some(Some((them, rest))) = super::pronoun_groups::plural_object_ref("them", b)
            else {
                return None;
            };
            if !rest.is_empty() {
                return None;
            }
            parts.push(Filter::In(Box::new(them)));
        }
        _ => return None,
    }
    // Permanents, unless the phrase names cards ("a nonland card from among them").
    let cards = serde_json::to_string(&f).is_ok_and(|j| j.contains("\"Card\""));
    let zone = f.zone().unwrap_or(if cards { ZoneKind::Library } else { ZoneKind::Battlefield });
    if f.zone().is_none() && !cards {
        parts.push(Filter::InZone(ZoneKind::Battlefield));
    }
    if cards && f.zone().is_none() && !parts.iter().any(|p| matches!(p, Filter::In(_))) {
        return None;
    }
    if different || unchosen {
        parts.push(Filter::not(Filter::In(Box::new(Sel::Var(CHOSEN)))));
    }
    let noun = r.split([' ', ',']).find(|w| head_noun(w).is_some())?;
    let noun = if zone == ZoneKind::Battlefield { noun } else { "card" };
    let choose = Effect::Store {
        var: CHOSEN,
        sel: Sel::Union(vec![
            Sel::Var(CHOSEN),
            Sel::Choose {
                chooser: PlayerRef::Iterated,
                filter: Filter::and(parts),
                count: Value::c(count),
                up_to,
                store: None,
            },
        ]),
    };
    name_chosen(b, noun);
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN,
            sel: Sel::Union(vec![]),
        },
        Effect::InTurnOrder {
            first,
            who,
            effect: Box::new(choose),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "iteration: starting with [player], each player chooses [objects]", priority: 100, parse: starting_with_choose } }

/// "for each kind of counter on target permanent or player, give that permanent or player
/// another counter of that kind" (Maulfist Revolutionary), "for each kind of counter on
/// target permanent, put another counter of that kind on it or remove one from it"
/// (Quarry Hauler), "for each kind of counter on permanents you control, you may put your
/// choice of a +1/+1 counter or a counter of that kind on ~" (Bribe Taker).
fn each_counter_kind(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each kind of counter on ")?;
    let k = CHOSEN_COUNTER_KIND;
    for (x, y) in comma_splits(r) {
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let from = if let Some(sel) = super::counter_grammar::holder(x, b) {
            sel
        } else if let Some((f, true, tail)) = parse_object_phrase(x) {
            if !tail.trim().is_empty() {
                continue;
            }
            Sel::All(f)
        } else {
            b.targets.truncate(saved.0);
            continue;
        };
        let mut t = y.to_string();
        // "put another counter of that kind on it or remove one from it".
        if let Some(head) = t.strip_suffix(" or remove one from it") {
            let add = head.replace("another counter of that kind", &format!("another {k} counter"));
            let add = crate::oracle::effects::parse_clause(&add, b)?;
            let Effect::AddCounters { what, .. } = &add else {
                return None;
            };
            let remove = Effect::RemoveCounters {
                what: what.clone(),
                kind: Some(k.into()),
                n: Value::c(1),
            };
            return Some(Effect::ForEachCounterKind {
                from,
                then: Box::new(Effect::ChooseOne {
                    who: PlayerRef::You,
                    options: vec![
                        ("put another counter of that kind".into(), add),
                        ("remove one".into(), remove),
                    ],
                }),
            });
        }
        for (a, c) in [
            ("another counter of that kind", format!("another {k} counter")),
            ("a counter of that kind", format!("a {k} counter")),
        ] {
            t = t.replace(a, &c);
        }
        if let Some(rest) = t.strip_prefix("give that permanent or player ") {
            t = format!("put {rest} on it");
        }
        let then = crate::oracle::effects::parse_clause(&t, b);
        let Some(then) = then.filter(|e| mentions_kind(e)) else {
            b.targets.truncate(saved.0);
            (b.it, b.it_player) = (saved.1, saved.2);
            return None;
        };
        return Some(Effect::ForEachCounterKind {
            from,
            then: Box::new(then),
        });
    }
    None
}

/// Whether the effect uses the kind of counter being iterated.
fn mentions_kind(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|j| j.contains(CHOSEN_COUNTER_KIND))
}

inventory::submit! { EffectPattern { name: "iteration: for each kind of counter on [holder], [instruction with that kind]", priority: 100, parse: each_counter_kind } }

/// "Prevent all damage that would be dealt to target multicolored creature this turn. For
/// each 1 damage prevented this way, put a +1/+1 counter on that creature." (Brace for
/// Impact), "... For each 1 damage prevented this way, create a 2/1 white and black
/// Inkling creature token with flying." (Inkshield): the rest of the prevention effect,
/// performed right after damage is prevented (CR 615.5), once for each 1 damage
/// prevented.
fn f_for_each_damage_prevented(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = l.strip_prefix("for each 1 damage prevented this way, ") else {
        return false;
    };
    let Some(slot) = last_prevent_damage(prev) else {
        return false;
    };
    // "that creature": the one the damage would have been dealt to (the event's object).
    let saved = (b.targets.len(), b.it.clone(), b.in_trigger);
    b.it = Sel::TriggerObject;
    b.in_trigger = true;
    let e = crate::oracle::effects::parse_clause(r, b);
    let ok = b.targets.len() == saved.0;
    b.targets.truncate(saved.0);
    (b.it, b.in_trigger) = (saved.1, saved.2);
    let Some(e) = e.filter(|_| ok) else {
        return false;
    };
    let e = match e {
        Effect::AddCounters {
            what,
            kind,
            n: Value::Const(1),
        } => Effect::AddCounters {
            what,
            kind,
            n: Value::EventAmount,
        },
        e => Effect::Repeat {
            times: Value::EventAmount,
            effect: Box::new(e),
        },
    };
    *slot = Some(Box::new(e));
    true
}

/// The `then` of the last prevention shield an effect creates, if it has none yet.
fn last_prevent_damage(e: &mut Effect) -> Option<&mut Option<Box<Effect>>> {
    match e {
        Effect::Seq(v) => v.iter_mut().rev().find_map(last_prevent_damage),
        Effect::PreventDamage { then, .. } if then.is_none() => Some(then),
        _ => None,
    }
}

inventory::submit! { super::FollowupPattern { name: "iteration: for each 1 damage prevented this way, [instruction]", priority: 50, apply: f_for_each_damage_prevented } }

/// "For each land destroyed this way, its controller may search their library for a basic
/// land card and put it onto the battlefield. Then each player who searched their library
/// this way shuffles." (From the Ashes): each search is followed by its player's shuffle.
fn f_searchers_shuffle(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if l.strip_prefix("then ").unwrap_or(l) != "each player who searched their library this way shuffles" {
        return false;
    }
    fn search_in(e: &mut Effect) -> Option<&mut bool> {
        match e {
            Effect::Seq(v) => v.last_mut().and_then(search_in),
            Effect::ForEach { effect, var, .. } if *var == EACH => search_in(effect),
            Effect::May { effect, .. } => search_in(effect),
            Effect::Search { shuffle, .. } if !*shuffle => Some(shuffle),
            _ => None,
        }
    }
    if !matches!(last_of(prev), Effect::ForEach { var, .. } if *var == EACH) {
        return false;
    }
    match search_in(prev) {
        Some(shuffle) => {
            *shuffle = true;
            true
        }
        None => false,
    }
}

/// The last instruction of a sequence.
fn last_of(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_of),
        e => e,
    }
}

inventory::submit! { super::FollowupPattern { name: "iteration: then each player who searched this way shuffles", priority: 50, apply: f_searchers_shuffle } }

/// "{u}{u} spent to cast it" → how many times that much mana of that color was spent to
/// cast the object (CR 601.2h; a copy or a permanent that wasn't cast saw none spent).
fn spent_symbols(x: &str) -> Option<Value> {
    let body = [" spent to cast it", " spent to cast ~", " spent to cast this spell"]
        .iter()
        .find_map(|s| x.strip_suffix(s))?;
    let inner = body.strip_prefix('{')?.strip_suffix('}')?;
    let syms: Vec<&str> = inner.split("}{").collect();
    let letter = syms.first()?.to_ascii_uppercase();
    if letter.len() != 1 || !"WUBRGC".contains(letter.as_str()) || syms.iter().any(|s| s.to_ascii_uppercase() != letter) {
        return None;
    }
    let spent = Value::Custom(format!("mana_spent_of:{letter}").into());
    Some(match syms.len() {
        1 => spent,
        n => Value::Div(Box::new(spent), n as i32, false),
    })
}

/// "When this creature enters, for each {U}{U} spent to cast it, draw a card." (Bladecoil
/// Serpent): the instruction is performed that many times.
fn for_each_spent(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (x, y) = r.split_once(", ")?;
    let times = spent_symbols(x)?;
    let effect = parse_clause(y, b)?;
    Some(Effect::Repeat {
        times,
        effect: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "iteration: for each {C}{C} spent to cast it, [instruction]", priority: 100, parse: for_each_spent } }

/// "For each color, return up to one target card of that color from your graveyard to your
/// hand." (All Suns' Dawn), "for each color, put a +1/+1 counter on a Dragon you control
/// of that color" (Call the Spirit Dragons), "For each permanent type, return up to one
/// card of that type from your graveyard to the battlefield." (Revival Experiment): the
/// instruction once for each color (CR 105.1) or permanent type (CR 110.4), each about an
/// object of that quality; objects moved to the same place move together.
fn for_each_quality(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    let (qualities, marker, y): (&[&str], &str, &str) =
        if let Some(y) = r.strip_prefix("color, ") {
            (&["white", "blue", "black", "red", "green"], " of that color", y)
        } else if let Some(y) = r.strip_prefix("permanent type, ") {
            (
                &["artifact", "battle", "creature", "enchantment", "land", "planeswalker"],
                " of that type",
                y,
            )
        } else {
            return None;
        };
    let i = y.find(marker)?;
    if y[i + marker.len()..].contains(marker) {
        return None;
    }
    let (pre, post) = (&y[..i], &y[i + marker.len()..]);
    // Where the object phrase begins: after its determiner ("up to one target ", "a ").
    let start = ["target ", " a ", " an ", "up to one "]
        .iter()
        .filter_map(|d| pre.rfind(d).map(|j| j + d.len()))
        .max()?;
    let saved = b.targets.len();
    let mut effects = Vec::new();
    for q in qualities {
        let mut head = pre[..start].to_string();
        // "a artifact card" → "an artifact card".
        if head.ends_with(" a ") && q.starts_with(['a', 'e', 'i', 'o', 'u']) {
            head.truncate(head.len() - 2);
            head.push_str("an ");
        }
        let text = format!("{head}{q} {}{post}", &pre[start..]);
        let Some(e) = parse_clause(&text, b) else {
            b.targets.truncate(saved);
            return None;
        };
        effects.push(e);
    }
    // One instruction moving them all at once.
    let same_move = effects.windows(2).all(|w| {
        matches!((&w[0], &w[1]), (Effect::Move { to: a, .. }, Effect::Move { to: c, .. })
            if serde_json::to_string(a).ok() == serde_json::to_string(c).ok())
    });
    if same_move {
        if let Some(Effect::Move { to, .. }) = effects.first() {
            let to = to.clone();
            let what = effects
                .iter()
                .filter_map(|e| match e {
                    Effect::Move { what, .. } => Some(what.clone()),
                    _ => None,
                })
                .collect();
            return Some(Effect::Move {
                what: Sel::Union(what),
                to,
            });
        }
    }
    Some(Effect::Seq(effects))
}

inventory::submit! { EffectPattern { name: "iteration: for each color/permanent type, [instruction about one of that quality]", priority: 100, parse: for_each_quality } }

/// The total amount of mana the players paid with "Join forces — Starting with you, each
/// player may pay any amount of mana."
pub const MANA_PAID: Var = vars::USER + 7703;

/// Marks, in [`Builder::named`], that [`MANA_PAID`] holds the mana the players paid.
const MANA_PAID_NAME: &str = "\u{1}join forces";

/// "Join forces — Starting with you, each player may pay any amount of mana.":
/// each player in turn order, starting with you (knowing how much the players before paid,
/// CR 101.4b), chooses an amount and pays that much mana; the amounts are added up for
/// "the total amount of mana paid this way".
fn join_forces(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "starting with you, each player may pay any amount of mana" {
        return None;
    }
    let pay = Effect::Custom(crate::kw::join_forces::PAY_ANY_AMOUNT.into());
    b.named.push((MANA_PAID_NAME.to_string(), Sel::None));
    Some(Effect::seq(vec![
        Effect::StoreValue {
            var: MANA_PAID,
            value: Value::c(0),
        },
        Effect::InTurnOrder {
            first: TurnOrderStart::You,
            who: PlayerFilter::Any,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(pay),
            }),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "iteration: join forces (each player may pay any amount of mana)", priority: 100, parse: join_forces } }

/// "Each player draws X cards, where X is the total amount of mana paid this way." after
/// [`join_forces`].
fn total_mana_paid(l: &str, b: &mut Builder) -> Option<Effect> {
    const WHERE: &str = ", where x is the total amount of mana paid this way";
    let l = end(l);
    if !l.contains(WHERE) || !b.named.iter().any(|(p, _)| p == MANA_PAID_NAME) {
        return None;
    }
    let head = l.replacen(WHERE, "", 1);
    let e = super::value_grammar::with_x_defined(true, || parse_clause(&head, b))?;
    Some(Effect::seq(vec![
        Effect::SetX {
            value: Value::Var(MANA_PAID),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "iteration: [instruction], where X is the total amount of mana paid this way", priority: 50, parse: total_mana_paid } }

/// Later sentences' names for the objects in [`CHOSEN`] ("the chosen creatures", "each
/// permanent chosen this way", "creatures they control not chosen this way").
fn name_chosen(b: &mut Builder, noun: &str) {
    for name in [
        format!("the chosen {noun}s"),
        format!("those {noun}s"),
        format!("{noun}s chosen this way"),
        format!("each {noun} chosen this way"),
        "the chosen permanents".to_string(),
        "permanents chosen this way".to_string(),
        "each permanent chosen this way".to_string(),
    ] {
        b.named.push((name, Sel::Var(CHOSEN)));
    }
    b.it = Sel::Var(CHOSEN);
    CHOSEN_TEXT.with(|t| *t.borrow_mut() = crate::oracle::raw_text());
}

/// "for each player, destroy up to one nonbasic land that player controls" (Krenko's
/// Buzzcrusher): the controller chooses that object for each player (not targeted), then
/// the instruction is performed on all of them at once.
fn act_on_one_for_each(who: PlayerRef, y: &str, b: &mut Builder) -> Option<Effect> {
    let (verb, det, rest) = ["up to one ", "a ", "an "].iter().find_map(|d| {
        let i = y.find(&format!(" {d}"))?;
        Some((&y[..i], *d, &y[i + 1 + d.len()..]))
    })?;
    if !matches!(verb, "destroy" | "exile" | "tap" | "untap") {
        return None;
    }
    let saved = (b.named.len(), b.it.clone());
    let choose = choose_for_each(who, &format!("choose {det}{rest}"), b)?;
    let act = parse_clause(&format!("{verb} the chosen permanents"), b);
    let Some(act) = act else {
        b.named.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    Some(Effect::seq(vec![choose, act]))
}

/// "not chosen this way" / "that weren't chosen this way" after an object phrase, in a
/// text where "for each player, choose ..." chose them: the objects other than those.
fn not_chosen_suffix<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let rest = ["not chosen this way", "that weren't chosen this way", "that wasn't chosen this way"]
        .iter()
        .find_map(|p| s.strip_prefix(p))?;
    let ours = CHOSEN_TEXT.with(|t| *t.borrow() == crate::oracle::raw_text());
    if !ours {
        return None;
    }
    Some((Filter::not(Filter::In(Box::new(Sel::Var(CHOSEN)))), rest))
}

inventory::submit! { super::FilterSuffixPattern { name: "iteration: not chosen this way", priority: 100, parse: not_chosen_suffix } }

/// "Each opponent discards a card. Each opponent who can't loses 3 life." (Entropic
/// Battlecruiser): "for each opponent who can't, that player loses 3 life".
fn each_who_cant(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, pred) = if let Some(p) = end(l).strip_prefix("each opponent who can't ") {
        ("opponent", p)
    } else {
        ("player", end(l).strip_prefix("each player who can't ")?)
    };
    for_each_player(&format!("for each {who} who can't, that player {pred}"), b)
}

inventory::submit! { EffectPattern { name: "iteration: each opponent who can't [instruction]", priority: 100, parse: each_who_cant } }

/// "they"/"their" as the iterated player: "unless they pay {1}" → "unless that player
/// pays {1}".
fn they_as_that_player(y: &str) -> String {
    let mut s = format!(" {y} ");
    for (from, to) in [
        (" unless they pay ", " unless that player pays "),
        (" they control", " that player controls"),
        (" their graveyard", " that player's graveyard"),
        (" their library", " that player's library"),
        (" their hand", " that player's hand"),
    ] {
        s = s.replace(from, to);
    }
    s.trim().to_string()
}

/// Whether the effect refers to the iterated player anywhere.
fn mentions_iterated_player(e: &Effect) -> bool {
    let j = serde_json::to_string(e).unwrap_or_default();
    j.contains("\"Iterated\"")
}

/// The objects a "for each [objects]" phrase names (the whole phrase), and the head nouns
/// "that [noun]" may name them by.
fn iterated_objects(x: &str, b: &mut Builder) -> Option<(Sel, Vec<String>)> {
    let nouns = |s: &str| -> Vec<String> {
        s.split(' ')
            .filter(|w| head_noun(w).is_some() || subtype_word(w).is_some())
            .map(|w| w.to_string())
            .collect()
    };
    // "Look at the top five cards of your library. For each card, ...": each of those cards.
    if x == "card" {
        let dug = b
            .named
            .iter()
            .rev()
            .find(|(p, _)| p == super::dig_grammar::DIG_MARK)
            .map(|(_, s)| s.clone())?;
        return Some((dug, vec!["card".to_string()]));
    }
    // "for each of X target permanents", "for each of up to X target creatures".
    if let Some(r) = x.strip_prefix("of ").filter(|r| r.contains("target ")) {
        let saved = b.targets.len();
        let (sel, rest) = object_ref(r, b)?;
        if !rest.trim().is_empty() || !matches!(sel, Sel::Target(_)) {
            b.targets.truncate(saved);
            return None;
        }
        return Some((sel, nouns(r)));
    }
    // "for each of them", "for each of those creatures".
    if let Some(r) = x.strip_prefix("of ") {
        if let Some(Some((sel, rest))) = super::pronoun_groups::plural_object_ref(r, b) {
            if rest.trim().is_empty() {
                let noun = r
                    .strip_prefix("those ")
                    .map(|n| n.trim_end_matches('s').to_string());
                return Some((sel, noun.into_iter().collect()));
            }
        }
        return None;
    }
    // "for each permanent chosen this way": objects an earlier sentence chose.
    if let Some(noun) = x.strip_suffix(" chosen this way") {
        let (f, false, tail) = parse_object_phrase(noun)? else {
            return None;
        };
        if !tail.trim().is_empty() {
            return None;
        }
        let chosen = b
            .named
            .iter()
            .rev()
            .find(|(p, _)| p == "the chosen permanents" || p == &format!("the chosen {noun}s"))
            .map(|(_, s)| s.clone())?;
        let sel = Sel::All(Filter::and(vec![f, Filter::In(Box::new(chosen))]));
        return Some((sel, nouns(noun)));
    }
    // "for each creature destroyed this way".
    if let Some((sel, rest)) = super::value_results::this_way_sel(x, b) {
        if rest.trim().is_empty() {
            return Some((sel, nouns(x.split(" this way").next().unwrap_or(""))));
        }
        return None;
    }
    // "for each creature", "for each attacking creature without flying", "for each
    // creature your opponents control".
    let saved = b.targets.len();
    let (sel, rest) = object_ref(&format!("each {x}"), b)?;
    if !end(&rest).trim().is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    // Permanents: "for each card" means cards some earlier instruction is about ("Look at
    // the top five cards of your library. For each card, ..."), not every card.
    let Sel::All(f) = &sel else {
        return None;
    };
    if f.zone().is_none() && serde_json::to_string(f).is_ok_and(|j| j.contains("\"Card\"")) {
        return None;
    }
    Some((sel, nouns(x)))
}

/// "for each [objects], [instruction about it]".
fn for_each_object(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each ")?;
    if before_unless(l) {
        return None;
    }
    for (x, y) in comma_splits(r) {
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone(), b.named.len());
        let Some((sel, nouns)) = iterated_objects(x, b) else {
            b.targets.truncate(saved.0);
            continue;
        };
        if y.contains("target ") {
            b.targets.truncate(saved.0);
            return None;
        }
        b.it = Sel::Var(EACH);
        // "that player" is the object's controller ("its owner" if the instruction names
        // its owner).
        b.it_player = if y.contains("its owner") {
            PlayerRef::OwnerOf(Box::new(Sel::Var(EACH)))
        } else {
            PlayerRef::ControllerOf(Box::new(Sel::Var(EACH)))
        };
        for n in &nouns {
            b.named.push((format!("that {n}"), Sel::Var(EACH)));
        }
        let body = parse_clause(&they_as_its_controller(y), b);
        b.named.truncate(saved.3);
        b.it = saved.1;
        b.it_player = saved.2;
        let Some(body) = body else {
            b.targets.truncate(saved.0);
            return None;
        };
        return Some(each_object(sel, body));
    }
    None
}

/// Whether the effect mentions the variable.
fn mentions_var(e: &Effect, v: Var) -> bool {
    serde_json::to_string(e).is_ok_and(|j| j.contains(&format!("{{\"Var\":{v}}}")))
}

/// Whether the effect creates tokens (and so records them in `vars::CREATED`).
fn creates_tokens(e: &Effect) -> bool {
    serde_json::to_string(e).is_ok_and(|j| {
        ["\"CreateToken\"", "\"CreateTokenCopy\"", "\"CreateTokenWithPT\""]
            .iter()
            .any(|k| j.contains(k))
    })
}

/// All the tokens a "for each" instruction created.
const ALL_CREATED: Var = vars::USER + 7701;

/// The instruction performed for each of the objects.
fn each_object(sel: Sel, body: Effect) -> Effect {
    // "For each of them, create a token that's a copy of that creature": a token copy of
    // each of them, as one instruction (CR 707.2); later sentences name those tokens.
    if let Effect::CreateTokenCopy { of: Sel::Var(v), .. } = &body {
        if *v == EACH {
            let mut copy = body.clone();
            if let Effect::CreateTokenCopy { of, .. } = &mut copy {
                *of = sel.clone();
            }
            if !mentions_var(&copy, EACH) {
                return copy;
            }
        }
    }
    if !creates_tokens(&body) {
        return Effect::ForEach {
            sel,
            var: EACH,
            effect: Box::new(body),
        };
    }
    // The tokens created for each of them are all the tokens created ("those tokens"),
    // not only the last ones.
    let collect = Effect::Store {
        var: ALL_CREATED,
        sel: Sel::Union(vec![Sel::Var(ALL_CREATED), Sel::Var(vars::CREATED)]),
    };
    Effect::Seq(vec![
        Effect::Store {
            var: ALL_CREATED,
            sel: Sel::Union(vec![]),
        },
        Effect::ForEach {
            sel,
            var: EACH,
            effect: Box::new(Effect::seq(vec![body, collect])),
        },
        Effect::Store {
            var: vars::CREATED,
            sel: Sel::Var(ALL_CREATED),
        },
    ])
}

/// "they"/"their" for the controller the instruction names: "its controller sacrifices
/// it unless they pay X life".
fn they_as_its_controller(y: &str) -> String {
    let mut s = format!(" {y} ");
    if s.contains(" its controller ") || s.contains(" its owner ") {
        for (from, to) in [
            (" unless they pay ", " unless that player pays "),
            (" of their choice", " of that player's choice"),
        ] {
            s = s.replace(from, to);
        }
    }
    s.trim().to_string()
}

/// "For each color among permanents you control, add one mana of that color." (Bloom
/// Tender): one mana of each of those colors.
fn each_color_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each color among ")?;
    let noun = r.strip_suffix(", add one mana of that color")?;
    let (f, true, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).trim().is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::EachColorAmong(f),
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "iteration: for each color among [objects], add one mana of that color", priority: 100, parse: each_color_mana } }
inventory::submit! { EffectPattern { name: "iteration: for each [players], [instruction about that player]", priority: 980, parse: for_each_player } }
inventory::submit! { EffectPattern { name: "iteration: for each [objects], [instruction about it]", priority: 980, parse: for_each_object } }

#[cfg(test)]
mod tests {
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    /// Compiles `text` on a card of the given type line; the unsupported blocks.
    fn unsupported(text: &str, type_line: &str) -> Vec<String> {
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
        compile(text, &ctx).unsupported
    }

    /// `PROBE_TEXT="..." [PROBE_TYPE="Sorcery"] cargo test -p mtg-engine --lib probe --
    /// --nocapture`: prints how a text compiles (a development aid).
    #[test]
    fn probe() {
        let Ok(text) = std::env::var("PROBE_TEXT") else {
            return;
        };
        let tl = std::env::var("PROBE_TYPE").unwrap_or_else(|_| "Sorcery".into());
        for t in text.split(" || ") {
            let tl2 = TypeLine::parse(&tl);
            let ctx = CompileContext {
                card_name: "Probe",
                full_name: "Probe",
                type_line: &tl2,
                layout: crate::card::Layout::Normal,
                face_index: 0,
                keywords: &[],
                power: None,
                toughness: None,
            };
            let c = compile(t, &ctx);
            println!("== {t}\n  unsupported: {:?}", c.unsupported);
            if std::env::var("PROBE_AST").is_ok() {
                println!("{:#?}", c.abilities);
            }
        }
        let _ = unsupported;
    }
}
