//! "If a spell cast this way would be put into a graveyard, exile it instead." after "you
//! may cast [cards] without paying their mana costs" (Diluvian Primordial): a replacement
//! effect for the spells cast (CR 614.1a), which the casting effect records as "it"
//! (`vars::IT`, CR 400.7h). It applies wherever they would go to a graveyard from —
//! resolving or countered — and only to those spells: once one has left the stack it's a
//! new object the effect doesn't apply to.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// Whether the effect ends by casting cards.
fn ends_with_casting(e: &Effect) -> bool {
    match e {
        Effect::CastCard { .. } => true,
        Effect::May { effect, .. } => ends_with_casting(effect),
        Effect::Seq(v) => v.last().is_some_and(ends_with_casting),
        _ => false,
    }
}

fn spell_cast_this_way_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "if a spell cast this way would be put into a graveyard, exile it instead"
            | "if a spell cast this way would be put into a graveyard, exile that card instead"
    ) || !ends_with_casting(prev)
    {
        return false;
    }
    let replacement = Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: Filter::In(Box::new(Sel::Var(vars::IT))),
                from: None,
                to: Some(ZoneKind::Graveyard),
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::Permanent,
        uses: None,
    };
    *prev = Effect::seq(vec![std::mem::take(prev), replacement]);
    true
}

inventory::submit! { FollowupPattern { name: "if a spell cast this way would be put into a graveyard, exile it instead", priority: 90, apply: spell_cast_this_way_exiled } }
