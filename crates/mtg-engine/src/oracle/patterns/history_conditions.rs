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
    let (s, exactly, at_least) = if let Some(r) = s.strip_prefix("exactly ") {
        (r, true, false)
    } else if let Some(r) = s.strip_prefix("at least ") {
        (r, false, true)
    } else {
        (s, false, false)
    };
    let (v, r) = parse_number(s)?;
    let Value::Const(n) = v else { return None };
    if exactly {
        return Some((Cmp::Eq, n, n != 1, r));
    }
    if at_least {
        return Some((Cmp::Ge, n, n != 1, r));
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
    ("came", "come", "comes"),
    ("fought", "fight", "fights"),
    ("turned", "turn", "turns"),
    ("exerted", "exert", "exerts"),
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
    ("blocked or was blocked by", "blocks or becomes blocked by"),
    ("was removed", "is removed"),
    ("were removed", "is removed"),
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
    /// For a clause about objects, its predicate as written ("attacked this combat").
    pred: String,
    /// An amount some one of the objects reached ("a source you controlled dealt 5 or
    /// more damage this turn"): the greatest among them.
    per_object: bool,
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
            pred: String::new(),
            per_object: false,
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
            VERBS
                .iter()
                .find(|(_, b, _)| *b == w)
                .map(|(_, b, t)| (*b, *t))?
        } else {
            VERBS
                .iter()
                .find(|(p, _, _)| *p == w)
                .map(|(_, b, t)| (*b, *t))?
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
    let np = if plural {
        singular(np)?
    } else {
        np.to_string()
    };
    let obj = if np.starts_with("another ") {
        format!("{np}{tail}")
    } else {
        format!("{} {np}{tail}", article(&np))
    };
    Some((obj, cmp, n))
}

/// "[player] [verb past] [object phrase]" → present tense. "an opponent" / "a player"
/// with a count of two or more is some one of them ("if an opponent cast two or more
/// spells this turn": not two opponents casting one each).
fn player_clause(c: &str) -> Option<Clause> {
    let cl = player_clause_as(c)?;
    if cl.per_player.is_some() || cl.distinct || cl.cmp != Cmp::Ge || cl.n < 2 {
        return Some(cl);
    }
    let Some((words, pf)) = [
        ("an opponent", PlayerFilter::Opponent),
        ("a player", PlayerFilter::Any),
    ]
    .into_iter()
    .find(|(w, _)| c.starts_with(&format!("{w} "))) else {
        return Some(cl);
    };
    let rest = &c[words.len()..];
    // The rest must not mention "you" (it becomes each player in turn).
    if rest
        .split([' ', '\''])
        .any(|w| matches!(w, "you" | "your" | "you've"))
    {
        return None;
    }
    let mut each = player_clause_as(&format!("you{rest}"))?;
    each.per_player = Some(pf);
    each.negated = cl.negated;
    Some(each)
}

