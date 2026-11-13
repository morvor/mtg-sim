//! Goad with durations other than "until your next turn" (CR 701.15), and ending goad:
//!
//! * "It's goaded for the rest of the game.", "The tokens are goaded for the rest of the
//!   game." (Jon Irenicus, Life of the Party, Nettling Nuisance);
//! * "The tokens are goaded for as long as ~ remains on the battlefield." (The War Games),
//!   "As long as ~ remains on the battlefield, that creature is also goaded." (Hot Pursuit:
//!   if ~ has already left, the creature isn't goaded at all, CR 611.2b);
//! * "If you do, it's goaded for as long as they control it." (Vislor Turlough);
//! * "Then each creature you control is no longer goaded." (Serene Sleuth).
//!
//! A creature goaded this way is goaded by the controller of the effect (CR 701.15a-b):
//! the effect is a [`Restriction::Goaded`] locked onto the objects named as it begins
//! (CR 611.2c), which `Game::goaders` reads.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_clause, Builder};
use crate::oracle::phrases::end;

/// The duration phrase after "goaded".
fn goad_duration(d: &str, b: &Builder) -> Option<Duration> {
    Some(match d.trim() {
        "for the rest of the game" => Duration::Permanent,
        "for as long as ~ remains on the battlefield"
        | "as long as ~ remains on the battlefield" => Duration::WhileSourceOnBattlefield,
        // "for as long as they control it": the player the text named, fixed as the
        // effect begins (see `Effect::AddRestriction`).
        "for as long as they control it" | "for as long as that player controls it" => {
            if super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
                return None;
            }
            Duration::WhileCondition(Condition::SelMatches(
                Sel::Var(vars::AFFECTED),
                Filter::ControlledByPlayer(Box::new(b.it_player.clone())),
            ))
        }
        _ => return None,
    })
}

/// The objects a goad sentence's subject names: specific objects only ("it", "that
/// creature", "the tokens").
fn goaded_subject(s: &str, b: &mut Builder) -> Option<Filter> {
    let s = s.trim();
    let s = match s {
        "it's" | "it is" => "it",
        "they're" | "they are" => "they",
        other => other,
    };
    let (what, rest) = object_ref(s, b)?;
    // The source only when the text names it ("~", or "it" meaning it).
    let names_source = s.starts_with('~') || s == "it";
    if !end(&rest).is_empty()
        || matches!(what, Sel::All(_) | Sel::Players(_) | Sel::None)
        || (matches!(what, Sel::This) && !names_source)
        || super::oracle_hardening_referents::is_no_referent(&what)
    {
        return None;
    }
    Some(Filter::In(Box::new(what)))
}

fn goaded(f: Filter, duration: Duration) -> Effect {
    Effect::AddRestriction {
        restriction: Restriction::Goaded(f),
        duration,
    }
}

/// "[objects] is/are goaded [duration]", "as long as ~ remains on the battlefield,
/// [objects] is also goaded".
fn goaded_for(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    // "As long as ~ remains on the battlefield, that creature is also goaded."
    if let Some((cond, main)) = l.split_once(", ") {
        let d = goad_duration(cond, b)?;
        let subject = main
            .strip_suffix(" is also goaded")
            .or_else(|| main.strip_suffix(" are also goaded"))?;
        let f = goaded_subject(subject, b)?;
        return Some(goaded(f, d));
    }
    // "It's goaded for the rest of the game and it gains "~ can't be sacrificed."".
    if let Some((first, second)) = l.split_once(" and ") {
        let a = goaded_for(first, b)?;
        let c = parse_clause(second, b)?;
        return Some(Effect::seq(vec![a, c]));
    }
    let (subject, d) = ["'s goaded ", " is goaded ", " are goaded ", "'re goaded "]
        .iter()
        .find_map(|v| l.split_once(v).map(|(s, d)| (s, d, *v)))
        .map(|(s, d, v)| {
            let s = match v {
                "'s goaded " => format!("{s}'s"),
                "'re goaded " => format!("{s}'re"),
                _ => s.to_string(),
            };
            (s, d)
        })?;
    let d = goad_duration(d, b)?;
    let f = goaded_subject(&subject, b)?;
    Some(goaded(f, d))
}

inventory::submit! { EffectPattern { name: "goad durations: [objects] are goaded [duration]", priority: 95, parse: goaded_for } }

/// "Then each creature you control is no longer goaded.", "it's no longer goaded": the
/// goad designation ends (CR 701.15).
fn no_longer_goaded(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let subject = l
        .strip_suffix(" is no longer goaded")
        .or_else(|| l.strip_suffix(" are no longer goaded"))
        .or_else(|| l.strip_suffix("'s no longer goaded"))?;
    let (what, rest) = object_ref(subject, b)?;
    if !end(&rest).is_empty() || super::oracle_hardening_referents::is_no_referent(&what) {
        return None;
    }
    let mut spec = crate::kwa::Spec::new(KeywordAction::Goad, PlayerRef::You, what, Value::c(1));
    spec.undo = true;
    Some(spec.effect())
}

inventory::submit! { EffectPattern { name: "goad durations: [objects] are no longer goaded", priority: 95, parse: no_longer_goaded } }

/// "Creatures your opponents control get -2/-0 until end of turn. If any of them are
/// suspected, they're no longer suspected." (Eliminate the Impossible): the creatures the
/// previous sentence affected (fixed as it resolved) that are suspected stop being
/// suspected (CR 701.60a).
fn f_any_of_them_unsuspected(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l.trim()) != "if any of them are suspected, they're no longer suspected" {
        return false;
    }
    fn affected(e: &Effect) -> Option<Sel> {
        match e {
            Effect::Modify {
                what: w @ Sel::All(_),
                ..
            } => Some(w.clone()),
            Effect::Seq(v) => v.last().and_then(affected),
            _ => None,
        }
    }
    let Some(Sel::All(f)) = affected(prev) else {
        return false;
    };
    // The objects the earlier instruction affected, as it did (a later creature isn't
    // "them").
    let what = Sel::All(f);
    let mut spec = crate::kwa::Spec::new(KeywordAction::Suspect, PlayerRef::You, what, Value::c(1));
    spec.undo = true;
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, spec.effect()]);
    true
}

inventory::submit! { FollowupPattern { name: "goad durations: if any of them are suspected, they're no longer suspected", priority: 40, apply: f_any_of_them_unsuspected } }

/// "Each opponent creates a token that's a copy of it. The tokens are goaded for the rest
/// of the game." (Life of the Party): the tokens each of those players created.
fn f_each_players_tokens_goaded(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim());
    let Some(d) = ["the tokens are goaded ", "those tokens are goaded "]
        .iter()
        .find_map(|p| l.strip_prefix(p))
    else {
        return false;
    };
    let Some(duration) = goad_duration(d, b) else {
        return false;
    };
    // The creation is the last instruction, done by each of those players (the ability's
    // controller stays the goading player: no `AsPlayer`).
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        e => Some(e),
    };
    let Some(Effect::ForEachPlayer { effect, .. }) = last else {
        return false;
    };
    if !matches!(
        **effect,
        Effect::CreateToken { .. }
            | Effect::CreateTokenWithPT { .. }
            | Effect::CreateTokenCopy { .. }
    ) {
        return false;
    }
    let create = std::mem::take(&mut **effect);
    **effect = Effect::seq(vec![
        create,
        goaded(Filter::In(Box::new(Sel::Var(vars::CREATED))), duration),
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "goad durations: the tokens each player created are goaded", priority: 39, apply: f_each_players_tokens_goaded } }
