//! CR 702.151 Reconfigure. "Reconfigure [cost]" means "[Cost]: Attach this permanent to
//! another target creature you control. Activate only as a sorcery" and "[Cost]: Unattach
//! this permanent. Activate only if this permanent is attached to a creature and only as
//! a sorcery." (CR 702.151a). An Equipment that's also a creature can equip a creature
//! only if it has reconfigure (CR 301.5c, see `attach.rs`).
//!
//! Attaching an Equipment with reconfigure to another creature, by any means, causes the
//! Equipment to stop being a creature until it becomes unattached from that creature
//! (CR 702.151b); it also loses its creature types (CR 205.1a). This is a
//! characteristic-changing effect in layer 4 generated along with the keyword, so it
//! keeps applying if the Equipment loses its abilities in layer 6 (CR 613.6, Bronzeplate
//! Boar rulings). An Equipment with reconfigure that's still a creature while attached
//! (another effect makes it one) becomes unattached (see `attach::legal_attachment`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::CardType;

pub struct Reconfigure;

impl KeywordRules for Reconfigure {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Reconfigure]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Reconfigure.name();
        let cost = kw.cost.clone().unwrap_or_default();
        let attached_to_creature =
            Condition::SelMatches(Sel::AttachedTo, Filter::Type(CardType::Creature));
        let mut attach = ActivatedAbility::new(
            cost.clone(),
            Body::simple(
                vec![TargetSpec::object(
                    Filter::and(vec![
                        Filter::creature(),
                        Filter::ControlledBy(PlayerRel::You),
                        Filter::Other,
                    ]),
                    "another target creature you control",
                )],
                Effect::Attach {
                    what: Sel::This,
                    to: Sel::Target(0),
                },
            ),
        );
        attach.timing = ActivationTiming::Sorcery;
        let mut unattach = ActivatedAbility::new(
            cost,
            Body::effect(Effect::Unattach { what: Sel::This }),
        );
        unattach.timing = ActivationTiming::Sorcery;
        unattach.condition = Some(attached_to_creature.clone());
        let mut not_a_creature = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![
                Modification::RemoveTypes(vec![CardType::Creature]),
                Modification::RemoveAllCreatureTypes,
            ],
        });
        not_a_creature.condition = Some(attached_to_creature);
        Some(vec![
            AbilityDef::new(AbilityKind::Activated(attach), text),
            AbilityDef::new(AbilityKind::Activated(unattach), text),
            AbilityDef::new(AbilityKind::Static(not_a_creature), text),
        ])
    }
}

inventory::submit! { KeywordRegistration(&Reconfigure) }
