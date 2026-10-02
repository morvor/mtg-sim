//! Conditions about referents: SUBJECT + PREDICATE, where the subject is resolved by the
//! effect parser's referent machinery (the [`Builder`]'s "it", "that creature", "that
//! player", the sacrificed or discarded card, enchanted/equipped creature, ~, X) and the
//! predicate is an object or player state phrase:
//!
//! - "it's white", "that creature was a Human", "it isn't a token", "the sacrificed
//!   creature was legendary", "that land is a Forest", "it wasn't attacking";
//! - "it has flying", "it doesn't have suspend", "that creature had a +1/+1 counter on
//!   it", "it has mana value 2 or less", "it had power 3 or greater";
//! - "its mana value is 3 or less", "that creature's power is greater than ~'s power",
//!   "its toughness is less than or equal to the number of cards in your graveyard";
//! - "it shares a color with a creature you control", "it has the same mana value as the
//!   discarded card", "it has greater power or toughness than ~";
//! - "that player has two or fewer cards in hand", "that player controls more lands than
//!   you", "that player is you", "it's not their turn", "its controller is poisoned";
//! - "you control that creature", "an opponent controls that creature";
//! - "X is 5 or more".
//!
//! [`parse_condition_with`] is the Builder-aware condition entry point: the effect parser
//! uses it for leading "If ..., " sentences, trailing " if ..." conditions and "instead"
//! replacements whose condition refers back to something, and the trigger parser for
//! intervening-if clauses about the trigger object (CR 603.4, 603.10a). Conditions are
//! checked as that part of the effect happens (CR 608.2c), using last known information
//! for objects that have left their zone (CR 608.2h).

use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::oracle::effects::{object_ref, player_ref, Builder};
use crate::oracle::patterns::oracle_hardening_referents::{is_no_player_referent, is_no_referent};
use crate::oracle::patterns::statics_conditions::{
    amount_cmp, color_or_phrase, player_predicate, state_filter,
};
use crate::oracle::phrases::*;
use crate::types::*;

use crate::discard_rules::DISCARDED;

/// Parses a condition, resolving its pronouns and object references with `b`. Tries the
/// subject-predicate grammar first, then conditions joined by "and"/"or" in which at
/// least one part has a referent. Never adds targets. Conditions without referents are
/// left to [`crate::oracle::statics::parse_condition`].
pub fn parse_condition_with(c: &str, b: &mut Builder) -> Option<Condition> {
    let c = end(c.trim());
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let r = referent_condition(c, b)
        .or_else(|| joined(c, b))
        .or_else(|| referent_free(c, b));
    // Resolving a subject never changes what later pronouns mean, nor adds targets.
    b.targets.truncate(saved.0);
    b.it = saved.1;
    b.it_player = saved.2;
    let r = r?;
    // A pronoun that had no antecedent isn't understood.
    let d = serde_json::to_string(&r).unwrap_or_default();
    let needle = format!(
        "\"Var\":{}",
        crate::oracle::patterns::oracle_hardening_referents::NO_REFERENT
    );
    if d.contains(&needle) {
        return None;
    }
    Some(r)
}

/// Whether a condition's "it" would mean the ability's source only by default: after an
/// earlier sentence that may have named something the parser didn't track ("Target
/// player exiles a card from their graveyard. If it's a creature card, ...").
fn it_is_source_by_default(c: &str, b: &Builder) -> bool {
    matches!(b.it, Sel::This)
        && !b.in_trigger
        && b.sentences > 0
        && c.split(' ').any(|w| matches!(w, "it" | "its" | "it's"))
}

/// "If [condition about a referent], [effect]." (the effect parser's fallback for a
/// leading "if" its core didn't understand).
pub fn leading_if(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (c, rest) = r.split_once(", ")?;
    // "..., [effect] instead" replaces the previous sentence's effect (follow-ups).
    if rest.ends_with(" instead") || rest.starts_with("instead ") {
        return None;
    }
    if it_is_source_by_default(c, b) {
        return None;
    }
    let cond = parse_condition_with(c, b)?;
    // "If enchanted creature is red, tap it.": the object the condition is about is what
    // a following "it" refers to, when nothing more specific is.
    if let Some(sel) = condition_subject(c, b) {
        if matches!(b.it, Sel::This) || is_no_referent(&b.it) {
            b.it = sel;
        }
    }
    if rest.split(' ').any(|w| matches!(w, "it" | "its")) {
        if let Some(sel) = named_subject(c, b) {
            b.it = sel;
        }
    }
    // "~ deals damage equal to the number of counters on it": whether "it" is the
    // sentence's subject or the condition's object is ambiguous.
    if rest.starts_with("~ ")
        && !matches!(b.it, Sel::This)
        && (rest.contains(" on it") || rest.contains(" its "))
    {
        return None;
    }
    let then = crate::oracle::effects::parse_sentence(rest, b)?;
    let then = other_than_antecedent(then, b)?;
    Some(Effect::If {
        cond,
        then: Box::new(then),
        otherwise: Box::new(Effect::Noop),
    })
}

/// In a spell's text, "other" ("destroy all other creatures") can't mean other than the
/// spell: it means other than what the text named before ("Create X 1/1 white Soldier
/// creature tokens. If X is 5 or more, destroy all other creatures."). Without such an
/// antecedent, not understood.
fn other_than_antecedent(e: Effect, b: &Builder) -> Option<Effect> {
    if !b.ctx.is_spell() {
        return Some(e);
    }
    let json = serde_json::to_value(&e).ok()?;
    fn has_other(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::String(s) => s == "Other",
            serde_json::Value::Array(a) => a.iter().any(has_other),
            serde_json::Value::Object(m) => m.values().any(has_other),
            _ => false,
        }
    }
    if !has_other(&json) {
        return Some(e);
    }
    if matches!(b.it, Sel::This | Sel::None) || is_no_referent(&b.it) {
        return None;
    }
    let not_it = serde_json::to_value(Filter::not(Filter::In(Box::new(b.it.clone())))).ok()?;
    fn replace(v: serde_json::Value, with: &serde_json::Value) -> serde_json::Value {
        use serde_json::Value as J;
        match v {
            J::String(s) if s == "Other" => with.clone(),
            J::Array(a) => J::Array(a.into_iter().map(|x| replace(x, with)).collect()),
            J::Object(m) => J::Object(m.into_iter().map(|(k, x)| (k, replace(x, with))).collect()),
            other => other,
        }
    }
    serde_json::from_value(replace(json, &not_it)).ok()
}

/// The object a condition is about ("enchanted creature is red" → the enchanted
/// creature), if its subject is an object.
pub fn condition_subject(c: &str, b: &mut Builder) -> Option<Sel> {
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let r = match subject(end(c.trim()), b) {
        Some((Subject::Object(sel), _)) if !is_no_referent(&sel) => Some(sel),
        _ => None,
    };
    b.targets.truncate(saved.0);
    b.it = saved.1;
    b.it_player = saved.2;
    r
}

