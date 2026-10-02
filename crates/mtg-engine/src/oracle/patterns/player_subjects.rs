//! Instructions with any player as the subject: "that player exiles the top four cards
//! of their library", "target opponent reveals their hand", "each player may discard a
//! card", "each player who controls six or more lands chooses five lands they control and
//! sacrifices the rest", "you and target opponent each draw two cards", "if they do, they
//! put a rope counter on a creature they control".
//!
//! The grammar is generic: the subject is stripped, the predicate is reworded as the same
//! instruction for "you" (third-person verbs to the base form, "their"/"they" to
//! "your"/"you"), parsed with the ordinary instruction patterns, and performed by the
//! subject player (`Effect::AsPlayer`: "you" in it is that player, who makes its choices
//! and whose library, hand and graveyard it uses). A predicate that also mentions "you"
//! (the ability's controller) isn't reworded, since its "you" would then mean the
//! subject.
//!
//! "Each [player] may [instruction]": the players choose in APNAP order whether to do it,
//! each knowing the earlier choices (CR 101.4, 101.4b); then those who accepted do it.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, parse_sentence, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::kw::each_player_collect::{COLLECT, COLLECTED, PICK};
use crate::scry_rules::{OPTED, OPT_IN};

/// The players who accepted the most recent "each [player] may [instruction]".
pub const ACCEPTED: Var = vars::USER + 2741;

/// Marks, in [`Builder::named`], that an earlier sentence was "each [player] may
/// [instruction]" (the later "each player who does" refers to it).
const ACCEPTED_NAME: &str = "\u{1}the players who accepted";

/// Who performs the instruction.
enum Subject {
    /// One player (or the players a reference names, each in turn when it names several).
    One(PlayerRef),
    /// Each of the players, in APNAP order (CR 101.4), each performing it; only those
    /// for whom the condition (worded for "you") holds.
    Each(PlayerRef, Option<Condition>),
    /// "you and [player] each": you, then that player.
    YouAnd(PlayerRef),
}

/// Base verbs a third-person form may be turned into (the whitelist keeps nouns and
/// other words ending in "s" from being "conjugated").
const VERBS: &[&str] = &[
    "attach", "become", "bid", "cast", "choose", "control", "create", "discard",
    "discover", "draw", "exile", "gain", "get", "have", "investigate", "look", "lose",
    "mill", "own", "pay", "play", "proliferate", "put", "reveal", "return", "sacrifice",
    "scry", "search", "separate", "shuffle", "surveil", "take", "tap", "untap", "do",
];

/// The base form of a third-person singular verb ("draws" → "draw", "searches" →
/// "search", "scries" → "scry", "has" → "have", "does" → "do").
fn base_form(w: &str) -> Option<&'static str> {
    let cand: String = match w {
        "has" => "have".into(),
        "does" => "do".into(),
        _ if w.ends_with("ies") => format!("{}y", &w[..w.len() - 3]),
        _ if ["ches", "shes", "sses", "xes"].iter().any(|s| w.ends_with(s)) => {
            w[..w.len() - 2].to_string()
        }
        _ => w.strip_suffix('s')?.to_string(),
    };
    VERBS.iter().copied().find(|v| *v == cand)
}

/// Whether `t` mentions "you" (the ability's controller) as a word.
fn mentions_you(t: &str) -> bool {
    t.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .any(|w| matches!(w, "you" | "your" | "yours" | "yourself" | "you're" | "you've"))
}

