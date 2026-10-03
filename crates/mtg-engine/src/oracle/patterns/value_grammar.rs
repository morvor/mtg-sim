//! The value grammar (CR 107): the numbers instructions use, read compositionally.
//!
//! ```text
//! value  := term (("plus" | "minus") term)*
//! term   := "twice" term | "half" value ", rounded up|down" | atom
//! atom   := "the number of" count | "the total number of" count
//!         | ("the greatest" | "the highest" | "the least" | "the lowest") stat "among" objects
//!         | "the total" stat "of" objects
//!         | referent "'s" stat | "the" stat "of" referent
//!         | player-possessive "life total"
//!         | (the core phrases of `statics::parse_value_phrase_core`, numbers, "X")
//! count  := players ("opponents you have", "opponents who control an artifact")
//!         | counters ("experience counters you have", "+1/+1 counters among creatures
//!           you control", "rad counters among players")
//!         | kinds "among" objects ("card types", "colors", "basic land types",
//!           "creature types", "different mana values", "different powers", ...)
//!         | objects (an object phrase with zone, controller and owner qualifiers:
//!           "cards in their hand", "cards named ~ in all graveyards", "creatures
//!           target opponent controls")
//! ```
//!
//! Counted objects are counted as the instruction is performed (CR 608.2h). A negative
//! result of a calculation is treated as 0 where the instruction uses it (CR 107.1b).

use super::oracle_hardening_referents::{is_no_player_referent, is_no_referent};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::types::CounterKind;

thread_local! {
    static X_DEFINED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static HALF_ROUNDING: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// How "half" rounds where the text says so once for the whole ability ("Round up each
/// time.", CR 107.1a): Some(true) for up.
pub fn half_rounding() -> Option<bool> {
    HALF_ROUNDING.with(|c| c.get())
}

/// Runs `f` with [`half_rounding`] set.
pub fn with_half_rounding<T>(up: bool, f: impl FnOnce() -> T) -> T {
    let saved = HALF_ROUNDING.with(|c| c.replace(Some(up)));
    let out = f();
    HALF_ROUNDING.with(|c| c.set(saved));
    out
}

/// Whether the text being compiled defines X: the ability's cost has an X in it (CR
/// 107.3a, 107.3k), or the sentence says "where X is ..." (CR 107.3c). Instructions that
/// only make sense with a known X ("look at the top X cards of your library") accept X
/// then.
pub fn x_defined() -> bool {
    X_DEFINED.with(|c| c.get())
}

/// Runs `f` with [`x_defined`] set to `defined` (restored afterwards).
pub fn with_x_defined<T>(defined: bool, f: impl FnOnce() -> T) -> T {
    let saved = X_DEFINED.with(|c| c.replace(defined));
    let out = f();
    X_DEFINED.with(|c| c.set(saved));
    out
}

/// Whether an activation cost has an X in it: "{X}", "Remove X counters", "Tap X
/// untapped creatures you control", "Pay X life".
pub fn cost_has_x(cost: &str) -> bool {
    let c = cost.to_lowercase();
    c.contains("{x}") || c.split(|ch: char| !ch.is_alphanumeric()).any(|w| w == "x")
}

/// Parses a value phrase at the start of `s`; returns the value and the rest of the text.
pub fn parse_value(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let s = s.trim_start();
    let (mut v, mut rest) = term(s, b)?;
    loop {
        let (minus, r) = if let Some(r) = rest.trim_start().strip_prefix("plus ") {
            (false, r.to_string())
        } else if let Some(r) = rest.trim_start().strip_prefix("minus ") {
            (true, r.to_string())
        } else {
            break;
        };
        let saved = b.targets.len();
        let Some((w, r2)) = term(&r, b) else {
            b.targets.truncate(saved);
            break;
        };
        v = if minus {
            Value::Diff(Box::new(v), Box::new(w))
        } else {
            match v {
                Value::Sum(mut xs) => {
                    xs.push(w);
                    Value::Sum(xs)
                }
                v => Value::Sum(vec![v, w]),
            }
        };
        rest = r2;
    }
    Some((v, rest))
}

/// Whether `rest` ends a word (so a prefix match is a whole phrase).
fn word_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.', ';', '"'])
}

fn term(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(r) = s.strip_prefix("twice ") {
        let (v, rest) = term(r, b)?;
        return Some((Value::Mul(Box::new(Value::c(2)), Box::new(v)), rest));
    }
    // CR 107.1a: "half ..., rounded up/down" (the text says how to round).
    if let Some(r) = s.strip_prefix("half ") {
        let (v, rest) = parse_value(r, b)?;
        let (up, rest) = if let Some(x) = rest.strip_prefix(", rounded up") {
            (true, x)
        } else if let Some(x) = rest.strip_prefix(", rounded down") {
            (false, x)
        } else {
            (half_rounding()?, rest.as_str())
        };
        return Some((Value::Div(Box::new(v), 2, up), rest.to_string()));
    }
    atom(s, b)
}

fn atom(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let saved = b.targets.len();
    let saved_player = b.it_player.clone();
    let ext = atom_ext(s, b);
    if let Some((v, rest)) = ext {
        if plausible_end(&rest) {
            return Some((v, rest));
        }
        // A partial reading: the core phrases may read more of it.
        let ext_targets = b.targets.split_off(saved);
        let ext_player = std::mem::replace(&mut b.it_player, saved_player.clone());
        if let Some(core) = crate::oracle::statics::parse_value_phrase_core(s, b) {
            if plausible_end(&core.1) {
                return Some(core);
            }
        }
        b.targets.truncate(saved);
        b.targets.extend(ext_targets);
        b.it_player = ext_player;
        return Some((v, rest));
    }
    b.targets.truncate(saved);
    b.it_player = saved_player;
    crate::oracle::statics::parse_value_phrase_core(s, b)
}

/// Whether the text after a value phrase plausibly continues the sentence (rather than
/// being an unread part of the phrase).
fn plausible_end(rest: &str) -> bool {
    let t = rest.trim_start();
    t.is_empty()
        || t.starts_with([',', '.', ';', '"'])
        || [
            "plus ",
            "minus ",
            "as you ",
            "and ",
            "to ",
            "then ",
            "until ",
            "for ",
            "unless ",
            "from ",
            "this turn",
            "instead",
            "in addition",
            "if ",
            "on ",
            "among ",
            "divided ",
        ]
        .iter()
        .any(|p| t.starts_with(p))
}

