//! "You may cast this card from your graveyard by paying {3}{R} and exiling four other
//! cards from your graveyard rather than paying its mana cost." (Squee, Dubious Monarch),
//! "You may cast this card from your graveyard by discarding two cards in addition to
//! paying its other costs." (Rona, Sheoldred's Faithful): a static ability that functions
//! while the card is in its owner's graveyard (CR 113.6m) and lets them cast it from there
//! (CR 601.3) for an alternative cost (CR 118.9) or with an additional cost (CR 601.2f).
//! The spell's timing is the normal one for its type; as it's cast, the card moves to the
//! stack first (CR 601.2a) and the costs are paid last (CR 601.2h).
//!
//! The ability is a `StaticEffect::Custom` named by [`ability_name`] (compiled in
//! `oracle/patterns/cast_self_from_graveyard.rs`); the cost text is parsed again here.
//! The spell is cast with `CastMethod::Alternative` (that ability's uid) and [`TAG`] in
//! its cast information: it's cast with this ability's own permission, not another one
//! (such as Lurrus's).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::CastMethod;
use crate::types::*;

/// Recorded in `CastInfo::paid` for a spell cast with such an ability.
pub const TAG: &str = "cast from its owner's graveyard by its own ability";

const ALTERNATIVE: &str = "cast from graveyard instead:";
const ADDITIONAL: &str = "cast from graveyard also:";

/// The `StaticEffect::Custom` name of the ability: `cost` is the cost's text (as
/// [`crate::oracle::costs::parse_cost`] reads it), paid instead of the mana cost or in
/// addition to the other costs.
pub fn ability_name(cost: &str, instead: bool) -> String {
    format!("{}{cost}", if instead { ALTERNATIVE } else { ADDITIONAL })
}

/// The cost of such an ability, and whether it's paid instead of the mana cost.
fn ability_cost(name: &str) -> Option<(Cost, bool)> {
    let (text, instead) = match name.strip_prefix(ALTERNATIVE) {
        Some(t) => (t, true),
        None => (name.strip_prefix(ADDITIONAL)?, false),
    };
    let (cost, loyalty) = crate::oracle::costs::parse_cost(text)?;
    (!loyalty).then_some((cost, instead))
}

pub struct CastSelfFromGraveyard;

impl KeywordRules for CastSelfFromGraveyard {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        if !crate::as_though::in_graveyard_for(g, p, card) {
            return vec![];
        }
        let mut out = Vec::new();
        for a in &g.obj(card).chars.abilities {
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            let StaticEffect::Custom(name) = &s.effect else {
                continue;
            };
            if s.zone != FunctionZone::Graveyard {
                continue;
            }
            let Some((cost, instead)) = ability_cost(name) else {
                continue;
            };
            for face in crate::casting::castable_faces(g, card) {
                let mut opt = CastOption::normal(face);
                opt.method = CastMethod::Alternative(a.uid);
                if instead {
                    opt.alt_cost = Some(cost.clone());
                } else {
                    opt.extra_cost = Some(cost.clone());
                }
                opt.tag = Some(TAG);
                out.push(opt);
            }
        }
        out
    }
}

inventory::submit! { KeywordRegistration(&CastSelfFromGraveyard) }
