//! "Whenever a player casts a card, if it has the same name as one of the cards exiled
//! with ~, you may copy the other." (Spellweaver Helix, CR 707.12): of the cards exiled
//! with the source (CR 607.2a), one that has the same name as the spell is set aside and
//! the other is copied (one copy even when both have that name; nothing with only one
//! card exiled, its rulings). The copy is "the copy" later instructions cast.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;

/// [`Effect::Custom`]: "copy the other".
pub const COPY_THE_OTHER: &str = "copy the other exiled card";

/// The cards exiled with the source by its linked ability, still in exile.
pub fn exiled_with_source() -> Filter {
    Filter::and(vec![
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ])
}

pub struct CopyTheOther;

impl KeywordRules for CopyTheOther {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != COPY_THE_OTHER {
            return false;
        }
        ctx.prev_happened = false;
        let exiled: Vec<_> = g
            .objects_matching(&exiled_with_source(), ctx)
            .into_iter()
            .filter(|o| g.obj(*o).zone == Zone::Exile)
            .collect();
        let Some(spell) = g
            .eval_sel(&Sel::TriggerSpell, ctx)
            .into_iter()
            .find_map(|e| e.object())
        else {
            return true;
        };
        let spell_chars = g.obj(spell).chars.clone();
        let Some(same) = exiled
            .iter()
            .position(|o| g.obj(*o).chars.shares_name_with(&spell_chars))
        else {
            return true;
        };
        let Some(other) = exiled
            .iter()
            .enumerate()
            .find(|(i, _)| *i != same)
            .map(|(_, o)| *o)
        else {
            return true;
        };
        let copy = Effect::CopyCard {
            what: Sel::All(Filter::Objects(vec![other])),
            named: None,
        };
        g.exec(&copy, ctx);
        ctx.prev_happened = ctx.vars.get(&vars::CREATED).is_some_and(|v| !v.is_empty());
        true
    }
}

inventory::submit! { KeywordRegistration(&CopyTheOther) }
