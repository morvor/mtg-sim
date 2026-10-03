//! Choices made at random (nobody chooses; each possibility is equally likely):
//!
//! - players: "choose an opponent at random", "choose a player at random" (then "that
//!   player");
//! - objects as the effect happens ([`Sel::AtRandom`]): "exile a card at random from your
//!   graveyard", "exile three cards at random from your graveyard", "mill a card, then
//!   exile a card from your graveyard at random", "choose a card at random in your
//!   graveyard", "choose a creature at random", "it deals 7 damage to a creature an
//!   opponent controls chosen at random", "destroy one of them at random";
//! - targets ([`TargetSpec::random`]): "~ deals 2 damage to any target chosen at random",
//!   "it fights target creature an opponent controls chosen at random" (the target is
//!   chosen at random among the legal ones as the ability is put on the stack);
//! - numbers and words: "choose 1, 2, or 3 at random" (then "that many"), "choose flying
//!   or indestructible at random" (then "that ability").

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::kw::choice_grammar::{random_number_effect, random_player_effect};
use crate::oracle::effects::{parse_clause, parse_sentence, player_ref, Builder};
use crate::oracle::patterns::hand_graveyard_grammar::{cards, Qty};
use crate::oracle::phrases::*;

/// The player chosen at random.
pub const RANDOM_PLAYER: Var = vars::USER + 4420;
/// The objects chosen at random by a "choose ... at random" sentence.
const RANDOM_OBJECTS: Var = vars::USER + 4421;
/// The number or word chosen at random (its index, from 1).
pub const RANDOM_NUMBER: Var = vars::USER + 4422;
/// The object an instruction describes as "chosen at random" (not targeted).
const RANDOM_OBJECT: Var = vars::USER + 4423;

/// "choose an opponent at random", "choose a player at random", "choose another player
/// at random".
fn choose_player_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let from = match r {
        "an opponent at random" => PlayerRef::EachOpponent,
        "a player at random" => PlayerRef::EachPlayer,
        "another player at random" => PlayerRef::EachOtherPlayer,
        _ => return None,
    };
    b.it_player = PlayerRef::Var(RANDOM_PLAYER);
    b.named.push((
        "the chosen player".into(),
        Sel::Players(PlayerRef::Var(RANDOM_PLAYER)),
    ));
    random_player_effect(RANDOM_PLAYER, from)
}

inventory::submit! { EffectPattern { name: "choice grammar: choose a player at random", priority: 80, parse: choose_player_at_random } }

/// A card phrase with "at random" in it ("a card at random from your graveyard", "a card
/// from your graveyard at random", "three cards at random from your graveyard"): the
/// selection of that many cards at random, and the rest of the text.
fn random_cards(s: &str, b: &mut Builder, subject: &PlayerRef) -> Option<(Sel, String)> {
    let s = s.trim_start();
    let k = s.find(" at random")?;
    let without = format!("{}{}", &s[..k], &s[k + " at random".len()..]);
    let (c, rest) = cards(&without, b, Some(subject))?;
    let Qty::Exactly(n) = c.qty else {
        return None;
    };
    // The phrase names its zone ("from your graveyard", "from their hand").
    c.zone?;
    if c.any_owner {
        return None;
    }
    Some((
        Sel::AtRandom {
            filter: c.filter,
            count: n,
            store: None,
        },
        rest,
    ))
}

/// "[player] exile(s) [cards] at random [from zone]".
fn exile_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" at random") {
        return None;
    }
    let (who, r) = if let Some(r) = l.strip_prefix("exile ") {
        (PlayerRef::You, r.to_string())
    } else {
        let (who, rest) = player_ref(l, b)?;
        (who, rest.trim_start().strip_prefix("exiles ")?.to_string())
    };
    let (what, rest) = random_cards(&r, b, &who)?;
    if !end(&rest).is_empty() {
        return None;
    }
    b.it = Sel::Var(vars::IT);
    Some(Effect::Exile {
        what,
        face_down: false,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "choice grammar: exile cards at random", priority: 80, parse: exile_at_random } }

