//! "[Effect] unless [a player] [does something]" (CR 118.12a: the same as "[that player]
//! may [do something]. If they don't, [effect]"), the punisher and tax family:
//!
//! * the player: "you", "they" / "that player" (the player the text is about), "its
//!   controller" / "that creature's controller" / "that spell's controller" (the controller
//!   of what the effect is about), "target opponent", "any player" / "a player" (each
//!   player in turn may, until one does), "an opponent";
//! * the action, from that player's point of view (their permanents, their hand): paying
//!   ("pays {2}", "pays 3 life", "pays {1} or 1 life", "pays {2} and 2 life", "pays {X}",
//!   "pays life equal to its toughness", "pays mana equal to [value]"), sacrificing ("a
//!   nonland permanent of their choice", "that artifact", "it"), discarding ("a card",
//!   "their hand"), exiling ("all cards from their graveyard", "a historic card from your
//!   graveyard"), returning ("a land you control to its owner's hand", "a basic land card
//!   from your graveyard to your hand"), removing counters, "has ~ deal 6 damage to
//!   them", or any other cost the cost grammar reads;
//! * several actions the player chooses among: "unless that player sacrifices a nonland
//!   permanent of their choice or discards a card" (the player may do one of them; if they
//!   do neither, the effect happens).
//!
//! An action is a cost paid as the spell or ability resolves (CR 118.12): a player can't
//! choose an action they can't perform in full (CR 118.3), and the effect happens if they
//! don't perform one. "Any player" may pay: each player in turn gets the option until one
//! does.
//!
//! Also "[effect] unless [a condition about a referent]": "sacrifice it unless {U} was
//! spent to cast it", "~ deals 2 damage to that player unless they control two or more
//! basic lands" — the effect happens if the condition, checked as it resolves, is false.

use super::counters_resources_pay::resolution_cost;
use super::oracle_hardening_referents::{is_no_player_referent, is_no_referent};
use super::EffectPattern;
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::effects::{object_ref, parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;

/// One action the player may perform instead of suffering the effect: a cost, and the
/// value of X it uses when it's "mana equal to [value]".
struct Payment {
    cost: Cost,
    x: Option<Value>,
}

/// Verbs an action may start with (base form).
const VERBS: &[&str] = &[
    "pay", "sacrifice", "discard", "exile", "return", "remove", "put", "have", "tap",
    "waterbend", "reveal", "mill",
];

/// The base form of an action verb, third-person ("pays", "has") or base ("pay").
fn verb_base(w: &str) -> Option<&'static str> {
    let cand = match w {
        "has" => "have".to_string(),
        _ => w.strip_suffix('s').unwrap_or(w).to_string(),
    };
    VERBS.iter().copied().find(|v| *v == cand || *v == w)
}

/// The action worded from the acting player's point of view: the verb in the base form
/// and "their"/"they"/"them" as "your"/"you".
fn as_actor(alt: &str) -> Option<String> {
    let (w, rest) = split_word(alt.trim());
    let verb = verb_base(w)?;
    let mut out = vec![verb.to_string()];
    for word in rest.split(' ') {
        let (core, punct) = match word.find([',', '.']) {
            Some(at) => (&word[..at], &word[at..]),
            None => (word, ""),
        };
        let r = match core {
            "their" => "your",
            "they" => "you",
            "them" => "you",
            "theirs" => "yours",
            "themselves" | "themself" => "yourself",
            // A character card's pronouns are the card itself ("has ~ deal damage to them
            // equal to his power").
            "his" | "her" => "~'s",
            // An object's owner or controller stays.
            c => c,
        };
        out.push(format!("{r}{punct}"));
    }
    Some(out.join(" ").trim().to_string())
}

