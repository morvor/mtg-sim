//! Conditions that compare numbers, players and groups of objects:
//!
//! - parity: "the number of permanents is even", "its mana value is odd" (CR 107.1: zero
//!   is even);
//! - two values compared: "the number of attacking creatures is greater than the number
//!   of quest counters on ~", "the difference between your life total and target
//!   player's life total is 5 or less";
//! - counts of kinds and groups: "there are four or more basic land types among lands
//!   that player controls", "you control three or more lands with the same name",
//!   "you control eight or more artifacts with the same name as one another", "there are
//!   nine or more lands with different names among the basic, Sphere, and Locus lands
//!   you control" (CR 201.2);
//! - player comparisons: "you control more creatures than each other player", "you
//!   control fewer creatures than each opponent", "that player has more cards in hand
//!   than you", "that player has more life than you";
//! - extremes with ties: "you control the creature with the greatest toughness or tied
//!   for the greatest toughness";
//! - existence: "a creature has a -1/-1 counter on it", "a creature is attacking",
//!   "another permanent with the same name is on the battlefield", "a card with the same
//!   name is in a graveyard";
//! - game state: "you're not the monarch", "it's your combat phase", "two or more players
//!   have lost the game" (CR 800.4).
//!
//! [`parse_with`] reads them with the effect parser's [`Builder`] (so "that player",
//! "its" and "target player" mean what the text says); conditions without referents are
//! also registered as a [`ConditionPattern`] for static and activation conditions.

use super::ConditionPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::oracle_hardening_referents::{
    is_no_player_referent, is_no_referent, no_player_referent, no_referent, NO_REFERENT,
};
use crate::oracle::patterns::statics_conditions::{amount_cmp, color_or_phrase, state_filter};
use crate::oracle::patterns::value_grammar::parse_value;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// Custom value: the number of players who have lost the game or left it (CR 800.4).
pub const PLAYERS_LOST_GAME: &str = "players_lost_game";

/// A value that is 1 if `v` is odd and 0 if it's even.
pub fn parity(v: Value) -> Value {
    Value::Diff(
        Box::new(v.clone()),
        Box::new(Value::Mul(
            Box::new(Value::c(2)),
            Box::new(Value::Div(Box::new(v), 2, false)),
        )),
    )
}

/// The value is odd (`odd`) or even.
pub fn is_parity(v: Value, odd: bool) -> Condition {
    Condition::Compare(parity(v), Cmp::Eq, Value::c(if odd { 1 } else { 0 }))
}

/// |a - b|.
fn difference(a: Value, b: Value) -> Value {
    Value::Max(
        Box::new(Value::Diff(Box::new(a.clone()), Box::new(b.clone()))),
        Box::new(Value::Diff(Box::new(b), Box::new(a))),
    )
}

fn dummy_ctx() -> CompileContext<'static> {
    use std::sync::OnceLock;
    static TL: OnceLock<TypeLine> = OnceLock::new();
    CompileContext {
        card_name: "",
        full_name: "",
        type_line: TL.get_or_init(TypeLine::default),
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    }
}

