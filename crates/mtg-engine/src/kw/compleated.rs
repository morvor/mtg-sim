//! CR 702.150 Compleated: a static ability found on some planeswalker cards. Compleated
//! means "If this permanent would enter with one or more loyalty counters on it and the
//! player who cast it chose to pay life for any part of its cost represented by Phyrexian
//! mana symbols, it instead enters the battlefield with that many loyalty counters minus
//! two for each of those mana symbols." (CR 702.150a).
//!
//! As a spell is cast, the Phyrexian mana symbols its caster pays 2 life for (whether they
//! choose to as the total cost is determined, CR 118.13a, or the payment uses life for
//! them) are counted in `CastInfo::phyrexian_paid_with_life`, which the permanent keeps.
//! Compleated is a replacement effect on the loyalty counters the permanent is given as
//! it enters (CR 122.6, 614.1c); other replacement effects that change that number apply
//! as normal, in the order its controller chooses (CR 616.1).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;
use crate::types::*;

/// `Filter::Custom`: the object is entering the battlefield right now (its counters are
/// being put on it as it enters), and the player who cast it paid life for one or more
/// Phyrexian mana symbols of its cost.
pub const ENTERING: &str = "compleated:entering the battlefield after life was paid";
/// `Value::Custom`: two for each Phyrexian mana symbol of the source's cost its caster
/// paid life for.
pub const LOYALTY_REDUCTION: &str = "compleated:two for each phyrexian symbol paid with life";

/// Records that `symbols` Phyrexian mana symbols of the cost of the spell `spell` were
/// paid with life.
pub fn record_phyrexian_life(g: &mut Game, spell: ObjectId, symbols: u32) {
    if symbols == 0 {
        return;
    }
    let o = &mut g.objects[spell.0 as usize];
    if o.zone != Zone::Stack {
        return;
    }
    if let Some(si) = o.stack.as_mut() {
        si.cast.phyrexian_paid_with_life += symbols;
    }
}

/// How many Phyrexian mana symbols of the cost of the spell that became `id` its caster
/// paid life for.
fn paid_with_life(g: &Game, id: ObjectId) -> u32 {
    g.obj(id)
        .cast
        .as_deref()
        .filter(|c| c.was_cast)
        .map_or(0, |c| c.phyrexian_paid_with_life)
}

pub struct Compleated;

impl KeywordRules for Compleated {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Compleated]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::PutCounters {
                on_objects: Some(Filter::And(vec![
                    Filter::Source,
                    Filter::Custom(ENTERING.into()),
                ])),
                on_players: None,
                kind: Some(counters::LOYALTY.into()),
            },
            action: ReplacementAction::Subtract(Value::Custom(LOYALTY_REDUCTION.into())),
            self_replacement: false,
            optional: false,
        }));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Compleated.name(),
        )])
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == ENTERING)
            .then(|| paid_with_life(g, id) > 0 && g.entering.iter().any(|o| g.current(*o) == id))
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != LOYALTY_REDUCTION {
            return None;
        }
        Some(2 * ctx.source.map_or(0, |s| paid_with_life(g, s)) as i64)
    }
}

inventory::submit! { KeywordRegistration(&Compleated) }