/// The action text split into the alternatives the player chooses among: "sacrifices a
/// creature or planeswalker of their choice or discards a card" → two (an "or" starts an
/// alternative only before a verb).
fn split_alternatives(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let words: Vec<&str> = s.split(' ').collect();
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        if w == "or" && i + 1 < words.len() && !cur.is_empty() && verb_base(words[i + 1]).is_some()
        {
            out.push(cur.trim().to_string());
            cur = String::new();
            i += 1;
            continue;
        }
        cur.push_str(w);
        cur.push(' ');
        i += 1;
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// Whether a value is read the same from any player's point of view (it doesn't mention
/// "you", which in a cost paid by another player would mean that player).
fn impersonal(v: &Value) -> bool {
    !serde_json::to_string(v).unwrap_or_default().contains("\"You\"")
}

/// "{2}", "3 life", "{1} or 1 life", "{2} and 2 life", "{x}", "life equal to [value]",
/// "mana equal to [value]" (after "pay").
fn pay_alternatives(r: &str, b: &mut Builder) -> Option<Vec<Payment>> {
    let r = end(r);
    if r == "{x}" {
        // The X of the spell (CR 107.3): only a spell's own X is known here.
        if !b.ctx.is_spell() {
            return None;
        }
        return Some(vec![Payment {
            cost: Cost::mana(ManaCost::parse("{X}")?),
            x: None,
        }]);
    }
    if let Some(v) = r.strip_prefix("life equal to ") {
        let (value, rest) = parse_value_phrase(v, b)?;
        if !end(&rest).is_empty() || !impersonal(&value) {
            return None;
        }
        return Some(vec![Payment {
            cost: Cost::default().with(CostPart::PayLife(value)),
            x: None,
        }]);
    }
    if let Some(v) = r.strip_prefix("mana equal to ") {
        let (value, rest) = parse_value_phrase(v, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        return Some(vec![Payment {
            cost: Cost::mana(ManaCost::parse("{X}")?),
            x: Some(value),
        }]);
    }
    if let Some((a, c)) = r.split_once(" or ") {
        let mut out = pay_alternatives(a, b)?;
        out.extend(pay_alternatives(c, b)?);
        return Some(out);
    }
    if let Some((a, c)) = r.split_once(" and ") {
        let a = resolution_cost(a)?;
        let c = resolution_cost(c)?;
        let mut cost = a;
        if let Some(m) = c.mana {
            match cost.mana.as_mut() {
                Some(t) => t.add(&m),
                None => cost.mana = Some(m),
            }
        }
        cost.parts.extend(c.parts);
        return Some(vec![Payment { cost, x: None }]);
    }
    Some(vec![Payment {
        cost: resolution_cost(r)?,
        x: None,
    }])
}

/// Whether the engine can tell if an action performed as a cost can be performed in full
/// (CR 118.3; see `cost_effects`).
fn known_payability(e: &Effect) -> bool {
    match e {
        // Always possible.
        Effect::DealDamage { .. } => true,
        Effect::Exile {
            what: Sel::All(_), ..
        } => true,
        // Needs the objects (`cost_effects`).
        Effect::SacrificeObjects { .. } => true,
        Effect::Move {
            what: Sel::Choose { up_to: false, .. },
            ..
        } => true,
        Effect::Exile {
            what: Sel::Choose { up_to: false, .. },
            ..
        } => true,
        _ => false,
    }
}

/// One action (worded for the actor, base form) as a payment.
fn action_payment(a: &str, b: &mut Builder) -> Option<Vec<Payment>> {
    let a = end(a);
    if let Some(r) = a.strip_prefix("pay ") {
        return pay_alternatives(r, b);
    }
    // "of their choice" is how the player who sacrifices or discards chooses anyway.
    let a = a.strip_suffix(" of your choice").unwrap_or(a).trim();
    // "remove two oil counters from it" where "it" is ~.
    let a = if matches!(b.it, Sel::This) {
        a.split(' ')
            .map(|w| if w == "it" { "~" } else { w })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        a.to_string()
    };
    let single = |cost: Cost| Some(vec![Payment { cost, x: None }]);
    // "have ~ deal 6 damage to you" (to the player who chooses it).
    if let Some(r) = a.strip_prefix("have ~ deal ") {
        let e = parse_clause(&format!("~ deals {r}"), b)?;
        if !matches!(&e, Effect::DealDamage { source: Sel::This, to: Sel::Players(PlayerRef::You), .. })
        {
            return None;
        }
        return single(Cost::default().with(CostPart::Effect(Box::new(e))));
    }
    // "sacrifice it", "sacrifice that artifact": that object, if they control it.
    if let Some(r) = a.strip_prefix("sacrifice ") {
        if parse_number(r).is_none() && !r.starts_with("another ") {
            let saved = (b.targets.len(), b.it.clone());
            if let Some((what, rest)) = object_ref(r, b) {
                if end(&rest).is_empty() && b.targets.len() == saved.0 && !is_no_referent(&what)
                {
                    if matches!(what, Sel::This) {
                        return single(Cost::default().with(CostPart::SacrificeSelf));
                    }
                    let e = Effect::SacrificeObjects { what };
                    return single(Cost::default().with(CostPart::Effect(Box::new(e))));
                }
            }
            b.targets.truncate(saved.0);
            b.it = saved.1;
        }
    }
    // "return another creature you control to its owner's hand".
    if let Some(r) = a.strip_prefix("return another ") {
        if let Some((cost, _)) = crate::oracle::costs::parse_cost(&format!("return a {r}")) {
            if let [CostPart::ReturnToHand { filter, count }] = cost.parts.as_slice() {
                if cost.mana.is_none() {
                    let part = CostPart::ReturnToHand {
                        filter: Filter::and(vec![filter.clone(), Filter::Other]),
                        count: count.clone(),
                    };
                    return single(Cost::default().with(part));
                }
            }
        }
        return None;
    }
    if let Some((cost, loyalty)) = crate::oracle::costs::parse_cost(&a) {
        let ok = !loyalty
            && cost.mana.is_none()
            && !cost.parts.is_empty()
            && cost.parts.iter().all(|p| match p {
                CostPart::Effect(e) => {
                    matches!(&**e, Effect::KeywordAction { .. }) || known_payability(e)
                }
                CostPart::Tap | CostPart::Untap | CostPart::Loyalty(_) => false,
                _ => true,
            });
        if ok {
            return single(cost);
        }
        return None;
    }
    // An instruction performed as a cost: "exile all cards from your graveyard", "return a
    // basic land card from your graveyard to your hand".
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let e = parse_clause(&a, b);
    let new_targets = b.targets.len() != saved.0;
    b.targets.truncate(saved.0);
    b.it = saved.1;
    b.it_player = saved.2;
    let e = e?;
    if new_targets || !known_payability(&e) {
        return None;
    }
    single(Cost::default().with(CostPart::Effect(Box::new(e))))
}

/// "Another" in an action about an object the text names ("~ deals damage equal to that
/// creature's power to that player unless they sacrifice another creature", where ~ is
/// an Aura): other than that object, not other than the source.
fn other_than_referent(cost: Cost, it: &Sel) -> Option<Cost> {
    if !matches!(it, Sel::AttachedTo | Sel::Target(_)) {
        return Some(cost);
    }
    let json = serde_json::to_value(&cost).ok()?;
    let not_it = serde_json::to_value(Filter::not(Filter::In(Box::new(it.clone())))).ok()?;
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

/// The alternatives the player chooses among.
fn payments(action: &str, b: &mut Builder) -> Option<Vec<Payment>> {
    let mut out = Vec::new();
    for alt in split_alternatives(action) {
        let worded = as_actor(&alt)?;
        for p in action_payment(&worded, b)? {
            out.push(Payment {
                cost: other_than_referent(p.cost, &b.it)?,
                x: p.x,
            });
        }
    }
    (!out.is_empty()).then_some(out)
}

/// "[otherwise] unless [who] [one of the payments]".
fn unless_paid(who: PlayerRef, pays: Vec<Payment>, otherwise: Effect) -> Effect {
    let mut e = otherwise;
    for p in pays.into_iter().rev() {
        let pay = Effect::PayOptional {
            who: who.clone(),
            cost: p.cost,
            then: Box::new(Effect::Noop),
            otherwise: Box::new(e),
        };
        e = match p.x {
            Some(value) => Effect::seq(vec![Effect::SetX { value }, pay]),
            None => pay,
        };
    }
    e
}

/// A player subject at the start of an instruction ("each opponent loses ...", "target
/// opponent discards ..."): the player grammar (`player_subjects`) words the rest for that
/// player, so "they" in the action is that player.
fn has_player_subject(eff: &str) -> bool {
    const SUBJECTS: &[&str] = &[
        "each player ",
        "each opponent ",
        "each other player ",
        "target player ",
        "target opponent ",
        "that player ",
        "that opponent ",
        "enchanted player ",
        "its controller ",
        "defending player ",
        "you ",
    ];
    SUBJECTS.iter().any(|s| eff.starts_with(s))
}

/// The player phrase after "unless" and the rest (the action).
enum PayerPhrase<'a> {
    /// "they", "that player".
    Pronoun(&'a str),
    /// "its controller", "that creature's controller".
    ControllerOfIt(&'a str),
    /// A player phrase `player_ref` reads ("you", "target opponent", "defending player").
    Phrase(&'a str, &'a str),
    /// "any player", "a player": any one of them may (each in turn until one does).
    Any(PlayerRef, &'a str),
}

fn payer_phrase(r: &str) -> Option<PayerPhrase<'_>> {
    for p in ["they ", "that player ", "that opponent "] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some(PayerPhrase::Pronoun(rest));
        }
    }
    for p in [
        "its controller ",
        "that creature's controller ",
        "that spell's controller ",
        "that permanent's controller ",
        "that land's controller ",
        "that artifact's controller ",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some(PayerPhrase::ControllerOfIt(rest));
        }
    }
    for (p, who) in [
        ("any player ", PlayerRef::EachPlayer),
        ("a player ", PlayerRef::EachPlayer),
        ("any opponent ", PlayerRef::EachOpponent),
        ("an opponent ", PlayerRef::EachOpponent),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some(PayerPhrase::Any(who, rest));
        }
    }
    for p in ["you ", "target opponent ", "target player ", "defending player "] {
        if let Some(rest) = r.strip_prefix(p) {
            return Some(PayerPhrase::Phrase(&p[..p.len() - 1], rest));
        }
    }
    None
}