fn atom_ext(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = super::value_results::atom_ext(s, b) {
        return Some(v);
    }
    // "the number of cards you've drawn this turn" (Fractal Anomaly).
    if let Some(r) = s.strip_prefix("the number of cards you've drawn this turn") {
        if word_end(r) {
            return Some((Value::CardsDrawnThisTurn(PlayerRef::You), r.to_string()));
        }
    }
    if let Some(r) = s
        .strip_prefix("the total number of ")
        .or_else(|| s.strip_prefix("the number of "))
    {
        return count(r, b);
    }
    for (p, op) in [
        ("the greatest ", AggOp::Max),
        ("the highest ", AggOp::Max),
        ("the least ", AggOp::Min),
        ("the lowest ", AggOp::Min),
        ("the smallest ", AggOp::Min),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return extreme(op, r, b);
        }
    }
    if let Some(r) = s.strip_prefix("the total ") {
        let (stat, r) = stat_word(r)?;
        let r = r.strip_prefix(" of ")?;
        let (f, rest) = objects(r, b)?;
        return Some((
            Value::Aggregate(AggOp::Sum, stat, Box::new(Sel::All(f))),
            rest,
        ));
    }
    // "the power of target creature you control", "the mana value of that spell".
    if let Some(r) = s.strip_prefix("the ") {
        if let Some((stat, r)) = stat_word(r) {
            if let Some(r) = r.strip_prefix(" of ") {
                if let Some((sel, rest)) = referent(r, b) {
                    return Some((of_referent(stat, sel), rest));
                }
            }
        }
    }
    // "target creature's power", "that creature card's toughness", "the card's mana
    // value".
    if let Some((who, r)) = split_possessive(s) {
        if let Some((stat, rest)) = stat_word(r) {
            if word_end(rest) {
                let (sel, tail) = referent(who, b)?;
                if !tail.trim().is_empty() {
                    return None;
                }
                return Some((of_referent(stat, sel), rest.to_string()));
            }
        }
        if let Some(rest) = r.strip_prefix("life total") {
            if word_end(rest) {
                let (p, tail) = player_word(who, b)?;
                if !tail.trim().is_empty() {
                    return None;
                }
                return Some((Value::LifeTotal(p), rest.to_string()));
            }
        }
    }
    if let Some(rest) = s.strip_prefix("their life total") {
        let p = their(b)?;
        return Some((Value::LifeTotal(p), rest.to_string()));
    }
    None
}

/// "X's [rest]": the possessor and the rest, at the first possessive.
fn split_possessive(s: &str) -> Option<(&str, &str)> {
    let i = s.find("'s ")?;
    Some((&s[..i], &s[i + 3..]))
}

fn stat_word(s: &str) -> Option<(Stat, &str)> {
    for (p, st) in [
        ("power and/or toughness", Stat::PowerOrToughness),
        ("power", Stat::Power),
        ("toughness", Stat::Toughness),
        ("mana value", Stat::ManaValue),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if word_end(r) {
                return Some((st, r));
            }
        }
    }
    None
}

fn of_referent(stat: Stat, sel: Sel) -> Value {
    let sel = Box::new(sel);
    match stat {
        Stat::Power => Value::PowerOf(sel),
        Stat::Toughness => Value::ToughnessOf(sel),
        Stat::ManaValue => Value::ManaValueOf(sel),
        stat => Value::Aggregate(AggOp::Max, stat, sel),
    }
}

/// An object the text refers to: "~", "it", "that creature", "target creature you
/// control", "the card" (what "it" names), "that creature card".
fn referent(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim_start();
    // A phrase the text gave a meaning of its own ("that card" for the card drawn and
    // revealed, `dig_grammar::draw_and_reveal`).
    for (p, sel) in &b.named {
        if let Some(rest) = s.strip_prefix(p.as_str()) {
            if p == "that card" && (word_end(rest) || rest.starts_with('\'')) {
                return Some((sel.clone(), rest.to_string()));
            }
        }
    }
    for p in [
        "the card",
        "that creature card",
        "the artifact",
        "the permanent",
        "the spell",
        "that card",
        "that spell",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) || rest.starts_with('\'') {
                if is_no_referent(&b.it) || matches!(b.it, Sel::This) {
                    return None;
                }
                return Some((b.it.clone(), rest.to_string()));
            }
        }
    }
    crate::oracle::effects::object_ref(s, b)
}

/// "they"/"their": the player the text is about.
fn their(b: &Builder) -> Option<PlayerRef> {
    if is_no_player_referent(&b.it_player) {
        return None;
    }
    Some(b.it_player.clone())
}

/// A player named by a word or phrase: "you", "they", "that player", "target opponent"
/// (adding the target), "defending player", "its controller", "that creature's
/// controller". Returns the player and the rest.
fn player_word<'a>(s: &'a str, b: &mut Builder) -> Option<(PlayerRef, &'a str)> {
    let s = s.trim_start();
    let fixed: [(&str, Option<PlayerRef>); 4] = [
        ("you", Some(PlayerRef::You)),
        ("defending player", Some(PlayerRef::DefendingPlayer)),
        ("they", their(b)),
        ("that player", their(b)),
    ];
    for (p, r) in fixed {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) || rest.starts_with('\'') {
                return Some((r?, rest));
            }
        }
    }
    for p in [
        "its controller",
        "that creature's controller",
        "that permanent's controller",
        "their controller",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) || rest.starts_with('\'') {
                if is_no_referent(&b.it) {
                    return None;
                }
                return Some((PlayerRef::ControllerOf(Box::new(b.it.clone())), rest));
            }
        }
    }
    for (p, pf) in [
        ("another target player", PlayerFilter::Any),
        ("another target opponent", PlayerFilter::Opponent),
        ("target player", PlayerFilter::Any),
        ("target opponent", PlayerFilter::Opponent),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) || rest.starts_with('\'') {
                let it = b.it.clone();
                let slot = b.add_target(TargetSpec::player(pf, p), p);
                b.it = it;
                b.it_player = PlayerRef::Target(slot);
                return Some((PlayerRef::Target(slot), rest));
            }
        }
    }
    None
}

/// "your", "their", "that player's", "target opponent's", "defending player's", "its
/// controller's", "that creature's controller's".
fn player_possessive<'a>(s: &'a str, b: &mut Builder) -> Option<(PlayerRef, &'a str)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("your ") {
        return Some((PlayerRef::You, r));
    }
    if let Some(r) = s.strip_prefix("their ") {
        return Some((their(b)?, r));
    }
    let (p, rest) = player_word(s, b)?;
    let rest = rest.strip_prefix("'s ")?;
    Some((p, rest))
}

/// Objects controlled by the player(s) a reference names.
pub fn controlled_by(p: &PlayerRef) -> Filter {
    match p {
        PlayerRef::You => Filter::ControlledBy(PlayerRel::You),
        PlayerRef::Target(s) => Filter::ControlledBy(PlayerRel::Target(*s)),
        PlayerRef::TriggerPlayer => Filter::ControlledBy(PlayerRel::TriggerPlayer),
        PlayerRef::Iterated => Filter::ControlledBy(PlayerRel::Iterated),
        PlayerRef::DefendingPlayer => Filter::ControlledBy(PlayerRel::Defending),
        PlayerRef::EachOpponent => Filter::ControlledBy(PlayerRel::Opponent),
        PlayerRef::EachPlayer => Filter::Any,
        p => Filter::ControlledByPlayer(Box::new(p.clone())),
    }
}