fn player_clause_as(c: &str) -> Option<Clause> {
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
            .replace(
                "under their control",
                &format!("under {possessive} control"),
            )
            .replace(
                "into their graveyard",
                &format!("into {possessive} graveyard"),
            );
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
        let v = if verb.starts_with("los") {
            "lose"
        } else {
            "gain"
        };
        let v = if second {
            v.to_string()
        } else {
            format!("{v}s")
        };
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
                pred: String::new(),
                per_object: false,
            },
            subj,
        ));
    }
    // "both a creature spell and a noncreature spell" is two clauses (see `joined`).
    if rest.starts_with("both ") {
        return None;
    }
    // "[did something] four or more times": the number of events (Yidaro, Wandering
    // Monster: "you've cycled a card named ~ four or more times this game").
    if let Some(head) = rest.strip_suffix(" or more times") {
        if let Some((obj, num)) = head.rsplit_once(' ') {
            if let Some((Value::Const(k), "")) = parse_number(num) {
                if obj.starts_with("a ") || obj.starts_with("an ") {
                    return Some(Clause::new(format!("{subj} {verb} {obj}"), Cmp::Ge, k, negated));
                }
            }
        }
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
        let (src, c2, n2) = if by == "~" {
            ("~".to_string(), Cmp::Ge, 1)
        } else {
            counted_object(by)?
        };
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
            pred: String::new(),
            per_object: false,
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
            pred: String::new(),
            per_object: false,
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
        // "has dealt", "have attacked": the perfect is the past here.
        let rest = match rest
            .strip_prefix("has ")
            .or_else(|| rest.strip_prefix("have "))
        {
            Some(r) if !r.starts_with("been ") => r.to_string(),
            _ => rest,
        };
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
        // "was dealt 4 or more damage": the total dealt to it; "dealt 5 or more damage
        // [to a player]": the total it dealt.
        let mut amount = None;
        let mut present = present;
        for verb in ["is dealt ", "deals "] {
            let Some((Value::Const(k), r)) = present.strip_prefix(verb).and_then(parse_number)
            else {
                continue;
            };
            let Some(kind) = r.strip_prefix("or more ") else {
                continue;
            };
            if kind.starts_with("damage") || kind.starts_with("combat damage") {
                present = format!("{verb}{kind}");
                amount = Some(k);
                break;
            }
        }
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
                // "a source you controlled": the trigger grammar reads damage sources.
                _ if matches!(np.as_str(), "source" | "source you control") => np.clone(),
                _ => continue,
            }
        };
        // "enters the battlefield under an opponent's control": "[objects] an opponent
        // controls enters" (the trigger grammar's wording).
        let (np_s, present) = control_qualifier(np_s, present);
        // "one other creature": "another creature".
        let np_s = match np_s.strip_prefix("other ") {
            Some(x) => format!("another {x}"),
            None => np_s,
        };
        let subj = if np_s == "~" || np_s.starts_with("~ ") || np_s.starts_with("another ") {
            np_s
        } else {
            format!("{} {np_s}", article(&np_s))
        };
        let mut cl = Clause::new(format!("{subj} {present}"), cmp, n, negated);
        cl.pred = rest;
        if let Some(k) = amount {
            if cmp != Cmp::Ge || n != 1 {
                return None;
            }
            cl.n = k;
            cl.amount = true;
            // "a source you controlled dealt 5 or more damage": some one source did.
            cl.per_object = subj != "~" && present.starts_with("deals ");
        }
        return Some(cl);
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
            // "~ entered the battlefield under your control": "~ you control enters" (read
            // about "a permanent you control", see `trigger_of`).
            if present == format!("{verb}{q}") && (np != "~" || !ctl.is_empty()) {
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
    // "[objects] [verb] or come(s) under your control": either.
    if let Some(head) = text.strip_suffix(" or comes under your control") {
        let (subj, _) = head.rsplit_once(' ')?;
        let a = trigger_of(head)?;
        let b = trigger_of(&format!("{subj} comes under your control"))?;
        return Some(TriggerCond::AnyOf(vec![a, b]));
    }
    // "[objects] come(s) under your control": they enter the battlefield under your
    // control, or you gain control of them (Erg Raiders's ruling).
    if let Some(subj) = text.strip_suffix(" comes under your control") {
        let TriggerCond::EntersBattlefield(f) = trigger_of(&format!("{subj} enters"))? else {
            return None;
        };
        let f = Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]);
        return Some(TriggerCond::AnyOf(vec![
            TriggerCond::EntersBattlefield(f.clone()),
            TriggerCond::ControlChanged(f),
        ]));
    }
    let parse =
        |t: &str| crate::oracle::triggers::parse_trigger_condition(&format!("whenever {t}"));
    // "~ is dealt excess damage": the trigger grammar reads it about "a permanent"; the
    // permanent is then ~.
    // Or about "a creature" for what only creatures do ("~ fights").
    let about_source = || {
        let r = text.strip_prefix("~ ")?;
        let (c, x, y) = parse(&format!("a permanent {r}"))?;
        let json = serde_json::to_string(&c).ok()?;
        if json.matches("\"Permanent\"").count() != 1 {
            return None;
        }
        let c = serde_json::from_str(&json.replace("\"Permanent\"", "\"Source\"")).ok()?;
        Some((c, x, y))
    };
    let about_source_creature = || {
        let r = text.strip_prefix("~ ")?;
        let (c, x, y) = parse(&format!("a creature {r}"))?;
        let json = serde_json::to_string(&c).ok()?;
        let creature = "{\"Type\":\"Creature\"}";
        if json.matches(creature).count() != 1 {
            return None;
        }
        let c = serde_json::from_str(&json.replace(creature, "\"Source\"")).ok()?;
        Some((c, x, y))
    };
    // A card put into a hand from the battlefield is returned there.
    let (cond, _, _) = parse(text).or_else(about_source).or_else(about_source_creature).or_else(|| {
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
fn clause_condition(cl: Clause, w: &Option<HistoryWindow>) -> Option<Condition> {
    let mut cond = trigger_of(&cl.trigger)?;
    if cl.per_player.is_some() {
        cond = iterated_player(cond)?;
    }
    let tally = if cl.amount {
        Tally::Amount
    } else {
        Tally::Events
    };
    // Some one source's total: the greatest among the sources (each in `vars::TESTED`).
    if cl.per_object {
        let TriggerCond::DealsDamage {
            to, combat_only, ..
        } = cond.clone()
        else {
            return None;
        };
        let each = TriggerCond::DealsDamage {
            source: Filter::In(Box::new(Sel::Var(vars::TESTED))),
            to,
            combat_only,
        };
        let (total, sources) = match w {
            None => (
                Value::EventsThisTurn(Box::new(each), Tally::Amount),
                Sel::ThisTurn(Box::new(cond)),
            ),
            Some(w) => (
                Value::EventsDuring(w.clone(), Box::new(each), Tally::Amount),
                Sel::EventsDuring(w.clone(), Box::new(cond)),
            ),
        };
        let v = Value::Extreme(Box::new(total), Box::new(sources), true);
        let cmp = Condition::Compare(v, cl.cmp, Value::Const(cl.n));
        return Some(if cl.negated { not(cmp) } else { cmp });
    }
    let mut v = match (cl.distinct, w) {
        (true, None) => Value::CountSel(Box::new(Sel::ThisTurn(Box::new(cond)))),
        (true, Some(w)) => Value::CountSel(Box::new(Sel::EventsDuring(w.clone(), Box::new(cond)))),
        (false, None) => Value::EventsThisTurn(Box::new(cond), tally),
        (false, Some(w)) => Value::EventsDuring(w.clone(), Box::new(cond), tally),
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
    let mut json = serde_json::to_value(&c).ok()?;
    fn walk(v: &mut serde_json::Value, n: &mut usize) {
        match v {
            serde_json::Value::String(s) if s == "You" => {
                *s = "Iterated".into();
                *n += 1;
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, n)),
            serde_json::Value::Object(m) => m.values_mut().for_each(|x| walk(x, n)),
            _ => {}
        }
    }
    let mut n = 0;
    walk(&mut json, &mut n);
    if n == 0 {
        return None;
    }
    serde_json::from_value(json).ok()
}

/// A history clause (without its window): one clause, or two joined by "and"/"or" ("you
/// gained and lost life", "you've played a land or cast a spell", "you attacked with a
/// Hero this turn or a Hero entered the battlefield under your control"), "both [A] and
/// [B]", or two subjects of one predicate ("~ and at least two other creatures
/// attacked").
fn clause_or_joined(c: &str, w: &Option<HistoryWindow>) -> Option<Condition> {
    let c = c.strip_suffix(window_suffix(w)).unwrap_or(c);
    if let Some(cl) = player_clause(c).or_else(|| object_clause(c)) {
        if let Some(cond) = clause_condition(cl, w) {
            return Some(cond);
        }
    }
    // "[player] [verb] both [A] and [B]".
    if let Some((subj, second, negated, r)) = player_subject(c) {
        if !negated {
            if let Some((_, rest)) = conjugate(r, second, false) {
                if let Some(x) = rest.strip_prefix("both ") {
                    let (a, b) = x.split_once(" and ")?;
                    let past = split_word(r).0;
                    let ca = clause_or_joined(&format!("{subj} {past} {a}"), w)?;
                    let cb = clause_or_joined(&format!("{subj} {past} {b}"), w)?;
                    return Some(Condition::And(vec![ca, cb]));
                }
            }
        }
    }
    for (sep, all) in [(" or ", false), (" and ", true)] {
        for (i, _) in c.match_indices(sep) {
            let (x, y) = (&c[..i], &c[i + sep.len()..]);
            // Two subjects, one predicate: "~ and at least two other creatures attacked",
            // "~ or another artifact entered the battlefield under your control".
            if all || !x.contains(' ') {
                if let Some(cy) = object_clause(y) {
                    if let Some(cx) = object_clause(&format!("{x} {}", cy.pred)) {
                        if let (Some(a), Some(b)) =
                            (clause_condition(cx, w), clause_condition(cy, w))
                        {
                            return Some(if all {
                                Condition::And(vec![a, b])
                            } else {
                                Condition::Or(vec![a, b])
                            });
                        }
                    }
                }
            }
            // The same object, elided: "you gained and lost life" (the object after the
            // second verb).
            let Some(cx) = clause_or_joined(x, w).or_else(|| {
                let (_, _, _, r) = player_subject(x)?;
                if r.contains(' ') {
                    return None;
                }
                let obj = split_word(y).1;
                if obj.is_empty() {
                    return None;
                }
                clause_or_joined(&format!("{x} {obj}"), w)
            }) else {
                continue;
            };
            // The same subject, elided: "you gained and lost life".
            let cy = clause_or_joined(y, w).or_else(|| {
                let (subj, _, negated, _) = player_subject(x)?;
                if negated {
                    return None;
                }
                clause_or_joined(&format!("{subj} {y}"), w)
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

/// How a window is written at the end of a clause.
fn window_suffix(w: &Option<HistoryWindow>) -> &'static str {
    match w {
        None => " this turn",
        Some(HistoryWindow::ThisCombat) => " this combat",
        Some(HistoryWindow::LastTurn) => " last turn",
        Some(HistoryWindow::PlayersLastTurn(_)) => " during your last turn",
        Some(HistoryWindow::SinceLastTurn(_)) => " since your last turn",
        Some(HistoryWindow::SinceLastUpkeep(_)) => " since your last upkeep",
        Some(HistoryWindow::ThisGame) => " this game",
    }
}

/// The window a clause ends with, and the clause without it. "this turn" may come before
/// a "from anywhere ..." qualifier.
fn split_window(c: &str) -> Option<(String, Option<HistoryWindow>)> {
    let you = || PlayerRef::You;
    // Longer suffixes first ("since your last turn" before "last turn").
    for w in [
        None,
        Some(HistoryWindow::ThisCombat),
        Some(HistoryWindow::PlayersLastTurn(you())),
        Some(HistoryWindow::SinceLastTurn(you())),
        Some(HistoryWindow::SinceLastUpkeep(you())),
        Some(HistoryWindow::LastTurn),
        Some(HistoryWindow::ThisGame),
    ] {
        if let Some(b) = c.strip_suffix(window_suffix(&w)) {
            return Some((b.to_string(), w));
        }
    }
    let i = c.find(" this turn from ")?;
    Some((format!("{}{}", &c[..i], &c[i + " this turn".len()..]), None))
}

/// "if [history clause] [window]" without referents.
pub fn history_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    // A legendary card's short name ("if Kaito entered this turn", Kaito Shizuki) is ~.
    let short;
    let name = crate::oracle::card_name().to_lowercase();
    let c = match name.split_once(' ') {
        Some((first, _)) if !name.contains(',') && c.starts_with(&format!("{first} ")) => {
            short = format!("~{}", &c[first.len()..]);
            short.as_str()
        }
        _ => c,
    };
    // "if your team gained life this turn": you and/or your teammates (CR 102.4).
    if let Some(r) = c.strip_prefix("your team ") {
        let cond = history_condition(&format!("you {r}"))?;
        return team_condition(cond);
    }
    let (body, w) = split_window(c)?;
    clause_or_joined(&body, &w)
}

/// A condition about your events made about your team's: each event of yours may be a
/// teammate's too (CR 102.4: "your team" means you and/or your teammates).
fn team_condition(cond: Condition) -> Option<Condition> {
    fn teamed(t: &TriggerCond) -> Option<TriggerCond> {
        let mate = match t {
            TriggerCond::GainsLife { who: PlayerRel::You } => TriggerCond::GainsLife {
                who: PlayerRel::Teammate,
            },
            TriggerCond::LosesLife { who: PlayerRel::You } => TriggerCond::LosesLife {
                who: PlayerRel::Teammate,
            },
            _ => return None,
        };
        Some(TriggerCond::AnyOf(vec![t.clone(), mate]))
    }
    Some(match cond {
        Condition::Compare(Value::EventsThisTurn(t, Tally::Events), cmp, n) => {
            Condition::Compare(Value::EventsThisTurn(Box::new(teamed(&t)?), Tally::Events), cmp, n)
        }
        Condition::Not(c) => Condition::Not(Box::new(team_condition(*c)?)),
        _ => return None,
    })
}

inventory::submit! { super::ConditionPattern { name: "history: [subject] [past verb] [objects] this turn", priority: 900, parse: history_condition } }

/// A history clause about a referent: "it attacked or blocked this turn", "that creature
/// entered this turn", "equipped creature didn't deal combat damage to a creature this
/// turn". The clause is read about "~" and then made about the referent: an object the
/// events were about is that object (the same object while it stays where it is, CR
/// 400.7).
pub fn referent_history(c: &str, b: &mut crate::oracle::effects::Builder) -> Option<Condition> {
    let c = end(c);
    // "if it's a creature or if {G}{W} was spent to cast ~": either condition.
    if let Some((x, y)) = c.split_once(" or if ") {
        let cx = super::conditions_referents::parse_condition_with(x, b)?;
        let cy = super::conditions_referents::parse_condition_with(y, b)?;
        return Some(Condition::Or(vec![cx, cy]));
    }
    if let Some(cond) = cast_condition(c, b)
        .or_else(|| their_last_turn(c, Some(b)))
        .or_else(|| source_to_referent(c, b))
        .or_else(|| nth_time_counters(c, b))
        .or_else(|| super::history_conditions_4::first_time_referent(c, b))
        .or_else(|| phase_condition(c))
        .or_else(|| resolved_ordinal(c))
        .or_else(|| nth_spell(c, b))
    {
        return Some(cond);
    }
    // "Exile target creature, then return it to the battlefield under its owner's control.
    // If it entered under your control, ...": the permanent the instruction just put onto
    // the battlefield, under whose control it entered (it's checked right away).
    if matches!(
        c,
        "it entered under your control" | "it entered the battlefield under your control"
    ) && matches!(b.it, Sel::Var(vars::IT))
    {
        return Some(Condition::SelMatches(
            b.it.clone(),
            Filter::ControlledBy(PlayerRel::You),
        ));
    }
    if c.contains('~') || split_window(c).is_none() {
        return None;
    }
    // "4 or more damage was dealt to it this turn": "it was dealt 4 or more damage".
    let swapped;
    let c = match c.split_once(" damage was dealt to ") {
        Some((amount, r)) if parse_number(amount).is_some() => {
            let (who, window) = r.split_once(' ').unwrap_or((r, ""));
            swapped = format!("{who} was dealt {amount} damage {window}");
            swapped.as_str()
        }
        _ => c,
    };
    // "... to that player": the player the ability is about.
    if let Some((x, y)) = c.split_once(" to that player ") {
        let rel = match &b.it_player {
            PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
            PlayerRef::Target(t) => PlayerRel::Target(*t),
            _ => return None,
        };
        let cond = referent_history(&format!("{x} to a player {y}"), b)?;
        let mut json = serde_json::to_value(&cond).ok()?;
        let rel = serde_json::to_value(rel).ok()?;
        let mut n = 0;
        fn walk(v: &mut serde_json::Value, rel: &serde_json::Value, n: &mut usize) {
            match v {
                serde_json::Value::Object(m) => {
                    for (k, x) in m.iter_mut() {
                        if k == "to" && x.get("Player").is_some_and(|p| p == "Any") {
                            *x = serde_json::json!({ "Player": rel });
                            *n += 1;
                        } else {
                            walk(x, rel, n);
                        }
                    }
                }
                serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, rel, n)),
                _ => {}
            }
        }
        walk(&mut json, &rel, &mut n);
        if n == 0 {
            return None;
        }
        return serde_json::from_value(json).ok();
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

/// "[the object] [past predicate] [window]" for an object a static ability names ("~
/// can't attack if it attacked during your last turn", "enchanted creature doesn't untap
/// during its controller's untap step if it attacked during its controller's last turn").
pub fn object_history(pred: &str, sel: &Sel) -> Option<Condition> {
    let pred = end(pred);
    if pred.contains('~') {
        return None;
    }
    let (pred, controllers) = match pred.strip_suffix(" during its controller's last turn") {
        Some(p) => (format!("{p} during your last turn"), true),
        None => (pred.to_string(), false),
    };
    let cond = history_condition(&format!("~ {pred}"))?;
    let cond = if matches!(sel, Sel::This) {
        cond
    } else {
        about(cond, sel)?
    };
    if !controllers {
        return Some(cond);
    }
    // The window is its controller's last turn.
    let mut json = serde_json::to_value(&cond).ok()?;
    let whose = serde_json::to_value(PlayerRef::ControllerOf(Box::new(sel.clone()))).ok()?;
    fn walk(v: &mut serde_json::Value, whose: &serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, x) in m.iter_mut() {
                    if k == "PlayersLastTurn" {
                        *x = whose.clone();
                    } else {
                        walk(x, whose);
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, whose)),
            _ => {}
        }
    }
    walk(&mut json, &whose);
    serde_json::from_value(json).ok()
}

/// The condition with each "~" ([`Filter::Source`]) in its trigger conditions meaning the
/// selected object instead.
pub fn about(cond: Condition, sel: &Sel) -> Option<Condition> {
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

// ---------------------------------------------------------------------------
// How a spell was cast and paid for
// ---------------------------------------------------------------------------

use crate::kw::cast_history as ch;
use crate::oracle::effects::Builder;

/// The spell a cast-status or mana-spent condition is about.
#[derive(Clone, Debug)]
enum Whose {
    /// The source's own spell: a resolving spell, or the spell a permanent was.
    Own,
    /// The spell that caused the ability to trigger.
    TriggerSpell,
    /// Another object (a permanent that entered, a target).
    Obj(Sel),
}

/// The trigger condition of the ability being compiled, lowercased with the card's name
/// as "~" (granted abilities in quotes included).
fn trigger_texts() -> Vec<String> {
    let mut raw = crate::oracle::raw_text().to_lowercase();
    let name = crate::oracle::card_name().to_lowercase();
    if !name.is_empty() {
        raw = raw.replace(&name, "~");
        if let Some((short, _)) = name.split_once(", ") {
            raw = raw.replace(short, "~");
        }
    }
    let raw = raw.replace("this creature", "~");
    raw.split(['"', '\n'])
        .filter_map(|part| {
            let i = part.find("whenever ").or_else(|| part.find("when "))?;
            Some(part[i..].split(',').next().unwrap_or("").to_string())
        })
        .collect()
}

/// Whether the ability triggers on a spell being cast ("whenever you cast a noncreature
/// spell").
fn cast_trigger(b: &Builder) -> bool {
    b.in_trigger
        && trigger_texts()
            .iter()
            .any(|t| t.contains(" cast ") || t.contains(" casts ") || t.ends_with(" cast"))
}

/// Whether the trigger object is the source itself ("When ~ enters").
fn self_trigger(b: &Builder) -> bool {
    b.in_trigger
        && trigger_texts().iter().any(|t| {
            let t = t
                .trim_start_matches("whenever ")
                .trim_start_matches("when ");
            t.starts_with("~ enters") || t.starts_with("~ enter ")
        })
}

/// What a subject word of a cast-status condition refers to.
fn whose(word: &str, b: Option<&mut Builder>) -> Option<Whose> {
    if matches!(word, "~" | "this spell" | "this creature") {
        return Some(Whose::Own);
    }
    let b = b?;
    let pronoun = matches!(
        word,
        "it" | "he" | "she" | "they" | "him" | "her" | "them" | "that spell" | "that creature"
    );
    if !pronoun {
        return None;
    }
    if cast_trigger(b)
        && matches!(b.it, Sel::TriggerSpell | Sel::TriggerObject)
        && word != "that creature"
    {
        return Some(Whose::TriggerSpell);
    }
    if word == "that creature" || word == "that spell" {
        let before = b.targets.len();
        let (sel, rest) = crate::oracle::effects::object_ref(word, b)?;
        if b.targets.len() != before || !rest.trim().is_empty() {
            b.targets.truncate(before);
            return None;
        }
        return Some(Whose::Obj(sel));
    }
    match &b.it {
        Sel::This => Some(Whose::Own),
        Sel::TriggerObject if self_trigger(b) => Some(Whose::Own),
        it if super::oracle_hardening_referents::is_no_referent(it) => None,
        it => Some(Whose::Obj(it.clone())),
    }
}

/// A status of the spell: own conditions for the source's own spell, filters otherwise.
fn status(w: &Whose, own: Condition, filter: Filter) -> Condition {
    match w {
        Whose::Own => own,
        Whose::TriggerSpell => Condition::SelMatches(Sel::TriggerSpell, filter),
        Whose::Obj(sel) => Condition::SelMatches(sel.clone(), filter),
    }
}

fn not(c: Condition) -> Condition {
    Condition::Not(Box::new(c))
}

/// "was cast from [zone]" / "entered from [zone]": the zone.
fn zone_phrase(z: &str) -> Option<ZoneKind> {
    Some(match z {
        "your hand" => ZoneKind::Hand,
        "your graveyard" | "a graveyard" => ZoneKind::Graveyard,
        "exile" => ZoneKind::Exile,
        "your library" => ZoneKind::Library,
        _ => return None,
    })
}

fn was_cast(w: &Whose) -> Condition {
    status(w, Condition::WasCast, Filter::Custom(ch::WAS_CAST.into()))
}

/// Whether a zone phrase names your zone ("your graveyard"): a card in it is yours (CR
/// 400.3), unlike one in "a graveyard".
fn yours(z: &str) -> bool {
    z.starts_with("your ")
}

/// The quality, and that the object is yours if the zone was (`yours`).
fn owned(f: Filter, yours: bool) -> Filter {
    if yours {
        Filter::and(vec![f, Filter::OwnedBy(PlayerRel::You)])
    } else {
        f
    }
}

fn cast_from(w: &Whose, z: ZoneKind, yours: bool) -> Condition {
    status(
        w,
        Condition::CastFrom(z),
        owned(Filter::Custom(format!("{}{z:?}", ch::CAST_FROM).into()), yours),
    )
}

/// "[S] entered from [zone]": the permanent came from that zone (CR 400.7).
fn entered_from(w: &Whose, z: ZoneKind, yours: bool) -> Condition {
    let f = owned(
        Filter::Custom(crate::kw::played_from_zone::came_from(z).into()),
        yours,
    );
    match w {
        Whose::Own => Condition::SelMatches(Sel::This, f),
        Whose::TriggerSpell => Condition::Never,
        Whose::Obj(sel) => Condition::SelMatches(sel.clone(), f),
    }
}

const SUBJECTS: &[&str] = &[
    "~",
    "this spell",
    "this creature",
    "it",
    "he",
    "she",
    "they",
    "that spell",
    "that creature",
];
const OBJECTS: &[&str] = &[
    "~",
    "this spell",
    "this creature",
    "it",
    "him",
    "her",
    "them",
    "that spell",
    "that creature",
];

/// One cast-status clause: "[S] was cast [from Z]", "[S] wasn't cast", "you cast [S]
/// [from Z | from anywhere other than your hand]", "you didn't cast [S]", "[S] was
/// kicked", "[S] entered from [Z]".
fn cast_status_clause(c: &str, b: &mut Option<&mut Builder>) -> Option<Condition> {
    let c = c.trim();
    for (p, neg) in [("you cast ", false), ("you didn't cast ", true)] {
        if let Some(r) = c.strip_prefix(p) {
            let (word, rest) = OBJECTS
                .iter()
                .filter_map(|w| r.strip_prefix(w).map(|x| (*w, x)))
                .find(|(_, x)| word_end(x))?;
            let w = whose(word, b.as_deref_mut())?;
            let cond = match rest.trim() {
                "" => was_cast(&w),
                "from anywhere other than your hand" => {
                    Condition::And(vec![was_cast(&w), not(cast_from(&w, ZoneKind::Hand, false))])
                }
                z => {
                    let z = z.strip_prefix("from ")?;
                    cast_from(&w, zone_phrase(z)?, yours(z))
                }
            };
            return Some(if neg { not(cond) } else { cond });
        }
    }
    let (word, rest) = SUBJECTS
        .iter()
        .filter_map(|w| c.strip_prefix(w).map(|x| (*w, x)))
        .find(|(_, x)| x.starts_with(' '))?;
    let rest = rest.trim();
    let w = whose(word, b.as_deref_mut())?;
    if matches!(rest, "was cast this turn" | "were cast this turn") {
        return Some(status(
            &w,
            Condition::SelMatches(Sel::This, Filter::Custom(ch::CAST_THIS_TURN.into())),
            Filter::Custom(ch::CAST_THIS_TURN.into()),
        ));
    }
    // "was cast for its warp cost" (an alternative cost, CR 118.9).
    if let Some(k) = rest
        .strip_prefix("was cast for its ")
        .and_then(|r| r.strip_suffix(" cost"))
    {
        let kind = crate::keywords::KeywordKind::from_name(k)?;
        let f = Filter::Custom(format!("{}{kind:?}", ch::CAST_FOR).into());
        return Some(status(&w, Condition::SelMatches(Sel::This, f.clone()), f));
    }
    for (p, neg) in [
        ("was cast", false),
        ("were cast", false),
        ("wasn't cast", true),
        ("weren't cast", true),
    ] {
        if let Some(x) = rest.strip_prefix(p) {
            let cond = match x.trim() {
                "" => was_cast(&w),
                z => {
                    let z = z.strip_prefix("from ")?;
                    cast_from(&w, zone_phrase(z)?, yours(z))
                }
            };
            return Some(if neg { not(cond) } else { cond });
        }
    }
    if matches!(rest, "was kicked" | "were kicked") {
        return Some(status(
            &w,
            Condition::CostPaid("kicker".into()),
            Filter::Custom(ch::KICKED.into()),
        ));
    }
    if let Some(z) = rest.strip_prefix("entered from ") {
        return Some(entered_from(&w, zone_phrase(z)?, yours(z)));
    }
    None
}

/// Cast-status clauses joined by "or" ("it entered from your graveyard or you cast it
/// from your graveyard", "it entered from your library or was cast from your library").
fn cast_status(c: &str, mut b: Option<&mut Builder>) -> Option<Condition> {
    if let Some(cond) = cast_status_clause(c, &mut b) {
        return Some(cond);
    }
    let (x, y) = c.split_once(" or ")?;
    let cx = cast_status_clause(x, &mut b)?;
    let cy = cast_status_clause(y, &mut b).or_else(|| {
        // The same subject, elided: "... or was cast from your library".
        let subj = SUBJECTS.iter().find(|w| x.starts_with(&format!("{w} ")))?;
        cast_status_clause(&format!("{subj} {y}"), &mut b)
    })?;
    Some(Condition::Or(vec![cx, cy]))
}

/// "[amount] [mana] was spent to cast [S]" / "... to activate this ability": the
/// payment's Value::Custom prefix argument.
fn payment_whose(r: &str, b: &mut Option<&mut Builder>) -> Option<(&'static str, Whose)> {
    if r == "activate this ability" {
        return Some(("this ability", Whose::Own));
    }
    let word = r.strip_prefix("cast ")?;
    if !OBJECTS.contains(&word) {
        return None;
    }
    match whose(word, b.as_deref_mut())? {
        Whose::Own => Some(("self", Whose::Own)),
        Whose::TriggerSpell => Some(("that spell", Whose::TriggerSpell)),
        // A target spell ("Counter target spell if no mana was spent to cast it").
        Whose::Obj(Sel::Target(i)) => Some((
            ["target 0", "target 1", "target 2"].get(i as usize)?,
            Whose::Obj(Sel::Target(i)),
        )),
        Whose::Obj(_) => None,
    }
}

/// An amount of the mana spent on a spell or ability (the existing values for the total
/// spent on the source's own spell and on the triggering spell).
fn spent_value(key: &str, what: &str) -> Value {
    match (key, what) {
        ("self", "total") => Value::ManaSpent,
        ("that spell", "total") => Value::Custom(crate::kw::opus::MANA_SPENT_ON_THAT_SPELL.into()),
        _ => Value::Custom(ch::spent(key, what).into()),
    }
}

/// Mana-spent conditions: "no mana was spent to cast it", "no colored mana was spent to
/// cast it", "at least four mana was spent to cast it", "mana from a Treasure was spent
/// to cast it", "three or more mana from creatures was spent to cast it", "{C} wasn't spent
/// to cast it", "exactly three colors of mana were spent to activate this ability", "the
/// amount of mana spent to cast it was less than its mana value".
fn mana_spent(c: &str, mut b: Option<&mut Builder>) -> Option<Condition> {
    let c = c.trim();
    // "the amount of mana spent to cast [S] was less than its mana value".
    if let Some(r) = c.strip_prefix("the amount of mana spent to ") {
        for (sep, cmp) in [
            (" was less than ", Cmp::Lt),
            (" is less than ", Cmp::Lt),
            (" was greater than ", Cmp::Gt),
            (" is greater than ", Cmp::Gt),
        ] {
            let Some((x, y)) = r.split_once(sep) else {
                continue;
            };
            let (key, w) = payment_whose(x, &mut b)?;
            let sel = match w {
                Whose::Own => Sel::This,
                Whose::TriggerSpell => Sel::TriggerSpell,
                Whose::Obj(sel) => sel,
            };
            let other = match y {
                "its mana value" | "that spell's mana value" => Value::ManaValueOf(Box::new(sel)),
                "~'s power" => Value::PowerOf(Box::new(Sel::This)),
                _ => return None,
            };
            return Some(Condition::Compare(spent_value(key, "total"), cmp, other));
        }
        return None;
    }
    let (amount, neg, r) = [
        (" was spent to ", false),
        (" were spent to ", false),
        (" wasn't spent to ", true),
    ]
    .iter()
    .find_map(|(sep, neg)| c.split_once(sep).map(|(a, r)| (a, *neg, r)))?;
    let (key, w) = payment_whose(r, &mut b)?;
    // Mana symbols: the symbol grammar's, about the source's own spell.
    if amount.starts_with('{') {
        // The symbol grammar reads "[symbols] was spent to cast ~" itself.
        if !neg || !matches!(w, Whose::Own) || key != "self" {
            return None;
        }
        let cond =
            crate::oracle_ext::parse_condition_ext(&format!("{amount} was spent to cast ~"))?;
        return Some(if neg { not(cond) } else { cond });
    }
    if neg {
        return None;
    }
    let v = |what: &str| spent_value(key, what);
    let cmp = |what: &str, cmp: Cmp, n: i32| Condition::Compare(v(what), cmp, Value::Const(n));
    let cond = match amount {
        "no mana" => cmp("total", Cmp::Eq, 0),
        "no colored mana" => cmp("colored", Cmp::Eq, 0),
        "mana from a treasure" => cmp("source subtype:Treasure", Cmp::Ge, 1),
        _ => {
            let (q, n, _, rest) = quantity(amount)?;
            if q != Cmp::Ge && q != Cmp::Eq {
                return None;
            }
            match rest {
                "mana" => cmp("total", q, n),
                "mana from creatures" | "mana from a creature" => cmp("source type:Creature", q, n),
                "colors of mana" | "color of mana" => cmp("colors", q, n),
                _ => return None,
            }
        }
    };
    Some(cond)
}

/// "create a Treasure token for each mana from a Treasure spent to cast it" (Marut, in
/// "When ~ enters, ..."): as many as that mana (CR 400.7d: the permanent knows what was
/// spent to cast the spell it was).
fn for_each_mana_from(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, r) = l.split_once(" for each mana from ")?;
    let (kind, whose) = r.split_once(" spent to cast ")?;
    let own = whose == "~" || (whose == "it" && self_trigger(b));
    if !own {
        return None;
    }
    let what = match kind {
        "a treasure" => "source subtype:Treasure",
        "a creature" | "creatures" => "source type:Creature",
        _ => return None,
    };
    let saved = b.targets.len();
    let e = crate::oracle::effects::parse_clause(head, b);
    match e {
        Some(Effect::CreateToken {
            spec,
            count: Value::Const(1),
            controller,
            tapped,
            attacking,
        }) => Some(Effect::CreateToken {
            spec,
            count: spent_value("self", what),
            controller,
            tapped,
            attacking,
        }),
        _ => {
            b.targets.truncate(saved);
            None
        }
    }
}

inventory::submit! { super::EffectPattern { name: "history: [create a token] for each mana from [a kind] spent to cast it", priority: 900, parse: for_each_mana_from } }

/// "Some of [group] [condition]": each condition about the group made about some object of
/// it.
fn some_of(c: Condition, group: &Sel) -> Option<Condition> {
    Some(match c {
        Condition::SelMatches(s, f) if format!("{s:?}") == format!("{group:?}") => {
            Condition::SelNonEmpty(Sel::Matching(Box::new(s), f))
        }
        Condition::Or(v) => Condition::Or(
            v.into_iter()
                .map(|x| some_of(x, group))
                .collect::<Option<Vec<_>>>()?,
        ),
        _ => return None,
    })
}

/// Cast-status and mana-spent conditions without referents ("~ wasn't cast from your
/// hand", "at least three mana was spent to cast ~").
fn cast_condition_free(c: &str) -> Option<Condition> {
    let c = end(c);
    cast_status(c, None).or_else(|| mana_spent(c, None))
}

inventory::submit! { super::ConditionPattern { name: "history: how ~ was cast and paid for", priority: 900, parse: cast_condition_free } }

/// Cast-status and mana-spent conditions about a referent ("if it was cast", "if he was
/// kicked", "if no mana was spent to cast it" in a cast trigger).
pub fn cast_condition(c: &str, b: &mut Builder) -> Option<Condition> {
    let c = end(c);
    // "if one or more of them entered from a graveyard or was cast from a graveyard" (the
    // objects of a "one or more" trigger): some of them.
    if let Some(r) = c.strip_prefix("one or more of them ") {
        if !matches!(b.it, Sel::TriggerObjects) {
            return None;
        }
        return some_of(
            cast_status(&format!("it {r}"), Some(b))?,
            &Sel::TriggerObjects,
        );
    }
    cast_status(c, Some(b)).or_else(|| mana_spent(c, Some(b)))
}

// ---------------------------------------------------------------------------
// Ordinals and totals: "the first combat phase", "the second creature spell you cast
// this turn", "the second time this ability has resolved", "total power 6 or greater"
// ---------------------------------------------------------------------------

/// "first", "second", ... as a number.
fn ordinal(w: &str) -> Option<i32> {
    Some(match w {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        _ => return None,
    })
}

/// "it's the first combat phase of the turn" (CR 506.1: a turn may have several), "of
/// your turn".
fn phase_condition(c: &str) -> Option<Condition> {
    let first = || Condition::Custom(crate::kw::activated_ability_kind::FIRST_COMBAT_PHASE.into());
    match c {
        "it's the first combat phase of the turn" => Some(first()),
        "it's the first combat phase of your turn" => {
            Some(Condition::And(vec![Condition::YourTurn, first()]))
        }
        _ => None,
    }
}

/// "this ability has been activated four or more times this turn" (in the ability's own
/// effect; the activation being resolved counts).
fn activated_times(c: &str) -> Option<Condition> {
    let r = c.strip_prefix("this ability has been activated ")?;
    let (n, r) = parse_number(r)?;
    let cmp = match r.trim() {
        "or more times this turn" => Cmp::Ge,
        "times this turn" => Cmp::Eq,
        _ => return None,
    };
    Some(Condition::Compare(
        Value::Custom(ch::TIMES_ACTIVATED.into()),
        cmp,
        n,
    ))
}

/// "this is the first or second time this ability has resolved this turn" (the count
/// includes this resolution); "it's the second time" after such a condition earlier in the
/// ability ("..., if this is the first time this ability has resolved this turn. If it's
/// the second time, ...").
fn resolved_ordinal(c: &str) -> Option<Condition> {
    let times =
        |n: i32, cmp: Cmp| Condition::Compare(Value::TimesResolvedThisTurn, cmp, Value::c(n));
    if let Some(r) = c.strip_prefix("this is the ") {
        let r = r.strip_suffix(" time this ability has resolved this turn")?;
        let (a, b) = r.split_once(" or ")?;
        let (a, b) = (ordinal(a)?, ordinal(b)?);
        if a != 1 || b != 2 {
            return None;
        }
        return Some(times(2, Cmp::Le));
    }
    let r = c.strip_prefix("it's the ")?.strip_suffix(" time")?;
    let n = ordinal(r)?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("time this ability has resolved this turn") {
        return None;
    }
    Some(times(n, Cmp::Eq))
}

/// "you attacked with creatures with total power 6 or greater this combat" (pack tactics):
/// the creatures you attacked with, each with its power now (or as it last existed).
fn attacked_total_power(c: &str) -> Option<Condition> {
    let (body, w) = split_window(c)?;
    let r = body.strip_prefix("you attacked with ")?;
    let (np, n) = r.split_once(" with total power ")?;
    let (n, rest) = parse_number(n)?;
    if rest.trim() != "or greater" {
        return None;
    }
    let (f, true, tail) = parse_object_phrase(np)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    let cond = TriggerCond::Attacks(Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]));
    let sel = match w {
        None => Sel::ThisTurn(Box::new(cond)),
        Some(w) => Sel::EventsDuring(w, Box::new(cond)),
    };
    Some(Condition::Compare(
        Value::Aggregate(AggOp::Sum, Stat::Power, Box::new(sel)),
        Cmp::Ge,
        n,
    ))
}

/// Ordinals and totals without referents.
fn ordinal_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    phase_condition(c)
        .or_else(|| activated_times(c))
        .or_else(|| attacked_total_power(c))
        .or_else(|| {
            if c.starts_with("this is the ") {
                resolved_ordinal(c)
            } else {
                None
            }
        })
}

