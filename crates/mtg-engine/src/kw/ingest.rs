//! CR 702.115 Ingest: "Whenever this creature deals combat damage to a player, that
//! player exiles the top card of their library." (CR 702.115a). Each instance triggers
//! separately (CR 702.115b). The card is exiled face up; a player with an empty library
//! exiles nothing (and doesn't lose the game).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Ingest;

impl KeywordRules for Ingest {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Ingest]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::DealsDamage {
                    source: Filter::Source,
                    to: DamageRecipient::Player(PlayerRel::Any),
                    combat_only: true,
                },
                Body::effect(Effect::Exile {
                    what: Sel::TopOfLibrary(PlayerRef::TriggerPlayer, Value::c(1)),
                    face_down: false,
                    link: false,
                }),
            )),
            KeywordKind::Ingest.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Ingest) }
