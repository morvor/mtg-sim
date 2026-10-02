//! Choosing objects as an instruction of its own (CR 608.2d; not targeted, CR 115.10),
//! by you or by other players, whose result later sentences refer to:
//!
//! - "Choose up to one creature. Destroy the rest." / "Choose five permanents you
//!   control." / "Choose exactly two creatures you control." / "Choose a creature card
//!   exiled with ~." / "choose a card in your hand";
//! - "Each opponent chooses a creature card in their graveyard. Put those cards onto the
//!   battlefield under your control." / "Each player chooses six lands they control.
//!   Destroy all other permanents." / "Target opponent chooses X cards from their hand." /
//!   "Defending player chooses a nonland card in your graveyard." (each of those players
//!   chooses in APNAP order, CR 101.4);
//! - "An opponent chooses one of them." (the opponent is chosen as CR 801.5a describes).
//!
//! The chosen objects are "the chosen [noun]s", "those [noun]s", "that [noun]", "it" and
//! "them"; what wasn't chosen is "the rest", "all other [objects]" or "[objects] not
//! chosen this way".

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, player_ref, Builder};
use crate::oracle::patterns::hand_graveyard_grammar::{cards, Qty};
use crate::oracle::phrases::*;

/// The objects chosen.
pub const CHOSEN: Var = vars::USER + 4430;
/// The opponent who makes a choice "an opponent" makes (CR 801.5a).
const DECIDER: Var = vars::USER + 4431;
/// Marks, in [`Builder::named`], what the choice chose among (for "the rest"): the
/// selection is all such objects.
const UNIVERSE: &str = "\u{1}choice grammar: chosen among";

/// Who chooses: the chooser, and the players each of whom chooses (for "each opponent
/// chooses"), with the rest after the verb.
struct Chooser {
    chooser: PlayerRef,
    each: Option<PlayerRef>,
    pre: Option<Effect>,
    rest: String,
}

fn chooser(l: &str, b: &mut Builder) -> Option<Chooser> {
    let simple = |who: PlayerRef, rest: &str| Chooser {
        chooser: who,
        each: None,
        pre: None,
        rest: rest.to_string(),
    };
    if let Some(r) = l.strip_prefix("choose ").or_else(|| l.strip_prefix("you choose ")) {
        return Some(simple(PlayerRef::You, r));
    }
    if let Some(r) = l.strip_prefix("an opponent chooses ") {
        let pre = crate::kw::choice_grammar::deciding_opponent_effect(DECIDER)?;
        return Some(Chooser {
            chooser: PlayerRef::Var(DECIDER),
            each: None,
            pre: Some(pre),
            rest: r.to_string(),
        });
    }
    let (who, rest) = player_ref(l, b)?;
    let r = rest.trim_start().strip_prefix("chooses ")?;
    if matches!(who, PlayerRef::You) {
        return None;
    }
    let each = matches!(
        who,
        PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer | PlayerRef::Each(_)
    );
    Some(if each {
        Chooser {
            chooser: PlayerRef::Iterated,
            each: Some(who),
            pre: None,
            rest: r.to_string(),
        }
    } else {
        simple(who, r)
    })
}

/// The quantity of a choice: (count, up to?, "any number"?) and the rest.
fn quantity(s: &str) -> Option<(Value, bool, bool, &str)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("any number of ") {
        return Some((Value::c(0), true, true, r));
    }
    if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        return Some((n, true, false, r));
    }
    let s = s.strip_prefix("exactly ").unwrap_or(s);
    let (n, r) = parse_number(s)?;
    Some((n, false, false, r))
}

/// The head noun of a phrase, for naming the chosen objects afterward ("creature",
/// "card", "land", "permanent").
fn head_word(phrase: &str) -> &'static str {
    let words: Vec<&str> = phrase
        .split([' ', ','])
        .filter(|w| !w.is_empty())
        .collect();
    for w in words.iter() {
        let w = w.trim_end_matches('s');
        match w {
            "card" => return "card",
            "creature" if !words.contains(&"card") && !words.contains(&"cards") => {
                return "creature"
            }
            "land" if !words.contains(&"card") && !words.contains(&"cards") => return "land",
            "permanent" => return "permanent",
            "artifact" if !words.iter().any(|x| x.starts_with("card") || x.starts_with("creature")) => {
                return "artifact"
            }
            _ => {}
        }
    }
    if words.iter().any(|w| w.starts_with("card")) {
        "card"
    } else {
        "permanent"
    }
}

