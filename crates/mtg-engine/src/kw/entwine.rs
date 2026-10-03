//! CR 702.42 Entwine: "Entwine [cost]" means "You may choose all modes of this spell
//! instead of just the number specified. If you do, you pay an additional [cost]."
//! (CR 702.42a). The modes of an entwined spell are followed in the order written
//! (CR 702.42b, 700.2).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::ObjectId;
use smol_str::SmolStr;

/// The name recorded in the spell's paid costs when its entwine cost is paid.
pub const ENTWINE: &str = "entwine";

pub struct Entwine;

impl KeywordRules for Entwine {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Entwine]
    }

    /// An additional cost announced while casting (CR 601.2b, 601.2f–h), for a modal
    /// spell (CR 700.2).
    fn optional_costs(&self, g: &Game, spell: ObjectId, kw: &Keyword) -> Vec<(SmolStr, Cost, bool)> {
        let modal = g.spell_body(spell).modal.is_some();
        match (&kw.cost, modal) {
            (Some(c), true) => vec![(SmolStr::new(ENTWINE), c.clone(), false)],
            _ => vec![],
        }
    }

    /// With the entwine cost paid, all modes are chosen.
    fn adjust_spell_body(&self, g: &Game, spell: ObjectId, _kw: &Keyword, mut body: Body) -> Body {
        let entwined = g
            .obj(spell)
            .stack
            .as_deref()
            .is_some_and(|s| s.cast.paid.iter().any(|p| p == ENTWINE));
        if let (true, Some(modal)) = (entwined, body.modal.as_mut()) {
            let n = modal.modes.len() as i32;
            modal.min = Value::c(n);
            modal.max = Value::c(n);
            modal.allow_repeat = false;
        }
        body
    }
}

inventory::submit! { KeywordRegistration(&Entwine) }