/// The referent-free conditions (no "it", "that player" or targets).
fn parse_free(c: &str) -> Option<Condition> {
    let ctx = dummy_ctx();
    let mut b = Builder::new(&ctx);
    b.it = no_referent();
    b.it_player = no_player_referent();
    let cond = parse_with(c, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    let d = serde_json::to_string(&cond).ok()?;
    let needle = format!("\"Var\":{NO_REFERENT}");
    let no_player = serde_json::to_string(&no_player_referent()).ok()?;
    (!d.contains(&needle) && !d.contains(&no_player)).then_some(cond)
}

/// Equivalent wordings of conditions about a referent that the referent grammar reads:
/// "it was a Jace planeswalker spell" (a spell that has left the stack: what it was, CR
/// 608.2h) → "it was a Jace planeswalker"; "the exiled card is a God card" → "the card is
/// a God card"; "it entered under your control" → "you control it" (checked right after
/// it entered).
pub fn referent_rewording(c: &str) -> Option<String> {
    let c = end(c.trim());
    for p in ["it was a ", "it was an ", "that spell was a ", "that spell was an "] {
        if let Some(r) = c.strip_prefix(p) {
            let r = r.strip_suffix(" spell")?;
            return Some(format!("{p}{r}"));
        }
    }
    if let Some(r) = c.strip_prefix("the exiled card ") {
        return Some(format!("the card {r}"));
    }
    if c == "it entered under your control" {
        return Some("you control it".to_string());
    }
    None
}

/// [`parse_with`] when it adds no targets (conditions of the referent grammar,
/// `conditions_referents::parse_condition_with`, never add targets).
pub fn parse_without_targets(c: &str, b: &mut Builder) -> Option<Condition> {
    let before = b.targets.len();
    let r = parse_with(c, b)?;
    if b.targets.len() != before {
        b.targets.truncate(before);
        return None;
    }
    Some(r)
}

/// "If target player has exactly 10 life, ~ deals 10 damage to that player.", "If the
/// difference between your life total and target player's life total is 5 or less,
/// exchange life totals with that player.": a leading condition that introduces a
/// target (CR 601.2c: chosen as the spell is cast, wherever the text names it). "That
/// player" in the effect is the target player.
fn leading_if_with_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("if ")?;
    let (c, rest) = r.split_once(", ")?;
    let before = b.targets.len();
    let saved = (b.it.clone(), b.it_player.clone());
    let cond = parse_with(c, b)?;
    if b.targets.len() == before {
        b.targets.truncate(before);
        (b.it, b.it_player) = saved;
        return None;
    }
    if let Some(i) = (before..b.targets.len())
        .rev()
        .find(|i| matches!(b.targets[*i].what, TargetKind::Player(_)))
    {
        b.it_player = PlayerRef::Target(i as u8);
    }
    let Some(then) = crate::oracle::effects::parse_clause(rest, b) else {
        b.targets.truncate(before);
        (b.it, b.it_player) = saved;
        return None;
    };
    Some(Effect::If {
        cond,
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! {
    super::EffectPattern { name: "if [condition with a target], [effect]", priority: 5, parse: leading_if_with_targets }
}

inventory::submit! {
    ConditionPattern { name: "comparisons, parity and groups", priority: 400, parse: parse_free }
}

/// Parses a condition of this module's grammar. May add targets ("target player's life
/// total"); the caller decides whether that's allowed.
pub fn parse_with(c: &str, b: &mut Builder) -> Option<Condition> {
    let c = end(c.trim());
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let r = parse_inner(c, b);
    if r.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    r
}

fn parse_inner(c: &str, b: &mut Builder) -> Option<Condition> {
    match c {
        "you're not the monarch" | "you aren't the monarch" => {
            return Some(Condition::Not(Box::new(Condition::IsMonarch)))
        }
        // CR 505.1 / 506.1: the combat phase of your turn.
        "it's your combat phase" => {
            return Some(Condition::And(vec![
                Condition::YourTurn,
                Condition::Phase(PhaseCond::Combat),
            ]))
        }
        _ => {}
    }
    // "two or more players have lost the game" (CR 800.4).
    if let Some(r) = c.strip_suffix(" players have lost the game") {
        let (cmp, n, rest) = amount_cmp(r)?;
        if !rest.trim().is_empty() {
            return None;
        }
        return Some(Condition::Compare(
            Value::Custom(PLAYERS_LOST_GAME.into()),
            cmp,
            n,
        ));
    }
    if let Some(r) = c.strip_prefix("the difference between ") {
        return difference_cond(r, b);
    }
    if let Some(r) = c
        .strip_prefix("there are ")
        .or_else(|| c.strip_prefix("there is "))
    {
        return there_are(r, b);
    }
    if let Some(r) = c.strip_prefix("you control ") {
        if let Some(cond) = you_control(r) {
            return Some(cond);
        }
        // "you control a white permanent other than that creature or ~": an object
        // described relative to what the text named.
        if let Some(x) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) {
            let (f, false, rest) = parse_object_phrase(x)? else {
                return None;
            };
            if !rest.trim().is_empty() || !super::filters_relational::mentions_referent(&f) {
                return None;
            }
            let f = super::filters_relational::resolve_referent(f, b)?;
            return Some(Condition::Exists(f.you_control()));
        }
    }
    if let Some(cond) = source_state_list(c) {
        return Some(cond);
    }
    if let Some(cond) = chosen_player_controls(c) {
        return Some(cond);
    }
    if let Some(cond) = player_comparison(c, b) {
        return Some(cond);
    }
    if let Some(cond) = player_owns(c, b) {
        return Some(cond);
    }
    if let Some(cond) = exists(c, b) {
        return Some(cond);
    }
    if let Some(cond) = value_comparison(c, b) {
        return Some(cond);
    }
    // "a creature is attacking and a creature is blocking", "a card with the same name is
    // in a graveyard or a nontoken permanent with the same name is on the battlefield":
    // both parts of this grammar.
    for (sep, all) in [(" and ", true), (" or ", false)] {
        for (i, _) in c.match_indices(sep) {
            let saved = b.targets.len();
            if let Some(x) = parse_with(&c[..i], b) {
                if let Some(y) = parse_with(&c[i + sep.len()..], b) {
                    return Some(if all {
                        Condition::And(vec![x, y])
                    } else {
                        Condition::Or(vec![x, y])
                    });
                }
            }
            b.targets.truncate(saved);
        }
    }
    None
}

/// "[value] is odd", "[value] is even", "[value] is greater than [value]".
fn value_comparison(c: &str, b: &mut Builder) -> Option<Condition> {
    // Only values that are phrases of their own ("the number of ...", "its mana value"),
    // never a subject a state follows ("~ is attacking").
    let (v, rest) = parse_value(c, b)?;
    if matches!(v, Value::Const(_)) {
        return None;
    }
    let rest = rest.trim();
    let r = rest.strip_prefix("is ")?;
    match r {
        "odd" => return Some(is_parity(v, true)),
        "even" => return Some(is_parity(v, false)),
        _ => {}
    }
    for (p, cmp) in [
        ("less than or equal to ", Cmp::Le),
        ("greater than or equal to ", Cmp::Ge),
        ("less than ", Cmp::Lt),
        ("greater than ", Cmp::Gt),
        ("equal to ", Cmp::Eq),
    ] {
        if let Some(x) = r.strip_prefix(p) {
            let (w, rest) = parse_value(x, b)?;
            if !end(&rest).trim().is_empty() {
                return None;
            }
            return Some(Condition::Compare(v, cmp, w));
        }
    }
    None
}

/// "the difference between [value] and [value] is N or less".
fn difference_cond(r: &str, b: &mut Builder) -> Option<Condition> {
    let (a, rest) = parse_value(r, b)?;
    let rest = rest.trim_start().strip_prefix("and ")?.to_string();
    let (bv, rest) = parse_value(&rest, b)?;
    let rest = rest.trim_start().strip_prefix("is ")?;
    let (cmp, n, tail) = amount_cmp(rest)?;
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(difference(a, bv), cmp, n))
}

/// "there are four or more basic land types among lands that player controls", "there
/// are five basic land types among lands you control", "there are nine or more lands with
/// different names among the basic, Sphere, and Locus lands you control".
fn there_are(r: &str, b: &mut Builder) -> Option<Condition> {
    let (cmp, n, rest) = match amount_cmp(r) {
        Some(x) => x,
        None => {
            let (n, rest) = parse_number(r)?;
            (Cmp::Eq, n, rest)
        }
    };
    let rest = rest.trim();
    // "[nouns] with different names among [objects]"
    if let Some((_, among)) = rest.split_once(" with different names among ") {
        let (noun, _) = rest.split_once(" with different names among ")?;
        let among = among.strip_prefix("the ").unwrap_or(among);
        let (f, _, tail) = parse_object_phrase(among)?;
        if !end(tail).trim().is_empty() {
            return None;
        }
        let (kind, _, t2) = parse_object_phrase(noun)?;
        if !t2.trim().is_empty() {
            return None;
        }
        return Some(Condition::Compare(
            Value::DistinctNames(Filter::and(vec![kind, f])),
            cmp,
            n,
        ));
    }
    // Only kinds among objects ("basic land types among ...", "card types among ...").
    if !rest.contains(" among ") {
        return None;
    }
    let (v, tail) = parse_value(&format!("the number of {rest}"), b)?;
    if !tail.trim().is_empty() || !matches!(v, Value::DistinctAmong(..) | Value::Domain | Value::CardTypesAmong(_) | Value::ColorsAmong(_)) {
        return None;
    }
    Some(Condition::Compare(v, cmp, n))
}

/// "you control three or more lands with the same name", "you control eight or more
/// artifacts with the same name as one another", "you control more creatures than each
/// other player", "you control fewer creatures than each opponent", "you control the
/// creature with the greatest toughness or tied for the greatest toughness".
fn you_control(r: &str) -> Option<Condition> {
    for suffix in [" with the same name as one another", " with the same name"] {
        if let Some(x) = r.strip_suffix(suffix) {
            let (cmp, n, rest) = amount_cmp(x)?;
            let f = color_or_phrase(rest.trim())?;
            return Some(Condition::Compare(
                Value::LargestSameNameGroup(f.you_control()),
                cmp,
                n,
            ));
        }
    }
    for (p, more) in [("more ", true), ("fewer ", false)] {
        let Some(x) = r.strip_prefix(p) else {
            continue;
        };
        for (q, who) in [
            (" than each other player", PlayerFilter::NotYou),
            (" than each opponent", PlayerFilter::Opponent),
        ] {
            let Some(noun) = x.strip_suffix(q) else {
                continue;
            };
            let f = noun_phrase(noun)?;
            // No such player controls as many (more: at least as many; fewer: at most).
            let theirs = PlayerFilter::Controls(
                Box::new(f.clone()),
                if more { Cmp::Ge } else { Cmp::Le },
                Box::new(Value::Count(f.you_control())),
            );
            return Some(Condition::Not(Box::new(Condition::PlayerMatches(
                PlayerRef::Each(who),
                theirs,
            ))));
        }
    }
    // "the creature with the greatest toughness or tied for the greatest toughness"
    for (p, stat) in [("power", true), ("toughness", false)] {
        let x = format!(
            "the creature with the greatest {p} or tied for the greatest {p}"
        );
        if r == x {
            let mine = Filter::creature().you_control();
            let val = |f: Filter| {
                let of = if stat {
                    Value::PowerOf(Box::new(Sel::Var(vars::TESTED)))
                } else {
                    Value::ToughnessOf(Box::new(Sel::Var(vars::TESTED)))
                };
                Value::Extreme(Box::new(of), Box::new(Sel::All(f)), true)
            };
            return Some(Condition::And(vec![
                Condition::Exists(mine.clone()),
                Condition::Compare(val(mine), Cmp::Ge, val(Filter::creature())),
            ]));
        }
    }
    None
}

/// An object noun phrase, also "creatures of that type" / "of the chosen type".
fn noun_phrase(noun: &str) -> Option<Filter> {
    for suffix in [" of that type", " of the chosen type"] {
        if let Some(n) = noun.strip_suffix(suffix) {
            let f = color_or_phrase(n)?;
            return Some(Filter::and(vec![f, Filter::ChosenType]));
        }
    }
    color_or_phrase(noun)
}

/// "that player has more cards in hand than you", "that player has more life than you",
/// "that player or that planeswalker's controller has more life than you", "target player
/// has exactly 10 life", "that player has 5 or less life".
fn player_comparison(c: &str, b: &mut Builder) -> Option<Condition> {
    let (who, pred) = c.split_once(" has ")?;
    let f = match pred {
        "more cards in hand than you" => {
            PlayerFilter::HandSize(Cmp::Gt, Box::new(Value::HandSize(PlayerRef::You)))
        }
        "more life than you" => {
            PlayerFilter::Life(Cmp::Gt, Box::new(Value::LifeTotal(PlayerRef::You)))
        }
        _ => {
            let (cmp, n, rest) = amount_cmp(pred)?;
            if rest.trim() != "life" {
                return None;
            }
            PlayerFilter::Life(cmp, Box::new(n))
        }
    };
    let p = player_subject(who, b)?;
    Some(Condition::PlayerMatches(p, f))
}

/// "they own three or more exiled cards with hit counters on them": how many objects
/// matching the phrase the player owns.
fn player_owns(c: &str, b: &mut Builder) -> Option<Condition> {
    let (who, r) = c
        .split_once(" own ")
        .or_else(|| c.split_once(" owns "))?;
    let (cmp, n, rest) = amount_cmp(r)?;
    let (f, _, tail) = parse_object_phrase(rest.trim())?;
    if !tail.trim().is_empty() {
        return None;
    }
    let p = match who {
        "they" => {
            if is_no_player_referent(&b.it_player) || matches!(b.it_player, PlayerRef::You) {
                return None;
            }
            b.it_player.clone()
        }
        "you" => PlayerRef::You,
        _ => player_subject(who, b)?,
    };
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![f, Filter::OwnedByPlayer(Box::new(p))])),
        cmp,
        n,
    ))
}