/// The player a pronoun ("they", "that player") in the "unless" part refers to: the one
/// player target the effect introduced, the controller the effect mentions ("... deals
/// damage to its controller unless that player sacrifices it"), or what "that player"
/// means.
fn pronoun_player(eff: &str, b: &Builder, first_new: usize) -> Option<PlayerRef> {
    let new_players: Vec<u8> = (first_new..b.targets.len())
        .filter(|i| matches!(b.targets[*i].what, TargetKind::Player(_)))
        .map(|i| i as u8)
        .collect();
    let mentions_controller = eff.ends_with(" its controller")
        || eff.contains(" to its controller ")
        || eff.starts_with("its controller ");
    match new_players.as_slice() {
        [one] => Some(PlayerRef::Target(*one)),
        [] if mentions_controller && !is_no_referent(&b.it) && !matches!(b.it, Sel::This) => {
            Some(PlayerRef::ControllerOf(Box::new(b.it.clone())))
        }
        [] if !is_no_player_referent(&b.it_player) => Some(b.it_player.clone()),
        _ => None,
    }
}

/// "[effect] unless [player] [action]".
fn split_unless(eff: &str, rest: &str, b: &mut Builder) -> Option<Effect> {
    let phrase = payer_phrase(rest)?;
    // "Each opponent loses 3 life unless that player sacrifices ...": the player grammar
    // performs the instruction as each of them; "they" in its action is that player.
    if let PayerPhrase::Pronoun(action) = phrase {
        if has_player_subject(eff) && !eff.starts_with("you ") {
            if rest.starts_with("they ") {
                return None;
            }
            return parse_clause(&format!("{eff} unless they {action}"), b);
        }
    }
    let first_new = b.targets.len();
    let effect = parse_clause(eff, b)?;
    let (who, action) = match phrase {
        PayerPhrase::Pronoun(action) => (pronoun_player(eff, b, first_new)?, action),
        PayerPhrase::ControllerOfIt(action) => {
            if is_no_referent(&b.it) || matches!(b.it, Sel::This) {
                return None;
            }
            (PlayerRef::ControllerOf(Box::new(b.it.clone())), action)
        }
        PayerPhrase::Any(who, action) => (who, action),
        PayerPhrase::Phrase(p, action) => {
            let (who, tail) = player_ref(p, b)?;
            if !tail.trim().is_empty() {
                return None;
            }
            (who, action)
        }
    };
    let pays = payments(action, b)?;
    Some(unless_paid(who, pays, effect))
}