/// The predicate `r` (after a third-person subject, lowercase) worded for "you": the
/// first word must be a third-person verb; verbs starting the later joined clauses are
/// put in the base form too, and the subject's "their"/"they" become "your"/"you".
fn as_you(r: &str) -> Option<String> {
    if mentions_you(r) {
        return None;
    }
    let (first, rest) = split_word(r);
    let verb = base_form(first)?;
    let mut t = format!(" {verb} {} ", rest.trim());
    // Possessives of other things ("their owners' hands", "their mana costs") stay.
    const KEEP: &[&str] = &[
        "owner", "owners", "owner's", "owners'", "controller", "controllers", "controller's",
        "controllers'", "mana", "power", "toughness",
    ];
    let words: Vec<String> = t.split(' ').map(str::to_string).collect();
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    for (i, w) in words.iter().enumerate() {
        let next = words.get(i + 1).map(String::as_str).unwrap_or("");
        let next_clean = next.trim_end_matches([',', '.']);
        let (core, punct) = match w.find([',', '.']) {
            Some(at) => (&w[..at], &w[at..]),
            None => (w.as_str(), ""),
        };
        // "discard their hands" (several players, one hand each).
        if core == "their" && next_clean == "hands" {
            out.push("your".to_string());
            continue;
        }
        if core == "hands" && i > 0 && words[i - 1] == "their" {
            out.push(format!("hand{punct}"));
            continue;
        }
        let replaced = match core {
            "their" if !KEEP.contains(&next_clean) => Some("your"),
            "theirs" => Some("yours"),
            "they" => Some("you"),
            "themselves" | "themself" => Some("yourself"),
            "they're" | "they've" | "them" => {
                // "them" is usually objects; contractions aren't reworded: give up only
                // on contractions (objects as "them" stay).
                if core == "them" {
                    None
                } else {
                    return None;
                }
            }
            _ => None,
        };
        match replaced {
            Some(x) => out.push(format!("{x}{punct}")),
            None => out.push(w.clone()),
        }
    }
    t = out.join(" ");
    // Verbs starting later clauses: ", then draws", " and sacrifices", ", loses ...".
    for sep in [", then ", " then ", ", and ", " and ", ", "] {
        let mut s = String::new();
        let mut rest = t.as_str();
        while let Some(at) = rest.find(sep) {
            s.push_str(&rest[..at + sep.len()]);
            rest = &rest[at + sep.len()..];
            let (w, _) = split_word(rest);
            if let Some(v) = base_form(w) {
                s.push_str(v);
                rest = &rest[w.len()..];
            }
        }
        s.push_str(rest);
        t = s;
    }
    Some(t.trim().to_string())
}

/// "[cond]" after "each player who"/"each opponent with": the condition worded for
/// "you" ("controls six or more lands" → "you control six or more lands", "no cards in
/// hand" after "with" → "you have no cards in hand").
fn player_condition(r: &str, with: bool, b: &Builder) -> Option<(Condition, String)> {
    // The condition ends where the predicate's verb starts: try each split.
    let words: Vec<&str> = r.split(' ').collect();
    for i in 1..words.len() {
        let cond = words[..i].join(" ");
        let pred = words[i..].join(" ");
        let first_pred = words[i];
        if base_form(first_pred).is_none() && first_pred != "may" {
            continue;
        }
        if mentions_you(&cond) {
            continue;
        }
        let worded = if with {
            format!("you have {cond}")
        } else {
            let Some(c) = as_you(&cond) else { continue };
            format!("you {c}")
        };
        if let Some(c) = crate::oracle::statics::parse_condition(&worded, b.ctx) {
            return Some((c, pred));
        }
    }
    None
}

