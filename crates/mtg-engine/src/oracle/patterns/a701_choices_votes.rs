//! Oracle patterns for villainous choices (CR 701.55) and votes (CR 701.38): "[player]
//! faces a villainous choice — [A], or [B]", "If an opponent would face a villainous
//! choice, they face that choice an additional time", "starting with you, each player
//! votes for [A] or [B]" / "for [an object]", "if [A] gets more votes [or the vote is
//! tied]", "for each [A] vote", "exile each permanent with the most votes or tied for most
//! votes", and "While voting, you get an additional vote".

use super::{ConditionPattern, EffectPattern, StaticPattern};
use crate::ability::*;
use crate::kwa::vote::{word_var, CHOICES_WITH_MOST, MOST_VOTES, WINNERS};
use crate::kwa::Spec;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// One option of a villainous choice, with "they" / "that player" / "that opponent" as the
/// player facing it and "you" as the controller.
fn villainous_option(text: &str, b: &mut Builder) -> Option<Effect> {
    let t = text.trim();
    let t = if let Some(r) = t.strip_prefix("you ") {
        r.to_string()
    } else if let Some(r) = t.strip_prefix("they ") {
        // "they lose 2 life" is "that player loses 2 life".
        let (verb, rest) = split_word(r);
        let verb = if verb.ends_with('s') {
            verb.to_string()
        } else {
            format!("{verb}s")
        };
        format!("that player {verb} {rest}")
    } else if let Some(r) = ["that opponent ", "that player "]
        .iter()
        .find_map(|p| t.strip_prefix(p))
    {
        format!("that player {r}")
    } else {
        t.to_string()
    };
    let saved = b.it_player.clone();
    b.it_player = PlayerRef::Iterated;
    let e = crate::oracle::effects::parse_effect_text(&t, b);
    b.it_player = saved;
    e
}

/// "[player] faces a villainous choice — [option A], or [option B]" (CR 701.55a).
fn villainous_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let marker = "faces a villainous choice — ";
    let i = l.find(marker)?;
    let (subject, options) = (l[..i].trim(), &l[i + marker.len()..]);
    let who = if subject.is_empty() {
        b.it_player.clone()
    } else {
        let (who, rest) = player_ref(subject, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        who
    };
    // The options are separated by ", or "; an option may itself contain commas.
    let mut start = 0;
    while let Some(k) = options[start..].find(", or ") {
        let k = start + k;
        let (a, c) = (&options[..k], &options[k + 5..]);
        let saved_targets = b.targets.len();
        if let (Some(ea), Some(ec)) = (villainous_option(a, b), villainous_option(c, b)) {
            let mut spec = Spec::new(KeywordAction::VillainousChoice, who, Sel::None, Value::c(1));
            spec.options = vec![(a.to_string(), ea), (c.to_string(), ec)];
            return Some(spec.effect());
        }
        b.targets.truncate(saved_targets);
        start = k + 5;
    }
    None
}

inventory::submit! { EffectPattern { name: "a701 villainous choice", priority: 60, parse: villainous_choice } }

/// Rules-modifying statics: "If an opponent would face a villainous choice, they face that
/// choice an additional time." (CR 701.55c), "While voting, you get an additional vote." /
/// "While voting, you may vote an additional time." (CR 701.38d).
fn choice_statics(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let name = match end(l) {
        "if an opponent would face a villainous choice, they face that choice an additional time" => {
            crate::kwa::villainous::FACE_AGAIN
        }
        "while voting, you get an additional vote" => crate::kwa::vote::EXTRA_VOTE,
        "while voting, you may vote an additional time" => crate::kwa::vote::OPTIONAL_EXTRA_VOTE,
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(SmolStr::new(name)))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 villainous choice and vote statics", priority: 60, parse: choice_statics } }

/// "starting with you, each player votes for [A] or [B]" / "for [A], [B], or [C]" /
/// "for a [object]" (CR 701.38a–b).
fn vote(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("starting with you, ")?;
    let r = r.strip_prefix("each player votes for ")?;
    let _ = b;
    // Objects: "a nonland permanent you don't control", "an artifact ... card in your
    // graveyard".
    if let Some(x) = r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) {
        if let Some((f, false, tail)) = parse_object_phrase(x) {
            if end(tail).is_empty() {
                let f = if f.zone().is_some() {
                    f
                } else {
                    Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])
                };
                return Some(
                    Spec::new(KeywordAction::Vote, PlayerRef::You, Sel::All(f), Value::c(1))
                        .effect(),
                );
            }
        }
    }
    // Words: "time or knowledge", "blue, black, red, or green".
    let words: Vec<String> = r
        .replace(", or ", ", ")
        .replace(" or ", ", ")
        .split(", ")
        .map(|w| w.trim().to_string())
        .filter(|w| !w.is_empty())
        .collect();
    if words.len() < 2 || words.iter().any(|w| w.split(' ').count() > 3) {
        return None;
    }
    let mut spec = Spec::new(KeywordAction::Vote, PlayerRef::You, Sel::None, Value::c(1));
    spec.options = words.into_iter().map(|w| (w, Effect::Noop)).collect();
    Some(spec.effect())
}

inventory::submit! { EffectPattern { name: "a701 vote", priority: 60, parse: vote } }

/// "[word] gets more votes", "[word] gets more votes or the vote is tied", "the vote is
/// tied".
fn vote_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    let tied = || Condition::Compare(Value::Var(CHOICES_WITH_MOST), Cmp::Ge, Value::c(2));
    if c == "the vote is tied" {
        return Some(tied());
    }
    let (word, or_tied) = if let Some(w) = c.strip_suffix(" gets more votes or the vote is tied") {
        (w, true)
    } else if let Some(w) = c.strip_suffix(" gets more votes") {
        (w, false)
    } else {
        return None;
    };
    if word.is_empty() || word.split(' ').count() > 3 {
        return None;
    }
    let more = Condition::And(vec![
        Condition::Compare(Value::Var(word_var(word)), Cmp::Eq, Value::Var(MOST_VOTES)),
        Condition::Compare(Value::Var(CHOICES_WITH_MOST), Cmp::Eq, Value::c(1)),
    ]);
    Some(if or_tied {
        Condition::Or(vec![more, tied()])
    } else {
        more
    })
}

inventory::submit! { ConditionPattern { name: "a701 vote results", priority: 60, parse: vote_condition } }

/// "exile each permanent with the most votes or tied for most votes", "for each [word]
/// vote, [effect]".
fn vote_results(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    for verb in ["exile", "destroy"] {
        if l == format!("{verb} each permanent with the most votes or tied for most votes") {
            let what = Sel::Var(WINNERS);
            return Some(if verb == "exile" {
                Effect::Exile {
                    what,
                    face_down: false,
                    link: false,
                }
            } else {
                Effect::Destroy {
                    what,
                    no_regen: false,
                }
            });
        }
    }
    let r = l.strip_prefix("for each ")?;
    let (word, rest) = r.split_once(" vote, ")?;
    if word.split(' ').count() > 3 {
        return None;
    }
    let e = parse_clause(rest, b)?;
    Some(Effect::Repeat {
        times: Value::Var(word_var(word)),
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "a701 vote results", priority: 60, parse: vote_results } }