/// Objects owned by the player(s) a reference names.
pub fn owned_by(p: &PlayerRef) -> Filter {
    match p {
        PlayerRef::You => Filter::OwnedBy(PlayerRel::You),
        PlayerRef::Target(s) => Filter::OwnedBy(PlayerRel::Target(*s)),
        PlayerRef::TriggerPlayer => Filter::OwnedBy(PlayerRel::TriggerPlayer),
        PlayerRef::Iterated => Filter::OwnedBy(PlayerRel::Iterated),
        PlayerRef::DefendingPlayer => Filter::OwnedBy(PlayerRel::Defending),
        PlayerRef::EachOpponent => Filter::OwnedBy(PlayerRel::Opponent),
        PlayerRef::EachPlayer => Filter::Any,
        p => Filter::OwnedByPlayer(Box::new(p.clone())),
    }
}

fn zone_word(s: &str) -> Option<(ZoneKind, &str)> {
    for (p, z) in [
        ("hand", ZoneKind::Hand),
        ("graveyard", ZoneKind::Graveyard),
        ("library", ZoneKind::Library),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if word_end(r) {
                return Some((z, r));
            }
        }
    }
    None
}

/// Qualifiers after an object phrase that the shared phrase parser leaves: "in their
/// hand", "in each graveyard", "named ~", "[player] controls", "attacking you", "you
/// own in exile" (already read), "attached to ~", "with the same name as that spell".
fn suffix<'a>(t: &'a str, b: &mut Builder) -> Option<(Filter, &'a str)> {
    // "Auras you control that are attached to creatures", "that's attached to a
    // creature": attached to an object of that kind.
    for p in ["that's attached to ", "that are attached to ", "attached to "] {
        if let Some(r) = t.strip_prefix(p) {
            let r2 = r
                .strip_prefix("a ")
                .or_else(|| r.strip_prefix("an "))
                .unwrap_or(r);
            if let Some((f, false, rest)) = parse_object_phrase(r2) {
                if matches!(f, Filter::Type(_) | Filter::Permanent) {
                    let n = rest.len();
                    return Some((
                        Filter::AttachedToAnyOf(Box::new(Sel::All(f))),
                        &t[t.len() - n..],
                    ));
                }
            }
        }
    }
    // "attached to it", "attached to ~", "attached to them" (a player).
    if let Some(r) = t.strip_prefix("attached to ") {
        if let Some(r2) = r.strip_prefix("them") {
            if word_end(r2) {
                let p = their(b)?;
                return Some((Filter::AttachedToAnyOf(Box::new(Sel::Players(p))), r2));
            }
        }
        let (sel, rest) = referent(r, b)?;
        let n = rest.len();
        return Some((Filter::AttachedToAnyOf(Box::new(sel)), &t[t.len() - n..]));
    }
    // "exiled with ~" (CR 607.2a: the cards its linked ability exiled); on a card with
    // delve, the cards exiled to pay for it (CR 702.66a).
    for p in ["exiled with ~", "exiled with it"] {
        if let Some(r) = t.strip_prefix(p) {
            if b.ctx.keywords.iter().any(|k| k.eq_ignore_ascii_case("delve")) {
                return word_end(r).then(|| {
                    (
                        Filter::and(vec![
                            Filter::InZone(ZoneKind::Exile),
                            Filter::Custom(crate::kw::delve::EXILED_WITH_IT.into()),
                        ]),
                        r,
                    )
                });
            }
            if word_end(r) && (p.ends_with('~') || matches!(b.it, Sel::This)) {
                return Some((
                    Filter::and(vec![
                        Filter::In(Box::new(Sel::Linked)),
                        Filter::InZone(ZoneKind::Exile),
                    ]),
                    r,
                ));
            }
        }
    }
    // "permanents you own that your opponents control".
    for (p, rel) in [
        ("that your opponents control", PlayerRel::Opponent),
        ("that an opponent controls", PlayerRel::Opponent),
        ("that you control", PlayerRel::You),
        ("that you don't control", PlayerRel::NotYou),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if word_end(r) {
                return Some((Filter::ControlledBy(rel), r));
            }
        }
    }
    // "that are instant cards, sorcery cards, and/or have an Adventure", "that are
    // Oozes or are named ~": either description.
    if let Some(r) = t.strip_prefix("that are ") {
        return relative_list(r, b);
    }
    // "that shares a creature type with it" (it shares one with itself if it has any;
    // "other ..." excludes it, see `objects`).
    for p in [
        "that shares a creature type with it",
        "that share a creature type with it",
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if word_end(r) && !is_no_referent(&b.it) {
                return Some((Filter::SharesCreatureType(Box::new(b.it.clone())), r));
            }
        }
    }
    if let Some(r) = t.strip_prefix("in ") {
        if let Some(r2) = r
            .strip_prefix("each graveyard")
            .or_else(|| r.strip_prefix("all graveyards"))
            .or_else(|| r.strip_prefix("graveyards"))
        {
            return Some((Filter::InZone(ZoneKind::Graveyard), r2));
        }
        let (p, r2) = player_possessive(r, b)?;
        let (z, r3) = zone_word(r2)?;
        return Some((Filter::and(vec![Filter::InZone(z), owned_by(&p)]), r3));
    }
    if let Some(r) = t.strip_prefix("named ~") {
        if word_end(r) {
            return Some((named_this(b)?, r));
        }
        return None;
    }
    if let Some(r) = t.strip_prefix("attacking ") {
        if let Some(r2) = r.strip_prefix("you") {
            if word_end(r2) {
                return Some((Filter::AttackingPlayer(PlayerRel::You), r2));
            }
        }
        if let Some(r2) = r.strip_prefix("them") {
            if word_end(r2) {
                let p = their(b)?;
                let rel = match p {
                    PlayerRef::Iterated => PlayerRel::Iterated,
                    PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
                    PlayerRef::Target(s) => PlayerRel::Target(s),
                    _ => return None,
                };
                return Some((Filter::AttackingPlayer(rel), r2));
            }
        }
        return None;
    }
    if let Some(r) = t.strip_prefix("with the same name as ") {
        let (sel, rest) = referent(r, b)?;
        let n = rest.len();
        // "with the same name as another permanent": another than the object itself
        // (CR 201.2), not than the source.
        if r.trim_start().starts_with("another ") {
            let f = crate::kw::basic_effects::same_name_as_another(&sel)?;
            return Some((f, &t[t.len() - n..]));
        }
        return Some((Filter::SameNameAs(Box::new(sel)), &t[t.len() - n..]));
    }
    if let Some(r) = t.strip_prefix("on the battlefield") {
        if word_end(r) {
            return Some((Filter::InZone(ZoneKind::Battlefield), r));
        }
    }
    // "creatures it's blocking": the creatures the source blocks.
    if let Some(r) = t.strip_prefix("it's blocking") {
        if word_end(r) && matches!(b.it, Sel::This) {
            return Some((Filter::BlockedBySource, r));
        }
        return None;
    }
    // "with base power or toughness 1", "with base power and toughness 2/2" (CR 208.4b).
    if let Some(r) = t.strip_prefix("with base power or toughness ") {
        let (n, rest) = parse_number(r)?;
        let n = n.as_const()?;
        return Some((
            Filter::Custom(format!("{}{n}", crate::kw::value_counts::BASE_POWER_OR_TOUGHNESS).into()),
            rest,
        ));
    }
    if let Some(r) = t.strip_prefix("with base power and toughness ") {
        let (pt, rest) = split_word(r);
        let (p, q) = pt.split_once('/')?;
        let (p, q): (i32, i32) = (p.parse().ok()?, q.parse().ok()?);
        return Some((
            Filter::Custom(
                format!("{}{p}/{q}", crate::kw::value_counts::BASE_POWER_AND_TOUGHNESS).into(),
            ),
            rest,
        ));
    }
    // "with power less than 0", "with toughness greater than 3".
    for (p, cmp, power) in [
        ("with power less than ", Cmp::Lt, true),
        ("with power greater than ", Cmp::Gt, true),
        ("with toughness less than ", Cmp::Lt, false),
        ("with toughness greater than ", Cmp::Gt, false),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            let (n, rest) = parse_number(r)?;
            n.as_const()?;
            let f = if power {
                Filter::Power(cmp, Box::new(n))
            } else {
                Filter::Toughness(cmp, Box::new(n))
            };
            return Some((f, rest));
        }
    }
    // "lands you control named Wastes": a card name (CR 201.2).
    if let Some(r) = t.strip_prefix("named ") {
        let (name, rest) = match r.find([',', '.']) {
            Some(i) => (&r[..i], &r[i..]),
            None => (r, ""),
        };
        let title: Vec<String> = name
            .split(' ')
            .map(|w| {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default()
            })
            .collect();
        let title = title.join(" ");
        if crate::card::CardDb::global().get(&title).is_some() {
            return Some((Filter::Named(title.into()), rest));
        }
        return None;
    }
    // "[player] controls", "[player] owns".
    let (p, r) = player_word(t, b)?;
    for (verb, owns) in [(" owns", true), (" own", true)] {
        if let Some(r2) = r.strip_prefix(verb) {
            if word_end(r2) && owns {
                return Some((owned_by(&p), r2));
            }
        }
    }
    let r = r
        .strip_prefix(" controls")
        .or_else(|| r.strip_prefix(" control"))?;
    if !word_end(r) {
        return None;
    }
    Some((controlled_by(&p), r))
}