/// The subject at the start of `l` and the rest (a predicate).
fn subject(l: &str, b: &mut Builder) -> Option<(Subject, String)> {
    use super::oracle_hardening_referents::is_no_player_referent;
    // "you and target opponent each draw two cards"
    if let Some(r) = l.strip_prefix("you and ") {
        let (other, rest) = if let Some(r) = r.strip_prefix("another target player each ") {
            let slot = b.add_target(
                TargetSpec::player(PlayerFilter::NotYou, "another target player"),
                "another target player",
            );
            b.it_player = PlayerRef::Target(slot);
            (PlayerRef::Target(slot), r.to_string())
        } else if let Some(r) = r.strip_prefix("the attacking player each ") {
            (PlayerRef::ActivePlayer, r.to_string())
        } else {
            let (who, rest) = player_ref(r, b)?;
            let rest = rest.trim_start().strip_prefix("each ")?.to_string();
            if matches!(
                who,
                PlayerRef::You
                    | PlayerRef::EachPlayer
                    | PlayerRef::EachOpponent
                    | PlayerRef::EachOtherPlayer
            ) {
                return None;
            }
            (who, rest)
        };
        return Some((Subject::YouAnd(other), rest));
    }
    // "any number of target players each discard a card", "up to two target players each
    // draw a card".
    let multi = if let Some(r) = l.strip_prefix("any number of target ") {
        Some((Value::c(99), r))
    } else if let Some(r) = l.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        r.trim_start().strip_prefix("target ").map(|r| (n, r))
    } else {
        None
    };
    if let Some((max, r)) = multi {
        let (filter, text, r) = if let Some(r) = r.strip_prefix("players each ") {
            (PlayerFilter::Any, "target players", r)
        } else {
            (
                PlayerFilter::Opponent,
                "target opponents",
                r.strip_prefix("opponents each ")?,
            )
        };
        let mut spec = TargetSpec::player(filter, text);
        spec.min = 0;
        spec.max = max;
        let slot = b.add_target(spec, text);
        // The predicate is in the plural ("each discard"): reword it as a singular one.
        return Some((
            Subject::Each(PlayerRef::Target(slot), None),
            plural_to_singular(r)?,
        ));
    }
    // "each player who controls ...", "each opponent with no cards in hand ..."
    for (p, who) in [
        ("each player ", PlayerRef::EachPlayer),
        ("each opponent ", PlayerRef::EachOpponent),
        ("each other player ", PlayerRef::EachOtherPlayer),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            for (rel, with) in [("who ", false), ("with ", true)] {
                if let Some(c) = r.strip_prefix(rel) {
                    let (cond, pred) = player_condition(c, with, b)?;
                    return Some((Subject::Each(who, Some(cond)), pred));
                }
            }
            return Some((Subject::Each(who, None), r.to_string()));
        }
    }
    // Pronouns and references for a player mentioned earlier.
    for p in ["they ", "the player ", "that opponent "] {
        if let Some(r) = l.strip_prefix(p) {
            let who = b.it_player.clone();
            if matches!(who, PlayerRef::You) || is_no_player_referent(&who) {
                return None;
            }
            // "they" for objects ("they get +1/+1") isn't a player.
            if p == "they "
                && (b.group.is_some()
                    || super::pronoun_groups::plural_referent(&b.it, b.ctx))
            {
                return None;
            }
            // "they" takes the plural verb: reword as singular.
            let rest = if p == "they " {
                plural_to_singular(r)?
            } else {
                r.to_string()
            };
            return Some((Subject::One(who), rest));
        }
    }
    if let Some(r) = l.strip_prefix("enchanted player ") {
        if !b.ctx.type_line.subtypes.iter().any(|s| s.as_str() == "Aura") {
            return None;
        }
        let who = PlayerRef::ControllerOf(Box::new(Sel::AttachedTo));
        b.it_player = who.clone();
        return Some((Subject::One(who), r.to_string()));
    }
    // "that creature's controller", "that spell's controller"
    for p in ["that creature's controller ", "that spell's controller "] {
        if let Some(r) = l.strip_prefix(p) {
            if super::oracle_hardening_referents::is_no_referent(&b.it) {
                return None;
            }
            let who = PlayerRef::ControllerOf(Box::new(b.it.clone()));
            b.it_player = who.clone();
            return Some((Subject::One(who), r.to_string()));
        }
    }
    if l.starts_with("you ") || l.starts_with("your ") {
        return None;
    }
    let (who, rest) = player_ref(l, b)?;
    if matches!(
        who,
        PlayerRef::You | PlayerRef::EachPlayer | PlayerRef::EachOpponent | PlayerRef::EachOtherPlayer
    ) {
        return None;
    }
    // "its controller"/"its owner": later "that player" is that player.
    if matches!(who, PlayerRef::ControllerOf(_) | PlayerRef::OwnerOf(_) | PlayerRef::DefendingPlayer) {
        b.it_player = who.clone();
    }
    Some((Subject::One(who), rest.trim_start().to_string()))
}

/// A predicate with the plural verb ("discard a card, then draw seven cards") in the
/// singular, so it reads as after a singular subject.
fn plural_to_singular(r: &str) -> Option<String> {
    let (first, rest) = split_word(r);
    if first == "may" {
        // "may" takes the base form either way.
        let (verb, _) = split_word(rest);
        VERBS.iter().find(|v| **v == verb)?;
        return Some(format!("may {}", rest.trim_start()));
    }
    let v = VERBS.iter().copied().find(|v| *v == first)?;
    let s = match v {
        "have" => "has".to_string(),
        "do" => "does".to_string(),
        v if v.ends_with('y') && !v.ends_with("ay") => format!("{}ies", &v[..v.len() - 1]),
        v if v.ends_with("ch") || v.ends_with("sh") || v.ends_with("ss") || v.ends_with('x') => {
            format!("{v}es")
        }
        v => format!("{v}s"),
    };
    // Later clauses already in the base form are fine for [`as_you`].
    Some(format!("{s} {rest}").trim_end().to_string())
}

/// [`as_you`] for a predicate whose verb is already in the base form (after "may").
fn as_you_base(r: &str) -> Option<String> {
    as_you(&plural_to_singular(r)?)
}

/// The value an instruction for another player hoisted out of its predicate because it
/// mentions "you" (the ability's controller): determined before that player performs it.
const HOISTED: Var = vars::USER + 2742;