/// "~ is enchanted, equipped, and has a counter on it": each of a list of states of the
/// source.
fn source_state_list(c: &str) -> Option<Condition> {
    let r = c.strip_prefix("~ is ")?;
    if !r.contains(", ") {
        return None;
    }
    let r = r.replace(", and ", ", ");
    let mut conds = vec![];
    for item in r.split(", ") {
        if let Some(h) = item.strip_prefix("has ") {
            let (cmp, n, kind) = crate::oracle::patterns::statics_conditions::counters_on_it(h)?;
            conds.push(Condition::Compare(
                Value::CountersOn(Box::new(Sel::This), kind),
                cmp,
                n,
            ));
        } else {
            conds.push(Condition::SelMatches(Sel::This, state_filter(item)?));
        }
    }
    (conds.len() >= 2).then_some(Condition::And(conds))
}

/// "the chosen player controls a nontoken permanent of the chosen color", "the chosen
/// player controls no nontoken permanents of the chosen color" (the player chosen as the
/// permanent entered, CR 607.2d).
fn chosen_player_controls(c: &str) -> Option<Condition> {
    let r = c.strip_prefix("the chosen player controls ")?;
    let (neg, x) = if let Some(x) = r.strip_prefix("no ") {
        (true, x)
    } else {
        (false, r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?)
    };
    let (f, _, tail) = parse_object_phrase(x)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let cond = Condition::Exists(Filter::and(vec![f, Filter::ControlledBy(PlayerRel::Chosen)]));
    Some(if neg {
        Condition::Not(Box::new(cond))
    } else {
        cond
    })
}

