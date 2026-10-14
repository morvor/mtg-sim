//! Game-history conditions (CR 603.4, 608.2c): what happened earlier this turn, and how
//! a spell was cast.
//!
//! A history clause is read compositionally as SUBJECT + VERB (past or perfect tense,
//! possibly negated) + OBJECT PHRASE + WINDOW, with a count on the subject or the object:
//! "if you cycled two or more cards this turn", "if a creature card left your graveyard
//! this turn", "if an opponent discarded a card this turn", "if no permanents left the
//! battlefield this turn", "if you didn't attack with a creature this turn". The clause is
//! put in the present tense, singular ("you cycle a card", "a creature card leaves your
//! graveyard") and read by the trigger grammar; the condition counts this turn's events
//! that a triggered ability with that trigger condition would have triggered on
//! ([`Value::EventsThisTurn`]), whether or not anything triggered (CR 603.1b, 603.2).

use crate::ability::*;
use crate::oracle::phrases::*;

/// Whether `rest` ends a word.
fn word_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.', ';', '"'])
}

/// A count on a noun phrase: "a", "one or more", "two or more", "more than one",
/// "exactly one", "no". Returns (comparison, number, whether the noun is plural, rest).
fn quantity(s: &str) -> Option<(Cmp, i32, bool, &str)> {
    let s = s.trim_start();
    // "another creature": the word stays part of the noun phrase.
    if s.starts_with("another ") {
        return Some((Cmp::Ge, 1, false, s));
    }
    for (p, cmp, n, plural) in [
        ("a ", Cmp::Ge, 1, false),
        ("an ", Cmp::Ge, 1, false),
        ("one or more ", Cmp::Ge, 1, true),
        ("more than one ", Cmp::Ge, 2, false),
        ("no ", Cmp::Eq, 0, true),
        ("any ", Cmp::Ge, 1, true),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            return Some((cmp, n, plural, r));
        }
    }
    let (s, exactly) = match s.strip_prefix("exactly ") {
        Some(r) => (r, true),
        None => (s.strip_prefix("at least ").unwrap_or(s), false),
    };
    let (v, r) = parse_number(s)?;
    let Value::Const(n) = v else { return None };
    if exactly {
        return Some((Cmp::Eq, n, n != 1, r));
    }
    let r = r.strip_prefix("or more ")?;
    Some((Cmp::Ge, n, true, r))
}

fn filter_key(f: &Filter) -> String {
    serde_json::to_string(f).unwrap_or_default()
}

/// The singular of a plural noun phrase that means the same objects ("cards" → "card",
/// "creature cards with flying" → "creature card with flying", "Merfolk" → "Merfolk"),
/// checked with the object phrase grammar. Returns the singular phrase and the rest.
fn singular(np: &str) -> Option<String> {
    let (f, plural, tail) = parse_object_phrase(np)?;
    if !tail.trim().is_empty() {
        return None;
    }
    if !plural {
        return Some(np.to_string());
    }
    let key = filter_key(&f);
    let words: Vec<&str> = np.split(' ').collect();
    for i in 0..words.len() {
        let w = words[i];
        let mut cands = vec![w.to_string()];
        if let Some(x) = w.strip_suffix("ies") {
            cands.push(format!("{x}y"));
        }
        if let Some(x) = w.strip_suffix("es") {
            cands.push(x.to_string());
        }
        if let Some(x) = w.strip_suffix('s') {
            cands.push(x.to_string());
        }
        if let Some(x) = w.strip_suffix("ves") {
            cands.push(format!("{x}f"));
        }
        for c in cands {
            let mut ws: Vec<String> = words.iter().map(|x| x.to_string()).collect();
            ws[i] = c;
            let s2 = ws.join(" ");
            if let Some((f2, false, t2)) = parse_object_phrase(&s2) {
                if t2.trim().is_empty() && filter_key(&f2) == key {
                    return Some(s2);
                }
            }
        }
    }
    None
}