/// The object a condition names explicitly as its subject ("equipped creature is a
/// Vampire", "that creature was a Human", "the sacrificed creature was legendary"), not by
/// a pronoun: what a following "it" refers to ("If equipped creature is a Vampire, put two
/// +1/+1 counters on it instead.").
pub fn named_subject(c: &str, b: &mut Builder) -> Option<Sel> {
    let first = split_word(c.trim()).0;
    if matches!(
        first,
        "it" | "its" | "it's" | "he" | "she" | "he's" | "she's" | "his" | "her" | "~" | "~'s"
    ) {
        return None;
    }
    condition_subject(c, b).filter(|s| !matches!(s, Sel::This))
}

/// Whether "it" in a trailing condition is ambiguous: after an earlier sentence, in a
/// triggered ability whose "it" is the trigger's object, it may as well mean an object
/// that sentence named ("Whenever a creature you control dies, put a phyresis counter on
/// ~. Then draw a card if it has seven or more phyresis counters on it.").
pub fn ambiguous_it(c: &str, b: &Builder) -> bool {
    b.sentences > 0
        && matches!(b.it, Sel::TriggerObject | Sel::TriggerLki)
        && c.split(' ').any(|w| matches!(w, "it" | "its" | "it's"))
}

/// An intervening-if clause of a triggered ability (CR 603.4) whose pronouns refer to the
/// trigger's object and player (`it`, `it_player`): "Whenever a creature dies, if it had
/// a +1/+1 counter on it, ...", "At the beginning of each opponent's upkeep, if that
/// player has two or fewer cards in hand, ...". The trigger object of a leaves-the-
/// battlefield trigger is its last known information (CR 603.10a). Also returns the object
/// the condition is about ("if enchanted creature is untapped, tap it").
pub fn intervening(
    c: &str,
    ctx: &crate::oracle::CompileContext,
    it: &Sel,
    it_player: &PlayerRef,
) -> Option<(Condition, Option<Sel>)> {
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = it.clone();
    b.it_player = match it_player {
        PlayerRef::You => crate::oracle::patterns::oracle_hardening_referents::no_player_referent(),
        p => p.clone(),
    };
    if matches!(b.it, Sel::None) {
        b.it = crate::oracle::patterns::oracle_hardening_referents::no_referent();
    }
    let cond = parse_condition_with(c, &mut b)?;
    let subject = condition_subject(c, &mut b);
    Some((cond, subject))
}

/// A condition whose only pronouns are inside an object phrase ("you control a creature
/// with a counter on it"), or none: the referent-free parser's.
fn referent_free(c: &str, b: &Builder) -> Option<Condition> {
    let words = c.replace(" on it", " ").replace(" on them", " ");
    if words.split(' ').any(|w| {
        matches!(
            w,
            "it" | "its" | "it's" | "that" | "they" | "their" | "them" | "those" | "he" | "she"
        )
    }) {
        return None;
    }
    crate::oracle::statics::parse_condition(c, b.ctx)
}

/// Either a referent condition or one without referents.
fn any_condition(c: &str, b: &mut Builder) -> Option<Condition> {
    referent_condition(c, b).or_else(|| crate::oracle::statics::parse_condition(c, b.ctx))
}

/// "[a] and [b]", "[a] or [b]" where at least one part is about a referent. A part that
/// starts with a verb shares the subject of the first ("that player is your opponent and
/// has four or more cards in hand").
fn joined(c: &str, b: &mut Builder) -> Option<Condition> {
    for (sep, all) in [(" and ", true), (" or ", false)] {
        for (i, _) in c.match_indices(sep) {
            let (x, y) = (&c[..i], &c[i + sep.len()..]);
            let Some(cx) = referent_condition(x, b) else {
                if let (Some(cx), Some(cy)) = (
                    crate::oracle::statics::parse_condition(x, b.ctx),
                    referent_condition(y, b),
                ) {
                    return Some(combine(all, cx, cy));
                }
                continue;
            };
            // Same subject, elided: "is your opponent and has ...".
            let cy = match subject_text(x) {
                Some(subj) if starts_with_verb(y) => any_condition(&format!("{subj} {y}"), b),
                _ => any_condition(y, b),
            };
            if let Some(cy) = cy {
                return Some(combine(all, cx, cy));
            }
        }
    }
    None
}

fn combine(all: bool, a: Condition, b: Condition) -> Condition {
    if all {
        Condition::And(vec![a, b])
    } else {
        Condition::Or(vec![a, b])
    }
}

fn starts_with_verb(s: &str) -> bool {
    let w = split_word(s).0;
    matches!(
        w,
        "is" | "was"
            | "isn't"
            | "wasn't"
            | "has"
            | "had"
            | "doesn't"
            | "didn't"
            | "controls"
            | "shares"
    )
}

/// The subject words of a simple "[subject] [verb] ..." condition.
fn subject_text(c: &str) -> Option<&str> {
    for p in [
        "that player",
        "that opponent",
        "the player",
        "that creature",
        "that permanent",
        "that card",
        "that spell",
        "it",
    ] {
        if c.starts_with(&format!("{p} ")) {
            return Some(p);
        }
    }
    None
}

/// A subject: an object or a player.
enum Subject {
    Object(Sel),
    Player(PlayerRef),
}