/// "named ~": the card's name (CR 201.2).
fn named_this(b: &Builder) -> Option<Filter> {
    // Without the card (a static ability read on its own, maybe granted to another
    // object), "~" can't be told apart from the object it's granted to.
    (!b.ctx.card_name.is_empty()).then(|| Filter::Named(b.ctx.card_name.into()))
}

/// "instant cards, sorcery cards, and/or have an Adventure", "Oozes or are named ~":
/// alternatives after "that are" (to the end of the phrase).
fn relative_list<'a>(r: &'a str, b: &mut Builder) -> Option<(Filter, &'a str)> {
    let (body, rest) = match r.find(['.', ';']) {
        Some(i) => (&r[..i], &r[i..]),
        None => (r, ""),
    };
    let norm = body
        .replace(", and/or ", ", ")
        .replace(" and/or ", ", ")
        .replace(", or ", ", ")
        .replace(" or ", ", ");
    let mut alts = Vec::new();
    for item in norm.split(", ") {
        let item = item.trim();
        let item = item.strip_prefix("are ").unwrap_or(item);
        if item == "have an adventure" {
            alts.push(Filter::Custom(crate::adventure::HAS_ADVENTURE.into()));
            continue;
        }
        if item == "named ~" {
            alts.push(named_this(b)?);
            continue;
        }
        let (f, _, tail) = parse_object_phrase(item)?;
        if !tail.trim().is_empty() {
            return None;
        }
        alts.push(f);
    }
    if alts.len() < 2 {
        return None;
    }
    Some((Filter::Or(alts), rest))
}

/// An object phrase with the qualifiers above, in any order. Returns the filter and the
/// rest.
pub fn objects(s: &str, b: &mut Builder) -> Option<(Filter, String)> {
    // "the milled cards" after milling (`dig_grammar`).
    if let Some(x) = super::dig_grammar::dug_objects(s, b) {
        return Some(x);
    }
    // CR 702.62b: a "suspended" card is in exile with suspend and a time counter on it.
    if let Some(r) = s.strip_prefix("suspended ") {
        let (f, rest) = objects(r, b)?;
        let suspended = Filter::and(vec![
            Filter::InZone(ZoneKind::Exile),
            Filter::HasKeyword(crate::keywords::KeywordKind::Suspend),
            Filter::HasCounter(Some(crate::types::counters::TIME.into())),
        ]);
        return Some((Filter::and(vec![suspended, f]), rest));
    }
    // "Mountain and red card": each card of either description.
    {
        let words: Vec<&str> = s.splitn(4, ' ').collect();
        if words.len() >= 3
            && words[1] == "and"
            && matches!(head_noun(words[0]), Some(Filter::Subtype(_) | Filter::Type(_)))
            && crate::types::Color::from_word(words[2]).is_some()
        {
            let rest = words.get(3).copied().unwrap_or("");
            let (f2, r2) = objects(&format!("{} {rest}", words[2]), b)?;
            let (f1, _, _) = parse_object_phrase(words[0])?;
            // The second description's head ("card") and qualifiers apply to both.
            let Filter::And(parts) = &f2 else {
                return None;
            };
            let color = parts.first()?.clone();
            if !matches!(color, Filter::Color(_)) {
                return None;
            }
            let mut v = vec![Filter::Or(vec![f1, color])];
            v.extend(parts[1..].iter().cloned());
            return Some((Filter::and(v), r2));
        }
    }
    // "Aura and Equipment attached to it": either kind (a union of two nouns).
    let owned;
    let s = match s.split_once(' ') {
        Some((w1, r)) => match r.split_once(' ') {
            Some(("and", r2))
                if head_noun(w1).is_some()
                    && head_noun(r2.split(' ').next().unwrap_or("")).is_some() =>
            {
                owned = format!("{w1} or {r2}");
                owned.as_str()
            }
            _ => s,
        },
        None => s,
    };
    let (f, _, rest) = super::statics::object_phrase(s)?;
    let mut parts = vec![f];
    let mut rest = rest.to_string();
    loop {
        let t = rest.trim_start().to_string();
        if t.is_empty() {
            rest = t;
            break;
        }
        let saved = b.targets.len();
        let saved_player = b.it_player.clone();
        if let Some((f, r)) = suffix(&t, b) {
            parts.push(f);
            rest = r.to_string();
            continue;
        }
        b.targets.truncate(saved);
        b.it_player = saved_player;
        // The shared parser's qualifiers after one of ours ("creatures that player
        // controls with flying"): read after a stand-in noun.
        if t.starts_with("with ") || t.starts_with("without ") || t.starts_with("that ") {
            let probe = format!("elf {t}");
            if let Some((Filter::And(mut v), _, r)) = super::statics::object_phrase(&probe) {
                if r.len() < t.len() && matches!(v.first(), Some(Filter::Subtype(s)) if s == "Elf")
                {
                    v.remove(0);
                    parts.extend(v);
                    rest = r.to_string();
                    continue;
                }
            }
        }
        break;
    }
    // "other creatures with the same name as that creature": other than that creature,
    // not other than the source.
    let referent = parts.iter().find_map(|p| match p {
        Filter::SameNameAs(sel) | Filter::SharesCreatureType(sel) if !matches!(**sel, Sel::This) => {
            Some(sel.clone())
        }
        _ => None,
    });
    if let Some(sel) = referent {
        parts = parts
            .into_iter()
            .flat_map(|p| match p {
                Filter::And(v) => v,
                p => vec![p],
            })
            .map(|p| match p {
                Filter::Other => Filter::not(Filter::In(sel.clone())),
                p => p,
            })
            .collect();
    }
    Some((Filter::and(parts), rest))
}