/// A predicate whose amount mentions "you" ("reveals X cards from their hand, where X is
/// the number of Faeries you control", "mills cards equal to the number of cards in your
/// hand", "reveals a number of cards from their hand equal to the number of Allies you
/// control"): the predicate with the amount as "x", and the amount read for "you".
fn hoist_value(pred: &str, b: &mut Builder) -> Option<(String, Option<Value>)> {
    let has_x = |t: &str| t.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x");
    if let Some((clause, v)) = pred.rsplit_once(", where x is ") {
        if !mentions_you(v) {
            return Some((pred.to_string(), None));
        }
        let (val, tail) = crate::oracle::statics::parse_value_phrase(v, b)?;
        if !end(&tail).is_empty() || !has_x(clause) {
            return None;
        }
        return Some((clause.to_string(), Some(val)));
    }
    let Some(at) = pred.find(" equal to ") else {
        return Some((pred.to_string(), None));
    };
    let (head, v) = (&pred[..at], &pred[at + " equal to ".len()..]);
    if !mentions_you(v) {
        return Some((pred.to_string(), None));
    }
    if has_x(head) || mentions_you(head) {
        return None;
    }
    let (val, tail) = crate::oracle::statics::parse_value_phrase(v, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let head = if head.contains(" a number of ") {
        head.replacen(" a number of ", " x ", 1)
    } else {
        let (verb, rest) = split_word(head);
        let (noun, after) = split_word(rest);
        if !matches!(noun, "cards" | "life") {
            return None;
        }
        format!("{verb} x {noun} {after}").trim_end().to_string()
    };
    Some((head, Some(val)))
}

/// The predicate split before a later clause with another subject: "discards a card, you
/// draw a card, and you gain 2 life" → ("discards a card", "you draw a card, and you gain
/// 2 life"); "exiles the top four cards of their library, then you may put ...".
fn split_tail(pred: &str) -> (&str, Option<&str>) {
    let mut best: Option<(usize, usize)> = None;
    for sep in [", then you ", ", and you ", " and you ", ", you ", ", then each player who does "] {
        if let Some(at) = pred.find(sep) {
            if best.is_none_or(|(b, _)| at < b) {
                best = Some((at, sep.len()));
            }
        }
    }
    match best {
        Some((at, len)) => {
            let sep = &pred[at..at + len];
            // Keep the new subject: "you ..." / "each player who does ...".
            let start = if sep.ends_with("you ") {
                at + len - "you ".len()
            } else {
                at + len - "each player who does ".len()
            };
            (&pred[..at], Some(&pred[start..]))
        }
        None => (pred, None),
    }
}

/// An instruction worded for "you" (`text`, without the subject): as the patterns read it
/// without a subject ("draw a card") or with "you" ("you become the monarch"); failing
/// that, the original third-person predicate (`orig`) as patterns for "that player" read
/// it, with "that player" meaning the player performing it ("you" in `Effect::AsPlayer`).
fn instruction(text: &str, orig: Option<&str>, b: &mut Builder) -> Option<Effect> {
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        b.it = saved.1.clone();
        b.it_player = saved.2.clone();
    };
    if let Some(e) = parse_clause(text, b) {
        return Some(e);
    }
    restore(b);
    if let Some(e) = parse_clause(&format!("you {text}"), b) {
        return Some(e);
    }
    restore(b);
    let orig = orig?;
    // Only a predicate about that player alone: no other player is "that player".
    if orig.contains("that player") || orig.contains(" the player") {
        return None;
    }
    b.it_player = PlayerRef::You;
    let e = parse_clause(&format!("that player {orig}"), b)
        // A pattern that read "that player" as the trigger's player isn't about the
        // player performing it.
        .filter(|e| !format!("{e:?}").contains("TriggerPlayer"));
    b.it_player = saved.2.clone();
    if e.is_none() {
        restore(b);
    }
    e
}

/// Parses predicate `pred` (third person, after the subject; `base`: its verb is already
/// in the base form, as after "each" or "they") as an instruction for "you". Returns
/// whether it's optional ("may [instruction]"), the instruction (for "may", the whole
/// optional instruction for a one-player subject, `one`), and the hoisted amount.
fn predicate(
    pred: &str,
    one: bool,
    b: &mut Builder,
) -> Option<(bool, Effect, Option<Value>)> {
    let (pred, hoisted) = hoist_value(pred, b)?;
    let (may, pred) = match pred.strip_prefix("may ") {
        Some(r) => (true, r.to_string()),
        None => (false, pred),
    };
    let text = if may {
        as_you_base(&pred)?
    } else {
        as_you(&pred)?
    };
    let orig = if may {
        plural_to_singular(&pred)?
    } else {
        pred.clone()
    };
    let e = if may && one {
        // "you may pay {2}" (an optional cost, CR 118.12) and "you may [instruction]".
        match parse_sentence(&format!("you may {text}"), b) {
            Some(e) => e,
            None => Effect::May {
                who: PlayerRef::You,
                effect: Box::new(instruction(&text, Some(&orig), b)?),
            },
        }
    } else {
        instruction(&text, Some(&orig), b)?
    };
    let e = match &hoisted {
        Some(_) => {
            if !super::r107_numbers::uses_x(&e) {
                return None;
            }
            super::r107_numbers::substitute_x(&e, &Value::Var(HOISTED))?
        }
        None => e,
    };
    Some((may, e, hoisted))
}