inventory::submit! { super::ConditionPattern { name: "history: ordinals and totals this turn", priority: 900, parse: ordinal_condition } }

/// "it's the second creature spell you cast this turn", "it was the second spell you cast
/// this turn": the spell (the triggering spell, or the source's own) is the Nth matching
/// spell its controller cast this turn, counting the ones cast up to it.
fn nth_spell(c: &str, b: &mut Builder) -> Option<Condition> {
    let r = c
        .strip_prefix("it's the ")
        .or_else(|| c.strip_prefix("it was the "))?;
    let (w, r) = split_word(r);
    let n = ordinal(w)?;
    let np = r
        .strip_suffix(" you cast this turn")
        .or_else(|| r.strip_suffix(" you've cast this turn"))?;
    let (f, false, tail) = parse_object_phrase(np)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    let (key, subject) = match whose("it", Some(b))? {
        Whose::Own => ("self", None),
        Whose::TriggerSpell => ("that spell", Some(Sel::TriggerSpell)),
        Whose::Obj(_) => return None,
    };
    let plain_spell = matches!(f, Filter::Spell);
    let count = Condition::Compare(
        Value::EventsThisTurn(
            Box::new(TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::and(vec![
                    f.clone(),
                    Filter::Custom(format!("{}{key}", ch::CAST_BY_THEN).into()),
                ]),
            }),
            Tally::Events,
        ),
        Cmp::Eq,
        Value::c(n),
    );
    Some(match subject {
        Some(sel) if !plain_spell => Condition::And(vec![Condition::SelMatches(sel, f), count]),
        _ if !plain_spell => return None,
        _ => count,
    })
}