/// The subject at the start of `c` and the rest. A possessive subject ("its", "that
/// creature's") leaves the rest starting with "'s ".
fn subject(c: &str, b: &mut Builder) -> Option<(Subject, String)> {
    // Players first ("that player", "its controller": "its" isn't the possessive here).
    for p in ["that player", "its controller", "its owner"] {
        if c.starts_with(p) {
            let (pr, rest) = player_ref(c, b)?;
            if is_no_player_referent(&pr) {
                return None;
            }
            return Some((Subject::Player(pr), rest));
        }
    }
    for p in ["that opponent", "the player"] {
        if let Some(rest) = c.strip_prefix(p) {
            if is_no_player_referent(&b.it_player) || matches!(b.it_player, PlayerRef::You) {
                return None;
            }
            return Some((Subject::Player(b.it_player.clone()), rest.to_string()));
        }
    }
    // "they" as a player ("if they do"): only when "that player" has an antecedent and
    // "it" doesn't name a group.
    if let Some(rest) = c.strip_prefix("they ") {
        if !is_no_player_referent(&b.it_player)
            && !matches!(b.it_player, PlayerRef::You)
            && player_verb(rest)
        {
            return Some((Subject::Player(b.it_player.clone()), format!(" {rest}")));
        }
    }
    // Possessive pronouns.
    for p in ["its ", "his ", "her "] {
        if let Some(rest) = c.strip_prefix(p) {
            let it = it_referent(b)?;
            return Some((Subject::Object(it), format!("'s {rest}")));
        }
    }
    // Contractions.
    for p in ["it's ", "he's ", "she's "] {
        if let Some(rest) = c.strip_prefix(p) {
            let it = it_referent(b)?;
            return Some((Subject::Object(it), format!(" 's-contracted {rest}")));
        }
    }
    // "the card", "the spell": what "it" refers to, when that's an object an earlier
    // instruction found or targeted.
    for p in ["the card", "the spell"] {
        if let Some(rest) = c.strip_prefix(p) {
            if (rest.is_empty() || rest.starts_with(' ') || rest.starts_with("'s "))
                && matches!(b.it, Sel::Var(_) | Sel::Target(_))
                && !is_no_referent(&b.it)
            {
                return Some((Subject::Object(b.it.clone()), rest.to_string()));
            }
        }
    }
    for (p, sel) in [
        ("the sacrificed creature", Sel::Var(vars::SACRIFICED)),
        ("the sacrificed permanent", Sel::Var(vars::SACRIFICED)),
        ("the sacrificed artifact", Sel::Var(vars::SACRIFICED)),
        ("the sacrificed land", Sel::Var(vars::SACRIFICED)),
        ("the sacrificed enchantment", Sel::Var(vars::SACRIFICED)),
        ("the discarded card", Sel::Var(DISCARDED)),
    ] {
        if let Some(rest) = c.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with("'s ") {
                return Some((Subject::Object(sel), rest.to_string()));
            }
        }
    }
    // Objects the effect parser can name without adding a target.
    let first = split_word(c).0;
    if !matches!(
        first,
        "it" | "he" | "she" | "that" | "~" | "~'s" | "enchanted" | "equipped" | "the"
    ) && !first.starts_with("that")
    {
        return None;
    }
    let before = b.targets.len();
    let (sel, rest) = object_ref(c, b)?;
    if b.targets.len() != before || is_no_referent(&sel) || matches!(sel, Sel::All(_)) {
        b.targets.truncate(before);
        return None;
    }
    if !(rest.is_empty() || rest.starts_with(' ') || rest.starts_with("'s ")) {
        return None;
    }
    // "that land" can't be a target creature ("Whenever a land you control enters, tap
    // target creature an opponent controls. If that land is an Island, ..."): the phrase
    // names something the parser didn't track.
    if let (Some(noun), Sel::Target(slot)) = (c.strip_prefix("that ").map(|r| split_word(r).0), &sel)
    {
        let noun = noun.trim_end_matches("'s");
        if let Some(TargetKind::Object(f)) = b.targets.get(*slot as usize).map(|t| &t.what) {
            if !can_be(f, noun) {
                return None;
            }
        }
    }
    Some((Subject::Object(sel), rest))
}

/// Whether an object matching a target filter can be what `noun` names ("land",
/// "creature", ...). Unknown nouns and filters without card types are compatible.
fn can_be(f: &Filter, noun: &str) -> bool {
    let t = match noun {
        "creature" => CardType::Creature,
        "land" => CardType::Land,
        "artifact" => CardType::Artifact,
        "enchantment" => CardType::Enchantment,
        "planeswalker" => CardType::Planeswalker,
        _ => return true,
    };
    let mut types = Vec::new();
    fn collect(f: &Filter, out: &mut Vec<CardType>) {
        match f {
            Filter::Type(t) => out.push(*t),
            Filter::And(v) | Filter::Or(v) => v.iter().for_each(|x| collect(x, out)),
            _ => {}
        }
    }
    collect(f, &mut types);
    types.is_empty() || types.contains(&t)
}

/// What "it" refers to as a subject: an antecedent (not none, not a group).
fn it_referent(b: &Builder) -> Option<Sel> {
    let it = crate::oracle::patterns::pronoun_groups::singular_it(b);
    if is_no_referent(&it) || matches!(it, Sel::None | Sel::TriggerObjects) {
        return None;
    }
    Some(it)
}

fn player_verb(s: &str) -> bool {
    let w = split_word(s).0;
    matches!(
        w,
        "control" | "controls" | "have" | "has" | "are" | "aren't" | "own" | "don't" | "do"
    )
}

/// The subject-predicate grammar.
fn referent_condition(c: &str, b: &mut Builder) -> Option<Condition> {
    let c = end(c.trim());
    if let Some(cond) = special(c, b) {
        return Some(cond);
    }
    let (subj, rest) = subject(c, b)?;
    match subj {
        Subject::Object(sel) => object_predicate(&sel, &rest, b).or_else(|| {
            // The static-condition grammar's states of "it".
            let f = super::statics_conditions::pronoun_state(c)?;
            Some(Condition::SelMatches(sel, f))
        }),
        Subject::Player(p) => player_condition(&p, rest.trim(), b),
    }
}

/// Conditions whose subject isn't the referent: "you control that creature", "an
/// opponent controls it", "it's not their turn", "X is 5 or more".
fn special(c: &str, b: &mut Builder) -> Option<Condition> {
    if let Some(cond) = game_state(c, b) {
        return Some(cond);
    }
    for (p, rel, neg) in [
        ("you control ", PlayerRel::You, false),
        ("you don't control ", PlayerRel::You, true),
        ("an opponent controls ", PlayerRel::Opponent, false),
        ("another player controls ", PlayerRel::NotYou, false),
    ] {
        if let Some(r) = c.strip_prefix(p) {
            if !(r.starts_with("that ") || matches!(r, "it" | "him" | "her")) {
                continue;
            }
            let before = b.targets.len();
            let (sel, rest) = object_ref(r, b)?;
            if b.targets.len() != before || !rest.trim().is_empty() || is_no_referent(&sel) {
                b.targets.truncate(before);
                return None;
            }
            let cond = Condition::SelMatches(sel, Filter::ControlledBy(rel));
            return Some(if neg {
                Condition::Not(Box::new(cond))
            } else {
                cond
            });
        }
    }
    // "it's that player's turn", "it's not their turn".
    for (p, neg) in [
        ("it's that player's turn", false),
        ("it isn't that player's turn", true),
        ("it's not that player's turn", true),
        ("it's their turn", false),
        ("it's not their turn", true),
        ("it isn't their turn", true),
    ] {
        if c == p {
            if is_no_player_referent(&b.it_player) || matches!(b.it_player, PlayerRef::You) {
                return None;
            }
            let cond = Condition::PlayerMatches(b.it_player.clone(), PlayerFilter::Active);
            return Some(if neg {
                Condition::Not(Box::new(cond))
            } else {
                cond
            });
        }
    }
    // "you cast it", "you didn't cast it from your hand": about a permanent that was a
    // spell (the trigger's object: "Whenever a creature you control enters, if you cast
    // it, ..."). The source's own cast status is the core's (CR 601.2a).
    for (p, neg, name) in [
        ("you cast ", false, crate::custom::PERMANENT_WAS_CAST),
        ("you didn't cast ", true, crate::custom::PERMANENT_WAS_CAST),
    ] {
        let Some(r) = c.strip_prefix(p) else {
            continue;
        };
        let (who, from_hand) = match r.strip_suffix(" from your hand") {
            Some(w) => (w, true),
            None => (r, false),
        };
        if !matches!(who, "it" | "that creature" | "him" | "her") {
            continue;
        }
        let sel = it_referent(b)?;
        if matches!(sel, Sel::This) {
            return None;
        }
        let name = if from_hand {
            crate::custom::PERMANENT_CAST_FROM_HAND
        } else {
            name
        };
        let cond = Condition::SelMatches(sel, Filter::Custom(name.into()));
        return Some(if neg {
            Condition::Not(Box::new(cond))
        } else {
            cond
        });
    }
    // "X is 5 or more", "X is 3".
    if let Some(r) = c.strip_prefix("x is ") {
        let (cmp, v) = value_cmp(r, b)?;
        return Some(Condition::Compare(Value::X, cmp, v));
    }
    None
}

