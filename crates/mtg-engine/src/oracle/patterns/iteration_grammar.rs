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
    if !end(&rest).trim().is_empty() || b.targets.len() != saved {
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
