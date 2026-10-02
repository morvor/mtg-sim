//! Time and Tide: "Simultaneously, all phased-out creatures phase in and all creatures
//! with phasing phase out." (CR 702.26).

use super::{spell, ManualAbility};
use crate::ability::Effect;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const RESOLVE: &str = "card:Time and Tide:phased-out creatures phase in, creatures with phasing phase out";
const TEXT: &str = "Simultaneously, all phased-out creatures phase in and all creatures with phasing phase out.";

inventory::submit! { ManualAbility {
    card: "Time and Tide",
    face: 0,
    text: TEXT,
    build: |_| vec![spell(vec![], Effect::Custom(RESOLVE.into()), TEXT)],
    reason: "simultaneous phasing in and out of different groups: unique",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, _ctx: &mut Ctx) -> bool {
        if name != RESOLVE {
            return false;
        }
        // Both groups are determined first: a creature that phases in doesn't phase out.
        let out: Vec<ObjectId> = g
            .battlefield
            .iter()
            .copied()
            .filter(|id| {
                let o = g.obj(*id);
                !o.phased_out && o.is_creature() && o.has_keyword(KeywordKind::Phasing)
            })
            .collect();
        // Those that phased out indirectly phase in with what they're attached to.
        let back: Vec<ObjectId> = g
            .battlefield
            .iter()
            .copied()
            .filter(|id| {
                let o = g.obj(*id);
                o.phased_out && !o.phased_out_indirectly && o.is_creature()
            })
            .collect();
        crate::kw::phasing::phase_out(g, out);
        for id in back {
            crate::kw::phasing::phase_in(g, id);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
