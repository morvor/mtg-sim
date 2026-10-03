//! CR 702.52 Dredge. "Dredge N" means "As long as you have at least N cards in your
//! library, if you would draw a card, you may instead mill N cards and return this card
//! from your graveyard to your hand." (CR 702.52a). It's a static ability that functions
//! only while the card is in a graveyard: an optional replacement effect of each draw
//! (CR 614.1a, 616.1), which a player with fewer than N cards in their library can't use
//! (CR 702.52b). Each draw is replaced separately (CR 121.2), and a draw replaced by one
//! dredge ability is gone, so no other dredge ability can replace it.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};

pub struct Dredge;

impl KeywordRules for Dredge {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Dredge]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(0).max(0);
        let mut s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Draw(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(Effect::Seq(vec![
                Effect::Mill {
                    who: PlayerRef::You,
                    n: Value::c(n),
                },
                Effect::Move {
                    what: Sel::This,
                    to: Destination::zone(ZoneKind::Hand),
                },
            ]))),
            self_replacement: false,
            optional: true,
        }));
        s.zone = FunctionZone::Graveyard;
        // CR 702.52b: "As long as you have at least N cards in your library".
        s.condition = Some(Condition::Compare(
            Value::LibrarySize(PlayerRef::You),
            Cmp::Ge,
            Value::c(n),
        ));
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Dredge.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Dredge) }