fn player_subject(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    // "that player may draw a card if a player other than you lost life this turn": the
    // condition is the ability's ("you" is its controller), checked first.
    if let Some((head, c)) = l.rsplit_once(" if ") {
        if let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) {
            if let Some(e) = player_subject(head, b) {
                return Some(Effect::If {
                    cond,
                    then: Box::new(e),
                    otherwise: Box::new(Effect::Noop),
                });
            }
        }
    }
    let saved_targets = b.targets.len();
    let saved_player = b.it_player.clone();
    let r = (|| {
        let (subj, pred) = subject(l, b)?;
        let (pred, tail) = split_tail(&pred);
        let main = subject_effect(subj, pred, b)?;
        let Some(tail) = tail else {
            return Some(main);
        };
        let t = parse_sentence(tail, b)?;
        Some(Effect::seq(vec![main, t]))
    })();
    if r.is_none() {
        b.targets.truncate(saved_targets);
        b.it_player = saved_player;
    }
    r
}

/// The value of X as the instruction begins, for an instruction another player performs:
/// an X an outer pattern later defines ("..., where X is the number of Faeries you
/// control") is read for the ability's controller, not for that player.
const OUTER_X: Var = vars::USER + 2744;

/// `wrap` applied to `e`, with the X `e` uses read before it (see [`OUTER_X`]).
fn with_outer_x(e: Effect, wrap: impl FnOnce(Effect) -> Effect) -> Option<Effect> {
    if !super::r107_numbers::uses_x(&e) {
        return Some(wrap(e));
    }
    let inner = super::r107_numbers::substitute_x(&e, &Value::Var(OUTER_X))?;
    Some(Effect::seq(vec![
        Effect::StoreValue {
            var: OUTER_X,
            value: Value::X,
        },
        wrap(inner),
    ]))
}

/// The players' choices of "each [player] may [instruction]", in APNAP order (CR 101.4),
/// stored in [`ACCEPTED`].
fn opt_in(who: PlayerRef, guard: &dyn Fn(Effect) -> Effect) -> Vec<Effect> {
    vec![
        Effect::Store {
            var: OPTED,
            sel: Sel::None,
        },
        Effect::ForEachPlayer {
            who,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(guard(Effect::May {
                    who: PlayerRef::Iterated,
                    effect: Box::new(Effect::Custom(OPT_IN.into())),
                })),
            }),
        },
        Effect::Store {
            var: ACCEPTED,
            sel: Sel::Var(OPTED),
        },
    ]
}

/// "put a card from your hand onto the battlefield" (a choice, then a move) split into the
/// choice, which adds to [`COLLECTED`], and one move of all the players' choices (for the
/// players' own cards, under their own control).
fn split_choice_move(e: &Effect) -> Option<(Effect, Effect)> {
    let (mv, rest): (&Effect, &[Effect]) = match e {
        Effect::Seq(v) => (v.first()?, &v[1..]),
        e => (e, &[]),
    };
    let Effect::Move {
        what:
            Sel::Choose {
                chooser: PlayerRef::You,
                filter,
                count,
                up_to,
                store: None,
            },
        to,
    } = mv
    else {
        return None;
    };
    if to.zone != ZoneKind::Battlefield
        || !format!("{filter:?}").contains("OwnedBy(You)")
        || !matches!(to.controller, None | Some(PlayerRef::You))
        || rest.iter().any(|r| !matches!(r, Effect::Store { .. }))
    {
        return None;
    }
    let choose = Effect::seq(vec![
        Effect::Store {
            var: PICK,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: filter.clone(),
                count: count.clone(),
                up_to: *up_to,
                store: None,
            },
        },
        Effect::Custom(COLLECT.into()),
    ]);
    let mut to = to.clone();
    // Each card enters under its owner's control: the player who put it there.
    to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(COLLECTED))));
    let put = Effect::seq(
        std::iter::once(Effect::Move {
            what: Sel::Var(COLLECTED),
            to,
        })
        .chain(rest.iter().cloned())
        .collect(),
    );
    Some((choose, put))
}