/// "[thing] among [objects]" kinds of values.
fn among_kind(s: &str) -> Option<(Among, &str)> {
    for (p, k) in [
        ("card types", Among::CardTypes),
        ("card type", Among::CardTypes),
        ("permanent types", Among::PermanentTypes),
        ("permanent type", Among::PermanentTypes),
        ("creature types", Among::CreatureTypes),
        ("creature type", Among::CreatureTypes),
        ("basic land types", Among::BasicLandTypes),
        ("basic land type", Among::BasicLandTypes),
        ("colors", Among::Colors),
        ("color", Among::Colors),
        ("different mana values", Among::ManaValues),
        ("different mana value", Among::ManaValues),
        ("mana values", Among::ManaValues),
        ("different mana costs", Among::ManaCosts),
        ("different mana cost", Among::ManaCosts),
        ("different powers", Among::Powers),
        ("different power", Among::Powers),
        ("different names", Among::Names),
        ("different kinds of counters", Among::CounterKinds),
        ("kinds of counters", Among::CounterKinds),
        ("kind of counter", Among::CounterKinds),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if word_end(r) {
                return Some((k, r));
            }
        }
    }
    None
}

/// "[kind] counter(s)" / "counter(s)": the kind (None for any kind) and the rest.
fn counters_word(s: &str) -> Option<(Option<CounterKind>, &str)> {
    for p in ["counters", "counter"] {
        if let Some(r) = s.strip_prefix(p) {
            if word_end(r) {
                return Some((None, r));
            }
        }
    }
    let (k, rest) = crate::oracle::costs::counter_kind(s)?;
    let rest = rest.trim_start();
    for p in ["counters", "counter"] {
        if let Some(r) = rest.strip_prefix(p) {
            if word_end(r) {
                return Some((Some(k), r));
            }
        }
    }
    None
}

/// Opponents/players described by a relative clause: "opponents you have", "opponents
/// who control an artifact", "your opponents with four or more cards in hand",
/// "opponents who control more lands than you".
fn players(s: &str) -> Option<(PlayerFilter, &str)> {
    let (base, r) = if let Some(r) = s
        .strip_prefix("your opponents ")
        .or_else(|| s.strip_prefix("opponents "))
        .or_else(|| s.strip_prefix("opponent "))
    {
        (PlayerFilter::Opponent, r)
    } else if let Some(r) = s
        .strip_prefix("players ")
        .or_else(|| s.strip_prefix("player "))
    {
        (PlayerFilter::Any, r)
    } else {
        return None;
    };
    if let Some(rest) = r.strip_prefix("you have") {
        if matches!(base, PlayerFilter::Opponent) && word_end(rest) {
            return Some((base, rest));
        }
        return None;
    }
    // The relative clause runs to the end of the phrase.
    let (pred, rest) = match r.find([',', '.']) {
        Some(i) => (&r[..i], &r[i..]),
        None => (r, ""),
    };
    let pred = if let Some(p) = pred.strip_prefix("who ") {
        player_clause(p)?
    } else if let Some(p) = pred.strip_prefix("with ") {
        player_clause(&format!("has {p}"))?
    } else {
        return None;
    };
    Some((PlayerFilter::And(vec![base, pred]), rest))
}

/// "control an artifact", "controls more lands than you", "have three or more poison
/// counters", "has four or more cards in hand".
pub(crate) fn player_clause(p: &str) -> Option<PlayerFilter> {
    let p = p.trim();
    // Verb agreement: "who control", "who have".
    let p = if let Some(r) = p.strip_prefix("control ") {
        format!("controls {r}")
    } else if let Some(r) = p.strip_prefix("have ") {
        format!("has {r}")
    } else {
        p.to_string()
    };
    // "controls more [objects] than you" / "controls fewer [objects] than you".
    if let Some(r) = p.strip_prefix("controls ") {
        for (w, cmp) in [("more ", Cmp::Gt), ("fewer ", Cmp::Lt)] {
            if let Some(x) = r.strip_prefix(w).and_then(|x| x.strip_suffix(" than you")) {
                let (f, _, tail) = parse_object_phrase(x)?;
                if !tail.trim().is_empty() {
                    return None;
                }
                return Some(PlayerFilter::Controls(
                    Box::new(f.clone()),
                    cmp,
                    Box::new(Value::Count(f.you_control())),
                ));
            }
        }
        // "controls at least two more lands than you".
        if let Some(x) = r
            .strip_prefix("at least ")
            .and_then(|x| x.strip_suffix(" than you"))
        {
            let (n, x) = parse_number(x)?;
            n.as_const()?;
            let x = x.trim_start().strip_prefix("more ")?;
            let (f, _, tail) = parse_object_phrase(x)?;
            if !tail.trim().is_empty() {
                return None;
            }
            return Some(PlayerFilter::Controls(
                Box::new(f.clone()),
                Cmp::Ge,
                Box::new(Value::Sum(vec![Value::Count(f.you_control()), n])),
            ));
        }
        // "controls a creature with power 4 or greater".
        if let Some(x) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) {
            let (f, _, tail) = super::statics::object_phrase(x)?;
            if tail.trim().is_empty() {
                return Some(PlayerFilter::Controls(
                    Box::new(f),
                    Cmp::Ge,
                    Box::new(Value::c(1)),
                ));
            }
        }
    }
    // "has three or more poison counters".
    if let Some(r) = p.strip_prefix("has ") {
        if let Some((cmp, n, tail)) = amount(r) {
            if let Some((Some(k), t2)) = counters_word(tail.trim_start()) {
                if t2.trim().is_empty() {
                    return Some(PlayerFilter::Counters(k, cmp, Box::new(n)));
                }
            }
        }
    }
    super::statics_conditions::player_predicate(&p).or_else(|| {
        // "has more cards in their hand than you" (other comparisons with you are read
        // where conditions are).
        p.contains(" in their hand ")
            .then(|| super::choice_grammar_players::compared_with_you(&format!("who {p}")))
            .flatten()
    })
}

