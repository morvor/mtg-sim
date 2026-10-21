//! Temporary casting prohibitions from resolving spells and abilities (CR 601.3,
//! 611.2c): "Your opponents can't cast spells this turn.", "Target player can't cast
//! creature spells this turn.", "Its controller can't cast spells this turn.".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, player_ref, Builder};
use crate::oracle::patterns::statics::{
    filter_mentions, union_nouns, whole_object_phrase, without_spell,
};
use crate::oracle::phrases::end;

/// "[players] can't cast [spells] [this turn / until your next turn / until the end of
/// your next turn]".
fn players_cant_cast(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, main) = match l.strip_prefix("until end of turn, ") {
        Some(m) => (Duration::EndOfTurn, m),
        // "Until your next turn, your opponents can't cast spells from anywhere other
        // than their hands." (Avatar's Wrath)
        None if l.starts_with("until your next turn, ") => {
            (Duration::UntilYourNextTurn, &l["until your next turn, ".len()..])
        }
        // "Players can't cast creature or planeswalker spells until the end of your next
        // turn." (Single Combat)
        None => match l.strip_suffix(" until the end of your next turn") {
            Some(m) => (Duration::UntilEndOfYourNextTurn, m),
            None => duration_suffix(l),
        },
    };
    // "until a player planeswalks" (CR 901.11, Eloren Wilds).
    if !matches!(
        dur,
        Duration::EndOfTurn
            | Duration::UntilYourNextTurn
            | Duration::UntilEndOfYourNextTurn
            | Duration::UntilPlaneswalk { .. }
    ) {
        return None;
    }
    let (who, rest) = if let Some(r) = main.strip_prefix("players ") {
        (PlayerFilter::Any, r.to_string())
    } else {
        let (r, rest) = player_ref(main, b)?;
        let who = match r {
            PlayerRef::You => PlayerFilter::You,
            PlayerRef::EachOpponent => PlayerFilter::Opponent,
            PlayerRef::EachPlayer => PlayerFilter::Any,
            // Specific players are fixed as the effect begins (see `fix_restriction`).
            PlayerRef::Target(_)
            | PlayerRef::ControllerOf(_)
            | PlayerRef::OwnerOf(_)
            | PlayerRef::DefendingPlayer
            | PlayerRef::TriggerPlayer
            // "that player" of "each opponent ... during that player's next turn"
            // (`durations_players.rs`).
            | PlayerRef::Iterated => PlayerFilter::Ref(Box::new(r)),
            _ => return None,
        };
        (who, rest)
    };
    let what = rest.trim().strip_prefix("can't cast ")?;
    // "..., and that player can't activate abilities that aren't mana abilities"
    let (what, non_mana) = match [
        ", and that player can't activate abilities that aren't mana abilities",
        " or activate abilities that aren't mana abilities",
    ]
    .iter()
    .find_map(|suffix| what.strip_suffix(suffix))
    {
        Some(w) => (w, true),
        None => (what, false),
    };
    let what = if what == "spells" {
        Filter::Any
    } else if let Some(z) = what.strip_prefix("spells from ") {
        crate::oracle::patterns::statics::cast_from_zones(z)?
    } else {
        let (f, plural) = whole_object_phrase(&union_nouns(what))?;
        if !plural || !filter_mentions(&f, &|x| matches!(x, Filter::Spell)) {
            return None;
        }
        // The card being cast isn't a spell yet as the prohibition is checked.
        without_spell(f)?
    };
    let cast = Effect::AddRestriction {
        restriction: Restriction::CantCast {
            who: who.clone(),
            what,
        },
        duration: dur.clone(),
    };
    if !non_mana {
        return Some(cast);
    }
    Some(Effect::Seq(vec![
        cast,
        Effect::AddRestriction {
            restriction: Restriction::CantActivate {
                who,
                sources: Filter::Any,
                include_mana: false,
            },
            duration: dur,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "restrictions: players can't cast spells this turn", priority: 100, parse: players_cant_cast } }