/// Conditions about the game whose wording has a dummy "it" or names objects by
/// description: "it's night", "it's your main phase", "it's an opponent's turn", "you
/// don't control a creature named Keimi", "a graveyard has twenty or more cards in it",
/// "you control the creature with the greatest power or tied for the greatest power",
/// "you control more creatures than that spell's controller", "two or more permanents you
/// don't control have an aim counter on them", "there are no echo counters on ~", "no
/// opponent has more life than that player".
fn game_state(c: &str, b: &mut Builder) -> Option<Condition> {
    let cond = match c {
        "it's night" => Condition::IsNight,
        "it's day" => Condition::IsDay,
        "it's your turn" => Condition::YourTurn,
        "it's not your turn" => Condition::NotYourTurn,
        "it's your main phase" => {
            Condition::And(vec![Condition::YourTurn, Condition::Phase(PhaseCond::MainPhase)])
        }
        "it's an opponent's turn" => {
            Condition::PlayerMatches(PlayerRef::ActivePlayer, PlayerFilter::Opponent)
        }
        "a graveyard has twenty or more cards in it" => Condition::PlayerMatches(
            PlayerRef::EachPlayer,
            PlayerFilter::GraveyardSize(Cmp::Ge, Box::new(Value::c(20))),
        ),
        // Some creature you control has power at least as great as any creature's.
        "you control the creature with the greatest power or tied for the greatest power" => {
            Condition::And(vec![
                Condition::Exists(Filter::creature().you_control()),
                Condition::Compare(
                    Value::GreatestPower(Filter::creature().you_control()),
                    Cmp::Ge,
                    Value::GreatestPower(Filter::creature()),
                ),
            ])
        }
        _ => return game_state_phrases(c, b),
    };
    Some(cond)
}

fn game_state_phrases(c: &str, b: &mut Builder) -> Option<Condition> {
    // "you don't control a creature named Keimi", "you don't control a Pest creature token"
    for (p, neg) in [("you don't control ", true), ("you control no ", true)] {
        if let Some(r) = c.strip_prefix(p) {
            let f = named_phrase(r, b)?;
            let cond = Condition::Exists(f.you_control());
            return Some(if neg {
                Condition::Not(Box::new(cond))
            } else {
                cond
            });
        }
    }
    // "you control three or more permanents you don't own"
    if let Some(r) = c.strip_prefix("you control ") {
        if let Some((cmp, n, rest)) = amount_cmp(r) {
            let rest = rest.trim();
            let f = match rest.strip_suffix(" you don't own") {
                Some(p) => Filter::and(vec![
                    color_or_phrase(p)?,
                    Filter::not(Filter::OwnedBy(PlayerRel::You)),
                ]),
                None => color_or_phrase(rest)?,
            };
            return Some(Condition::Compare(Value::Count(f.you_control()), cmp, n));
        }
    }
    // "you control more creatures than that spell's controller"
    if let Some(r) = c.strip_prefix("you control more ") {
        let (noun, who) = r.split_once(" than ")?;
        let f = color_or_phrase(noun)?;
        let (p, rest) = player_subject(who, b)?;
        if !rest.trim().is_empty() {
            return None;
        }
        let theirs = Filter::and(vec![
            f.clone(),
            Filter::ControllerMatches(Box::new(PlayerFilter::Ref(Box::new(p)))),
        ]);
        return Some(Condition::Compare(
            Value::Count(f.you_control()),
            Cmp::Gt,
            Value::Count(theirs),
        ));
    }
    // "no opponent has more life than that player"
    if let Some(r) = c.strip_prefix("no opponent has more life than ") {
        let (p, rest) = player_subject(r, b)?;
        if !rest.trim().is_empty() {
            return None;
        }
        return Some(Condition::Not(Box::new(Condition::PlayerMatches(
            PlayerRef::EachOpponent,
            PlayerFilter::Life(Cmp::Gt, Box::new(Value::LifeTotal(p))),
        ))));
    }
    // "that opponent has more life than another of your opponents"
    if let Some(r) = c.strip_suffix(" has more life than another of your opponents") {
        let (p, rest) = player_subject(r, b)?;
        if !rest.trim().is_empty() {
            return None;
        }
        let others = PlayerRef::Each(PlayerFilter::And(vec![
            PlayerFilter::Opponent,
            PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(p.clone())))),
        ]));
        return Some(Condition::PlayerMatches(
            others,
            PlayerFilter::Life(Cmp::Lt, Box::new(Value::LifeTotal(p))),
        ));
    }
    // "no cards are in that graveyard" (the graveyard of the card an earlier instruction
    // named).
    if c == "no cards are in that graveyard" {
        let it = it_referent(b)?;
        if matches!(it, Sel::This) {
            return None;
        }
        return Some(Condition::Compare(
            Value::GraveyardSize(PlayerRef::OwnerOf(Box::new(it))),
            Cmp::Eq,
            Value::c(0),
        ));
    }
    // "there are no echo counters on ~", "there are three or more counters on it"
    if let Some(r) = c.strip_prefix("there are ").or_else(|| c.strip_prefix("there is ")) {
        let (body, on) = r.rsplit_once(" on ")?;
        let (s, rest) = subject(on, b)?;
        let Subject::Object(sel) = s else {
            return None;
        };
        if !rest.trim().is_empty() {
            return None;
        }
        return counters(&sel, &format!("{body} on it"));
    }
    // "two or more permanents you don't control have an aim counter on them"
    if let Some((cmp, n, rest)) = amount_cmp(c) {
        let (phrase, counter) = rest.split_once(" have ")?;
        let f = color_or_phrase(phrase.trim())?;
        let body = counter.strip_suffix(" on them")?;
        let kind = body
            .strip_prefix("a ")
            .or_else(|| body.strip_prefix("an "))
            .and_then(|k| k.strip_suffix(" counter"))
            .or_else(|| {
                body.strip_prefix("one or more ")
                    .and_then(|k| k.strip_suffix(" counters"))
            })?;
        let f = Filter::and(vec![f, Filter::HasCounter(Some(kind.into()))]);
        return Some(Condition::Compare(Value::Count(f), cmp, n));
    }
    None
}