/// Past-tense verbs and their present forms: (past, base form, third person singular).
const VERBS: &[(&str, &str, &str)] = &[
    ("cast", "cast", "casts"),
    ("drew", "draw", "draws"),
    ("drawn", "draw", "draws"),
    ("discarded", "discard", "discards"),
    ("cycled", "cycle", "cycles"),
    ("sacrificed", "sacrifice", "sacrifices"),
    ("created", "create", "creates"),
    ("gained", "gain", "gains"),
    ("lost", "lose", "loses"),
    ("searched", "search", "searches"),
    ("played", "play", "plays"),
    ("activated", "activate", "activates"),
    ("attacked", "attack", "attacks"),
    ("blocked", "block", "blocks"),
    ("died", "die", "dies"),
    ("entered", "enter", "enters"),
    ("left", "leave", "leaves"),
    ("dealt", "deal", "deals"),
    ("committed", "commit", "commits"),
    ("milled", "mill", "mills"),
    ("scried", "scry", "scries"),
    ("surveilled", "surveil", "surveils"),
    ("explored", "explore", "explores"),
    ("investigated", "investigate", "investigates"),
    ("put", "put", "puts"),
    ("tapped", "tap", "taps"),
];

/// Passive forms: "was put into", "were put into", "was dealt", "was blocked".
const PASSIVE: &[(&str, &str)] = &[
    ("was put", "is put"),
    ("were put", "is put"),
    ("was dealt", "is dealt"),
    ("were dealt", "is dealt"),
    ("was exiled", "is exiled"),
    ("were exiled", "is exiled"),
    ("was sacrificed", "is sacrificed"),
    ("were sacrificed", "is sacrificed"),
    ("was destroyed", "is destroyed"),
    ("were destroyed", "is destroyed"),
    ("was blocked", "becomes blocked"),
    ("was returned", "is returned"),
    ("were returned", "is returned"),
    ("was turned face up", "is turned face up"),
    ("became tapped", "becomes tapped"),
];

/// One reading of a history clause: the trigger text it corresponds to, the count it
/// needs, and whether it's negated.
struct Clause {
    trigger: String,
    cmp: Cmp,
    n: i32,
    negated: bool,
    /// The count is an amount (life, damage), not a number of events.
    amount: bool,
    /// "an opponent" / "a player" as the subject of an amount: some one player's amount.
    per_player: Option<PlayerFilter>,
    /// Counts the different objects the events were about ("attacked with three or more
    /// Merfolk", "dealt damage by two or more creatures"), not the events.
    distinct: bool,
}

impl Clause {
    fn new(trigger: String, cmp: Cmp, n: i32, negated: bool) -> Clause {
        Clause {
            trigger,
            cmp,
            n,
            negated,
            amount: false,
            per_player: None,
            distinct: false,
        }
    }
}

/// Players as subjects of a past action: (words, second person).
const PLAYERS: &[(&str, bool)] = &[
    ("you", true),
    ("an opponent", false),
    ("a player", false),
    ("your team", true),
    ("one or more opponents", false),
    ("one or more players", false),
    ("no opponent", false),
    ("no player", false),
];

/// Splits a player subject and auxiliary: (subject as written for the present tense,
/// second person, negated, rest). "no opponent" is "an opponent", negated.
fn player_subject(c: &str) -> Option<(&'static str, bool, bool, &str)> {
    let (subj, second, r) = PLAYERS.iter().find_map(|(p, second)| {
        let r = c.strip_prefix(p)?;
        (r.starts_with(' ') || r.starts_with("'ve ")).then_some((*p, *second, r))
    })?;
    let (subj, mut negated) = match subj {
        "no opponent" => ("an opponent", true),
        "no player" => ("a player", true),
        "one or more opponents" => ("an opponent", false),
        "one or more players" => ("a player", false),
        "your team" => ("you", false),
        s => (s, false),
    };
    let r = if let Some(x) = r.strip_prefix("'ve ") {
        x
    } else {
        let r = r.trim_start();
        let mut r2 = r;
        for (aux, neg) in [
            ("have ", false),
            ("has ", false),
            ("didn't ", true),
            ("did not ", true),
            ("haven't ", true),
            ("hasn't ", true),
            ("have not ", true),
        ] {
            if let Some(x) = r.strip_prefix(aux) {
                negated ^= neg;
                r2 = x;
                break;
            }
        }
        r2
    };
    Some((subj, second, negated, r))
}