/// "choose a creature at random", "choose a creature at random that attacked this turn":
/// the permanents are chosen at random as the effect happens; "it" / "that creature"
/// refer to them.
fn choose_objects_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let k = r.find(" at random")?;
    let without = format!("{}{}", &r[..k], &r[k + " at random".len()..]);
    // Permanents: "a creature", "a creature that attacked this turn". (Cards in a zone
    // chosen at random are the zone-move grammar's, `zone_move_grammar::p_choose_card`.)
    if without.split(' ').any(|w| w == "card" || w == "cards") {
        return None;
    }
    let (n, r2) = parse_number(&without)?;
    let (f, _, tail) = parse_object_phrase(r2)?;
    if !end(tail).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    let sel = Sel::AtRandom {
        filter: Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
        count: n,
        store: Some(RANDOM_OBJECTS),
    };
    b.it = Sel::Var(RANDOM_OBJECTS);
    for p in ["that card", "that creature", "the chosen card", "the chosen creature"] {
        b.named.push((p.into(), Sel::Var(RANDOM_OBJECTS)));
    }
    Some(Effect::Store {
        var: RANDOM_OBJECTS,
        sel,
    })
}

inventory::submit! { EffectPattern { name: "choice grammar: choose objects at random", priority: 80, parse: choose_objects_at_random } }

/// "... target [object] chosen at random", "... any target chosen at random", "... target
/// opponent chosen at random": the target is chosen at random among the legal ones as the
/// spell or ability is put on the stack. "... a creature an opponent controls chosen at
/// random": not a target; the creature is chosen at random as the effect happens.
fn chosen_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let k = l.find(" chosen at random")?;
    let after = &l[k + " chosen at random".len()..];
    if after.contains(" chosen at random") {
        return None;
    }
    let before = &l[..k];
    // A target: the sentence without "chosen at random", its new target made random.
    let target_at = ["target ", "any target"]
        .iter()
        .filter_map(|p| before.rfind(p))
        .max();
    // The article starting the object phrase, not one in a qualifier ("a creature an
    // opponent controls").
    let article_at = [" a ", " an "]
        .iter()
        .flat_map(|p| before.match_indices(p).map(|(i, m)| (i, m.len())))
        .filter(|(i, len)| {
            let next = &before[i + len..];
            !next.starts_with("opponent") && !next.starts_with("player")
        })
        .map(|(i, _)| i)
        .max();
    if let Some(t) = target_at.filter(|t| article_at.is_none_or(|a| a < *t)) {
        let _ = t;
        let text = format!("{before}{after}");
        let first = b.targets.len();
        let e = parse_sentence(&text, b)?;
        if b.targets.len() != first + 1 {
            return None;
        }
        let spec = &mut b.targets[first];
        if !matches!((&spec.min, &spec.max), (Value::Const(1), Value::Const(1))) {
            return None;
        }
        spec.random = true;
        return Some(e);
    }
    // Not targeted: "a creature an opponent controls chosen at random".
    let a = article_at?;
    let phrase = &before[a + 1..];
    // Permanents only: "a creature card with mana value X chosen at random" and "a copy
    // of a Liliana planeswalker chosen at random" choose among cards outside the game,
    // which the engine doesn't model.
    if phrase.split(' ').any(|w| w == "card" || w == "cards") || before.contains("copy of ") {
        return None;
    }
    let (n, r2) = parse_number(phrase)?;
    if n.as_const() != Some(1) {
        return None;
    }
    let (f, plural, tail) = parse_object_phrase(r2)?;
    if plural || !end(tail).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield)
    {
        return None;
    }
    let placeholder = "the object chosen at random";
    let text = format!("{} {placeholder}{after}", &before[..a]);
    b.named.push((placeholder.into(), Sel::Var(RANDOM_OBJECT)));
    let e = parse_sentence(&text, b);
    b.named.retain(|(p, _)| p != placeholder);
    let e = e?;
    Some(Effect::seq(vec![
        Effect::Store {
            var: RANDOM_OBJECT,
            sel: Sel::AtRandom {
                filter: Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
                count: Value::c(1),
                store: None,
            },
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "choice grammar: chosen at random", priority: 80, parse: chosen_at_random } }

/// "Destroy one of them at random." after choosing several targets ("Choose three target
/// nonenchantment permanents."): one of those objects, chosen at random.
fn one_of_them_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (verb, r) = l.split_once(' ')?;
    if r != "one of them at random" {
        return None;
    }
    let them = super::pronoun_groups::plural_object_ref("them", b)??.0;
    let what = Sel::AtRandom {
        filter: Filter::In(Box::new(them)),
        count: Value::c(1),
        store: None,
    };
    match verb {
        "destroy" => Some(Effect::Destroy {
            what,
            no_regen: false,
        }),
        "exile" => Some(Effect::Exile {
            what,
            face_down: false,
            link: false,
        }),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "choice grammar: [verb] one of them at random", priority: 80, parse: one_of_them_at_random } }

/// "choose 1, 2, or 3 at random": a number chosen at random, which "that many" / "that
/// number" means afterward (see [`that_many_random`]).
fn choose_number_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?.strip_suffix(" at random")?;
    let words: Vec<&str> = r
        .split(", ")
        .flat_map(|w| w.split(" or "))
        .map(|w| w.trim_start_matches("or ").trim())
        .filter(|w| !w.is_empty())
        .collect();
    if words.len() < 2 {
        return None;
    }
    let nums: Vec<i32> = words
        .iter()
        .map(|w| w.parse::<i32>().ok())
        .collect::<Option<Vec<_>>>()?;
    b.named.push((
        "\u{1}random number".into(),
        Sel::None,
    ));
    random_number_effect(RANDOM_NUMBER, &nums)
}

