//! CR 702.193 Power-up.
//!
//! * "Power-up — [Cost]: [Effect]" means "[Cost]: [Effect]. If this permanent entered this
//!   turn, this ability's cost is reduced by this permanent's mana cost. Activate this
//!   ability only once." (CR 702.193a). The ability keeps its full text ("Power-up — ..."),
//!   which identifies it as a power-up ability (`keyword_impls::ability_from_keyword`), so
//!   effects that modify "power-up abilities" apply to it.
//! * The reduction follows CR 118.7: generic mana in the permanent's mana cost reduces
//!   generic mana in the cost; colored and colorless mana reduce mana of the same type, and
//!   any excess reduces that much generic mana (CR 702.193b).
//! * "Activate this ability only once": each permanent's power-up ability, as that object
//!   (CR 400.7), counted in `KeywordState::power_up_activations`. "Each power-up ability
//!   of permanents you control can be activated an additional time" ([`ADDITIONAL_TIME`])
//!   raises the limit.
//! * "Power-up abilities of other creatures you control cost {N} less to activate"
//!   ([`others_cost_less`]) reduces the generic mana of those abilities' costs.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::{StackKind, Zone};
use crate::types::*;

/// `StaticEffect::Custom`: "Each power-up ability of permanents you control can be
/// activated an additional time."
pub const ADDITIONAL_TIME: &str = "power-up:can be activated an additional time";

/// Prefix of the `StaticEffect::Custom` "Power-up abilities of other creatures you
/// control cost {N} less to activate" (followed by N).
const OTHERS_COST_LESS: &str = "power-up:other creatures' cost less:";

/// The `StaticEffect::Custom` name of "Power-up abilities of other creatures you control
/// cost {n} less to activate".
pub fn others_cost_less(n: u32) -> smol_str::SmolStr {
    smol_str::SmolStr::new(format!("{OTHERS_COST_LESS}{n}"))
}

/// Whether `a` is a power-up ability.
pub fn is_power_up(a: &AbilityDef) -> bool {
    matches!(a.kind, AbilityKind::Activated(_))
        && crate::keyword_impls::ability_from_keyword(a) == Some(KeywordKind::PowerUp)
}

/// How many times the power-up abilities of permanents `p` controls can be activated.
fn limit(g: &Game, p: PlayerId) -> u32 {
    1 + g
        .statics
        .customs
        .iter()
        .filter(|(_, ctl, n)| *ctl == p && n == ADDITIONAL_TIME)
        .count() as u32
}

pub struct PowerUp;

impl KeywordRules for PowerUp {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::PowerUp]
    }

    fn activation_allowed(&self, g: &Game, _p: PlayerId, src: ObjectId, a: &Ability) -> bool {
        if !is_power_up(a) {
            return true;
        }
        let done = g
            .kw_state
            .power_up_activations
            .get(&(src, a.uid))
            .copied()
            .unwrap_or(0);
        done < limit(g, g.obj(src).controller)
    }

    /// CR 702.193a–b: reduced by the permanent's mana cost if it entered this turn.
    fn activation_cost(&self, g: &Game, _p: PlayerId, src: ObjectId, a: &Ability, cost: &mut Cost) {
        if !is_power_up(a) {
            return;
        }
        let o = g.obj(src);
        // "Power-up abilities of other creatures you control cost {N} less to activate."
        for (s, ctl, name) in &g.statics.customs {
            let Some(n) = name
                .strip_prefix(OTHERS_COST_LESS)
                .and_then(|n| n.parse::<u32>().ok())
            else {
                continue;
            };
            if *s != src && *ctl == o.controller && o.is_creature() {
                if let Some(m) = cost.mana.as_mut() {
                    m.reduce_generic(n);
                }
            }
        }
        if o.zone != Zone::Battlefield || o.entered_turn != g.turn.number {
            return;
        }
        let (Some(by), Some(m)) = (o.chars.mana_cost.clone(), cost.mana.as_mut()) else {
            return;
        };
        crate::cost_rules::reduce_by(m, &by, false, |_, cur, s| {
            crate::cost_rules::default_half(cur, s)
        });
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::AbilityActivated {
            ability: Some(ab),
            source,
            ..
        } = ev
        else {
            return;
        };
        let Some(StackKind::Activated { ability, .. }) =
            g.obj(*ab).stack.as_deref().map(|s| &s.kind)
        else {
            return;
        };
        if is_power_up(ability) {
            let key = (*source, ability.uid);
            *g.kw_state.power_up_activations.entry(key).or_insert(0) += 1;
        }
    }
}

inventory::submit! { KeywordRegistration(&PowerUp) }