/// An object phrase that may end with "named [name]": "a creature named Keimi", "a Pest
/// creature token", "a creature named Keeper of ~".
fn named_phrase(r: &str, b: &Builder) -> Option<Filter> {
    let r = r
        .strip_prefix("a ")
        .or_else(|| r.strip_prefix("an "))
        .unwrap_or(r);
    if let Some((phrase, name)) = r.split_once(" named ") {
        let (f, _, tail) = parse_object_phrase(phrase)?;
        if !end(tail).is_empty() || name.is_empty() {
            return None;
        }
        let name = name.replace('~', b.ctx.card_name);
        return Some(Filter::and(vec![f, Filter::Named(name.into())]));
    }
    // "a Pest creature token"
    if let Some(p) = r.strip_suffix(" token") {
        return Some(Filter::and(vec![color_or_phrase(p)?, Filter::Token]));
    }
    color_or_phrase(r)
}

/// A player named in a condition: "that player", "that spell's controller", "its
/// controller", "you".
fn player_subject(r: &str, b: &mut Builder) -> Option<(PlayerRef, String)> {
    if let Some(rest) = r.strip_prefix("you") {
        if rest.is_empty() || rest.starts_with(' ') {
            return Some((PlayerRef::You, rest.to_string()));
        }
    }
    match subject(r, b)? {
        (Subject::Player(p), rest) => Some((p, rest)),
        (Subject::Object(sel), rest) => {
            let rest = rest.strip_prefix("'s controller")?;
            Some((PlayerRef::ControllerOf(Box::new(sel)), rest.to_string()))
        }
    }
}

/// Predicates of an object subject. `r` is the rest after the subject: "'s [stat] is
/// ...", " 's-contracted [state]" (after "it's"), or " [verb] ...".
fn object_predicate(sel: &Sel, r: &str, b: &mut Builder) -> Option<Condition> {
    if let Some(state) = r.strip_prefix(" 's-contracted ") {
        // "it's a creature", "it's not a token", "it's attacking".
        let (neg, state) = match state.strip_prefix("not ") {
            Some(s) => (true, s),
            None => (false, state),
        };
        return is_state(sel, state, neg, b);
    }
    if let Some(r) = r.strip_prefix("'s ") {
        return possessive(sel, r, b);
    }
    let r = r.trim();
    for (p, neg) in [
        ("isn't ", true),
        ("wasn't ", true),
        ("is not ", true),
        ("was not ", true),
        ("is ", false),
        ("was ", false),
    ] {
        if let Some(s) = r.strip_prefix(p) {
            return is_state(sel, s, neg, b);
        }
    }
    for (p, neg) in [
        ("doesn't have ", true),
        ("didn't have ", true),
        ("does not have ", true),
        ("has ", false),
        ("had ", false),
    ] {
        if let Some(h) = r.strip_prefix(p) {
            let c = has(sel, h, b)?;
            return Some(if neg { Condition::Not(Box::new(c)) } else { c });
        }
    }
    if let Some(x) = r.strip_prefix("shares ") {
        return shares(sel, x, b);
    }
    None
}

/// "[subject] is/was [state]".
fn is_state(sel: &Sel, s: &str, neg: bool, b: &mut Builder) -> Option<Condition> {
    let s = end(s);
    let cond = if let Some(f) = object_state(s) {
        Condition::SelMatches(sel.clone(), f)
    } else if let Some(r) = s.strip_prefix("exiled with ") {
        // "~ is exiled with an egg counter on it"
        Condition::And(vec![
            Condition::SelMatches(sel.clone(), Filter::InZone(ZoneKind::Exile)),
            counters(sel, r)?,
        ])
    } else if let Some(r) = s.strip_prefix("enchanted by ") {
        // "~ is enchanted by two or more Auras"
        if !matches!(sel, Sel::This) {
            return None;
        }
        let (cmp, n, rest) = amount_cmp(r)?;
        let (f, _, tail) = parse_object_phrase(rest.trim())?;
        if !end(tail).is_empty() {
            return None;
        }
        Condition::Compare(
            Value::Count(Filter::and(vec![f, Filter::AttachedToSource])),
            cmp,
            n,
        )
    } else if let Some(p) = s.strip_prefix("attacking ") {
        // "it's attacking a battle", "it's attacking you".
        let f = match p {
            "you" => Filter::AttackingPlayer(PlayerRel::You),
            _ => return None,
        };
        Condition::SelMatches(sel.clone(), f)
    } else {
        // "it's a spell with lesser mana value" and other relational states: not here.
        let _ = b;
        return None;
    };
    Some(if neg {
        Condition::Not(Box::new(cond))
    } else {
        cond
    })
}