/// "three or more", "N or fewer", "no".
fn amount(s: &str) -> Option<(Cmp, Value, &str)> {
    let (n, r) = parse_number(s)?;
    if matches!(n, Value::X) {
        return None;
    }
    let r = r.trim_start();
    if let Some(x) = r.strip_prefix("or more") {
        return Some((Cmp::Ge, n, x));
    }
    if let Some(x) = r
        .strip_prefix("or fewer")
        .or_else(|| r.strip_prefix("or less"))
    {
        return Some((Cmp::Le, n, x));
    }
    None
}

/// After "the number of": what's counted.
fn count(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = super::value_results::count_ext(r, b) {
        return Some(v);
    }
    // "counters removed this way": an amount chosen for the cost (see `cost_parts`).
    if let Some(v) = super::cost_parts::paid_this_way_prefix(r) {
        return Some(v);
    }
    // "cards revealed this way", "creature cards exiled this way".
    if let Some(v) = super::hand_graveyard_grammar::count_phrase(r, b) {
        return Some(v);
    }
    // "Mountains returned this way".
    if let Some(v) = super::zone_move_grammar::returned_this_way(r, b) {
        return Some(v);
    }
    // "creatures that convoked ~" (CR 702.51c).
    for p in ["creatures that convoked ~", "creatures that convoked it"] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest) {
                return Some((
                    Value::Custom(crate::kw::convoke::CONVOKED_COUNT.into()),
                    rest.to_string(),
                ));
            }
        }
    }
    // "times ~ was kicked", "time it was kicked" (CR 702.33): the source's kicker count.
    for p in [
        "times ~ was kicked",
        "time ~ was kicked",
        "times it was kicked",
        "time it was kicked",
        "time he was kicked",
        "time she was kicked",
        "times he was kicked",
        "times she was kicked",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest)
                && (p.contains('~') || matches!(b.it, Sel::This) || is_no_referent(&b.it))
            {
                return Some((Value::TimesKicked, rest.to_string()));
            }
            return None;
        }
    }
    // "the number of cards looked at while scrying this way" in a "whenever you scry"
    // trigger: how many cards the player looked at (CR 701.22a, 701.22d).
    if b.in_trigger {
        for p in [
            "cards looked at while scrying this way",
            "card looked at while scrying this way",
        ] {
            if let Some(rest) = r.strip_prefix(p) {
                if word_end(rest) {
                    return Some((Value::EventAmount, rest.to_string()));
                }
            }
        }
    }
    // "for each 1 life you lost" in a "whenever you lose life" trigger: the amount of
    // that life loss (or gain), counted in groups of N.
    if b.in_trigger {
        if let Some((n, x)) = parse_number(r) {
            if let Some(k) = n.as_const().filter(|k| *k > 0) {
                for p in ["life you lost", "life you gained"] {
                    if let Some(rest) = x.trim_start().strip_prefix(p) {
                        if word_end(rest) {
                            let v = if k == 1 {
                                Value::EventAmount
                            } else {
                                Value::Div(Box::new(Value::EventAmount), k, false)
                            };
                            return Some((v, rest.to_string()));
                        }
                    }
                }
            }
        }
    }
    // "white mana symbols in its mana cost" (CR 107.4e: hybrid symbols of the color
    // count).
    {
        let (w, x) = split_word(r);
        if let Some(color) = crate::types::Color::from_word(w) {
            let x = x.trim_start();
            if let Some(y) = x
                .strip_prefix("mana symbols in ")
                .or_else(|| x.strip_prefix("mana symbol in "))
            {
                let (sel, rest) = if let Some(z) = y.strip_prefix("the mana cost of ") {
                    referent(z, b)?
                } else {
                    let (who, z) = split_possessive(y)
                        .or_else(|| y.strip_prefix("its ").map(|z| ("it", z)))?;
                    let rest = z.strip_prefix("mana cost")?;
                    let (sel, t) = referent(who, b)?;
                    if !t.trim().is_empty() {
                        return None;
                    }
                    (sel, rest.to_string())
                };
                return Some((
                    Value::Aggregate(AggOp::Sum, Stat::ManaSymbols(color), Box::new(sel)),
                    rest,
                ));
            }
        }
    }
    // CR 104.3: "players who have lost the game".
    for p in ["players who have lost the game", "player who has lost the game"] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some((
                Value::Custom(crate::kw::value_counts::PLAYERS_WHO_LOST.into()),
                rest.to_string(),
            ));
        }
    }
    // CR 700.8a: "creatures in your party".
    for p in ["creatures in your party", "creature in your party"] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some((
                Value::Custom(crate::game_terms::PARTY_SIZE.into()),
                rest.to_string(),
            ));
        }
    }
    // "graveyards with seven or more cards in them": players whose graveyard has that
    // many cards (CR 404.2: each player has their own graveyard).
    if let Some(x) = r
        .strip_prefix("graveyards with ")
        .or_else(|| r.strip_prefix("graveyard with "))
    {
        let (cmp, n, tail) = amount(x)?;
        let tail = tail.trim_start();
        let rest = tail
            .strip_prefix("cards in them")
            .or_else(|| tail.strip_prefix("cards in it"))?;
        return Some((
            Value::CountPlayers(PlayerFilter::GraveyardSize(cmp, Box::new(n))),
            rest.to_string(),
        ));
    }
    // Players.
    if let Some(rest) = r
        .strip_prefix("opponents you're attacking")
        .or_else(|| r.strip_prefix("opponent you're attacking"))
        .or_else(|| r.strip_prefix("opponents being attacked"))
        .or_else(|| r.strip_prefix("opponent being attacked"))
    {
        return Some((
            Value::Custom(crate::kw::value_counts::OPPONENTS_BEING_ATTACKED.into()),
            rest.to_string(),
        ));
    }
    // CR 508.6: "players you attacked this combat", "opponents you attacked [this turn]".
    for (p, v) in [
        (
            "players you attacked this combat",
            crate::kw::melee::OPPONENTS_ATTACKED,
        ),
        (
            "opponents you attacked this combat",
            crate::kw::melee::OPPONENTS_ATTACKED,
        ),
        (
            "opponent you attacked this combat",
            crate::kw::melee::OPPONENTS_ATTACKED,
        ),
        (
            "opponents you attacked this turn",
            crate::kw::value_counts::OPPONENTS_ATTACKED_THIS_TURN,
        ),
        (
            "opponent you attacked this turn",
            crate::kw::value_counts::OPPONENTS_ATTACKED_THIS_TURN,
        ),
        (
            "opponents you attacked",
            crate::kw::value_counts::OPPONENTS_ATTACKED_THIS_TURN,
        ),
        (
            "opponent you attacked",
            crate::kw::value_counts::OPPONENTS_ATTACKED_THIS_TURN,
        ),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest) {
                return Some((Value::Custom(v.into()), rest.to_string()));
            }
        }
    }
    if let Some((pf, rest)) = players(r) {
        return Some((Value::CountPlayers(pf), rest.to_string()));
    }
    // Counters on players and among objects.
    if let Some((kind, rest)) = counters_word(r) {
        let rest = rest.trim_start();
        // "experience counters you have", "poison counters they have", "poison counters
        // its controller has".
        if let Some(x) = rest.strip_prefix("your opponents have") {
            let v = Value::CountersOn(Box::new(Sel::Players(PlayerRef::Iterated)), kind);
            return Some((
                Value::OverPlayers(AggOp::Sum, PlayerFilter::Opponent, Box::new(v)),
                x.to_string(),
            ));
        }
        if let Some(x) = rest.strip_prefix("among players and permanents") {
            let on_players = Value::OverPlayers(
                AggOp::Sum,
                PlayerFilter::Any,
                Box::new(Value::CountersOn(
                    Box::new(Sel::Players(PlayerRef::Iterated)),
                    kind.clone(),
                )),
            );
            let on_permanents = Value::Aggregate(
                AggOp::Sum,
                Stat::Counters(kind),
                Box::new(Sel::All(Filter::Permanent)),
            );
            return Some((Value::Sum(vec![on_players, on_permanents]), x.to_string()));
        }
        if let Some(x) = rest.strip_prefix("among players") {
            let v = Value::CountersOn(Box::new(Sel::Players(PlayerRef::Iterated)), kind);
            return Some((
                Value::OverPlayers(AggOp::Sum, PlayerFilter::Any, Box::new(v)),
                x.to_string(),
            ));
        }
        if let Some(x) = rest.strip_prefix("among ") {
            let (f, tail) = objects(x, b)?;
            return Some((
                Value::Aggregate(AggOp::Sum, Stat::Counters(kind), Box::new(Sel::All(f))),
                tail,
            ));
        }
        // "counters it had on it" (its last known information).
        if let Some(x) = rest.strip_prefix("it had on it") {
            if is_no_referent(&b.it) {
                return None;
            }
            return Some((
                Value::CountersOn(Box::new(b.it.clone()), kind),
                x.to_string(),
            ));
        }
        if let Some((p, x)) = player_word(rest, b) {
            let x = x.trim_start();
            if let Some(x) = x.strip_prefix("have").or_else(|| x.strip_prefix("has")) {
                if word_end(x) {
                    if let Some(k) = kind {
                        return Some((Value::PlayerCounters(p, k), x.to_string()));
                    }
                    return Some((
                        Value::CountersOn(Box::new(Sel::Players(p)), None),
                        x.to_string(),
                    ));
                }
            }
            return None;
        }
        // "counters on ~" etc.: the core phrases.
        return None;
    }
    // Kinds of things among objects ("card types among cards in your graveyard"), or
    // in one object ("colors that spell is").
    if let Some((kind, rest)) = among_kind(r) {
        let rest = rest.trim_start();
        if let Some(x) = rest.strip_prefix("among ") {
            // Domain (CR 207.2c) keeps its own value.
            if kind == Among::BasicLandTypes {
                if let Some(t) = x.strip_prefix("lands you control") {
                    return Some((Value::Domain, t.to_string()));
                }
            }
            let (f, tail) = objects(x, b)?;
            return Some((Value::DistinctAmong(kind, Box::new(Sel::All(f))), tail));
        }
        if kind == Among::Colors {
            // "the number of colors that spell is", "... that creature was".
            if let Some(x) = rest.strip_prefix("that ") {
                for verb in [" is", " was"] {
                    if let Some(i) = x.find(verb) {
                        let after = &x[i + verb.len()..];
                        if word_end(after) {
                            let (sel, t) = referent(&format!("that {}", &x[..i]), b)?;
                            if !t.trim().is_empty() {
                                return None;
                            }
                            return Some((
                                Value::DistinctAmong(Among::Colors, Box::new(sel)),
                                after.to_string(),
                            ));
                        }
                    }
                }
            }
        }
        if kind == Among::CounterKinds {
            // "kinds of counters on it".
            if let Some(x) = rest.strip_prefix("on ") {
                let (sel, t) = referent(x, b)?;
                return Some((Value::DistinctAmong(Among::CounterKinds, Box::new(sel)), t));
            }
        }
        return None;
    }
    // "cards in their hand", "cards in that player's graveyard", "cards in your library".
    if let Some(x) = r
        .strip_prefix("cards in ")
        .or_else(|| r.strip_prefix("card in "))
    {
        let saved = b.targets.len();
        if let Some((p, x2)) = player_possessive(x, b) {
            if let Some((z, t)) = zone_word(x2) {
                // "in all graveyards with the same name as that spell": more qualifiers.
                if word_end(t)
                    && !t.trim_start().starts_with("with ")
                    && !t.trim_start().starts_with("that ")
                {
                    // A player who left the game: their hand as last known (CR 800.4i).
                    let v = match (&p, z) {
                        (_, ZoneKind::Hand) => Value::HandSize(p.clone()),
                        (_, ZoneKind::Library) => Value::LibrarySize(p.clone()),
                        (p, z) => Value::Count(Filter::and(vec![
                            Filter::Card,
                            Filter::InZone(z),
                            owned_by(p),
                        ])),
                    };
                    return Some((v, t.to_string()));
                }
            }
        }
        b.targets.truncate(saved);
    }
    // "suspended card you own and each other permanent you control with a time counter
    // on it": the sum of both counts.
    if let Some((x, y)) = r.split_once(" and each ") {
        let saved = b.targets.len();
        if let Some((v1, r1)) = count(x, b) {
            if r1.trim().is_empty() {
                if let Some((v2, r2)) = count(y, b) {
                    return Some((Value::Sum(vec![v1, v2]), r2));
                }
            }
        }
        b.targets.truncate(saved);
    }
    // "[cards] you own in exile and in your graveyard [that ...]": the cards in either
    // zone.
    for zones in [
        " you own in exile and in your graveyard",
        " in exile and in your graveyard",
    ] {
        if let Some((head, tail)) = r.split_once(zones) {
            let (f1, r1) = objects(&format!("{head} you own in exile{tail}"), b)?;
            let (f2, r2) = objects(&format!("{head} in your graveyard{tail}"), b)?;
            if r1 != r2 {
                return None;
            }
            return Some((Value::Sum(vec![Value::Count(f1), Value::Count(f2)]), r1));
        }
    }
    // Objects.
    let (f, rest) = objects(r, b)?;
    // Not the core's special phrases ("creatures in your party", "cards exiled with ~").
    let t = rest.trim_start();
    if !(t.is_empty()
        || t.starts_with([',', '.'])
        || t.starts_with("plus ")
        || t.starts_with("minus ")
        || t.starts_with("as you ")
        || t.starts_with("and ")
        || t.starts_with("to ")
        || t.starts_with("then "))
    {
        return None;
    }
    Some((Value::Count(f), rest))
}