/// "When the chosen player controls no nontoken permanents of the chosen color, sacrifice
/// ~." (Jihad): a state trigger (CR 603.8).
fn chosen_player_state_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let r = end(r);
    if !r.starts_with("the chosen player controls no ") {
        return None;
    }
    let cond = chosen_player_controls(r)?;
    Some((TriggerCond::State(cond), Sel::This, PlayerRef::You))
}

inventory::submit! { super::TriggerPattern { name: "state trigger: the chosen player controls no ...", priority: 100, parse: chosen_player_state_trigger } }

/// A player subject: "that player", "target player", "that player or that planeswalker's
/// controller" (the player a "target player or planeswalker" is, or the planeswalker's
/// controller).
fn player_subject(who: &str, b: &mut Builder) -> Option<PlayerRef> {
    if who == "that player or that planeswalker's controller" {
        // The last target that may be a player or a planeswalker: the player, or the
        // planeswalker's controller (a player's "controller" is the player).
        let t = b.targets.iter().rposition(|t| match &t.what {
            TargetKind::AnyTarget | TargetKind::ObjectOrPlayer(..) => true,
            _ => false,
        })?;
        return Some(PlayerRef::ControllerOf(Box::new(Sel::Target(t as u8))));
    }
    let (p, rest) = crate::oracle::effects::player_ref(who, b)?;
    if !rest.trim().is_empty() || is_no_player_referent(&p) {
        return None;
    }
    Some(p)
}