/// "[instruction] if [condition] or if [condition]" ("Destroy target nonland permanent if
/// it's a creature or if {G}{W} was spent to cast ~"): the instruction happens if either
/// holds. A pronoun in the conditions refers to the target the instruction names.
fn either_if(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (x, c) = l.split_once(" if ")?;
    if !c.contains(" or if ") || c.matches(" if ").count() != 1 || x.contains(',') {
        return None;
    }
    let first = b.targets.len();
    let e = crate::oracle::effects::parse_clause(x, b)?;
    let saved = b.it.clone();
    if let Some(i) = (first..b.targets.len()).find(|i| {
        matches!(
            b.targets[*i].what,
            TargetKind::Object(_) | TargetKind::Spell(_)
        )
    }) {
        b.it = Sel::Target(i as u8);
    }
    let cond = super::conditions_referents::parse_condition_with(c, b);
    b.it = saved;
    let Some(cond) = cond else {
        b.targets.truncate(first);
        return None;
    };
    Some(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { super::EffectPattern { name: "history: [instruction] if [condition] or if [condition]", priority: 240, parse: either_if } }

/// "[instruction] if [condition] and [instruction] if [condition]" where the second
/// instruction leaves out what it shares with the first ("Target player can't play lands
/// this turn if {R} was spent to cast ~ and can't cast creature spells this turn if {W}
/// was spent to cast ~", "You gain X life if {G} was spent to cast ~ and X life if {W} was
/// spent to cast ~"): each instruction happens if its own condition holds.
fn two_conditional_instructions(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l.matches(" if ").count() != 2 || l.contains(',') {
        return None;
    }
    let (x1, rest) = l.split_once(" if ")?;
    let (mid, c2) = rest.rsplit_once(" if ")?;
    let (c1, x2) = mid.rsplit_once(" and ")?;
    let cond1 = crate::oracle::statics::parse_condition(c1, b.ctx)?;
    let cond2 = crate::oracle::statics::parse_condition(c2, b.ctx)?;
    let first = b.targets.len();
    let e1 = crate::oracle::effects::parse_clause(x1, b)?;
    let after1 = b.targets.len();
    // The second instruction's shared words: the first instruction's opening words, with
    // "target player" as "that player" (the same player).
    let words: Vec<String> = x1
        .replacen("target player", "that player", 1)
        .split(' ')
        .map(str::to_string)
        .collect();
    for k in 1..words.len().min(5) {
        let x2_full = format!("{} {x2}", words[..k].join(" "));
        let saved = (b.it.clone(), b.it_player.clone());
        if let Some(e2) = crate::oracle::effects::parse_clause(&x2_full, b) {
            if b.targets.len() == after1 {
                return Some(Effect::Seq(vec![
                    Effect::If {
                        cond: cond1,
                        then: Box::new(e1),
                        otherwise: Box::new(Effect::Noop),
                    },
                    Effect::If {
                        cond: cond2,
                        then: Box::new(e2),
                        otherwise: Box::new(Effect::Noop),
                    },
                ]));
            }
        }
        b.targets.truncate(after1);
        b.it = saved.0;
        b.it_player = saved.1;
    }
    b.targets.truncate(first);
    None
}

inventory::submit! { super::EffectPattern { name: "history: [instruction] if [c1] and [instruction] if [c2]", priority: 245, parse: two_conditional_instructions } }

/// "[a player | that player] [past verb] ... during their last turn" ("if a player
/// attacked you during their last turn", "if that player attacked you during their last
/// turn"): what the player did during their own latest turn.
fn their_last_turn(c: &str, b: Option<&Builder>) -> Option<Condition> {
    let body = end(c).strip_suffix(" during their last turn")?;
    let (subj, rest) = body.split_once(' ')?;
    let (subj, rest) = match subj {
        "a" => ("a player", rest.strip_prefix("player ")?),
        "that" => ("that player", rest.strip_prefix("player ")?),
        "they" => ("that player", rest),
        _ => return None,
    };
    // Read as "a player [did it]", then made about the player whose turn it was.
    let cl = player_clause_as(&format!("a player {rest}"))?;
    if cl.amount || cl.distinct || cl.per_player.is_some() {
        return None;
    }
    let cond = trigger_of(&cl.trigger)?;
    let (rel, who, over) = match subj {
        "a player" => (PlayerRel::Iterated, PlayerRef::Iterated, true),
        _ => {
            let b = b?;
            let rel = match &b.it_player {
                PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
                PlayerRef::Target(t) => PlayerRel::Target(*t),
                _ => return None,
            };
            (rel, b.it_player.clone(), false)
        }
    };
    // The actor of the event ("attacker", "who") is that player.
    let mut json = serde_json::to_value(&cond).ok()?;
    let rel = serde_json::to_value(rel).ok()?;
    let mut n = 0;
    if let serde_json::Value::Object(m) = &mut json {
        for inner in m.values_mut() {
            if let serde_json::Value::Object(f) = inner {
                for key in ["attacker", "who"] {
                    if f.get(key).is_some_and(|v| v == "Any") {
                        f.insert(key.into(), rel.clone());
                        n += 1;
                    }
                }
            }
        }
    }
    if n != 1 {
        return None;
    }
    let cond: TriggerCond = serde_json::from_value(json).ok()?;
    let v = Value::EventsDuring(
        HistoryWindow::PlayersLastTurn(who),
        Box::new(cond),
        Tally::Events,
    );
    let v = if over {
        Value::OverPlayers(AggOp::Max, PlayerFilter::Any, Box::new(v))
    } else {
        v
    };
    let cmp = Condition::Compare(v, cl.cmp, Value::Const(cl.n));
    Some(if cl.negated { not(cmp) } else { cmp })
}

fn their_last_turn_free(c: &str) -> Option<Condition> {
    their_last_turn(c, None)
}

inventory::submit! { super::ConditionPattern { name: "history: a player [did] during their last turn", priority: 900, parse: their_last_turn_free } }

/// "~ dealt damage to it this turn" (the source to a referent): the damage events whose
/// recipient is that object.
fn source_to_referent(c: &str, b: &mut Builder) -> Option<Condition> {
    if !c.starts_with("~ ") || !c.contains(" to it ") {
        return None;
    }
    let sel = match &b.it {
        Sel::This => return None,
        it if super::oracle_hardening_referents::is_no_referent(it) => return None,
        it => it.clone(),
    };
    let cond = history_condition(&c.replacen(" to it ", " to a permanent ", 1))?;
    let mut json = serde_json::to_value(&cond).ok()?;
    let with = serde_json::to_value(DamageRecipient::Object(Filter::In(Box::new(sel)))).ok()?;
    let mut n = 0;
    fn walk(v: &mut serde_json::Value, with: &serde_json::Value, n: &mut usize) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, x) in m.iter_mut() {
                    if k == "to" && x.get("Object").is_some() {
                        *x = with.clone();
                        *n += 1;
                    } else {
                        walk(x, with, n);
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, with, n)),
            _ => {}
        }
    }
    walk(&mut json, &with, &mut n);
    if n != 1 {
        return None;
    }
    serde_json::from_value(json).ok()
}