/// A state phrase as a filter: "attacking", "a creature", "white or blue", "legendary",
/// "a token", "historic", "a nonland creature", "an enchantment creature or legendary
/// creature", "a 1/1".
pub(crate) fn object_state(s: &str) -> Option<Filter> {
    let s = end(s);
    match s {
        "a token" => return Some(Filter::Token),
        "not a token" => return Some(Filter::not(Filter::Token)),
        "historic" => return Some(Filter::Historic),
        "a card" => return Some(Filter::Card),
        "blocked" => return Some(Filter::Blocked),
        "unblocked" => return Some(Filter::Unblocked),
        // "it was dealt [noncombat] damage this turn" (Grisly Sigil).
        "dealt damage this turn" => return Some(Filter::DealtDamageThisTurn),
        "dealt noncombat damage this turn" => {
            return Some(Filter::Custom(
                crate::kw::noncombat_damage::DEALT_NONCOMBAT_DAMAGE_THIS_TURN.into(),
            ))
        }
        "on the battlefield" => return Some(Filter::InZone(ZoneKind::Battlefield)),
        "exiled" | "in exile" => return Some(Filter::InZone(ZoneKind::Exile)),
        "in your graveyard" => {
            return Some(Filter::and(vec![
                Filter::InZone(ZoneKind::Graveyard),
                Filter::OwnedBy(PlayerRel::You),
            ]))
        }
        "all colors" => {
            return Some(Filter::and(
                Color::ALL.iter().map(|c| Filter::Color(*c)).collect(),
            ))
        }
        _ => {}
    }
    // "a 1/1", "2/2".
    let pt = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    if let Some((p, t)) = pt.split_once('/') {
        if let (Ok(p), Ok(t)) = (p.parse::<i32>(), t.parse::<i32>()) {
            return Some(Filter::and(vec![
                Filter::Power(Cmp::Eq, Box::new(Value::c(p))),
                Filter::Toughness(Cmp::Eq, Box::new(Value::c(t))),
            ]));
        }
    }
    // A single adjective ("nonbasic", "suspected").
    if !s.contains(' ') {
        if let Some(f) = crate::oracle::phrases::adjective(s) {
            if crate::oracle::phrases::head_noun(s).is_none() {
                return Some(f);
            }
        }
    }
    // "an enchanted creature or enchantment creature": alternatives of several words each
    // are separate states (not adjectives sharing the last noun, as in "a red or green
    // creature").
    let parts: Vec<&str> = s.split(" or ").collect();
    let first = parts[0]
        .strip_prefix("a ")
        .or_else(|| parts[0].strip_prefix("an "))
        .unwrap_or(parts[0]);
    let separate = parts.len() > 1 && (first.contains(' ') || parts.iter().all(|p| !p.contains(' ')));
    if !separate {
        if let Some(f) = state_filter(s) {
            return Some(f);
        }
    } else {
        let alts: Option<Vec<Filter>> = parts.iter().map(|p| object_state(p)).collect();
        if let Some(alts) = alts {
            return Some(Filter::Or(alts));
        }
        return None;
    }
    // "a land or double-faced card".
    let mut alts = Vec::new();
    for part in s.split(" or ") {
        let p = part
            .strip_prefix("a ")
            .or_else(|| part.strip_prefix("an "))
            .unwrap_or(part);
        let (f, _, tail) = parse_object_phrase(p)?;
        if !end(tail).is_empty() {
            return None;
        }
        alts.push(f);
    }
    match alts.len() {
        0 => None,
        1 => alts.pop(),
        _ => Some(Filter::Or(alts)),
    }
}

/// "[subject] has/had ...": counters, keywords, mana value, power, toughness.
fn has(sel: &Sel, h: &str, b: &mut Builder) -> Option<Condition> {
    let h = end(h);
    if let Some(c) = counters(sel, h) {
        return Some(c);
    }
    if h == "the chosen name" {
        return Some(Condition::SelMatches(sel.clone(), Filter::ChosenName));
    }
    if let Some(k) = KeywordKind::from_name(h) {
        return Some(Condition::SelMatches(sel.clone(), Filter::HasKeyword(k)));
    }
    for (p, stat) in [
        ("mana value ", Stat::ManaValue),
        ("power ", Stat::Power),
        ("toughness ", Stat::Toughness),
    ] {
        if let Some(r) = h.strip_prefix(p) {
            let (cmp, v) = value_cmp(r, b)?;
            return Some(Condition::Compare(stat.of(sel), cmp, v));
        }
    }
    // "the same mana value as the discarded card"
    for (p, stat) in [
        ("the same mana value as ", Stat::ManaValue),
        ("the same power as ", Stat::Power),
    ] {
        if let Some(r) = h.strip_prefix(p) {
            let other = other_object(r, b)?;
            return Some(Condition::Compare(stat.of(sel), Cmp::Eq, stat.of(&other)));
        }
    }
    // "greater power or toughness than ~" (either is greater).
    if let Some(r) = h.strip_prefix("greater power or toughness than ") {
        let other = other_object(r, b)?;
        return Some(Condition::Or(vec![
            Condition::Compare(Stat::Power.of(sel), Cmp::Gt, Stat::Power.of(&other)),
            Condition::Compare(Stat::Toughness.of(sel), Cmp::Gt, Stat::Toughness.of(&other)),
        ]));
    }
    for (p, stat, cmp) in [
        ("greater power than ", Stat::Power, Cmp::Gt),
        ("power greater than ", Stat::Power, Cmp::Gt),
        ("greater toughness than ", Stat::Toughness, Cmp::Gt),
        ("less power than ", Stat::Power, Cmp::Lt),
        ("lesser power than ", Stat::Power, Cmp::Lt),
    ] {
        if let Some(r) = h.strip_prefix(p) {
            // "power greater than ~'s power"
            if let Some((v, rest)) = crate::oracle::statics::parse_value_phrase(r, b) {
                if rest.trim().is_empty() {
                    return Some(Condition::Compare(stat.of(sel), cmp, v));
                }
            }
            let other = other_object(r, b)?;
            return Some(Condition::Compare(stat.of(sel), cmp, stat.of(&other)));
        }
    }
    None
}

#[derive(Clone, Copy)]
enum Stat {
    Power,
    Toughness,
    ManaValue,
}

impl Stat {
    fn of(self, sel: &Sel) -> Value {
        let s = Box::new(sel.clone());
        match self {
            Stat::Power => Value::PowerOf(s),
            Stat::Toughness => Value::ToughnessOf(s),
            Stat::ManaValue => Value::ManaValueOf(s),
        }
    }
}

/// Another object named in a comparison: "~", "that creature", "the discarded card".
fn other_object(r: &str, b: &mut Builder) -> Option<Sel> {
    let (s, rest) = subject(end(r), b)?;
    if !rest.trim().is_empty() {
        return None;
    }
    match s {
        Subject::Object(sel) => Some(sel),
        Subject::Player(_) => None,
    }
}

/// "[subject]'s power is 3 or less", "its mana value was 2 or less", "its power is
/// greater than ~'s power or its toughness is greater than ~'s toughness" (the latter
/// joined by [`joined`]).
fn possessive(sel: &Sel, r: &str, b: &mut Builder) -> Option<Condition> {
    for (p, stat) in [
        ("mana value ", Stat::ManaValue),
        ("power ", Stat::Power),
        ("toughness ", Stat::Toughness),
    ] {
        if let Some(r) = r.strip_prefix(p) {
            let r = r
                .strip_prefix("is ")
                .or_else(|| r.strip_prefix("was "))?;
            if let Some(k) = r.strip_prefix("different from its base ") {
                if p == "power " && k == "power" {
                    return Some(Condition::SelMatches(
                        sel.clone(),
                        Filter::Not(Box::new(Filter::PowerVsBase(Cmp::Eq))),
                    ));
                }
                return None;
            }
            // "its power is greater than ~'s"
            if let Some(o) = r.strip_prefix("greater than ").and_then(|o| o.strip_suffix("'s")) {
                let other = other_object(o, b)?;
                return Some(Condition::Compare(stat.of(sel), Cmp::Gt, stat.of(&other)));
            }
            let (cmp, v) = value_cmp(r, b)?;
            return Some(Condition::Compare(stat.of(sel), cmp, v));
        }
    }
    None
}