/// "[player] [verb] [rest]" in the present tense, conjugating each verb of "[verb] or
/// [verb]" ("scried or surveilled"). `base`: the verbs are in the base form ("didn't
/// attack").
fn conjugate(r: &str, second: bool, base: bool) -> Option<(String, &str)> {
    let (w, rest) = split_word(r);
    let one = |w: &str| -> Option<&'static str> {
        let (b, t) = if base {
            VERBS.iter().find(|(_, b, _)| *b == w).map(|(_, b, t)| (*b, *t))?
        } else {
            VERBS.iter().find(|(p, _, _)| *p == w).map(|(_, b, t)| (*b, *t))?
        };
        Some(if second { b } else { t })
    };
    let v = one(w)?;
    if let Some(x) = rest.strip_prefix("or ") {
        let (w2, rest2) = split_word(x);
        if let Some(v2) = one(w2) {
            return Some((format!("{v} or {v2}"), rest2));
        }
    }
    Some((v.to_string(), rest))
}

/// "a"/"an" for a noun phrase.
fn article(np: &str) -> &'static str {
    if np.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    }
}

/// A counted noun phrase in the singular with its article: "two or more cards" → ("a
/// card", ≥, 2).
fn counted_object(s: &str) -> Option<(String, Cmp, i32)> {
    let (cmp, n, plural, np) = quantity(s)?;
    let (np, tail) = split_qualifier(np);
    let np = if plural { singular(np)? } else { np.to_string() };
    let obj = if np.starts_with("another ") {
        format!("{np}{tail}")
    } else {
        format!("{} {np}{tail}", article(&np))
    };
    Some((obj, cmp, n))
}

/// "[player] [verb past] [object phrase]" → present tense.
fn player_clause(c: &str) -> Option<Clause> {
    let (subj, second, negated, r) = player_subject(c)?;
    // "[player] had [objects] enter the battlefield ...", "... had three or more cards put
    // into their graveyard ...": the objects did.
    if let Some(x) = r.strip_prefix("had ") {
        let possessive = match subj {
            "you" => "your",
            "an opponent" => "an opponent's",
            "a player" => "a player's",
            _ => return None,
        };
        let x = x
            .replace("under their control", &format!("under {possessive} control"))
            .replace("into their graveyard", &format!("into {possessive} graveyard"));
        let x = x
            .replacen(" enter ", " entered ", 1)
            .replacen(" put into ", " were put into ", 1);
        let mut cl = object_clause(&x)?;
        cl.negated ^= negated;
        return Some(cl);
    }
    // "[player] was dealt [combat] damage [by objects]": the objects dealt it.
    for p in ["was dealt ", "were dealt ", "been dealt "] {
        if let Some(x) = r.strip_prefix(p) {
            return dealt_damage(subj, negated, x);
        }
    }
    let (verb, rest) = conjugate(r, second, negated)?;
    // "attacked with [objects]": each of them attacked.
    if verb.starts_with("attack") && !verb.contains(" or ") {
        if let Some(x) = rest.strip_prefix("with ") {
            if subj != "you" {
                return None;
            }
            let (q, rest2) = match x.find(" this combat") {
                Some(i) => (&x[..i], &x[i..]),
                None => (x, ""),
            };
            if !rest2.is_empty() {
                return None;
            }
            let mut cl = object_clause(&format!("{q} you controlled attacked"))?;
            cl.negated ^= negated;
            cl.distinct = true;
            return Some(cl);
        }
    }
    // Amounts: "lost 3 or more life", "gained life".
    if verb.starts_with("los") || verb.starts_with("gain") {
        let v = if verb.starts_with("los") { "lose" } else { "gain" };
        let v = if second { v.to_string() } else { format!("{v}s") };
        let (cmp, n, amount) = if rest == "life" {
            (Cmp::Ge, 1, false)
        } else {
            let (num, r2) = parse_number(rest)?;
            let Value::Const(n) = num else { return None };
            if r2 != "or more life" {
                return None;
            }
            (Cmp::Ge, n, true)
        };
        return Some(per_player(
            Clause {
                trigger: format!("{subj} {v} life"),
                cmp,
                n,
                negated,
                amount,
                per_player: None,
                distinct: false,
            },
            subj,
        ));
    }
    // "both a creature spell and a noncreature spell" is two clauses (see `joined`).
    if rest.starts_with("both ") {
        return None;
    }
    // A counted object: "two or more cards", "a creature spell".
    let (obj, cmp, n) = counted_object(rest).unwrap_or((rest.to_string(), Cmp::Ge, 1));
    let trigger = if obj.is_empty() {
        format!("{subj} {verb}")
    } else {
        format!("{subj} {verb} {obj}")
    };
    Some(Clause::new(trigger, cmp, n, negated))
}