fn unless_player_does(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "Unless any player pays {1}, search your library for a card ...".
    if let Some(r) = l.strip_prefix("unless ") {
        let (cond, eff) = r.split_once(", ")?;
        let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
        let effect = parse_clause(eff, b);
        let r = effect.and_then(|effect| {
            let (who, action) = match payer_phrase(&format!("{cond} "))? {
                PayerPhrase::Any(who, action) => (who, action.trim().to_string()),
                _ => return None,
            };
            let pays = payments(&action, b)?;
            Some(unless_paid(who, pays, effect))
        });
        if r.is_none() {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
        }
        return r;
    }
    let splits: Vec<usize> = l.match_indices(" unless ").map(|(i, _)| i).collect();
    for i in splits.into_iter().rev() {
        let (eff, rest) = (&l[..i], &l[i + " unless ".len()..]);
        if eff.contains(" unless ") || eff.ends_with(',') {
            continue;
        }
        let saved = (
            b.targets.clone(),
            b.it.clone(),
            b.it_player.clone(),
            b.group.clone(),
        );
        if let Some(e) = split_unless(eff, rest, b) {
            return Some(e);
        }
        (b.targets, b.it, b.it_player, b.group) = saved;
    }
    None
}

inventory::submit! { EffectPattern { name: "unless grammar: [effect] unless [player] [action]", priority: 850, parse: unless_player_does } }