/// "[kind] counter(s) on it": "a +1/+1 counter on it", "two or more +1/+1 counters on
/// it", "fewer than four +1/+1 counters on it", "no counters on it", "a counter on it",
/// "loyalty counters on it".
fn counters(sel: &Sel, h: &str) -> Option<Condition> {
    let body = [" on it", " on ~", " on him", " on her", " on them"]
        .iter()
        .find_map(|s| h.strip_suffix(s))?;
    if let Some((cmp, n, kind)) = super::statics_conditions::counters_on_it(&format!("{body} on it"))
    {
        return Some(Condition::Compare(
            Value::CountersOn(Box::new(sel.clone()), kind),
            cmp,
            n,
        ));
    }
    // "fewer than three charge counters", "loyalty counters" (any number).
    let (cmp, n, rest) = match amount_cmp(body) {
        Some(x) => x,
        None => (Cmp::Ge, Value::c(1), body),
    };
    let rest = rest.trim();
    let kind = if matches!(rest, "counters" | "counter") {
        None
    } else {
        let (k, tail) = rest.split_once(' ')?;
        if !matches!(tail, "counters" | "counter") {
            return None;
        }
        Some(k.into())
    };
    Some(Condition::Compare(
        Value::CountersOn(Box::new(sel.clone()), kind),
        cmp,
        n,
    ))
}

/// "shares a color with [object]", "shares a creature type with ...", "shares a card
/// type with ...".
fn shares(sel: &Sel, x: &str, b: &mut Builder) -> Option<Condition> {
    let (mk, r): (fn(Box<Sel>) -> Filter, &str) = if let Some(r) = x.strip_prefix("a color with ")
    {
        (Filter::SharesColor, r)
    } else if let Some(r) = x.strip_prefix("a creature type with ") {
        (Filter::SharesCreatureType, r)
    } else if let Some(r) = x.strip_prefix("a card type with ") {
        (Filter::SharesCardType, r)
    } else {
        return None;
    };
    // "with that permanent", "with ~".
    if let Some(other) = other_object(r, b) {
        // Both phrases resolving to one object: one of them names something untracked.
        if format!("{other:?}") == format!("{sel:?}") {
            return None;
        }
        return Some(Condition::SelMatches(sel.clone(), mk(Box::new(other))));
    }
    // "with a creature you control": some such object shares one with it.
    let a = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(a)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Exists(Filter::and(vec![
        f,
        Filter::not(Filter::In(Box::new(sel.clone()))),
        mk(Box::new(sel.clone())),
    ])))
}

/// A comparison with a value: "3 or less", "2 or greater", "at least 3", "exactly 4",
/// "3", "x", "less than or equal to the number of cards in your hand", "greater than ~'s
/// power", "equal to the amount of unspent mana you have".
fn value_cmp(r: &str, b: &mut Builder) -> Option<(Cmp, Value)> {
    let r = end(r.trim());
    for (p, cmp) in [
        ("less than or equal to ", Cmp::Le),
        ("greater than or equal to ", Cmp::Ge),
        ("equal to ", Cmp::Eq),
        ("less than ", Cmp::Lt),
        ("greater than ", Cmp::Gt),
    ] {
        if let Some(v) = r.strip_prefix(p) {
            if let Some((n, rest)) = parse_number(v) {
                if rest.trim().is_empty() {
                    return Some((cmp, n));
                }
            }
            let (v, rest) = crate::oracle::statics::parse_value_phrase(v, b)?;
            if !rest.trim().is_empty() {
                return None;
            }
            return Some((cmp, v));
        }
    }
    if let Some((cmp, n, rest)) = amount_cmp(r) {
        if rest.trim().is_empty() {
            return Some((cmp, n));
        }
        return None;
    }
    let (n, rest) = parse_number(r)?;
    rest.trim().is_empty().then_some((Cmp::Eq, n))
}

/// "[player] [predicate]".
fn player_condition(p: &PlayerRef, r: &str, b: &mut Builder) -> Option<Condition> {
    let r = end(r);
    let m = |f: PlayerFilter| Condition::PlayerMatches(p.clone(), f);
    match r {
        "is you" => return Some(m(PlayerFilter::You)),
        "isn't you" => return Some(m(PlayerFilter::NotYou)),
        "is your opponent" | "is an opponent" => return Some(m(PlayerFilter::Opponent)),
        "is the monarch" => return Some(m(PlayerFilter::Monarch)),
        _ => {}
    }
    // "controls more lands than you", "has more life than you".
    if let Some(x) = r.strip_prefix("controls more ") {
        if let Some(noun) = x.strip_suffix(" than you") {
            let f = color_or_phrase(noun)?;
            return Some(m(PlayerFilter::Controls(
                Box::new(f.clone()),
                Cmp::Gt,
                Box::new(Value::Count(f.you_control())),
            )));
        }
        // "controls more lands than each other player"
        if let Some(noun) = x.strip_suffix(" than each other player") {
            // No other player controls as many.
            let f = color_or_phrase(noun)?;
            let theirs = Filter::and(vec![
                f.clone(),
                Filter::ControllerMatches(Box::new(PlayerFilter::Ref(Box::new(p.clone())))),
            ]);
            let others = PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::Ref(
                Box::new(p.clone()),
            ))));
            return Some(Condition::Not(Box::new(Condition::PlayerMatches(
                others,
                PlayerFilter::Controls(Box::new(f), Cmp::Ge, Box::new(Value::Count(theirs))),
            ))));
        }
        return None;
    }
    if let Some(x) = r
        .strip_prefix("doesn't control ")
        .or_else(|| r.strip_prefix("controls no "))
    {
        let f = other_than_referent(control_phrase(x)?, p)?;
        return Some(Condition::Not(Box::new(m(PlayerFilter::Controls(
            Box::new(f),
            Cmp::Ge,
            Box::new(Value::c(1)),
        )))));
    }
    // "has three or more poison counters", "doesn't have any rad counters".
    for (pre, neg) in [
        ("has ", false),
        ("have ", false),
        ("don't have ", true),
        ("doesn't have ", true),
    ] {
        let Some(x) = r.strip_prefix(pre) else {
            continue;
        };
        let (cmp, n, rest) = if let Some(k) = x.strip_prefix("any ") {
            (Cmp::Ge, Value::c(1), k)
        } else if let Some(k) = x.strip_prefix("no ") {
            (Cmp::Eq, Value::c(0), k)
        } else {
            match amount_cmp(x) {
                Some(v) => v,
                None => continue,
            }
        };
        let Some(kind) = rest.trim().strip_suffix(" counters").or_else(|| rest.trim().strip_suffix(" counter")) else {
            continue;
        };
        if kind.contains(' ') || kind == "cards" {
            continue;
        }
        let f = PlayerFilter::Counters(kind.into(), cmp, Box::new(n));
        return Some(if neg {
            Condition::Not(Box::new(m(f)))
        } else {
            m(f)
        });
    }
    if r == "has more life than each other player" {
        let others = PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
            p.clone(),
        )))));
        return Some(Condition::Not(Box::new(Condition::PlayerMatches(
            others,
            PlayerFilter::Life(Cmp::Ge, Box::new(Value::LifeTotal(p.clone()))),
        ))));
    }
    // "has more cards in hand than each other player"
    if r == "has more cards in hand than each other player" {
        let others = PlayerRef::Each(PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
            p.clone(),
        )))));
        return Some(Condition::Not(Box::new(Condition::PlayerMatches(
            others,
            PlayerFilter::HandSize(Cmp::Ge, Box::new(Value::HandSize(p.clone()))),
        ))));
    }
    // "has N or less life" (also "has N life or less").
    if let Some(x) = r.strip_prefix("has ") {
        if let Some((n, rest)) = parse_number(x) {
            for (s, cmp) in [(" life or less", Cmp::Le), (" life or more", Cmp::Ge)] {
                if rest == &s[1..] || format!(" {rest}") == s {
                    return Some(m(PlayerFilter::Life(cmp, Box::new(n))));
                }
            }
        }
    }
    if let Some(f) = player_predicate(r) {
        return Some(m(f));
    }
    // "controls one or more lands with contested counters on them"
    if let Some(x) = r.strip_prefix("controls ") {
        let (cmp, n, rest) = amount_cmp(x)?;
        let f = other_than_referent(control_phrase(rest)?, p)?;
        return Some(m(PlayerFilter::Controls(Box::new(f), cmp, Box::new(n))));
    }
    let _ = b;
    None
}