/// An amount some one player reached ("if an opponent lost 3 or more life"): the
/// greatest among them.
fn per_player(mut cl: Clause, subj: &str) -> Clause {
    if cl.amount {
        let pf = match subj {
            "an opponent" => Some(PlayerFilter::Opponent),
            "a player" => Some(PlayerFilter::Any),
            _ => None,
        };
        if let Some(pf) = pf {
            cl.trigger = cl.trigger.replacen(subj, "you", 1);
            if let Some(x) = cl.trigger.strip_prefix("you loses") {
                cl.trigger = format!("you lose{x}");
            }
            if let Some(x) = cl.trigger.strip_prefix("you gains") {
                cl.trigger = format!("you gain{x}");
            }
            cl.per_player = Some(pf);
        }
    }
    cl
}

/// "[player] was dealt [N or more] [combat|noncombat] damage [by objects]".
fn dealt_damage(subj: &str, negated: bool, x: &str) -> Option<Clause> {
    let (amount, cmp, n, x) = match parse_number(x) {
        Some((Value::Const(n), r)) if r.starts_with("or more ") => {
            (true, Cmp::Ge, n, &r["or more ".len()..])
        }
        _ => (false, Cmp::Ge, 1, x),
    };
    let (kind, x) = if let Some(r) = x.strip_prefix("combat damage") {
        ("combat damage", r)
    } else if let Some(r) = x.strip_prefix("noncombat damage") {
        ("noncombat damage", r)
    } else {
        ("damage", x.strip_prefix("damage")?)
    };
    let to = if subj == "you" { "you" } else { subj };
    if amount && !matches!(subj, "you" | "an opponent" | "a player") {
        return None;
    }
    if let Some(by) = x.strip_prefix(" by ") {
        if amount {
            return None;
        }
        // "by two or more creatures" counts the creatures, not the events.
        let (src, c2, n2) = counted_object(by)?;
        if c2 != Cmp::Ge || (n2 > 1 && subj != "you") {
            return None;
        }
        let t = format!("{src} deals {kind} to {to}");
        return Some(Clause {
            trigger: t,
            cmp: c2,
            n: n2,
            negated,
            amount,
            per_player: None,
            distinct: n2 > 1,
        });
    }
    if !x.is_empty() {
        return None;
    }
    let t = format!("a source deals {kind} to {to}");
    Some(per_player(
        Clause {
            trigger: t,
            cmp,
            n,
            negated,
            amount,
            per_player: None,
            distinct: false,
        },
        subj,
    ))
}

/// Splits a noun phrase from a following qualifier ("cards from anywhere" → "cards",
/// " from anywhere").
fn split_qualifier(np: &str) -> (&str, &str) {
    for q in [" from ", " onto ", " into ", " to ", " under "] {
        if let Some(i) = np.find(q) {
            return (&np[..i], &np[i..]);
        }
    }
    (np, "")
}