/// What may be chosen: the filter (with its zone), the head noun, and the rest.
fn chosen_kind(
    s: &str,
    c: &Chooser,
    b: &mut Builder,
) -> Option<(Filter, &'static str, String)> {
    let subject = c.each.clone().unwrap_or_else(|| c.chooser.clone());
    // "one of them", "two of those cards", "one of the exiled cards".
    for p in [
        "of them",
        "of those cards",
        "of the exiled cards",
        "of those creatures",
        "of the revealed cards",
    ] {
        if let Some(r) = s.trim_start().strip_prefix(p) {
            let them = match super::pronoun_groups::plural_object_ref("them", b) {
                Some(Some((sel, _))) => sel,
                _ => return None,
            };
            return Some((Filter::In(Box::new(them)), "card", r.to_string()));
        }
    }
    // Cards in a zone ("a creature card in their graveyard", "a card in your hand", "X
    // cards from their hand").
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let probe = format!("a {}", s.trim_start());
    if let Some((cs, rest)) = cards(&probe, b, Some(&subject)) {
        // Cards you choose are the zone-move grammar's (`zone_move_grammar::p_choose_card`).
        if cs.zone.is_some() && matches!(cs.qty, Qty::Exactly(_)) && !matches!(c.chooser, PlayerRef::You) {
            let word = head_word(&s[..s.len() - rest.trim_start().len().min(s.len())]);
            let filter = if c.each.is_some() {
                cs.filter
            } else {
                cs.filter
            };
            return Some((filter, if word == "permanent" { "card" } else { word }, rest));
        }
    }
    b.targets.truncate(saved.0);
    (b.it, b.it_player) = (saved.1, saved.2);
    // Permanents ("a creature they control", "six lands they control", "creatures target
    // player controls", "up to one creature").
    let (f, _, tail) = parse_object_phrase(s)?;
    if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    let text = &s[..s.len() - tail.len()];
    let word = head_word(text);
    let t = tail.trim_start();
    let (control, rest) = if let Some(r) = t.strip_prefix("they control") {
        let rel = match &c.each {
            Some(_) => Filter::ControlledBy(PlayerRel::Iterated),
            None => Filter::ControlledByPlayer(Box::new(c.chooser.clone())),
        };
        (Some(rel), r.to_string())
    } else {
        let (f2, r) = crate::oracle::effects::bind_target_player(Filter::Any, t, b);
        match f2 {
            Filter::Any => (None, r),
            f2 => (Some(f2), r),
        }
    };
    let mut parts = vec![f, Filter::InZone(ZoneKind::Battlefield)];
    parts.extend(control);
    Some((Filter::and(parts), word, rest))
}

fn choose_objects(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone(), b.named.len());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
        b.named.truncate(saved.3);
    };
    let r = (|| {
        let c = chooser(l, b)?;
        // Choices of words, types and players are other patterns'.
        if c.rest.contains(" at random") {
            return None;
        }
        let (count, up_to, any_number, r) = quantity(&c.rest)?;
        let (filter, word, rest) = chosen_kind(r, &c, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        Some((c, count, up_to, any_number, filter, word))
    })();
    let Some((c, count, up_to, any_number, filter, word)) = r else {
        restore(b);
        return None;
    };
    let count = if any_number {
        Value::CountSel(Box::new(Sel::All(filter.clone())))
    } else {
        count
    };
    let choose = Sel::Choose {
        chooser: c.chooser.clone(),
        filter: filter.clone(),
        count,
        up_to,
        store: None,
    };
    let mut out = Vec::new();
    out.extend(c.pre);
    match &c.each {
        Some(who) => {
            out.push(Effect::Store {
                var: CHOSEN,
                sel: Sel::Union(vec![]),
            });
            out.push(Effect::ForEachPlayer {
                who: who.clone(),
                effect: Box::new(Effect::Store {
                    var: CHOSEN,
                    sel: Sel::Union(vec![Sel::Var(CHOSEN), choose]),
                }),
            });
        }
        None => out.push(Effect::Store {
            var: CHOSEN,
            sel: choose,
        }),
    }
    // What it was chosen among, for "the rest" (each player's own, for "each player
    // chooses a creature they control": all such creatures).
    let universe = match &c.each {
        Some(_) => without_iterated(filter),
        None => filter,
    };
    let chosen = Sel::Var(CHOSEN);
    b.named.push((UNIVERSE.into(), Sel::All(universe.clone())));
    b.named.push((
        "the rest".into(),
        Sel::All(Filter::and(vec![
            universe,
            Filter::not(Filter::In(Box::new(chosen.clone()))),
        ])),
    ));
    let plural = format!("{word}s");
    for name in [
        format!("the chosen {plural}"),
        format!("the chosen {word}"),
        format!("those {plural}"),
        format!("that {word}"),
        "the chosen cards".to_string(),
        "the chosen permanents".to_string(),
    ] {
        b.named.push((name, chosen.clone()));
    }
    b.it = chosen;
    Some(Effect::seq(out))
}