/// "Other" in what a player controls is relative to the object that player was named by
/// ("target creature an opponent controls gets -2/-2 ... if that opponent controls no
/// other creatures"), not the ability's source. Without such an object, not understood.
fn other_than_referent(f: Filter, p: &PlayerRef) -> Option<Filter> {
    fn has_other(f: &Filter) -> bool {
        match f {
            Filter::Other => true,
            Filter::And(v) | Filter::Or(v) => v.iter().any(has_other),
            Filter::Not(x) => has_other(x),
            _ => false,
        }
    }
    if !has_other(&f) {
        return Some(f);
    }
    let PlayerRef::ControllerOf(sel) = p else {
        return None;
    };
    fn replace(f: Filter, sel: &Sel) -> Filter {
        match f {
            Filter::Other => Filter::not(Filter::In(Box::new(sel.clone()))),
            Filter::And(v) => Filter::And(v.into_iter().map(|x| replace(x, sel)).collect()),
            Filter::Or(v) => Filter::Or(v.into_iter().map(|x| replace(x, sel)).collect()),
            Filter::Not(x) => Filter::Not(Box::new(replace(*x, sel))),
            other => other,
        }
    }
    Some(replace(f, sel))
}

/// An object phrase after "controls": "a nonblack creature", "creatures", "a Plains",
/// "lands with contested counters on them".
fn control_phrase(x: &str) -> Option<Filter> {
    let x = end(x.trim());
    let a = x
        .strip_prefix("a ")
        .or_else(|| x.strip_prefix("an "))
        .unwrap_or(x);
    if let Some(f) = color_or_phrase(a) {
        return Some(f);
    }
    // "nonblack" alone ("controls a nonblack" means a nonblack permanent... not used).
    let (f, _, tail) = parse_object_phrase(a)?;
    let tail = end(tail.trim());
    if tail.is_empty() {
        return Some(f);
    }
    // "with [kind] counters on them/it"
    let w = tail.strip_prefix("with ")?;
    let body = w
        .strip_suffix(" on them")
        .or_else(|| w.strip_suffix(" on it"))?;
    let body = body
        .strip_prefix("a ")
        .or_else(|| body.strip_prefix("an "))
        .unwrap_or(body);
    let kind = if matches!(body, "counters" | "counter") {
        None
    } else {
        let (k, t) = body.split_once(' ')?;
        if !matches!(t, "counters" | "counter") {
            return None;
        }
        Some(k.into())
    };
    Some(Filter::and(vec![f, Filter::HasCounter(kind)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;

    fn ctx(tl: &TypeLine) -> CompileContext<'_> {
        CompileContext {
            card_name: "Test",
            full_name: "Test",
            type_line: tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        }
    }

    #[test]
    fn subject_predicate_conditions() {
        let tl = TypeLine::parse("Creature — Human");
        let c = ctx(&tl);
        let mut b = Builder::new(&c);
        b.it = Sel::Target(0);
        b.it_player = PlayerRef::ControllerOf(Box::new(Sel::Target(0)));
        for s in [
            "it's white",
            "that creature was a human",
            "it isn't a token",
            "it's not a token",
            "it has flying",
            "it doesn't have suspend",
            "that creature had a +1/+1 counter on it",
            "it has mana value 2 or less",
            "its mana value is 3 or less",
            "that creature's power is 2 or less",
            "its power is less than or equal to the number of cards in your hand",
            "it shares a color with a creature you control",
            "it has greater power or toughness than ~",
            "that player has two or fewer cards in hand",
            "that player controls more lands than you",
            "that player is you",
            "its controller is poisoned",
            "you control that creature",
            "x is 5 or more",
            "the sacrificed creature was legendary",
            "that creature is white or blue",
            "it's an enchantment creature or legendary creature",
            "that player is your opponent and has four or more cards in hand",
            "it's your main phase",
            "it's an opponent's turn",
            "it's not their turn",
            "you don't control a creature named keimi",
            "you control the creature with the greatest power or tied for the greatest power",
            "you control more creatures than that spell's controller",
            "no opponent has more life than that player",
            "its controller has three or more poison counters",
            "a graveyard has twenty or more cards in it",
            "there are no echo counters on ~",
            "two or more permanents you don't control have an aim counter on them",
            "you control three or more permanents you don't own",
            "that land was nonbasic",
            "it was dealt noncombat damage this turn",
            "~ is enchanted by two or more auras",
            "you control a creature with a +1/+1 counter on it",
        ] {
            assert!(parse_condition_with(s, &mut b).is_some(), "{s}");
        }
        assert!(b.targets.is_empty());
        // Without an antecedent, a pronoun isn't understood.
        let tl = TypeLine::parse("Instant");
        let c = ctx(&tl);
        let mut b = Builder::new(&c);
        for s in ["it's white", "that creature was a human", "that player is you"] {
            assert!(parse_condition_with(s, &mut b).is_none(), "{s}");
        }
    }
}