/// The instruction `pred` performed by `subj`.
fn subject_effect(subj: Subject, pred: &str, b: &mut Builder) -> Option<Effect> {
    let with_hoisted = |e: Effect, hoisted: Option<Value>| match hoisted {
        Some(v) => Effect::seq(vec![
            Effect::StoreValue {
                var: HOISTED,
                value: v,
            },
            e,
        ]),
        None => e,
    };
    match subj {
        Subject::One(who) => {
            // "target opponent may have you draw a card": that player chooses whether you
            // do it.
            if let Some(r) = pred.strip_prefix("may have you ") {
                let e = instruction(r, None, b)?;
                return Some(Effect::May {
                    who,
                    effect: Box::new(e),
                });
            }
            let (_, e, hoisted) = predicate(pred, true, b)?;
            let e = with_outer_x(e, |e| Effect::AsPlayer {
                who,
                effect: Box::new(e),
            })?;
            Some(with_hoisted(e, hoisted))
        }
        Subject::YouAnd(other) => {
            // The predicate is already worded for "you" ("each draw two cards",
            // "each reveal the top card of your library").
            if pred.starts_with("may ") {
                return None;
            }
            let orig = plural_to_singular(pred);
            let e = instruction(pred, orig.as_deref(), b)?;
            with_outer_x(e, |e| {
                Effect::seq(vec![
                    e.clone(),
                    Effect::AsPlayer {
                        who: other,
                        effect: Box::new(e),
                    },
                ])
            })
        }
        Subject::Each(who, cond) => {
            // "Any number of target opponents each sacrifice a creature with the greatest
            // power among creatures that player controls": "that player" is each of them
            // in turn ("you" as each performs it).
            let outer_player = std::mem::replace(&mut b.it_player, PlayerRef::You);
            let parsed = predicate(pred, false, b);
            b.it_player = outer_player;
            let (may, e, hoisted) = parsed?;
            let guard = |e: Effect| match &cond {
                Some(c) => Effect::If {
                    cond: c.clone(),
                    then: Box::new(e),
                    otherwise: Box::new(Effect::Noop),
                },
                None => e,
            };
            // "Each player may put a creature card from their hand onto the battlefield":
            // the players choose in APNAP order, then the cards are put onto the
            // battlefield at the same time (CR 101.4).
            if let Some((choose, put)) = split_choice_move(&e) {
                let players = if may { PlayerRef::Var(ACCEPTED) } else { who.clone() };
                let mut v = Vec::new();
                if may {
                    v.extend(opt_in(who, &guard));
                    b.named
                        .push((ACCEPTED_NAME.to_string(), Sel::Var(ACCEPTED)));
                }
                v.push(Effect::Store {
                    var: COLLECTED,
                    sel: Sel::None,
                });
                v.push(Effect::ForEachPlayer {
                    who: players,
                    effect: Box::new(Effect::AsPlayer {
                        who: PlayerRef::Iterated,
                        effect: Box::new(if may { choose } else { guard(choose) }),
                    }),
                });
                v.push(put);
                return with_outer_x(Effect::seq(v), |e| e).map(|e| with_hoisted(e, hoisted));
            }
            // "Each player chooses six lands they control, then sacrifices the rest": the
            // players choose in APNAP order, then all the rest are sacrificed at once
            // (CR 101.4).
            if let (false, None, Effect::KeepAndSacrificeRest { who: PlayerRef::You, among, keep, up_to }) = (may, &cond, &e) {
                return Some(Effect::KeepAndSacrificeRest {
                    who,
                    among: among.clone(),
                    keep: keep.clone(),
                    up_to: *up_to,
                });
            }
            if !may {
                let e = with_outer_x(e, |e| Effect::ForEachPlayer {
                    who,
                    effect: Box::new(Effect::AsPlayer {
                        who: PlayerRef::Iterated,
                        effect: Box::new(guard(e)),
                    }),
                })?;
                return Some(with_hoisted(e, hoisted));
            }
            let x_used = super::r107_numbers::uses_x(&e);
            let e = if x_used {
                super::r107_numbers::substitute_x(&e, &Value::Var(OUTER_X))?
            } else {
                e
            };
            b.named
                .push((ACCEPTED_NAME.to_string(), Sel::Var(ACCEPTED)));
            if is_search(&e) {
                b.named
                    .push((SEARCHED_NAME.to_string(), Sel::Var(ACCEPTED)));
            }
            let pre = x_used.then_some(Effect::StoreValue {
                var: OUTER_X,
                value: Value::X,
            });
            Some(with_hoisted(
                Effect::seq(pre.into_iter().chain(vec![
                    // CR 101.4: they decide in APNAP order...
                    Effect::Store {
                        var: OPTED,
                        sel: Sel::None,
                    },
                    Effect::ForEachPlayer {
                        who,
                        effect: Box::new(Effect::AsPlayer {
                            who: PlayerRef::Iterated,
                            effect: Box::new(guard(Effect::May {
                                who: PlayerRef::Iterated,
                                effect: Box::new(Effect::Custom(OPT_IN.into())),
                            })),
                        }),
                    },
                    Effect::Store {
                        var: ACCEPTED,
                        sel: Sel::Var(OPTED),
                    },
                    // ...then those who accepted do it.
                    Effect::ForEachPlayer {
                        who: PlayerRef::Var(ACCEPTED),
                        effect: Box::new(Effect::AsPlayer {
                            who: PlayerRef::Iterated,
                            effect: Box::new(e),
                        }),
                    },
                ]).collect()),
                hoisted,
            ))
        }
    }
}