inventory::submit! { EffectPattern { name: "choice grammar: [player] chooses [objects]", priority: 115, parse: choose_objects } }

/// The filter with "controlled by the player choosing" / "owned by" removed (each
/// player's own choice among all such objects).
fn without_iterated(f: Filter) -> Filter {
    let mine = |x: &Filter| {
        matches!(
            x,
            Filter::ControlledBy(PlayerRel::Iterated) | Filter::OwnedBy(PlayerRel::Iterated)
        )
    };
    match f {
        Filter::And(v) => Filter::and(v.into_iter().filter(|x| !mine(x)).collect()),
        f => f,
    }
}

/// Whether the effect ends with a choice of objects by [`choose_objects`].
fn ends_with_choice(e: &Effect) -> bool {
    match e {
        Effect::Store { var, .. } => *var == CHOSEN,
        Effect::ForEachPlayer { effect, .. } => ends_with_choice(effect),
        Effect::Seq(v) => v.last().is_some_and(ends_with_choice),
        _ => false,
    }
}

/// Adds "not one of the chosen objects" to each `Sel::All` selection of the effect's
/// main instruction ("Destroy all other permanents.", "Return all nonland permanents not
/// chosen this way to their owners' hands."). Returns whether any was changed.
fn exclude_chosen(e: &mut Effect, drop_other: bool) -> bool {
    let not_chosen = || Filter::not(Filter::In(Box::new(Sel::Var(CHOSEN))));
    let fix = |f: &mut Filter| {
        if drop_other {
            if let Filter::And(v) = f {
                v.retain(|x| !matches!(x, Filter::Other));
            } else if matches!(f, Filter::Other) {
                *f = Filter::Any;
            }
        }
        let old = std::mem::replace(f, Filter::Any);
        *f = Filter::and(vec![old, not_chosen()]);
    };
    match e {
        Effect::Destroy {
            what: Sel::All(f), ..
        }
        | Effect::Exile {
            what: Sel::All(f), ..
        }
        | Effect::Move {
            what: Sel::All(f), ..
        }
        | Effect::Tap { what: Sel::All(f) }
        | Effect::PhaseOut { what: Sel::All(f), .. }
        | Effect::SacrificeObjects { what: Sel::All(f) } => {
            fix(f);
            true
        }
        Effect::Seq(v) => {
            let mut any = false;
            for x in v {
                any |= exclude_chosen(x, drop_other);
            }
            any
        }
        _ => false,
    }
}

/// "Destroy all other permanents." / "Return all nonland permanents not chosen this way to
/// their owners' hands." / "Exile all other creatures." after a choice of objects.
fn others_than_chosen(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_choice(prev) {
        return false;
    }
    let l = end(l);
    let (text, drop_other) = if l.contains(" not chosen this way") {
        (l.replacen(" not chosen this way", "", 1), false)
    } else if l.contains(" all other ") {
        (l.to_string(), true)
    } else {
        return false;
    };
    let Some(mut e) = parse_sentence(&text, b) else {
        return false;
    };
    if !exclude_chosen(&mut e, drop_other) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "choice grammar: all other / not chosen this way", priority: 55, apply: others_than_chosen } }

/// Replaces "other" (than the source) filters in `e` with "not one of the chosen
/// objects". Returns `None` if there were none.
fn other_means_not_chosen(e: &Effect) -> Option<Effect> {
    fn walk(v: &mut serde_json::Value, with: &serde_json::Value, n: &mut usize) {
        match v {
            serde_json::Value::String(s) if s == "Other" => {
                *v = with.clone();
                *n += 1;
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, with, n)),
            serde_json::Value::Object(m) => {
                for (k, x) in m.iter_mut() {
                    // Not a filter: keyword actions and trigger batching.
                    if k != "action" && k != "per" {
                        walk(x, with, n);
                    }
                }
            }
            _ => {}
        }
    }
    let mut v = serde_json::to_value(e).ok()?;
    let with = serde_json::to_value(Filter::not(Filter::In(Box::new(Sel::Var(CHOSEN))))).ok()?;
    let mut n = 0;
    walk(&mut v, &with, &mut n);
    if n == 0 {
        return None;
    }
    serde_json::from_value(v).ok()
}

/// "Other creatures they control can't block this turn." after "Target opponent chooses a
/// creature they control": other than the chosen creature, not than the source.
fn other_than_the_chosen(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !ends_with_choice(prev) || !end(l).starts_with("other ") {
        return false;
    }
    let Some(e) = parse_sentence(end(l), b) else {
        return false;
    };
    let Some(e) = other_means_not_chosen(&e) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "choice grammar: other [objects] (than the chosen)", priority: 55, apply: other_than_the_chosen } }