/// After "the greatest"/"the least": "power among [objects]", "mana value among
/// [objects]", "life total among all players", "number of [objects] a player controls".
fn extreme(op: AggOp, r: &str, b: &mut Builder) -> Option<(Value, String)> {
    // CR 903.3: "your commanders", wherever they are.
    if op == AggOp::Max {
        if let Some(rest) = r.strip_prefix("mana value among your commanders") {
            return Some((
                Value::Custom(crate::commander_rules::YOUR_COMMANDER_MANA_VALUE.into()),
                rest.to_string(),
            ));
        }
        if let Some(rest) = r.strip_prefix(
            "mana value of a commander you own on the battlefield or in the command zone",
        ) {
            let among = |z: ZoneKind| {
                Value::Aggregate(
                    AggOp::Max,
                    Stat::ManaValue,
                    Box::new(Sel::All(Filter::and(vec![
                        Filter::Commander,
                        Filter::OwnedBy(PlayerRel::You),
                        Filter::InZone(z),
                    ]))),
                )
            };
            return Some((
                Value::Max(
                    Box::new(among(ZoneKind::Battlefield)),
                    Box::new(among(ZoneKind::Command)),
                ),
                rest.to_string(),
            ));
        }
    }
    if let Some((stat, x)) = stat_word(r) {
        let x = x.strip_prefix(" among ")?;
        let (f, rest) = objects(x, b)?;
        // Several kinds of objects joined by "and" ("other Dragons you control and Dragon
        // cards in your graveyard"): the extreme of either group.
        if let Some(y) = rest.strip_prefix(" and ") {
            let saved = b.targets.len();
            if let Some((g, rest2)) = objects(y, b) {
                let a = Value::Aggregate(op, stat.clone(), Box::new(Sel::All(f)));
                let c = Value::Aggregate(op, stat, Box::new(Sel::All(g)));
                let v = match op {
                    AggOp::Min => Value::Min(Box::new(a), Box::new(c)),
                    _ => Value::Max(Box::new(a), Box::new(c)),
                };
                return Some((v, rest2));
            }
            b.targets.truncate(saved);
        }
        return Some((Value::Aggregate(op, stat, Box::new(Sel::All(f))), rest));
    }
    if let Some(x) = r.strip_prefix("life total among ") {
        let (pf, rest) = if let Some(t) = x.strip_prefix("all players") {
            (PlayerFilter::Any, t)
        } else if let Some(t) = x.strip_prefix("your opponents") {
            (PlayerFilter::Opponent, t)
        } else {
            return None;
        };
        return Some((
            Value::OverPlayers(op, pf, Box::new(Value::LifeTotal(PlayerRef::Iterated))),
            rest.to_string(),
        ));
    }
    // "the greatest number of creatures a player controls", "... artifacts an opponent
    // controls".
    if let Some(x) = r.strip_prefix("number of ") {
        // "creatures you control that have a creature type in common".
        if let Some((head, rest)) = x.split_once(" that have a creature type in common") {
            let (f, t) = objects(head, b)?;
            if !t.trim().is_empty() {
                return None;
            }
            return Some((
                Value::DistinctAmong(Among::LargestCreatureTypeGroup, Box::new(Sel::All(f))),
                rest.to_string(),
            ));
        }
        for (p, pf) in [
            (" a player controls", PlayerFilter::Any),
            (" an opponent controls", PlayerFilter::Opponent),
        ] {
            if let Some((head, rest)) = x.split_once(p) {
                let (f, _, t) = super::statics::object_phrase(head)?;
                if !t.trim().is_empty() {
                    return None;
                }
                let v = Value::Count(Filter::and(vec![
                    f,
                    Filter::ControlledBy(PlayerRel::Iterated),
                ]));
                return Some((Value::OverPlayers(op, pf, Box::new(v)), rest.to_string()));
            }
        }
    }
    None
}