/// "[objects] [verb past] [rest]": "a creature card left your graveyard", "three or more
/// cards were put into your graveyard from anywhere", "~ attacked or blocked", "no
/// permanents left the battlefield", "~ didn't attack".
fn object_clause(c: &str) -> Option<Clause> {
    let (cmp, n, plural, np_rest) = if c.starts_with("~ ") {
        (Cmp::Ge, 1, false, c)
    } else {
        quantity(c)?
    };
    let words: Vec<&str> = np_rest.split(' ').collect();
    for i in 1..words.len() {
        let np = words[..i]
            .join(" ")
            .replace(" you controlled", " you control")
            .replace(" an opponent controlled", " an opponent controls")
            .replace(" your opponents controlled", " your opponents control");
        let rest = words[i..].join(" ");
        let mut negated = false;
        let mut present = None;
        for (p, pr) in PASSIVE {
            if let Some(x) = rest.strip_prefix(p) {
                if word_end(x) {
                    present = Some(format!("{pr}{x}"));
                    break;
                }
            }
        }
        if present.is_none() {
            if let Some(x) = rest.strip_prefix("didn't ") {
                if let Some((v, x2)) = conjugate(x, false, true) {
                    negated = true;
                    present = Some(format!("{v} {x2}"));
                }
            } else if let Some((v, x2)) = conjugate(&rest, false, false) {
                present = Some(format!("{v} {x2}"));
            }
        }
        let Some(present) = present else { continue };
        let present = present.trim_end().to_string();
        let np_s = if np == "~" {
            np.clone()
        } else if plural {
            match singular(&np) {
                Some(x) => x,
                None => continue,
            }
        } else {
            // A singular phrase must read whole.
            match parse_object_phrase(&np) {
                Some((_, false, tail)) if tail.trim().is_empty() => np.clone(),
                _ => continue,
            }
        };
        // "enters the battlefield under an opponent's control": "[objects] an opponent
        // controls enters" (the trigger grammar's wording).
        let (np_s, present) = control_qualifier(np_s, present);
        let subj = if np_s == "~" || np_s.starts_with("another ") {
            np_s
        } else {
            format!("{} {np_s}", article(&np_s))
        };
        return Some(Clause::new(format!("{subj} {present}"), cmp, n, negated));
    }
    None
}

/// Moves "under [player's] control" after "enters the battlefield" / "leaves the
/// battlefield" onto the noun phrase ("a creature an opponent controls enters").
fn control_qualifier(np: String, present: String) -> (String, String) {
    for (verb, short) in [
        ("enters the battlefield", "enters"),
        ("leaves the battlefield", "leaves the battlefield"),
    ] {
        for (q, ctl) in [
            (" under an opponent's control", "an opponent controls"),
            (" under your control", "you control"),
            (" under a player's control", ""),
        ] {
            if present == format!("{verb}{q}") && np != "~" {
                let np2 = if ctl.is_empty() {
                    np.clone()
                } else {
                    format!("{np} {ctl}")
                };
                return (np2, short.to_string());
            }
        }
    }
    (np, present)
}

/// The trigger condition for a present-tense clause, if the trigger grammar reads it as
/// one about events: batches are counted event by event.
fn trigger_of(text: &str) -> Option<TriggerCond> {
    let parse = |t: &str| crate::oracle::triggers::parse_trigger_condition(&format!("whenever {t}"));
    // A card put into a hand from the battlefield is returned there.
    let (cond, _, _) = parse(text).or_else(|| {
        let i = text.find(" is put into ")?;
        let r = &text[i + " is put into ".len()..];
        if !r.contains("hand from ") {
            return None;
        }
        parse(&format!("{} is returned to {r}", &text[..i]))
    })?;
    if matches!(cond, TriggerCond::Custom(_)) {
        return None;
    }
    let cond = match cond {
        TriggerCond::Batched { trigger, .. } => *trigger,
        c => c,
    };
    let json = serde_json::to_string(&cond).unwrap_or_default();
    // Not a delayed, state or once-per-turn trigger, nor one about a group at once.
    if [
        "\"Batched\"",
        "\"State\"",
        "\"FirstTimeEachTurn\"",
        "\"ThisTurn\"",
        "\"UntilYourNextTurn\"",
        "\"BeginningOf\"",
        "\"PlayerAttacksWith\"",
        "\"PlayerAttacks\"",
        "\"NthSpellCast\"",
    ]
    .iter()
    .any(|w| json.contains(w))
    {
        return None;
    }
    Some(cond)
}

