//! "Your commander" terms (CR 903.3d, 903.3e; cards3-costs-mana-replacements):
//!
//! - `Filter::Custom` [`SHARES_TYPE_WITH_YOUR_COMMANDER`]: "a creature spell that shares a
//!   creature type with your commander" (Path of Ancestry): the object has a creature type
//!   in common with one of the commanders the player ("you") owns, in whatever zone it is
//!   (CR 903.3e); changelings have every creature type (CR 702.73a).
//! - `Value::Custom` [`COLORS_IN_COMMANDERS_IDENTITY`]: "the number of colors in your
//!   commanders' color identity" (War Room): of the combined identity of the commanders
//!   the player owns (CR 903.4, 702.124c). Without a commander it's undefined, and a cost
//!   that refers to it is unpayable (CR 903.4f).

use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Characteristics;
use crate::types::*;

/// The `Filter::Custom` name.
pub const SHARES_TYPE_WITH_YOUR_COMMANDER: &str = "shares a creature type with your commander";

/// The amount of an undefined quality in a cost (CR 903.4f: unpayable).
const UNPAYABLE: i64 = i32::MAX as i64;

/// The `Value::Custom` name.
pub const COLORS_IN_COMMANDERS_IDENTITY: &str = "colors in your commanders' color identity";

/// Whether two objects' characteristics have a creature type in common.
fn share_creature_type(a: &Characteristics, b: &Characteristics) -> bool {
    let every = crate::kw::changeling::every_creature_type;
    let any_type = |x: &Characteristics| every(x) || x.subtypes.iter().any(|s| is_creature_type(s));
    a.subtypes
        .iter()
        .any(|s| is_creature_type(s) && b.has_subtype(s))
        || (every(a) && any_type(b))
        || (every(b) && any_type(a))
}

pub struct YourCommanderTerms;

impl super::KeywordRules for YourCommanderTerms {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != SHARES_TYPE_WITH_YOUR_COMMANDER {
            return None;
        }
        let me = &g.obj(id).chars;
        Some(
            crate::commander_rules::commanders_of(g, ctx.controller)
                .into_iter()
                .any(|c| share_creature_type(me, &g.obj(c).chars)),
        )
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        // Undefined without a commander (CR 903.4f): a cost of that much life is
        // unpayable, so it's more life than any player can have.
        (name == COLORS_IN_COMMANDERS_IDENTITY).then(|| {
            crate::commander_rules::commanders_identity(g, ctx.controller)
                .map_or(UNPAYABLE, |c| c.count() as i64)
        })
    }
}

inventory::submit! { super::KeywordRegistration(&YourCommanderTerms) }