/// "the number of [s]" read as a whole phrase outside an instruction (static abilities:
/// "~ gets +1/+1 for each Equipment attached to it", cost reductions), where "it" is the
/// object `it` names (the source by default) and there are no targets.
pub fn whole_count(s: &str, it: Option<&Sel>) -> Option<Value> {
    use crate::card::Layout;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;
    let tl = TypeLine::default();
    let ctx = CompileContext {
        card_name: "",
        full_name: "",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    whole_count_in(s, &ctx, it.cloned().unwrap_or(Sel::This))
}

/// [`whole_count`] for a card being compiled (so "named ~" is its name).
pub fn whole_count_in(s: &str, ctx: &crate::oracle::CompileContext, it: Sel) -> Option<Value> {
    let mut b = Builder::new(ctx);
    b.it = it;
    let (v, rest) = count(end(s), &mut b)?;
    (rest.trim().is_empty() && b.targets.is_empty()).then_some(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Layout;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;

    fn parse(s: &str) -> Option<Value> {
        let tl = TypeLine::parse("Creature — Test");
        let ctx = CompileContext {
            card_name: "Probe",
            full_name: "Probe",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let mut b = Builder::new(&ctx);
        b.in_trigger = true;
        b.it_player = PlayerRef::TriggerPlayer;
        let (v, rest) = parse_value(s, &mut b)?;
        rest.trim().is_empty().then_some(v)
    }

    #[test]
    fn value_phrases_parse() {
        for s in [
            "the number of cards in their hand minus 4",
            "2 minus the number of cards in their hand",
            "the number of cards named ~ in all graveyards",
            "one plus the number of cards named ~ in your graveyard",
            "the number of opponents you have",
            "the number of opponents who control an artifact",
            "the number of your opponents with four or more cards in hand",
            "the number of opponents who control more lands than you",
            "the greatest toughness among creatures you control",
            "the greatest power and/or toughness among other creatures you control",
            "the highest life total among all players",
            "the greatest number of creatures a player controls",
            "the total mana value of noncreature artifacts you control",
            "the total toughness of creatures you control",
            "the number of card types among other nonland permanents you control",
            "the number of basic land types among lands they control",
            "the number of experience counters you have",
            "the total number of rad counters among players",
            "the number of counters among creatures you control",
            "the number of creatures you control plus the number of foods you control",
            "twice the number of white creatures that player controls",
            "half your life total, rounded up",
            "the number of untapped lands they control",
            "the number of different kinds of counters among permanents you control",
            "the number of creature types among creatures you control",
            "the number of cards in your library",
        ] {
            assert!(parse(s).is_some(), "failed to parse {s:?}");
        }
        for s in [
            "half your life total",
            "the number of creatures in your hand minus",
        ] {
            assert!(parse(s).is_none(), "should not parse {s:?}");
        }
    }
}