/// A clause's condition.
fn clause_condition(cl: Clause) -> Option<Condition> {
    let mut cond = trigger_of(&cl.trigger)?;
    if cl.per_player.is_some() {
        cond = iterated_player(cond)?;
    }
    let tally = if cl.amount {
        Tally::Amount
    } else {
        Tally::Events
    };
    let mut v = if cl.distinct {
        Value::CountSel(Box::new(Sel::ThisTurn(Box::new(cond))))
    } else {
        Value::EventsThisTurn(Box::new(cond), tally)
    };
    if let Some(pf) = cl.per_player {
        v = Value::OverPlayers(AggOp::Max, pf, Box::new(v));
    }
    let cmp = Condition::Compare(v, cl.cmp, Value::Const(cl.n));
    Some(if cl.negated {
        Condition::Not(Box::new(cmp))
    } else {
        cmp
    })
}

/// The trigger condition with "you" as the player each player in turn ([`Value::OverPlayers`]).
fn iterated_player(c: TriggerCond) -> Option<TriggerCond> {
    Some(match c {
        TriggerCond::LosesLife { who: PlayerRel::You } => TriggerCond::LosesLife {
            who: PlayerRel::Iterated,
        },
        TriggerCond::GainsLife { who: PlayerRel::You } => TriggerCond::GainsLife {
            who: PlayerRel::Iterated,
        },
        TriggerCond::DealsDamage {
            source,
            to: DamageRecipient::Player(PlayerRel::You),
            combat_only,
        } => TriggerCond::DealsDamage {
            source,
            to: DamageRecipient::Player(PlayerRel::Iterated),
            combat_only,
        },
        TriggerCond::Noncombat(t) => TriggerCond::Noncombat(Box::new(iterated_player(*t)?)),
        _ => return None,
    })
}

/// A history clause (without its window): one clause, or two joined by "and"/"or" ("you
/// gained and lost life", "you've played a land or cast a spell", "you attacked with a
/// Hero this turn or a Hero entered the battlefield under your control"), or "both [A]
/// and [B]".
fn clause_or_joined(c: &str) -> Option<Condition> {
    let c = c.strip_suffix(" this turn").unwrap_or(c);
    if let Some(cl) = player_clause(c).or_else(|| object_clause(c)) {
        if let Some(cond) = clause_condition(cl) {
            return Some(cond);
        }
    }
    // "[player] [verb] both [A] and [B]".
    if let Some((subj, second, negated, r)) = player_subject(c) {
        if !negated {
            let (verb, rest) = conjugate(r, second, false)?;
            let _ = verb;
            if let Some(x) = rest.strip_prefix("both ") {
                let (a, b) = x.split_once(" and ")?;
                let past = split_word(r).0;
                let ca = clause_or_joined(&format!("{subj} {past} {a}"))?;
                let cb = clause_or_joined(&format!("{subj} {past} {b}"))?;
                return Some(Condition::And(vec![ca, cb]));
            }
        }
    }
    for (sep, all) in [(" or ", false), (" and ", true)] {
        for (i, _) in c.match_indices(sep) {
            let (x, y) = (&c[..i], &c[i + sep.len()..]);
            // The same object, elided: "you gained and lost life" (the object after the
            // second verb).
            let Some(cx) = clause_or_joined(x).or_else(|| {
                let (_, _, _, r) = player_subject(x)?;
                if r.contains(' ') {
                    return None;
                }
                let obj = split_word(y).1;
                if obj.is_empty() {
                    return None;
                }
                clause_or_joined(&format!("{x} {obj}"))
            }) else {
                continue;
            };
            // The same subject, elided: "you gained and lost life".
            let cy = clause_or_joined(y).or_else(|| {
                let (subj, _, negated, _) = player_subject(x)?;
                if negated {
                    return None;
                }
                clause_or_joined(&format!("{subj} {y}"))
            });
            if let Some(cy) = cy {
                return Some(if all {
                    Condition::And(vec![cx, cy])
                } else {
                    Condition::Or(vec![cx, cy])
                });
            }
        }
    }
    None
}

/// "if [history clause] this turn" without referents. "this turn" may come before a
/// "from anywhere ..." qualifier.
pub fn history_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    let body = if let Some(b) = c.strip_suffix(" this turn") {
        b.to_string()
    } else {
        let i = c.find(" this turn from ")?;
        format!("{}{}", &c[..i], &c[i + " this turn".len()..])
    };
    clause_or_joined(&body)
}