/// A condition after "unless" with "they" as its subject, worded with "that player" for
/// the referent condition grammar: "they control two or more basic lands" → "that player
/// controls two or more basic lands".
fn they_as_that_player(c: &str) -> String {
    let Some(r) = c.strip_prefix("they ") else {
        return c.to_string();
    };
    let (w, rest) = split_word(r);
    let verb = match w {
        "have" => "has".to_string(),
        "control" | "own" => format!("{w}s"),
        "are" => "is".to_string(),
        "were" => "was".to_string(),
        other => other.to_string(),
    };
    format!("that player {verb} {rest}")
        .trim()
        .replace(" their ", " that player's ")
}

/// "[effect] unless [condition]" where the condition refers to what the text is about:
/// "sacrifice it unless {U} was spent to cast it", "~ deals 2 damage to that player
/// unless they control two or more basic lands", "~ deals 1 damage to that player unless
/// that creature attacked this turn". The effect happens if the condition, checked as it
/// would happen, is false.
fn unless_state(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (eff, c) = l.rsplit_once(" unless ")?;
    if eff.contains(" unless ") || eff.ends_with(',') || eff.is_empty() {
        return None;
    }
    let c = they_as_that_player(c);
    let saved = (
        b.targets.clone(),
        b.it.clone(),
        b.it_player.clone(),
        b.group.clone(),
    );
    let r = (|| {
        let effect = parse_clause(eff, b)?;
        let cond = crate::oracle::patterns::conditions_referents::parse_condition_with(&c, b)?;
        Some(Effect::If {
            cond: Condition::Not(Box::new(cond)),
            then: Box::new(effect),
            otherwise: Box::new(Effect::Noop),
        })
    })();
    if r.is_none() {
        (b.targets, b.it, b.it_player, b.group) = saved;
    }
    r
}

inventory::submit! { EffectPattern { name: "unless grammar: [effect] unless [condition about a referent]", priority: 860, parse: unless_state } }