inventory::submit! { EffectPattern { name: "[player] [instruction]", priority: 900, parse: player_subject } }

/// "each player who does [instruction]" after "each [player] may [instruction]": each
/// player who accepted.
fn each_player_who_does(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.named.iter().any(|(p, _)| p == ACCEPTED_NAME) {
        return None;
    }
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let r = l.strip_prefix("each player who does ")?;
    let e = instruction(&as_you(r)?, Some(r), b)?;
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Var(ACCEPTED),
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e),
        }),
    })
}

/// Marks that the players who accepted "each [player] may search their library ..."
/// searched (a player who chooses to search searches, even if they find nothing).
const SEARCHED_NAME: &str = "\u{1}the players who searched";

fn is_search(e: &Effect) -> bool {
    match e {
        Effect::Search { .. } => true,
        Effect::Seq(v) => v.first().is_some_and(is_search),
        _ => false,
    }
}

/// "Then each player who searched their library this way shuffles." after "each player
/// may search their library for ...".
fn each_player_who_searched_shuffles(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.named.iter().any(|(p, _)| p == SEARCHED_NAME) {
        return None;
    }
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "each player who searched their library this way shuffles" {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Var(ACCEPTED),
        effect: Box::new(Effect::Shuffle {
            who: PlayerRef::Iterated,
        }),
    })
}

inventory::submit! { EffectPattern { name: "each player who searched their library this way shuffles", priority: 150, parse: each_player_who_searched_shuffles } }

inventory::submit! { EffectPattern { name: "each player who does [instruction]", priority: 150, parse: each_player_who_does } }

/// Whether `e` (the previous instruction) records whether its player did it, as the last
/// thing it does: an optional instruction or cost (CR 118.12), or one that can fail to do
/// what it says (sacrificing, discarding, exiling, moving, searching, casting).
fn records_whether_done(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(records_whether_done),
        Effect::AsPlayer { effect, .. } => records_whether_done(effect),
        Effect::May { .. }
        | Effect::PayOptional { .. }
        | Effect::Exile { .. }
        | Effect::Sacrifice { .. }
        | Effect::SacrificeObjects { .. }
        | Effect::Move { .. }
        | Effect::Discard { .. }
        | Effect::Search { .. }
        | Effect::CastCard { .. } => true,
        _ => false,
    }
}

/// "If they do, [instruction]." / "If that player doesn't, [instruction]." after a
/// player's optional instruction (or one that can fail): whether that player did it.
fn if_they_do(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    use super::oracle_hardening_referents::is_no_player_referent;
    let l = end(l);
    let Some((r, did)) = [
        ("if they do, ", true),
        ("if that player does, ", true),
        ("if the player does, ", true),
        ("if they don't, ", false),
        ("if that player doesn't, ", false),
        ("if the player doesn't, ", false),
    ]
    .iter()
    .find_map(|(p, d)| l.strip_prefix(p).map(|r| (r, *d))) else {
        return false;
    };
    if matches!(b.it_player, PlayerRef::You)
        || is_no_player_referent(&b.it_player)
        || !records_whether_done(prev)
    {
        return false;
    }
    let Some(e) = parse_sentence(r, b) else {
        return false;
    };
    let cond = if did {
        Condition::PrevHappened
    } else {
        Condition::Not(Box::new(Condition::PrevHappened))
    };
    let p = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![
        p,
        Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if they do, [instruction]", priority: 900, apply: if_they_do } }

/// The last instruction of `e` if another player performs it (`Effect::AsPlayer`).
fn last_as_player(e: &mut Effect) -> Option<(&PlayerRef, &mut Effect)> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_as_player),
        Effect::AsPlayer { who, effect } => Some((&*who, &mut **effect)),
        _ => None,
    }
}