/// "it's the first time +1/+1 counters have been put on that permanent this turn": the
/// times counters of that kind were put on it this turn (each event one time), this one
/// included.
fn nth_time_counters(c: &str, b: &mut Builder) -> Option<Condition> {
    let r = c.strip_prefix("it's the ")?;
    let (w, r) = split_word(r);
    let n = ordinal(w)?;
    let r = r.strip_prefix("time ")?;
    let (kind, r) = r.split_once(" counters have been put on ")?;
    let who = r.strip_suffix(" this turn")?;
    let before = b.targets.len();
    let saved = b.it.clone();
    let got = crate::oracle::effects::object_ref(who, b);
    b.it = saved;
    let (sel, rest) = got?;
    if b.targets.len() != before || !rest.trim().is_empty() {
        b.targets.truncate(before);
        return None;
    }
    let cond = trigger_of(&format!("one or more {kind} counters are put on ~"))?;
    let cond = Condition::Compare(
        Value::EventsThisTurn(Box::new(cond), Tally::Events),
        Cmp::Eq,
        Value::c(n),
    );
    if matches!(sel, Sel::This) {
        Some(cond)
    } else {
        about(cond, &sel)
    }
}

/// "the Ring has tempted you four or more times this game" (CR 701.54).
fn ring_tempted(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("the ring has tempted you ")?;
    let (n, r) = parse_number(r)?;
    if !matches!(r.trim(), "or more times this game" | "or more times") {
        return None;
    }
    Some(Condition::Compare(
        Value::Custom(crate::kwa::ring::TIMES_TEMPTED.into()),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! { super::ConditionPattern { name: "history: the Ring has tempted you N or more times this game", priority: 900, parse: ring_tempted } }

#[cfg(test)]
mod tests {
    use crate::ability::*;

    fn cond(c: &str) -> Condition {
        super::history_condition(c).unwrap_or_else(|| panic!("not read: {c}"))
    }

    fn json(c: &Condition) -> String {
        serde_json::to_string(c).unwrap()
    }

    #[test]
    fn player_clauses_count_this_turns_events() {
        match cond("you cycled two or more cards this turn") {
            Condition::Compare(
                Value::EventsThisTurn(t, Tally::Events),
                Cmp::Ge,
                Value::Const(2),
            ) => {
                assert!(matches!(
                    *t,
                    TriggerCond::Cycled {
                        who: PlayerRel::You,
                        ..
                    }
                ))
            }
            x => panic!("{x:?}"),
        }
        match cond("an opponent discarded a card this turn") {
            Condition::Compare(Value::EventsThisTurn(t, _), Cmp::Ge, Value::Const(1)) => {
                assert!(matches!(
                    *t,
                    TriggerCond::Discards {
                        who: PlayerRel::Opponent,
                        ..
                    }
                ))
            }
            x => panic!("{x:?}"),
        }
        // Negated: "didn't", "haven't".
        assert!(matches!(
            cond("you didn't attack with a creature this turn"),
            Condition::Not(_)
        ));
        assert!(matches!(
            cond("you haven't cast a spell this turn"),
            Condition::Not(_)
        ));
        // An amount some one opponent reached.
        let c = cond("an opponent lost 3 or more life this turn");
        assert!(matches!(
            c,
            Condition::Compare(
                Value::OverPlayers(AggOp::Max, PlayerFilter::Opponent, _),
                Cmp::Ge,
                Value::Const(3)
            )
        ));
        // A count some one opponent reached.
        let c = cond("an opponent cast two or more spells this turn");
        assert!(
            matches!(
                &c,
                Condition::Compare(
                    Value::OverPlayers(AggOp::Max, PlayerFilter::Opponent, _),
                    Cmp::Ge,
                    Value::Const(2)
                )
            ) && json(&c).contains("Iterated"),
            "{c:?}"
        );
        // Two verbs sharing an object, and "both [A] and [B]".
        assert!(
            matches!(cond("you gained and lost life this turn"), Condition::And(v) if v.len() == 2)
        );
        assert!(matches!(
            cond("you've cast both a creature spell and a noncreature spell this turn"),
            Condition::And(v) if v.len() == 2
        ));
        // "this turn" before a qualifier.
        assert!(json(&cond(
            "you've played a land or cast a spell this turn from anywhere other than your hand"
        ))
        .contains("CastFrom"));
    }

    #[test]
    fn object_clauses_count_this_turns_events() {
        assert!(matches!(
            cond("no permanents left the battlefield this turn"),
            Condition::Compare(_, Cmp::Eq, Value::Const(0))
        ));
        let j = json(&cond("a creature card left your graveyard this turn"));
        assert!(
            j.contains("\"from\":\"Graveyard\"") && j.contains("Creature"),
            "{j}"
        );
        let j = json(&cond(
            "three or more cards were put into your graveyard from anywhere other than the battlefield this turn",
        ));
        assert!(
            j.contains("\"Const\":3") && !j.contains("\"from\":\"Battlefield\""),
            "{j}"
        );
        let j = json(&cond(
            "a creature entered the battlefield under an opponent's control this turn",
        ));
        assert!(
            j.contains("EntersBattlefield") && j.contains("Opponent"),
            "{j}"
        );
        // "had [objects] enter": the objects entered.
        let j = json(&cond(
            "an opponent had three or more cards put into their graveyard from anywhere this turn",
        ));
        assert!(j.contains("\"Const\":3") && j.contains("Opponent"), "{j}");
        // Distinct attackers.
        assert!(
            json(&cond("you attacked with three or more merfolk this turn")).contains("CountSel")
        );
    }

    #[test]
    fn windows_other_than_this_turn() {
        assert!(json(&cond("an opponent lost life last turn")).contains("LastTurn"));
        let c = cond("you haven't been dealt combat damage since your last turn");
        assert!(matches!(&c, Condition::Not(_)) && json(&c).contains("SinceLastTurn"));
        let c = cond("~ and at least two other creatures attacked this combat");
        assert!(matches!(&c, Condition::And(v) if v.len() == 2) && json(&c).contains("ThisCombat"));
        // Not history: no window, or no past tense.
        assert!(super::history_condition("a creature died").is_none());
        assert!(super::history_condition("you control a creature this turn").is_none());
    }

    #[test]
    fn cast_status_and_mana_spent_about_the_source() {
        let free = |c: &str| super::cast_condition_free(c).unwrap_or_else(|| panic!("{c}"));
        assert!(matches!(
            free("~ wasn't cast from your hand"),
            Condition::Not(c) if matches!(*c, Condition::CastFrom(ZoneKind::Hand))
        ));
        assert!(matches!(
            free("at least four mana was spent to cast ~"),
            Condition::Compare(Value::ManaSpent, Cmp::Ge, Value::Const(4))
        ));
        assert!(json(&free("mana from a treasure was spent to cast ~")).contains("Treasure"));
        assert!(matches!(
            free("no colored mana was spent to cast ~"),
            Condition::Compare(_, Cmp::Eq, Value::Const(0))
        ));
        assert!(json(&free(
            "exactly three colors of mana were spent to activate this ability"
        ))
        .contains("this ability:colors"));
        // A pronoun needs a referent (the Builder-aware reading).
        assert!(super::cast_condition_free("it was cast").is_none());
    }

    #[test]
    fn ordinals_and_totals() {
        let o = |c: &str| super::ordinal_condition(c).unwrap_or_else(|| panic!("{c}"));
        assert!(matches!(
            o("it's the first combat phase of the turn"),
            Condition::Custom(_)
        ));
        assert!(matches!(
            o("it's the first combat phase of your turn"),
            Condition::And(_)
        ));
        assert!(matches!(
            o("this ability has been activated four or more times this turn"),
            Condition::Compare(Value::Custom(_), Cmp::Ge, Value::Const(4))
        ));
        assert!(matches!(
            o("this is the first or second time this ability has resolved this turn"),
            Condition::Compare(Value::TimesResolvedThisTurn, Cmp::Le, _)
        ));
        assert!(matches!(
            o("you attacked with creatures with total power 6 or greater this combat"),
            Condition::Compare(
                Value::Aggregate(AggOp::Sum, Stat::Power, _),
                Cmp::Ge,
                Value::Const(6)
            )
        ));
    }
}