inventory::submit! { super::ConditionPattern { name: "history: [subject] [past verb] [objects] this turn", priority: 50, parse: history_condition } }

/// A history clause about a referent: "it attacked or blocked this turn", "that creature
/// entered this turn", "equipped creature didn't deal combat damage to a creature this
/// turn". The clause is read about "~" and then made about the referent: an object the
/// events were about is that object (the same object while it stays where it is, CR
/// 400.7).
pub fn referent_history(c: &str, b: &mut crate::oracle::effects::Builder) -> Option<Condition> {
    let c = end(c);
    if c.contains('~') || !c.ends_with(" this turn") {
        return None;
    }
    let before = b.targets.len();
    let saved = b.it.clone();
    let r = crate::oracle::effects::object_ref(c, b);
    b.it = saved;
    let (sel, rest) = r?;
    if b.targets.len() != before
        || super::oracle_hardening_referents::is_no_referent(&sel)
        || !rest.starts_with(' ')
    {
        b.targets.truncate(before);
        return None;
    }
    let cond = history_condition(&format!("~{rest}"))?;
    if matches!(sel, Sel::This) {
        return Some(cond);
    }
    about(cond, &sel)
}

/// The condition with each "~" ([`Filter::Source`]) in its trigger conditions meaning the
/// selected object instead.
fn about(cond: Condition, sel: &Sel) -> Option<Condition> {
    let json = serde_json::to_value(&cond).ok()?;
    let with = serde_json::to_value(Filter::In(Box::new(sel.clone()))).ok()?;
    fn walk(v: serde_json::Value, with: &serde_json::Value, n: &mut usize) -> serde_json::Value {
        use serde_json::Value as J;
        match v {
            J::String(s) if s == "Source" => {
                *n += 1;
                with.clone()
            }
            J::Array(a) => J::Array(a.into_iter().map(|x| walk(x, with, n)).collect()),
            J::Object(m) => J::Object(m.into_iter().map(|(k, x)| (k, walk(x, with, n))).collect()),
            other => other,
        }
    }
    let mut n = 0;
    let out = walk(json, &with, &mut n);
    if n == 0 {
        return None;
    }
    serde_json::from_value(out).ok()
}

#[cfg(test)]
mod tests {
    use crate::card::Layout;
    use crate::oracle::effects::Builder;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;

    /// Developer probe: `HC_PROBE=<file>` with lines `C: <condition>` (a condition read
    /// in a creature's ability) or `U: <card name>` (the card's unsupported texts).
    #[test]
    fn probe() {
        let Ok(path) = std::env::var("HC_PROBE") else {
            return;
        };
        let text = std::fs::read_to_string(path).unwrap_or_default();
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
        for line in text.lines() {
            if let Some(c) = line.strip_prefix("C: ") {
                let c = c.to_lowercase();
                let mut b = Builder::new(&ctx);
                let s = crate::oracle::statics::parse_condition(&c, &ctx);
                let w =
                    super::super::conditions_referents::parse_condition_with(&c, &mut b);
                println!("C {c}\n  static={s:?}\n  with={w:?}");
            } else if let Some(c) = line.strip_prefix("W: ") {
                let c = c.to_lowercase();
                let t = crate::oracle::triggers::parse_trigger_condition(&c);
                println!("W {c}\n  {t:?}");
            } else if let Some(c) = line.strip_prefix("H: ") {
                let c = c.to_lowercase();
                let cl = super::player_clause(&c).or_else(|| super::object_clause(&c));
                let t = cl.as_ref().map(|x| x.trigger.clone());
                let tc = t.as_deref().and_then(super::trigger_of);
                let hc = super::history_condition(&format!("{c} this turn"));
                println!("H {c}\n  clause={t:?}\n  trigger={tc:?}\n  cond={hc:?}");
            } else if let Some(c) = line.strip_prefix("U: ") {
                let def = crate::card::card(c);
                for u in def.unsupported_text() {
                    println!("U {c}: {u}");
                }
            }
        }
    }
}
