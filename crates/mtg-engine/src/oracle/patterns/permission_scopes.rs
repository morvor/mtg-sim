//! Permissions for the card a triggered ability's source became (CR 400.7, 601.3):
//! "When ~ dies, you may cast it from your graveyard as an Adventure until the end of your
//! next turn." (Hildibrand Manderville): a permission for that card in the graveyard
//! ("it" is the card the source became, `Sel::This`), cast only as its Adventure
//! (`PlayTerms::cast_face`, CR 715.3) when it says so.

use super::EffectPattern;
use crate::ability::*;
use crate::object::FaceState;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn p_cast_it_from_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger || !matches!(b.it, Sel::This) || b.ctx.is_spell() {
        return None;
    }
    let l = end(l.trim());
    let r = l
        .strip_prefix("you may cast it from your graveyard")
        .or_else(|| l.strip_prefix("cast it from your graveyard"))?;
    let (face, r) = match r.strip_prefix(" as an adventure") {
        Some(r) => (Some(FaceState::Half(1)), r),
        None => (None, r),
    };
    if face.is_some() && b.ctx.layout != crate::card::Layout::Adventure {
        return None;
    }
    let duration = match r {
        " this turn" | " until end of turn" => Duration::EndOfTurn,
        " until the end of your next turn" => Duration::UntilEndOfYourNextTurn,
        _ => return None,
    };
    Some(Effect::WithPlayTerms {
        terms: PlayTerms {
            spells_only: true,
            cast_face: face,
            ..Default::default()
        },
        effect: Box::new(Effect::GrantPlayPermission {
            who: PlayerRef::You,
            what: Sel::This,
            duration,
            free: false,
        }),
    })
}

inventory::submit! { EffectPattern { name: "permission scopes: you may cast it from your graveyard [as an Adventure] for a while", priority: 120, parse: p_cast_it_from_graveyard } }