inventory::submit! { EffectPattern { name: "choice grammar: choose a number at random", priority: 80, parse: choose_number_at_random } }

/// "Its controller mills that many cards, ..." after "choose 1, 2, or 3 at random": the
/// sentence with "that many" meaning the number chosen.
fn that_many_random(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !b.named.iter().any(|(p, _)| p == "\u{1}random number") || !l.contains("that many") {
        return false;
    }
    let text = l.replacen("that many", "x", 1);
    let Some(e) = parse_effect_with_x(&text, b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

/// Parses `text` with X meaning the number chosen at random.
fn parse_effect_with_x(text: &str, b: &mut Builder) -> Option<Effect> {
    let e = crate::oracle::effects::parse_effect_text(text, b)?;
    super::r107_numbers::substitute_x(&e, &Value::Var(RANDOM_NUMBER))
}

inventory::submit! { FollowupPattern { name: "choice grammar: that many (the number chosen at random)", priority: 40, apply: that_many_random } }

/// The keyword abilities named by a "choose [A], [B], or [C]" list, if every option is
/// one ("first strike", "vigilance", "lifelink", "hexproof", "indestructible").
fn keyword_options(r: &str) -> Option<Vec<String>> {
    let words: Vec<String> = r
        .replace(", or ", ", ")
        .replace(" or ", ", ")
        .split(", ")
        .map(|w| w.trim().to_string())
        .filter(|w| !w.is_empty())
        .collect();
    if words.len() < 2 {
        return None;
    }
    words
        .iter()
        .all(|w| crate::oracle::effects::keyword_mods(w).is_some())
        .then_some(words)
}

/// "choose first strike, vigilance, or lifelink" / "choose flying or indestructible at
/// random": a keyword ability, which "that ability" names in the next sentence (see
/// [`that_ability`]). The choice itself does nothing.
fn choose_keyword(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (r, random) = match r.strip_suffix(" at random") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let words = keyword_options(r)?;
    let marker = if random { "\u{1}random ability:" } else { "\u{1}ability:" };
    b.named
        .push((format!("{marker}{}", words.join("|")), Sel::None));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choice grammar: choose a keyword ability", priority: 80, parse: choose_keyword } }

/// "Creatures you control gain that ability until end of turn." / "~ gains that ability
/// until end of turn." after choosing a keyword: the chosen one of the instructions with
/// each ability in place of "that ability" (chosen by you, or at random).
fn that_ability(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(prev, Effect::Noop) || !l.contains("that ability") {
        return false;
    }
    let Some((marker, _)) = b
        .named
        .iter()
        .rev()
        .find(|(p, _)| p.starts_with("\u{1}ability:") || p.starts_with("\u{1}random ability:"))
        .cloned()
    else {
        return false;
    };
    let random = marker.starts_with("\u{1}random");
    let words: Vec<String> = marker
        .split_once(':')
        .map(|(_, w)| w.split('|').map(str::to_string).collect())
        .unwrap_or_default();
    let mut options = Vec::new();
    for w in &words {
        let text = l.replacen("that ability", w, 1);
        let Some(e) = parse_clause(end(&text), b) else {
            return false;
        };
        options.push((w.clone(), e));
    }
    b.named.retain(|(p, _)| p != &marker);
    *prev = if random {
        let n = options.len() as i32;
        let nums: Vec<i32> = (1..=n).collect();
        let Some(pick) = random_number_effect(RANDOM_NUMBER, &nums) else {
            return false;
        };
        let mut chain = Effect::Noop;
        for (i, (_, e)) in options.into_iter().enumerate().rev() {
            chain = Effect::If {
                cond: Condition::Compare(
                    Value::Var(RANDOM_NUMBER),
                    Cmp::Eq,
                    Value::c(i as i32 + 1),
                ),
                then: Box::new(e),
                otherwise: Box::new(chain),
            };
        }
        Effect::seq(vec![pick, chain])
    } else {
        Effect::ChooseOne {
            who: PlayerRef::You,
            options,
        }
    };
    true
}

inventory::submit! { FollowupPattern { name: "choice grammar: that ability (the chosen keyword)", priority: 40, apply: that_ability } }

/// Whether the effect ends by exiling cards chosen at random (see [`exile_at_random`]).
fn ends_with_random_exile(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::AtRandom { .. },
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_random_exile),
        Effect::If {
            then, otherwise, ..
        } => ends_with_random_exile(then) && matches!(**otherwise, Effect::Noop),
        _ => false,
    }
}

/// Appends `e` to the effect, inside a trailing condition ("if there are seven or more
/// cards in your graveyard, exile a card at random ... You may play that card this turn.":
/// only if the card was exiled).
fn append_after_exile(prev: &mut Effect, e: Effect) {
    match prev {
        Effect::If { then, .. } => append_after_exile(then, e),
        Effect::Seq(v) if v.last().is_some_and(|x| matches!(x, Effect::If { .. })) => {
            if let Some(last) = v.last_mut() {
                append_after_exile(last, e);
            }
        }
        _ => {
            let old = std::mem::take(prev);
            *prev = Effect::seq(vec![old, e]);
        }
    }
}

/// After exiling a card at random: "You may play that card this turn.", "You may play the
/// exiled card this turn.", "You may cast it this turn." (a permission for that card, CR
/// 400.7), "Copy it." (a copy of the card in exile, CR 707.12).
fn after_random_exile(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !ends_with_random_exile(prev) {
        return false;
    }
    let l = end(l);
    let card = Sel::Var(vars::IT);
    let e = match l {
        "copy it" | "copy that card" | "copy the exiled card" => Effect::CopyCard {
            what: card,
            named: None,
        },
        _ => {
            let (cast_only, r) = match (l.strip_prefix("you may play "), l.strip_prefix("you may cast ")) {
                (Some(r), _) => (false, r),
                (None, Some(r)) => (true, r),
                _ => return false,
            };
            let Some(r) = ["that card", "the exiled card", "it"]
                .iter()
                .find_map(|p| r.strip_prefix(p).filter(|x| x.starts_with(' ')))
            else {
                return false;
            };
            let duration = match r.trim() {
                "this turn" | "until end of turn" => Duration::EndOfTurn,
                _ => return false,
            };
            let grant = Effect::GrantPlayPermission {
                who: PlayerRef::You,
                what: card,
                duration,
                free: false,
            };
            if cast_only {
                grant.cast_only()
            } else {
                grant
            }
        }
    };
    append_after_exile(prev, e);
    true
}

inventory::submit! { FollowupPattern { name: "choice grammar: after exiling a card at random", priority: 85, apply: after_random_exile } }