/// A later sentence about the player who performed the previous instruction ("Target
/// player reveals the top card of their library. If it's a land card, that player puts
/// it into their hand."; "... Otherwise, the player casts it without paying its mana cost
/// if able."): reworded for "you" and read as part of what that player does, so the
/// patterns that join such sentences for "you" apply.
fn same_player_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    use super::oracle_hardening_referents::is_no_player_referent;
    let l = end(l);
    let (prefix, rest) = if let Some(r) = l.strip_prefix("otherwise, ") {
        ("otherwise, ".to_string(), r)
    } else if let Some(r) = l.strip_prefix("if ") {
        let Some((c, r)) = r.split_once(", ") else {
            return false;
        };
        if mentions_you(c) {
            return false;
        }
        (format!("if {c}, "), r)
    } else {
        (String::new(), l)
    };
    let rest = rest.strip_prefix("then ").unwrap_or(rest);
    let (pred, plural) = if let Some(r) = rest.strip_prefix("that player ") {
        (r, false)
    } else if let Some(r) = rest.strip_prefix("the player ") {
        (r, false)
    } else if let Some(r) = rest.strip_prefix("that opponent ") {
        (r, false)
    } else if let Some(r) = rest.strip_prefix("they ") {
        (r, true)
    } else {
        return false;
    };
    if matches!(b.it_player, PlayerRef::You) || is_no_player_referent(&b.it_player) {
        return false;
    }
    let it_player = format!("{:?}", b.it_player);
    let Some((who, inner)) = last_as_player(prev) else {
        return false;
    };
    if format!("{who:?}") != it_player {
        return false;
    }
    let reworded = if plural {
        plural_to_singular(pred).and_then(|p| as_you(&p))
    } else {
        as_you(pred)
    };
    let Some(text) = reworded else {
        return false;
    };
    // The condition is reworded too ("if they do" → "if you do").
    let prefix = prefix
        .replace("if they do,", "if you do,")
        .replace("if they don't,", "if you don't,");
    let sentence = format!("{prefix}{text}");
    let saved_player = std::mem::replace(&mut b.it_player, PlayerRef::You);
    let saved_inner = inner.clone();
    let done = if crate::oracle_ext::apply_followup_ext(&sentence, inner, b) {
        true
    } else {
        *inner = saved_inner.clone();
        match parse_sentence(&sentence, b) {
            Some(e) if !format!("{e:?}").contains("TriggerPlayer") => {
                let old = std::mem::replace(inner, Effect::Noop);
                *inner = Effect::seq(vec![old, e]);
                true
            }
            _ => false,
        }
    };
    b.it_player = saved_player;
    if !done {
        *inner = saved_inner;
    }
    done
}

inventory::submit! { FollowupPattern { name: "a later sentence about the player performing the previous one", priority: 950, apply: same_player_followup } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewording_for_you() {
        assert_eq!(
            as_you("exiles the top four cards of their library").as_deref(),
            Some("exile the top four cards of your library")
        );
        assert_eq!(
            as_you("chooses five lands they control and sacrifices the rest").as_deref(),
            Some("choose five lands you control and sacrifice the rest")
        );
        assert_eq!(
            as_you("reveals the top card of their library, loses life equal to that card's mana value, then puts it into their hand").as_deref(),
            Some("reveal the top card of your library, lose life equal to that card's mana value, then put it into your hand")
        );
        assert_eq!(
            as_you("searches their library for a card").as_deref(),
            Some("search your library for a card")
        );
        // "you" in the predicate is the controller, not the subject.
        assert_eq!(as_you("mills cards equal to the number of cards in your hand"), None);
        // Not a verb.
        assert_eq!(as_you("cards equal to"), None);
        assert_eq!(
            plural_to_singular("discard their hands, then draw seven cards").as_deref(),
            Some("discards their hands, then draw seven cards")
        );
    }
}

#[cfg(test)]
mod probe {
    use crate::card::Layout;
    use crate::oracle::{compile, CompileContext};
    use crate::types::TypeLine;

    #[test]
    fn probe() {
        let Ok(texts) = std::env::var("PS_PROBE") else { return };
        let tl = TypeLine::parse(&std::env::var("PS_TYPE").unwrap_or("Sorcery".into()));
        let ctx = CompileContext {
            card_name: "Test Card",
            full_name: "Test Card",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        for t in texts.split('|') {
            let r = compile(t, &ctx);
            println!("== {t}\n   unsupported: {:?}\n   {:?}", r.unsupported, r.abilities.iter().map(|a| format!("{:?}", a.kind)).collect::<Vec<_>>());
        }
    }
}
