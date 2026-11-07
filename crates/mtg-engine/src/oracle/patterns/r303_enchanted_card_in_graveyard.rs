//! Instructions of an Aura that enchants a card in a graveyard ("Enchant instant card in a
//! graveyard", CR 303.4a, 702.5a; Spellweaver Volute):
//!
//! - "copy the enchanted instant card": a copy of the card the Aura enchants (as the Aura
//!   last existed on the battlefield, CR 608.2h), created in the zone the card is in
//!   (CR 707.12). See `copy_rules::copy_cards`.
//! - "attach ~ to another instant card in a graveyard": the Aura's controller chooses a
//!   card it can enchant other than the one it enchants (CR 701.3a); with none, nothing
//!   happens (CR 701.3b), and an Aura that's then attached to nothing is put into its
//!   owner's graveyard (CR 704.5m).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::types::CardType;

fn copy_enchanted_card(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("copy the enchanted ")?;
    if r != "card" && !r.strip_suffix(" card").is_some_and(|t| CardType::from_word(t).is_some())
    {
        return None;
    }
    Some(Effect::CopyCard {
        what: Sel::AttachedTo,
        named: None,
    })
}

inventory::submit! { EffectPattern { name: "r303 copy the enchanted [type] card", priority: 100, parse: copy_enchanted_card } }

fn attach_to_another_card_in_a_graveyard(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("attach ~ to another ")?
        .strip_suffix(" card in a graveyard")?;
    let t = CardType::from_word(r)?;
    Some(Effect::Attach {
        what: Sel::This,
        to: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::Type(t),
                Filter::Card,
                Filter::InZone(ZoneKind::Graveyard),
                Filter::not(Filter::In(Box::new(Sel::AttachedTo))),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    })
}

inventory::submit! { EffectPattern { name: "r303 attach ~ to another [type] card in a graveyard", priority: 100, parse: attach_to_another_card_in_a_graveyard } }
