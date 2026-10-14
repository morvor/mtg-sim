//! CR 603.6e: an Aura's ability that triggers when the enchanted permanent leaves the
//! battlefield can find the new object the Aura card became in the graveyard, also when
//! the trigger describes the enchanted permanent with more than "enchanted" ("When
//! enchanted creature dies": a creature that is the enchanted permanent).

use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn enchanted_creature_dies_returns_the_creature_and_then_the_aura() {
    cr!("603.6e");
    let mut t = TestGame::new(2);
    // "Enchant creature. When enchanted creature dies, return that card to the battlefield
    // under your control, then return this Aura to its owner's hand."
    let aura = CB::new("Undying Pact")
        .enchantment()
        .subtypes(&["Aura"])
        .ability(AbilityDef::new(
            AbilityKind::Keyword(mtg_engine::keywords::Keyword::with_filter(
                mtg_engine::keywords::KeywordKind::Enchant,
                Filter::creature(),
            )),
            "Enchant creature",
        ))
        .ability(trig(
            TriggerCond::Dies(Filter::And(vec![
                Filter::creature(),
                Filter::AttachedToSource,
            ])),
            Body::effect(Effect::Seq(vec![
                Effect::Move {
                    what: Sel::TriggerObject,
                    to: Destination::battlefield().under_your_control(),
                },
                Effect::Move {
                    what: Sel::This,
                    to: Destination::zone(ZoneKind::Hand),
                },
            ])),
        ))
        .build();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.custom(P0, aura, Zone::Battlefield);
    t.g.objects[a.0 as usize].attached_to = Some(Entity::Object(bears));
    t.g.recompute();
    t.g.destroy(bears, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Undying Pact"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_hand(P0, "Undying Pact"));
}
