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

/// "[players] can't cast [spells] [this turn / until your next turn]".
fn players_cant_cast(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, main) = duration_suffix(l);
    if !matches!(dur, Duration::EndOfTurn | Duration::UntilYourNextTurn) {
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
            | PlayerRef::TriggerPlayer => PlayerFilter::Ref(Box::new(r)),
            _ => return None,
        };
        (who, rest)
    };
    let what = rest.trim().strip_prefix("can't cast ")?;
    let what = if what == "spells" {
        Filter::Any
    } else {
        let (f, plural) = whole_object_phrase(&union_nouns(what))?;
        if !plural || !filter_mentions(&f, &|x| matches!(x, Filter::Spell)) {
            return None;
        }
        // The card being cast isn't a spell yet as the prohibition is checked.
        without_spell(f)?
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::CantCast { who, what },
        duration: dur,
    })
}

inventory::submit! { EffectPattern { name: "restrictions: players can't cast spells this turn", priority: 100, parse: players_cant_cast } }