/// "a creature has a -1/-1 counter on it", "a creature is attacking", "another permanent
/// with the same name is on the battlefield", "a card with the same name is in a
/// graveyard", "a nontoken permanent with the same name is on the battlefield".
/// Whether "it" in the triggered ability being compiled is its source: the trigger is
/// about the source ("Whenever ~ attacks, if a card is exiled with it, ...").
pub fn trigger_about_source(b: &Builder) -> bool {
    match &b.it {
        Sel::This => true,
        Sel::TriggerObject | Sel::TriggerLki => {
            let raw = crate::oracle::raw_text().to_lowercase();
            let name = crate::oracle::card_name().to_lowercase();
            !name.is_empty()
                && ["whenever ", "when "].iter().any(|w| {
                    raw.contains(&format!("{w}{name} ")) || raw.contains(&format!("{w}this "))
                })
        }
        _ => false,
    }
}

fn exists(c: &str, b: &mut Builder) -> Option<Condition> {
    // "a card is exiled with it": with the source (CR 607.2a).
    if c.ends_with(" exiled with it") && trigger_about_source(b) {
        let c2 = format!("{}~", &c[..c.len() - 2]);
        return crate::oracle::statics::parse_condition(&c2, b.ctx);
    }
    let r = c
        .strip_prefix("a ")
        .or_else(|| c.strip_prefix("an "))
        .or_else(|| c.starts_with("another ").then_some(c))?;
    // "a card with the same name is in a graveyard": the same name as what the text
    // named ("it").
    let r = r.replacen(" with the same name is ", " with the same name as it is ", 1);
    let (f, plural, rest) = parse_object_phrase(&r)?;
    if plural {
        return None;
    }
    let rest = rest.trim();
    let state = if rest == "is on the battlefield" {
        Filter::Permanent
    } else if rest == "is in a graveyard" {
        Filter::and(vec![Filter::Card, Filter::InZone(ZoneKind::Graveyard)])
    } else if let Some(s) = rest.strip_prefix("is ") {
        Filter::and(vec![Filter::Permanent, state_filter(s)?])
    } else if let Some(h) = rest.strip_prefix("has ") {
        let (cmp, n, kind) = crate::oracle::patterns::statics_conditions::counters_on_it(h)?;
        if !(cmp == Cmp::Ge && matches!(n, Value::Const(1))) {
            return None;
        }
        Filter::and(vec![Filter::Permanent, Filter::HasCounter(kind)])
    } else {
        return None;
    };
    let f = super::filters_relational::resolve_referent(Filter::and(vec![f, state]), b)?;
    if is_no_referent(&b.it) && super::filters_relational::mentions_referent(&f) {
        return None;
    }
    Some(Condition::Exists(f))
}
