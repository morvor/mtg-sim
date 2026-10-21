//! Bewitching Leechcraft: "Enchanted creature has "If ~ would untap during your untap step,
//! remove a +1/+1 counter from it instead. If you do, untap it."" A replacement effect of
//! untapping during the untap step (CR 502.3, 614.1a), granted to the creature (so it's
//! lost with the creature's abilities).

use super::{marker, ManualAbility};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{counters, Entity, ObjectId, PlayerId};

const LEECH: &str = "card:Bewitching Leechcraft:remove a +1/+1 counter instead of untapping";
const TEXT: &str = "Enchanted creature has \"If ~ would untap during your untap step, remove a +1/+1 counter from it instead. If you do, untap it.\"";
const GRANTED: &str = "If this creature would untap during your untap step, remove a +1/+1 counter from it instead. If you do, untap it.";

inventory::submit! { ManualAbility {
    card: "Bewitching Leechcraft",
    face: 0,
    text: TEXT,
    build: |_| {
        vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
                affected: Filter::AttachedToSource,
                mods: vec![Modification::AddAbility(marker(LEECH, GRANTED))],
            })),
            TEXT,
        )]
    },
    reason: "an untap-step replacement that untaps only if a +1/+1 counter was removed: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn untap_step(&self, g: &mut Game, active: PlayerId, to_untap: &mut Vec<ObjectId>) {
        let leeched: Vec<ObjectId> = to_untap
            .iter()
            .copied()
            .filter(|id| super::has_marker(g, *id, LEECH))
            .collect();
        for id in leeched {
            to_untap.retain(|x| *x != id);
            if g.remove_counters_by(Entity::Object(id), counters::PLUS1, 1, Some(active)) > 0 {
                g.untap(id);
            }
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
